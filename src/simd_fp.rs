use crate::{
    branch::condition_holds,
    cpu::Cpu,
    data_processing::shift_lsl,
    get_bits_ct,
    instruction::{InstDesc, Instruction},
    load_and_store::{ExtendType, extend_register},
    simd_fp_instr::*,
    utils::{BitUtils, bits_get, elem_get, elem_set, sign_extend},
};

/// Floating-point convert/move instruction types.
#[derive(Clone, Copy, Debug)]
pub enum FpConvOp {
    CvtFtoI,
    CvtItoF,
    CvtFtoIJs,
    MovFtoI,
    MovItoF,
}

/// Floating-point min/max instruction types.
#[derive(Clone, Copy, Debug)]
pub enum FpMaxMinOp {
    Max,
    Min,
    MaxNum,
    MinNum,
}

/// Unsigned move vector element to general-purpose register
/// MOV (alias when moving D lane to X register) / UMOV
#[derive(Clone, Copy, Debug)]
pub struct UmovAdvsimd {
    pub q: bool,
    pub imm5: u8,
    pub rn: u8,
    pub rd: u8,
}

impl UmovAdvsimd {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        // imm5 IN 'x0000' is UNDEFINED (reserved for ASIMDIMM group)
        if bits_get(self.imm5.into(), 0, 4) == 0 {
            panic!("Undefined: imm5<3:0> == 0000");
        }

        // Determine element size from lowest set bit in imm5<3:0>
        let size = bits_get(self.imm5.into(), 0, 4).trailing_zeros();
        let esize = 8 << size;
        let datasize = if self.q { 64 } else { 32 };

        // Validity checks per ARM spec
        // datasize == 64 && esize < 64 → UNDEFINED
        if datasize == 64 && esize < 64 {
            panic!("Undefined: 64-bit destination requires 64-bit element");
        }
        // datasize == 32 && esize >= 64 → UNDEFINED
        if datasize == 32 && esize >= 64 {
            panic!("Undefined: 32-bit destination cannot have 64-bit element");
        }

        // Calculate index from imm5<4:size+1>
        let index = bits_get(self.imm5.into(), (size + 1) as u8, 4 - size as u8) as usize;

        // Determine vector slice size: 64 << imm5<4>
        let idxdsize = 64 << ((self.imm5 >> 4) & 1);

        // Read the element from the vector register
        let operand = cpu.v_read(self.rn.into(), idxdsize);
        let element = elem_get(operand, index, esize as usize);

        // Zero-extend to destination size and write to general-purpose register
        cpu.x_write(self.rd.into(), element, datasize == 32);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) == 1;
        let imm5 = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::UmovAdvsimd(Self { q, imm5, rn, rd })
    }

    pub const UMOV_ADVSIMD: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0000_1110_0000_0000_0011_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Insert vector element from general-purpose register
/// INS <Vd>.<Ts>[<index>], <R><n>
/// Alias: MOV (from general)
#[derive(Clone, Copy, Debug)]
pub struct InsAdvsimdGen {
    pub imm5: u8,
    pub rn: u8,
    pub rd: u8,
}

impl InsAdvsimdGen {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        // imm5 IN 'x0000' is UNDEFINED
        if bits_get(self.imm5.into(), 0, 4) == 0 {
            panic!("Undefined: imm5<3:0> == 0000");
        }

        // Determine element size from lowest set bit in imm5<3:0>
        let size = bits_get(self.imm5.into(), 0, 4).trailing_zeros();
        let esize = (8 << size) as usize;

        // Calculate index from imm5<4:size+1>
        let index = bits_get(self.imm5.into(), (size + 1) as u8, 4 - size as u8) as usize;

        // Read the source general-purpose register, truncated to esize
        let element = cpu.x_read(self.rn.into(), esize as u8);

        // Read current 128-bit vector, insert element, write back
        let result = cpu.v_read(self.rd.into(), 128);
        let result = elem_set(result, index, esize, element);
        cpu.v_write(self.rd.into(), 128, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm5 = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::InsAdvsimdGen(Self { imm5, rn, rd })
    }

    pub const INS_ADVSIMD_GEN: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0100_1110_0000_0000_0001_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Duplicate general-purpose register to vector
#[derive(Clone, Copy, Debug)]
pub struct DupGeneral {
    pub q: bool,
    pub imm5: u8,
    pub rn: u8,
    pub rd: u8,
}

impl DupGeneral {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if bits_get(self.imm5.into(), 0, 4) == 0 {
            panic!("Undefined")
        }
        if bits_get(self.imm5.into(), 0, 4) == 0b1000 && !self.q {
            panic!("Undefined")
        }
        let size = bits_get(self.imm5.into(), 0, 4).trailing_zeros();

        let esize = 8 << size;
        let datasize = 64 << self.q as u8;
        dup_general_instruction(cpu, self.rn, self.rd, esize, datasize);
    }
    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) == 1;
        let imm5 = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::DupGeneral(Self { q, imm5, rn, rd })
    }
    pub const DUP_GENERAL: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0000_1110_0000_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct StrImdFpPostIndex {
    pub size: u8,
    pub opc: u8,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrImdFpPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.opc.single_bit(1) == 1 && self.size != 0 {
            panic!("Undefined");
        }

        let scale = if self.opc.single_bit(1) == 1 { 4 } else { self.size };
        let wback = true;
        let postindex = true;
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = 8 << scale;

        str_imd_fp_instruction(cpu, self.rn, self.rt, datasize, postindex, wback, offset);
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrImdFpPostIndex(Self { size, opc, imm9, rn, rt })
    }

    pub const STR_IMD_FP_POST_INDEX: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_1100_0000_0000,
        value: 0b0011_1100_0000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct StrImdFpPreIndex {
    pub size: u8,
    pub opc: u8,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrImdFpPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.opc.single_bit(1) == 1 && self.size != 0 {
            panic!("Undefined");
        }

        let scale = if self.opc.single_bit(1) == 1 { 4 } else { self.size };
        let wback = true;
        let postindex = false;
        let offset = sign_extend(self.imm9.into(), 9);

        let datasize = 8 << scale;

        str_imd_fp_instruction(cpu, self.rn, self.rt, datasize, postindex, wback, offset);
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrImdFpPreIndex(Self { size, opc, imm9, rn, rt })
    }
    pub const STR_IMD_FP_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_1100_0000_0000,
        value: 0b0011_1100_0000_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct StrImdFpUnsignedOffset {
    pub size: u8,
    pub opc: u8,
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrImdFpUnsignedOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.opc.single_bit(1) == 1 && self.size != 0 {
            panic!("Undefined");
        }

        let scale = if self.opc.single_bit(1) == 1 { 4 } else { self.size };
        let offset = (self.imm12 as u64) << scale;
        let datasize = 8 << scale;

        str_imd_fp_instruction(cpu, self.rn, self.rt, datasize, false, false, offset);
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrImdFpUnsignedOffset(Self { size, opc, imm12, rn, rt })
    }
    pub const STR_IMD_FP_UNSIGNED_OFFSET: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_0000_0000_0000,
        value: 0b0011_1101_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

// STP (SIMD&FP)
#[derive(Debug, Clone, Copy)]
pub struct StrPairFpPostIndex {
    pub opc: u8,
    pub imm7: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}
impl StrPairFpPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);

        str_pair_fp_instruction(cpu, self.rn, self.rt, self.rt2, offset, datasize, true, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 30, 2) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrPairFpPostIndex(Self { opc, imm7, rt2, rn, rt })
    }

    pub const STR_PAIR_FP_POST_INDEX: InstDesc = InstDesc {
        mask: 0b0011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1100_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

// STP (SIMD&FP)
#[derive(Debug, Clone, Copy)]
pub struct StrPairFpPreIndex {
    pub opc: u8,
    pub imm7: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}
impl StrPairFpPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);

        str_pair_fp_instruction(cpu, self.rn, self.rt, self.rt2, offset, datasize, false, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 30, 2) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrPairFpPreIndex(Self { opc, imm7, rt2, rn, rt })
    }
    pub const STR_PAIR_FP_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b0011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1101_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

// STP (SIMD&FP)
#[derive(Debug, Clone, Copy)]
pub struct StrPairFpSignedOffset {
    pub opc: u8,
    pub imm7: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}
impl StrPairFpSignedOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);

        str_pair_fp_instruction(cpu, self.rn, self.rt, self.rt2, offset, datasize, false, false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 30, 2) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrPairFpSignedOffset(Self { opc, imm7, rt2, rn, rt })
    }
    pub const STR_PAIR_FP_SIGNED_OFFSET: InstDesc = InstDesc {
        mask: 0b0011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1101_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load pair of SIMD&FP registers
#[derive(Debug, Clone, Copy)]
pub struct LdpSimdFpPostIndex {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl LdpSimdFpPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_ldp_simd_fp(cpu, self.rt, self.rt2, self.rn, datasize, offset, true, true);
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 30, 2) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdpSimdFpPostIndex(Self { opc, imm7, rt2, rn, rt })
    }
    pub const LDP_SIMD_FP_POST_INDEX: InstDesc = InstDesc {
        mask: 0b0011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1100_1100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load pair of SIMD&FP registers
#[derive(Debug, Clone, Copy)]
pub struct LdpSimdFpPreIndex {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl LdpSimdFpPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_ldp_simd_fp(cpu, self.rt, self.rt2, self.rn, datasize, offset, false, true);
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 30, 2) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdpSimdFpPreIndex(Self { opc, imm7, rt2, rn, rt })
    }
    pub const LDP_SIMD_FP_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b0011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1101_1100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load pair of SIMD&FP registers
#[derive(Debug, Clone, Copy)]
pub struct LdpSimdFpSignedOffset {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl LdpSimdFpSignedOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_ldp_simd_fp(cpu, self.rt, self.rt2, self.rn, datasize, offset, false, false);
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 30, 2) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdpSimdFpSignedOffset(Self { opc, imm7, rt2, rn, rt })
    }
    pub const LDP_SIMD_FP_SIGNED_OFFSET: InstDesc = InstDesc {
        mask: 0b0011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1101_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Advanced SIMD modified immediate
/// Handles MOVI, MVNI, ORR (immediate), BIC (immediate), FMOV (immediate)
#[derive(Debug, Clone, Copy)]
pub struct AsimdModImm {
    pub q: u8,
    pub op: u8,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub cmode: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub g: u8,
    pub h: u8,
    pub rd: u8,
}

impl AsimdModImm {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 64 << self.q;
        let imm8 = self.a << 7
            | self.b << 6
            | self.c << 5
            | self.d << 4
            | self.e << 3
            | self.f << 2
            | self.g << 1
            | self.h;
        let imm64 = adv_simd_expand_imm(self.op, self.cmode, imm8);

        // Determine operation based on op and cmode
        // ARM encoding table for asimdimm:
        //   cmode=0xx0, 10x0, 110x: MOVI (op=0) or MVNI (op=1)
        //   cmode=0xx1, 10x1:       ORR (op=0) or BIC (op=1)
        //   cmode=1110:             MOVI (both op values)
        //   cmode=1111:             FMOV (both op values)
        //
        // ORR/BIC condition: cmode[0]==1 AND cmode[3:2]!=11 (i.e., cmode < 12
        // && odd)
        let is_orr_bic = (self.cmode & 1) == 1 && (self.cmode >> 2) != 0b11;

        let imm128: u128 = if datasize == 128 {
            // Replicate 64-bit value to 128 bits
            (imm64 as u128) | ((imm64 as u128) << 64)
        } else {
            imm64 as u128
        };

        let result: u128 = if self.cmode == 0b1111 {
            // FMOV - move immediate to vector
            imm128
        } else if self.cmode == 0b1110 {
            // MOVI - 8-bit or 64-bit immediate
            imm128
        } else if is_orr_bic {
            // ORR (op=0) or BIC (op=1)
            let old_val = cpu.v_read(self.rd.into(), datasize as u8);
            if self.op == 0 {
                old_val | imm128 // ORR
            } else {
                old_val & !imm128 // BIC
            }
        } else {
            // MOVI (op=0) or MVNI (op=1)
            if self.op == 0 {
                imm128 // MOVI
            } else {
                // MVNI: invert and mask to datasize
                let mask = if datasize == 128 { !0u128 } else { (1u128 << 64) - 1 };
                !imm128 & mask
            }
        };

        cpu.v_write(self.rd.into(), datasize as u8, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let op = get_bits_ct!(word, 29, 1) as u8;
        let a = get_bits_ct!(word, 18, 1) as u8;
        let b = get_bits_ct!(word, 17, 1) as u8;
        let c = get_bits_ct!(word, 16, 1) as u8;
        let cmode = get_bits_ct!(word, 12, 4) as u8;
        let d = get_bits_ct!(word, 9, 1) as u8;
        let e = get_bits_ct!(word, 8, 1) as u8;
        let f = get_bits_ct!(word, 7, 1) as u8;
        let g = get_bits_ct!(word, 6, 1) as u8;
        let h = get_bits_ct!(word, 5, 1) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AsimdModImm(Self { q, op, a, b, c, cmode, d, e, f, g, h, rd })
    }

    pub const ASIMD_MOD_IMM: InstDesc = InstDesc {
        mask: 0b1001_1111_1111_1000_0000_1100_0000_0000,
        value: 0b0000_1111_0000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

pub fn adv_simd_expand_imm(op: u8, cmode: u8, imm8: u8) -> u64 {
    let mut imm64: u64 = 0;

    match cmode.bits_get(1, 3) {
        0b000 => {
            // 32-bit element: imm8 zero-extended, replicated 2x to fill 64 bits
            imm64 = (imm8 as u32).replicate_pattern::<u64>(32, 2);
        }
        0b001 => {
            // 32-bit element: imm8 << 8, replicated 2x
            let imm32 = (imm8 as u32) << 8;
            imm64 = imm32.replicate_pattern::<u64>(32, 2);
        }
        0b010 => {
            // 32-bit element: imm8 << 16, replicated 2x
            let imm32 = (imm8 as u32) << 16;
            imm64 = imm32.replicate_pattern::<u64>(32, 2);
        }
        0b011 => {
            // 32-bit element: imm8 << 24, replicated 2x
            let imm32 = (imm8 as u32) << 24;
            imm64 = imm32.replicate_pattern::<u64>(32, 2);
        }
        0b100 => {
            // 16-bit element: imm8 in low byte, replicated 4x
            imm64 = (imm8 as u16).replicate_pattern::<u64>(16, 4);
        }
        0b101 => {
            // 16-bit element: imm8 in high byte, replicated 4x
            let imm16 = (imm8 as u16) << 8;
            imm64 = imm16.replicate_pattern::<u64>(16, 4);
        }
        0b110 => {
            if cmode.single_bit(0) == 0 {
                // 32-bit element: (imm8 << 8) | 0xFF, replicated 2x
                let imm32 = ((imm8 as u32) << 8) | 0xFF;
                imm64 = imm32.replicate_pattern::<u64>(32, 2);
            } else {
                // 32-bit element: (imm8 << 16) | 0xFFFF, replicated 2x
                let imm32 = ((imm8 as u32) << 16) | 0xFFFF;
                imm64 = imm32.replicate_pattern::<u64>(32, 2);
            }
        }
        0b111 => {
            if cmode.single_bit(0) == 0 && op == 0 {
                // 8-bit element: imm8 replicated 8x
                imm64 = imm8.replicate_pattern::<u64>(8, 8);
            }
            if cmode.single_bit(0) == 0 && op == 1 {
                // 64-bit element: each bit of imm8 expanded to a byte
                let imm8a: u8 = imm8.single_bit(7).replicate_bit(8);
                let imm8b: u8 = imm8.single_bit(6).replicate_bit(8);
                let imm8c: u8 = imm8.single_bit(5).replicate_bit(8);
                let imm8d: u8 = imm8.single_bit(4).replicate_bit(8);
                let imm8e: u8 = imm8.single_bit(3).replicate_bit(8);
                let imm8f: u8 = imm8.single_bit(2).replicate_bit(8);
                let imm8g: u8 = imm8.single_bit(1).replicate_bit(8);
                let imm8h: u8 = imm8.single_bit(0).replicate_bit(8);
                imm64 =
                    u64::from_be_bytes([imm8a, imm8b, imm8c, imm8d, imm8e, imm8f, imm8g, imm8h]);
            }
            if cmode.single_bit(0) == 1 && op == 0 {
                // FMOV (single): IEEE 754 single-precision
                let bit7 = imm8.single_bit(7) as u32;
                let bit6 = imm8.single_bit(6) as u32;
                let bits5_0 = imm8.bits_get(0, 6) as u32;
                let imm32: u32 = (bit7 << 31)
                    | ((bit6 ^ 1) << 30)
                    | (bit6.replicate_bit::<u32>(5) << 25)
                    | (bits5_0 << 19);
                imm64 = imm32.replicate_pattern::<u64>(32, 2);
            }
            if cmode.single_bit(0) == 1 && op == 1 {
                // FMOV (double): IEEE 754 double-precision
                let bit7 = imm8.single_bit(7) as u64;
                let bit6 = imm8.single_bit(6) as u64;
                let bits5_0 = imm8.bits_get(0, 6) as u64;
                imm64 = (bit7 << 63)
                    | ((bit6 ^ 1) << 62)
                    | (bit6.replicate_bit::<u64>(8) << 54)
                    | (bits5_0 << 48);
            }
        }
        _ => panic!("Unknown cmode"),
    }

    imm64
}

/// Load SIMD&FP register (immediate offset)
#[derive(Debug, Clone, Copy)]
pub struct LdrSimdFpPostIndex {
    size: u8,
    opc: u8,
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl LdrSimdFpPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.opc.single_bit(1) == 1 && self.size != 0 {
            panic!("Undefined");
        }
        let scale = if self.opc.single_bit(1) == 1 { 4 } else { self.size };
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = 8 << scale;
        instruction_ldr_simd_fp(cpu, self.rt, self.rn, datasize, offset, true, true);
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrSimdFpPostIndex(Self { size, opc, imm9, rn, rt })
    }
    pub const LDR_SIMD_FP_POST_INDEX: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_1100_0000_0000,
        value: 0b0011_1100_0100_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Load SIMD&FP register (immediate offset)
#[derive(Debug, Clone, Copy)]
pub struct LdrSimdFpPreIndex {
    size: u8,
    opc: u8,
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl LdrSimdFpPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.opc.single_bit(1) == 1 && self.size != 0 {
            panic!("Undefined");
        }
        let scale = if self.opc.single_bit(1) == 1 { 4 } else { self.size };
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = 8 << scale;
        instruction_ldr_simd_fp(cpu, self.rt, self.rn, datasize, offset, false, true);
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrSimdFpPreIndex(Self { size, opc, imm9, rn, rt })
    }
    pub const LDR_SIMD_FP_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_1100_0000_0000,
        value: 0b0011_1100_0100_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load SIMD&FP register (immediate offset)
#[derive(Debug, Clone, Copy)]
pub struct LdrSimdFpUnsignedOffset {
    size: u8,
    opc: u8,
    imm12: u16,
    rn: u8,
    rt: u8,
}

impl LdrSimdFpUnsignedOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.opc.single_bit(1) == 1 && self.size != 0 {
            panic!("Undefined");
        }
        let scale = if self.opc.single_bit(1) == 1 { 4 } else { self.size };
        let offset = (self.imm12 as u64) << scale;
        let datasize = 8 << scale;
        instruction_ldr_simd_fp(cpu, self.rt, self.rn, datasize, offset, false, false);
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrSimdFpUnsignedOffset(Self { size, opc, imm12, rn, rt })
    }
    pub const LDR_SIMD_FP_UNSIGNED_OFFSET: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_0000_0000_0000,
        value: 0b0011_1101_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store SIMD&FP register (register offset)
#[derive(Debug, Clone, Copy)]
pub struct StrSimdRegOffset {
    pub size: u8,
    pub opc: u8,
    pub rm: u8,
    pub option: u8,
    pub s: u8,
    pub rn: u8,
    pub rt: u8,
}

impl StrSimdRegOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.option.single_bit(1) == 0 {
            panic!("Undefined");
        }
        if self.opc.single_bit(1) == 1 && self.size != 0 {
            panic!("Undefined");
        }

        let scale = if self.opc.single_bit(1) == 1 { 4 } else { self.size };

        let extend_type = ExtendType::from_u8(self.option);

        let shift = if self.s == 1 { scale } else { 0 };

        let datasize = 8 << scale;
        let offset = extend_register(cpu, self.rm, extend_type, shift, 64);

        let mut address = cpu.address_for_rn(self.rn);
        address = address.wrapping_add(offset);
        cpu.write_memory_128bit(address.try_into().unwrap(), cpu.v_read(self.rt.into(), datasize));
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let s = get_bits_ct!(word, 12, 1) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrSimdRegOffset(Self { size, opc, rm, option, s, rn, rt })
    }

    pub const STR_SIMD_REG_OFFSET: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_1100_0000_0000,
        value: 0b0011_1100_0010_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Store SIMD&FP register (unscaled offset)
#[derive(Debug, Clone, Copy)]
pub struct SturSimdUnscaledOffset {
    pub size: u8,
    pub opc: u8,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl SturSimdUnscaledOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.opc.single_bit(1) == 1 && self.size != 0 {
            panic!("Undefined");
        }

        let scale = if self.opc.single_bit(1) == 1 { 4 } else { self.size };
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = 8 << scale;

        let mut address = cpu.address_for_rn(self.rn);
        address = address.wrapping_add(offset);
        if datasize == 128 {
            cpu.write_memory_128bit(
                address.try_into().unwrap(),
                cpu.v_read(self.rt.into(), datasize),
            );
        } else {
            cpu.write_memory(
                address.try_into().unwrap(),
                datasize as usize / 8,
                cpu.v_read(self.rt.into(), datasize) as u64,
            );
        }
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::SturSimdUnscaledOffset(Self { size, opc, imm9, rn, rt })
    }

    pub const STUR_SIMD_UNSCALED_OFFSET: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_1100_0000_0000,
        value: 0b0011_1100_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Ld1NoOffset {
    pub q: u8,
    pub opcode: u8,
    pub size: u8,
    pub rn: u8,
    pub rt: u8,
}

impl Ld1NoOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 64 << self.q;
        let esize = 8 << self.size;
        let elements = datasize / esize;

        // number of iterations
        let rpt = match self.opcode {
            0b0010 => 4,
            0b0110 => 3,
            0b1010 => 2,
            0b0111 => 1,
            _ => panic!("Undefined"),
        };

        instruction_ld1(cpu, self.rn, 0, self.rt, elements, esize, datasize, rpt, false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let opcode = get_bits_ct!(word, 12, 4) as u8;
        let size = get_bits_ct!(word, 10, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ld1NoOffset(Self { q, opcode, size, rn, rt })
    }

    pub const LD1_NO_OFFSET: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_1111_0010_0000_0000_0000,
        value: 0b0000_1100_0100_0000_0010_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Ld1PostIndex {
    pub q: u8,
    pub opcode: u8,
    pub size: u8,
    pub rm: u8,
    pub rn: u8,
    pub rt: u8,
}

impl Ld1PostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 64 << self.q;
        let esize = 8 << self.size;
        let elements = datasize / esize;

        // number of iterations
        let rpt = match self.opcode {
            0b0010 => 4,
            0b0110 => 3,
            0b1010 => 2,
            0b0111 => 1,
            _ => panic!("Undefined"),
        };

        instruction_ld1(cpu, self.rn, self.rm, self.rt, elements, esize, datasize, rpt, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let opcode = get_bits_ct!(word, 12, 4) as u8;
        let size = get_bits_ct!(word, 10, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ld1PostIndex(Self { q, rm, opcode, size, rn, rt })
    }

    pub const LD1_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0010_0000_0000_0000,
        value: 0b0000_1100_1100_0000_0010_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Compare unsigned higher or same (vector)
#[derive(Debug, Clone, Copy)]
pub struct CmhsRegisterVector {
    pub q: u8,
    pub size: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl CmhsRegisterVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let size_and_q = (self.size << 1) | self.q;
        if size_and_q == 0b110 {
            panic!("Undefiend");
        }
        let esize = 8 << self.size;
        let datasize = 64 << self.q;
        let elements = datasize / esize;
        instruction_cmhs_register(cpu, self.rn, self.rm, self.rd, esize, datasize, elements);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let size = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::CmhsRegisterVector(Self { q, size, rm, rn, rd })
    }

    pub const CMHS_REGISTER_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_0010_0000_1111_1100_0000_0000,
        value: 0b0010_1110_0010_0000_0011_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Compare unsigned higher or same (vector)
#[derive(Debug, Clone, Copy)]
pub struct CmhsRegisterScalar {
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl CmhsRegisterScalar {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let esize = 8 << 0b11;
        let datasize = esize;
        let elements = 1;
        instruction_cmhs_register(cpu, self.rn, self.rm, self.rd, esize, datasize, elements);
    }

    pub const fn decode(word: u32) -> Instruction {
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::CmhsRegisterScalar(Self { rm, rn, rd })
    }

    pub const CMHS_REGISTER_SCALAR: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0111_1110_1110_0000_0011_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Compare bitwise equal to zero (vector)
#[derive(Debug, Clone, Copy)]
pub struct CmeqVector {
    pub rn: u8,
    pub q: u8,
    pub size: u8,
    pub rd: u8,
}

impl CmeqVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if ((self.size << 1) | self.q) == 0b110 {
            panic!("Undefined CMEQ");
        }

        let esize = 8 << self.size;
        let datasize = 64 << self.q;
        let elements = datasize / esize;

        instruction_cmeq(cpu, self.rn, self.rd, esize, datasize, elements);
    }

    pub const fn decode(word: u32) -> Instruction {
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let q = get_bits_ct!(word, 30, 1) as u8;
        let size = get_bits_ct!(word, 22, 2) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::CmeqVector(Self { rn, q, size, rd })
    }

    pub const CMEQ_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_0011_1111_1111_1100_0000_0000,
        value: 0b0000_1110_0010_0000_1001_1000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
/// Compare bitwise equal to zero (vector)
pub struct CmeqScalar {
    pub rn: u8,
    pub rd: u8,
}

impl CmeqScalar {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let esize = 8 << 0b11;
        let datasize = esize;
        let elements = 1;

        instruction_cmeq(cpu, self.rn, self.rd, esize, datasize, elements);
    }

    pub const fn decode(word: u32) -> Instruction {
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::CmeqScalar(Self { rn, rd })
    }

    pub const CMEQ_SCALAR: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1100_0000_0000,
        value: 0b0101_1110_1110_0000_1001_1000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
/// Compare bitwise equal (vector)
pub struct CmeqRegVector {
    pub q: u8,
    pub size: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl CmeqRegVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if ((self.size << 1) | self.q) == 0b110 {
            panic!("Undefined CMEQ");
        }

        let esize = 8 << self.size;
        let datasize = 64 << self.q;
        let elements = datasize / esize;

        instruction_cmeq_req(cpu, self.rn, self.rm, self.rd, esize, datasize, elements);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let size = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::CmeqRegVector(Self { q, size, rm, rn, rd })
    }

    pub const CMEQ_REG_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_0010_0000_1111_1100_0000_0000,
        value: 0b0010_1110_0010_0000_1000_1100_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
/// Compare bitwise equal (vector)
pub struct CmeqRegScalar {
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl CmeqRegScalar {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let esize = 8 << 0b11;
        let datasize = esize;
        let elements = 1;

        instruction_cmeq_req(cpu, self.rn, self.rm, self.rd, esize, datasize, elements);
    }

    pub const fn decode(word: u32) -> Instruction {
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::CmeqRegScalar(Self { rm, rn, rd })
    }

    pub const CMEQ_REG_SCALAR: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0111_1110_1110_0000_1000_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Bitwise insert if true
#[derive(Debug, Clone, Copy)]
pub struct BitwiseInsert {
    pub q: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl BitwiseInsert {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 64 << self.q;
        let operand1 = cpu.v_read(self.rd.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);
        let operand3 = cpu.v_read(self.rn.into(), datasize);

        let xor_op1_op3 = operand1 ^ operand3;
        let xor_and_op2 = xor_op1_op3 & operand2;

        let val = operand1 ^ xor_and_op2;

        cpu.v_write(self.rd.into(), datasize, val);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::BitwiseInsert(Self { q, rm, rn, rd })
    }

    pub const BITWISE_INSERT: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0010_1110_1010_0000_0001_1100_0000_0000,
        decode: Self::decode,
    };
}

/// AND (vector)
/// Bitwise AND of two SIMD registers
#[derive(Debug, Clone, Copy)]
pub struct AndVector {
    pub q: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AndVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 64 << self.q;
        let operand1 = cpu.v_read(self.rn.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);
        cpu.v_write(self.rd.into(), datasize, operand1 & operand2);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AndVector(Self { q, rm, rn, rd })
    }

    pub const AND_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0000_1110_0010_0000_0001_1100_0000_0000,
        decode: Self::decode,
    };
}

/// ORR (vector, register); MOV (vector) when Rm == Rn
#[derive(Debug, Clone, Copy)]
pub struct OrrVector {
    pub q: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl OrrVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 64 << self.q;
        let operand1 = cpu.v_read(self.rn.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);
        cpu.v_write(self.rd.into(), datasize, operand1 | operand2);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::OrrVector(Self { q, rm, rn, rd })
    }

    pub const ORR_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0000_1110_1010_0000_0001_1100_0000_0000,
        decode: Self::decode,
    };
}

/// BIC (vector, register)
#[derive(Debug, Clone, Copy)]
pub struct BicVector {
    pub q: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl BicVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 64 << self.q;
        let operand1 = cpu.v_read(self.rn.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);
        cpu.v_write(self.rd.into(), datasize, operand1 & !operand2);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::BicVector(Self { q, rm, rn, rd })
    }

    pub const BIC_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0000_1110_0110_0000_0001_1100_0000_0000,
        decode: Self::decode,
    };
}

/// NOT (vector); alias MVN
#[derive(Debug, Clone, Copy)]
pub struct NotVector {
    pub q: u8,
    pub rn: u8,
    pub rd: u8,
}

impl NotVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 64 << self.q;
        let operand = cpu.v_read(self.rn.into(), datasize);
        cpu.v_write(self.rd.into(), datasize, !operand);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::NotVector(Self { q, rn, rd })
    }

    pub const NOT_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_1111_1111_1100_0000_0000,
        value: 0b0010_1110_0010_0000_0101_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Shift right narrow (immediate)
#[derive(Debug, Clone, Copy)]
pub struct Shrn {
    pub q: u8,
    pub immh: u8,
    pub immb: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Shrn {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        // Maybe we should use assert for these ?
        // TODO
        if self.immh == 0 {
            panic!("See asimdim");
        }
        if self.immh.single_bit(3) == 1 {
            panic!("Undefined");
        }

        let highest_set_bit = self.immh.bits_get(0, 3).highest_one().unwrap();
        let esize = 8 << highest_set_bit;
        let datasize = 64;
        let part = self.q;
        let elements = datasize / esize;

        let immh_immb = (self.immh << 3) | self.immb;
        let shift = (2 * esize) - (immh_immb);

        let operand = cpu.v_read(self.rn.into(), datasize * 2);

        let mut result: u128 = 0;
        for e in 0..elements {
            let element = elem_get(operand, e as usize, (esize * 2) as usize) >> shift;
            let element = element.bits_get(0, esize);
            result = elem_set(result, e as usize, esize as usize, element);
        }
        cpu.v_part_write(self.rd.into(), part, datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let immh = get_bits_ct!(word, 19, 4) as u8;
        let immb = get_bits_ct!(word, 16, 3) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Shrn(Self { q, immh, immb, rn, rd })
    }

    // Split into 3 variants to exclude immh=0000 (asimdimm/MOVI) and immh=1xxx
    // (RESERVED) immh = 01xx (32-bit elements)
    pub const SHRN_01XX: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0000_1111_0010_0000_1000_0100_0000_0000,
        decode: Self::decode,
    };
    // immh = 001x (16-bit elements)
    pub const SHRN_001X: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_0000_1111_1100_0000_0000,
        value: 0b0000_1111_0001_0000_1000_0100_0000_0000,
        decode: Self::decode,
    };
    // immh = 0001 (8-bit elements)
    pub const SHRN_0001: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_1000_1111_1100_0000_0000,
        value: 0b0000_1111_0000_1000_1000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Signed shift left long (immediate)
/// SSHLL{2} <Vd>.<Ta>, <Vn>.<Tb>, #<shift>
/// Alias: SXTL/SXTL2 (when shift == 0)
#[derive(Clone, Copy, Debug)]
pub struct SshllAdvsimd {
    pub q: u8,
    pub immh: u8,
    pub immb: u8,
    pub rn: u8,
    pub rd: u8,
}

impl SshllAdvsimd {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.immh == 0 {
            panic!("See asimdimm");
        }
        if self.immh & 0b1000 != 0 {
            panic!("Undefined: immh<3> == 1");
        }

        // esize = 8 << HighestSetBitNZ(immh<2:0>)
        let highest_set_bit = self.immh.bits_get(0, 3).highest_one().unwrap();
        let esize = 8 << highest_set_bit;
        let datasize: u8 = 64;
        let part = self.q;
        let elements = datasize / esize;

        // shift = UInt(immh:immb) - esize
        let immh_immb = ((self.immh as u32) << 3) | self.immb as u32;
        let shift = immh_immb - esize as u32;

        // Read source: lower or upper 64 bits depending on Q
        let operand = cpu.v_part_read(self.rn.into(), part, datasize);

        let mut result: u128 = 0;
        for e in 0..elements {
            // Sign-extend element, shift left, truncate to 2*esize
            let raw = elem_get(operand, e as usize, esize as usize);
            let signed = sign_extend(raw, esize) as i64 as i128;
            let shifted = (signed << shift) as u128;
            let dest_esize = (esize * 2) as usize;
            let truncated = (shifted & ((1u128 << dest_esize) - 1)) as u64;
            result = elem_set(result, e as usize, dest_esize, truncated);
        }

        cpu.v_write(self.rd.into(), datasize * 2, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let immh = get_bits_ct!(word, 19, 4) as u8;
        let immb = get_bits_ct!(word, 16, 3) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::SshllAdvsimd(Self { q, immh, immb, rn, rd })
    }

    // Split into 3 variants to exclude immh=0000 (asimdimm) and immh=1xxx
    // (RESERVED) immh = 01xx (32→64 bit elements)
    pub const SSHLL_01XX: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0000_1111_0010_0000_1010_0100_0000_0000,
        decode: Self::decode,
    };
    // immh = 001x (16→32 bit elements)
    pub const SSHLL_001X: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_0000_1111_1100_0000_0000,
        value: 0b0000_1111_0001_0000_1010_0100_0000_0000,
        decode: Self::decode,
    };
    // immh = 0001 (8→16 bit elements)
    pub const SSHLL_0001: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_1000_1111_1100_0000_0000,
        value: 0b0000_1111_0000_1000_1010_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Floating-point move to or from general-purpose register without conversion
#[derive(Debug, Clone, Copy)]
pub struct FmovGeneral {
    pub sf: u8,
    pub ftype: u8,
    pub rmode: u8,
    pub opcode: u8,
    pub rn: u8,
    pub rd: u8,
}

impl FmovGeneral {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let opcode_rmode = (0b11 << 2) | self.rmode;
        if self.ftype == 0b10 && opcode_rmode != 0b1101 {
            panic!("Undefined")
        }

        let intsize = 32 << self.sf;
        let fltsize = if self.ftype == 0b10 { 64 } else { 8 << (self.ftype ^ 0b10) };
        let part = self.rmode.bits_get(0, 1);
        let op = match opcode_rmode {
            // FMOV
            0b11_00 => {
                if fltsize != 16 && fltsize != intsize {
                    panic!("Undefined");
                }
                if self.opcode.bits_get(0, 1) == 1 { FpConvOp::MovItoF } else { FpConvOp::MovFtoI }
            }
            0b11_01 => {
                if intsize != 64 || self.ftype != 0b10 {
                    panic!("Undefined");
                }
                if self.opcode.bits_get(0, 1) == 1 { FpConvOp::MovItoF } else { FpConvOp::MovFtoI }
            }
            _ => panic!("Unreachable"),
        };

        match op {
            FpConvOp::MovFtoI => {
                let fltval = cpu.v_part_read(self.rn.into(), part, fltsize) as u64;
                cpu.x_write(self.rd.into(), fltval, intsize == 32);
            }
            FpConvOp::MovItoF => {
                let intval = cpu.x_read(self.rn.into(), intsize);
                cpu.v_part_write(self.rd.into(), part, fltsize, intval.bits_get(0, fltsize).into());
            }
            _ => panic!("Unreachable"),
        }
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) as u8;
        let ftype = get_bits_ct!(word, 22, 2) as u8;
        let rmode = get_bits_ct!(word, 19, 2) as u8;
        let opcode = get_bits_ct!(word, 16, 3) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::FmovGeneral(Self { sf, ftype, rmode, opcode, rn, rd })
    }

    pub const FMOV_GENERAL: InstDesc = InstDesc {
        mask: 0b0111_1111_0011_0110_1111_1100_0000_0000,
        value: 0b0001_1110_0010_0110_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Unsigned maximum pairwise
#[derive(Debug, Clone, Copy)]
pub struct Umaxp {
    pub q: u8,
    pub size: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Umaxp {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.size == 0b11 {
            panic!("Undefined");
        }
        let esize = 8 << self.size;
        let datasize = 64 << self.q;
        let elements = datasize / esize;

        let mut result = 0;
        let operand1 = cpu.v_read(self.rn.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);

        // Pairwise operation: conceptually concat = operand2:operand1
        // Indices 0..elements are in operand1, indices elements..2*elements are
        // in operand2
        for e in 0..elements {
            let idx1 = (2 * e) as usize;
            let idx2 = (2 * e + 1) as usize;
            let elements = elements as usize;

            let element1 = if idx1 < elements {
                elem_get(operand1, idx1, esize as usize)
            } else {
                elem_get(operand2, idx1 - elements, esize as usize)
            };

            let element2 = if idx2 < elements {
                elem_get(operand1, idx2, esize as usize)
            } else {
                elem_get(operand2, idx2 - elements, esize as usize)
            };

            let max = element1.max(element2);
            result = elem_set(result, e as usize, esize as usize, max.bits_get(0, esize));
        }

        cpu.v_write(self.rd.into(), datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let size = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Umaxp(Self { q, size, rm, rn, rd })
    }

    pub const UMAXP: InstDesc = InstDesc {
        mask: 0b1011_1111_0010_0000_1111_1100_0000_0000,
        value: 0b0010_1110_0010_0000_1010_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Unsigned minimum pairwise
#[derive(Debug, Clone, Copy)]
pub struct Uminp {
    pub q: u8,
    pub size: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Uminp {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.size == 0b11 {
            panic!("Undefined");
        }
        let esize = 8 << self.size;
        let datasize = 64 << self.q;
        let elements = datasize / esize;

        let mut result = 0;
        let operand1 = cpu.v_read(self.rn.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);

        // Pairwise operation: conceptually concat = operand2:operand1
        // Indices 0..elements are in operand1, indices elements..2*elements are
        // in operand2
        for e in 0..elements {
            let idx1 = (2 * e) as usize;
            let idx2 = (2 * e + 1) as usize;
            let elements = elements as usize;

            let element1 = if idx1 < elements {
                elem_get(operand1, idx1, esize as usize)
            } else {
                elem_get(operand2, idx1 - elements, esize as usize)
            };

            let element2 = if idx2 < elements {
                elem_get(operand1, idx2, esize as usize)
            } else {
                elem_get(operand2, idx2 - elements, esize as usize)
            };

            let min = element1.min(element2);
            result = elem_set(result, e as usize, esize as usize, min.bits_get(0, esize));
        }

        cpu.v_write(self.rd.into(), datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let size = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Uminp(Self { q, size, rm, rn, rd })
    }

    pub const UMINP: InstDesc = InstDesc {
        mask: 0b1011_1111_0010_0000_1111_1100_0000_0000,
        value: 0b0010_1110_0010_0000_1010_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load SIMD&FP register (unscaled offset)
#[derive(Debug, Clone, Copy)]
pub struct LdurSimd {
    pub size: u8,
    pub opc: u8,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdurSimd {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let second_opc_bit = self.opc.single_bit(1);
        if second_opc_bit == 1 && self.size != 0b00 {
            panic!("Undefined");
        }
        let scale = if second_opc_bit == 1 { 4 } else { self.size };
        let offset = sign_extend(self.imm9.into(), 9);

        let datasize = 8 << scale;

        let address = cpu.address_for_rn(self.rn);

        let address = address.wrapping_add(offset);

        let val = if datasize == 128 {
            cpu.read_memory_128bit(address as usize).1
        } else {
            (cpu.read_memory(address as usize, datasize / 8).1) as u128
        };

        cpu.v_write(self.rt.into(), datasize as u8, val);
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdurSimd(Self { size, opc, imm9, rn, rt })
    }

    pub const LDUR_SIMD: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_1100_0000_0000,
        value: 0b0011_1100_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// EXT - Extract vector from pair of vectors
#[derive(Debug, Clone, Copy)]
pub struct Ext {
    pub q: u8,
    pub rm: u8,
    pub imm4: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Ext {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.q == 0 && self.imm4.single_bit(3) == 1 {
            panic!("Undefined");
        }

        let datasize = 64 << self.q;
        let position = (8 * self.imm4) as u32;

        let hi = cpu.v_read(self.rm.into(), datasize);
        let lo = cpu.v_read(self.rn.into(), datasize);

        let result = if position == 0 {
            lo
        } else {
            (lo >> position) | (hi << (datasize as u32 - position))
        };

        cpu.v_write(self.rd.into(), datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm4 = get_bits_ct!(word, 11, 4) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ext(Self { q, rm, imm4, rn, rd })
    }

    pub const EXT: InstDesc = InstDesc {
        mask: 0b1011_1111_0010_0000_1000_0100_0000_0000,
        value: 0b0010_1110_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// FCMP / FCMPE (scalar), register or zero variant.
/// One entry covers all four forms: opc<1> (signal_all_nans) only affects
/// FP exception traps, which are not modeled.
#[derive(Debug, Clone, Copy)]
pub struct FcmpFloat {
    pub ftype: u8,
    pub rm: u8,
    pub rn: u8,
    pub opc: u8,
}

impl FcmpFloat {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.ftype == 0b10 {
            panic!("Undefined");
        }
        if self.ftype == 0b11 {
            panic!("FCMP: FP16 not supported");
        }
        let datasize = 8 << (self.ftype ^ 0b10);
        let cmp_with_zero = self.opc & 0b01 != 0;

        let operand1 = cpu.v_read(self.rn.into(), datasize) as u64;
        let operand2 = if cmp_with_zero { 0 } else { cpu.v_read(self.rm.into(), datasize) as u64 };

        let ordering = if datasize == 32 {
            f32::from_bits(operand1 as u32).partial_cmp(&f32::from_bits(operand2 as u32))
        } else {
            f64::from_bits(operand1).partial_cmp(&f64::from_bits(operand2))
        };

        let (n, z, c, v) = match ordering {
            None => (false, false, true, true), // unordered (NaN)
            Some(std::cmp::Ordering::Equal) => (false, true, true, false),
            Some(std::cmp::Ordering::Less) => (true, false, false, false),
            Some(std::cmp::Ordering::Greater) => (false, false, true, false),
        };
        cpu.pstate.n = n;
        cpu.pstate.z = z;
        cpu.pstate.c = c;
        cpu.pstate.v = v;
    }

    pub const fn decode(word: u32) -> Instruction {
        let ftype = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let opc = get_bits_ct!(word, 3, 2) as u8;
        Instruction::FcmpFloat(Self { ftype, rm, rn, opc })
    }

    pub const FCMP_FLOAT: InstDesc = InstDesc {
        mask: 0b1111_1111_0010_0000_1111_1100_0000_0111,
        value: 0b0001_1110_0010_0000_0010_0000_0000_0000,
        decode: Self::decode,
    };
}

/// FABS (scalar)
#[derive(Debug, Clone, Copy)]
pub struct FabsFloat {
    pub ftype: u8,
    pub rn: u8,
    pub rd: u8,
}

impl FabsFloat {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.ftype == 0b10 {
            panic!("Undefined");
        }
        if self.ftype == 0b11 {
            panic!("FABS: FP16 not supported");
        }
        let esize = 8 << (self.ftype ^ 0b10);

        let operand = cpu.v_read(self.rn.into(), esize) as u64;
        // FPAbs is a pure sign-bit clear, NaNs included
        let result = operand & !(1u64 << (esize - 1));
        cpu.v_write(self.rd.into(), 128, result as u128);
    }

    pub const fn decode(word: u32) -> Instruction {
        let ftype = get_bits_ct!(word, 22, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::FabsFloat(Self { ftype, rn, rd })
    }

    pub const FABS_FLOAT: InstDesc = InstDesc {
        mask: 0b1111_1111_0011_1111_1111_1100_0000_0000,
        value: 0b0001_1110_0010_0000_1100_0000_0000_0000,
        decode: Self::decode,
    };
}

/// FNEG (scalar)
#[derive(Debug, Clone, Copy)]
pub struct FnegFloat {
    pub ftype: u8,
    pub rn: u8,
    pub rd: u8,
}

impl FnegFloat {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.ftype == 0b10 {
            panic!("Undefined");
        }
        if self.ftype == 0b11 {
            panic!("FNEG: FP16 not supported");
        }
        let esize = 8 << (self.ftype ^ 0b10);

        let operand = cpu.v_read(self.rn.into(), esize) as u64;
        // FPNeg is a pure sign-bit flip, NaNs included
        let result = operand ^ (1u64 << (esize - 1));
        cpu.v_write(self.rd.into(), 128, result as u128);
    }

    pub const fn decode(word: u32) -> Instruction {
        let ftype = get_bits_ct!(word, 22, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::FnegFloat(Self { ftype, rn, rd })
    }

    pub const FNEG_FLOAT: InstDesc = InstDesc {
        mask: 0b1111_1111_0011_1111_1111_1100_0000_0000,
        value: 0b0001_1110_0010_0001_0100_0000_0000_0000,
        decode: Self::decode,
    };
}

/// FCVT (scalar, precision conversion)
#[derive(Debug, Clone, Copy)]
pub struct FcvtFloat {
    pub ftype: u8,
    pub opc: u8,
    pub rn: u8,
    pub rd: u8,
}

impl FcvtFloat {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.ftype == self.opc || self.ftype == 0b10 || self.opc == 0b10 {
            panic!("Undefined");
        }
        if self.ftype == 0b11 || self.opc == 0b11 {
            panic!("FCVT: FP16 not supported");
        }
        let srcsize = 8 << (self.ftype ^ 0b10);

        let operand = cpu.v_read(self.rn.into(), srcsize) as u64;

        // Only S->D and D->S remain after the checks above
        let result = if srcsize == 32 {
            (f32::from_bits(operand as u32) as f64).to_bits() as u128
        } else {
            (f64::from_bits(operand) as f32).to_bits() as u128
        };
        cpu.v_write(self.rd.into(), 128, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let ftype = get_bits_ct!(word, 22, 2) as u8;
        let opc = get_bits_ct!(word, 15, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::FcvtFloat(Self { ftype, opc, rn, rd })
    }

    pub const FCVT_FLOAT: InstDesc = InstDesc {
        mask: 0b1111_1111_0011_1110_0111_1100_0000_0000,
        value: 0b0001_1110_0010_0010_0100_0000_0000_0000,
        decode: Self::decode,
    };
}

/// FSUB (scalar)
#[derive(Debug, Clone, Copy)]
pub struct FsubFloat {
    pub ftype: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl FsubFloat {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.ftype == 0b10 {
            panic!("Undefined");
        }
        if self.ftype == 0b11 {
            panic!("FSUB: FP16 not supported");
        }
        let esize = 8 << (self.ftype ^ 0b10);

        let operand1 = cpu.v_read(self.rn.into(), esize) as u64;
        let operand2 = cpu.v_read(self.rm.into(), esize) as u64;

        let result = if esize == 32 {
            let diff = f32::from_bits(operand1 as u32) - f32::from_bits(operand2 as u32);
            diff.to_bits() as u128
        } else {
            let diff = f64::from_bits(operand1) - f64::from_bits(operand2);
            diff.to_bits() as u128
        };
        cpu.v_write(self.rd.into(), 128, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let ftype = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::FsubFloat(Self { ftype, rm, rn, rd })
    }

    pub const FSUB_FLOAT: InstDesc = InstDesc {
        mask: 0b1111_1111_0010_0000_1111_1100_0000_0000,
        value: 0b0001_1110_0010_0000_0011_1000_0000_0000,
        decode: Self::decode,
    };
}

/// FCSEL (floating-point conditional select)
#[derive(Debug, Clone, Copy)]
pub struct FcselFloat {
    pub ftype: u8,
    pub rm: u8,
    pub cond: u8,
    pub rn: u8,
    pub rd: u8,
}

impl FcselFloat {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.ftype == 0b10 {
            panic!("Undefined");
        }
        if self.ftype == 0b11 {
            panic!("FCSEL: FP16 not supported");
        }
        let datasize = 8 << (self.ftype ^ 0b10);

        let result = if condition_holds(cpu, self.cond) {
            cpu.v_read(self.rn.into(), datasize)
        } else {
            cpu.v_read(self.rm.into(), datasize)
        };
        cpu.v_write(self.rd.into(), datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let ftype = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let cond = get_bits_ct!(word, 12, 4) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::FcselFloat(Self { ftype, rm, cond, rn, rd })
    }

    pub const FCSEL_FLOAT: InstDesc = InstDesc {
        mask: 0b1111_1111_0010_0000_0000_1100_0000_0000,
        value: 0b0001_1110_0010_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Add vector scalar
#[derive(Debug, Clone, Copy)]
pub struct AddVectorScalar {
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AddVectorScalar {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let esize = 8 << 0b11;
        let datasize = esize;
        let elements = 1;

        let mut result = 0;
        let operand1 = cpu.v_read(self.rn.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);

        for e in 0..elements {
            let element1 = elem_get(operand1, e, esize.into());
            let element2 = elem_get(operand2, e, esize.into());
            let val = element1.wrapping_add(element2);
            result = elem_set(result, e, esize as usize, val.bits_get(0, esize));
        }
        cpu.v_write(self.rd.into(), datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AddVectorScalar(Self { rm, rn, rd })
    }

    pub const ADD_VECTOR_SCALAR: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0101_1110_1110_0000_1000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Add vector vectoral
#[derive(Debug, Clone, Copy)]
pub struct AddVectorVectoral {
    pub q: u8,
    pub size: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AddVectorVectoral {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let size_and_q = (self.size << 1) | self.q;
        if size_and_q == 0b110 {
            panic!("Undefined");
        }
        let esize = 8 << self.size;
        let datasize = 64 << self.q;
        let elements = datasize / esize;

        let mut result = 0;
        let operand1 = cpu.v_read(self.rn.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);

        for e in 0..elements {
            let element1 = elem_get(operand1, e as usize, esize as usize);
            let element2 = elem_get(operand2, e as usize, esize as usize);
            let val = element1.wrapping_add(element2);
            result = elem_set(result, e as usize, esize as usize, val.bits_get(0, esize));
        }
        cpu.v_write(self.rd.into(), datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let size = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AddVectorVectoral(Self { q, size, rm, rn, rd })
    }

    pub const ADD_VECTOR_VECTORAL: InstDesc = InstDesc {
        mask: 0b1011_1111_0010_0000_1111_1100_0000_0000,
        value: 0b0000_1110_0010_0000_1000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// FADD (vector), single-precision and double-precision
#[derive(Debug, Clone, Copy)]
pub struct FaddVector {
    pub q: u8,
    pub sz: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl FaddVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let sz_and_q = (self.sz << 1) | self.q;
        if sz_and_q == 0b10 {
            panic!("Undefined");
        }
        let esize = 32 << self.sz;
        let datasize = 64 << self.q;
        let elements = datasize / esize;

        let mut result = 0;
        let operand1 = cpu.v_read(self.rn.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);

        for e in 0..elements {
            let element1 = elem_get(operand1, e as usize, esize as usize);
            let element2 = elem_get(operand2, e as usize, esize as usize);
            let val = if esize == 32 {
                let sum = f32::from_bits(element1 as u32) + f32::from_bits(element2 as u32);
                sum.to_bits() as u64
            } else {
                let sum = f64::from_bits(element1) + f64::from_bits(element2);
                sum.to_bits()
            };
            result = elem_set(result, e as usize, esize as usize, val);
        }
        cpu.v_write(self.rd.into(), datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let sz = get_bits_ct!(word, 22, 1) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::FaddVector(Self { q, sz, rm, rn, rd })
    }

    pub const FADD_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_1010_0000_1111_1100_0000_0000,
        value: 0b0000_1110_0010_0000_1101_0100_0000_0000,
        decode: Self::decode,
    };
}

/// FCMLT (zero, vector), single-precision and double-precision
#[derive(Debug, Clone, Copy)]
pub struct FcmltZeroVector {
    pub q: u8,
    pub sz: u8,
    pub rn: u8,
    pub rd: u8,
}

impl FcmltZeroVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let sz_and_q = (self.sz << 1) | self.q;
        if sz_and_q == 0b10 {
            panic!("Undefined");
        }
        let esize = 32 << self.sz;
        let datasize = 64 << self.q;
        let elements = datasize / esize;

        let mut result = 0;
        let operand = cpu.v_read(self.rn.into(), datasize);

        for e in 0..elements {
            let element = elem_get(operand, e as usize, esize as usize);
            let test_passed = if esize == 32 {
                f32::from_bits(element as u32) < 0.0
            } else {
                f64::from_bits(element) < 0.0
            };
            let val = if test_passed { u64::MAX.bits_get(0, esize) } else { 0 };
            result = elem_set(result, e as usize, esize as usize, val);
        }
        cpu.v_write(self.rd.into(), datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let sz = get_bits_ct!(word, 22, 1) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::FcmltZeroVector(Self { q, sz, rn, rd })
    }

    pub const FCMLT_ZERO_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_1011_1111_1111_1100_0000_0000,
        value: 0b0000_1110_1010_0000_1110_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Add pairwise (vector)
#[derive(Debug, Clone, Copy)]
pub struct AddpVector {
    pub q: u8,
    pub size: u8,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AddpVector {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let size_and_q = (self.size << 1) | self.q;
        if size_and_q == 0b110 {
            panic!("Undefined");
        }
        let esize = 8 << self.size;
        let datasize = 64 << self.q;
        let elements = datasize / esize;
        let mut result = 0;
        let operand1 = cpu.v_read(self.rn.into(), datasize);
        let operand2 = cpu.v_read(self.rm.into(), datasize);

        for e in 0..elements {
            let idx1 = (2 * e) as usize;
            let idx2 = (2 * e + 1) as usize;
            let elements = elements as usize;

            let element1 = if idx1 < elements {
                elem_get(operand1, idx1, esize as usize)
            } else {
                elem_get(operand2, idx1 - elements, esize as usize)
            };

            let element2 = if idx2 < elements {
                elem_get(operand1, idx2, esize as usize)
            } else {
                elem_get(operand2, idx2 - elements, esize as usize)
            };

            let val = element1 + element2;
            result = elem_set(result, e as usize, esize as usize, val.bits_get(0, esize));
        }
        cpu.v_write(self.rd.into(), datasize, result);
    }

    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 30, 1) as u8;
        let size = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AddpVector(Self { q, size, rm, rn, rd })
    }

    pub const ADDP_VECTOR: InstDesc = InstDesc {
        mask: 0b1011_1111_0010_0000_1111_1100_0000_0000,
        value: 0b0000_1110_0010_0000_1011_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Add pairwise (scalar)
#[derive(Debug, Clone, Copy)]
pub struct AddpScalar {
    pub rn: u8,
    pub rd: u8,
}

impl AddpScalar {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 128;
        let esize = 64;
        let input = cpu.v_read(self.rn.into(), datasize);
        let mut sum = 0u64;
        for e in 0..(128 / esize) {
            sum = sum.wrapping_add(elem_get(input, e, esize));
        }
        cpu.v_write(self.rd.into(), esize as u8, sum as u128);
    }

    pub const fn decode(word: u32) -> Instruction {
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AddpScalar(Self { rn, rd })
    }

    pub const ADDP_SCALAR: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1100_0000_0000,
        value: 0b0101_1110_1111_0001_1011_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Load SIMD&FP register (register offset)
#[derive(Debug, Clone, Copy)]
pub struct LdrSimdRegOffset {
    pub size: u8,
    pub opc: u8,
    pub rm: u8,
    pub option: u8,
    pub s: u8,
    pub rn: u8,
    pub rt: u8,
}

impl LdrSimdRegOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        // option<1> must be 1 (sub-word index check)
        if self.option.single_bit(1) == 0 {
            panic!("Undefined");
        }
        // opc<1> == 1 && size != 00 is undefined
        if self.opc.single_bit(1) == 1 && self.size != 0 {
            panic!("Undefined");
        }

        let scale = if self.opc.single_bit(1) == 1 { 4 } else { self.size };
        let extend_type = ExtendType::from_u8(self.option);
        let shift = if self.s == 1 { scale } else { 0 };
        let datasize: u8 = 8 << scale;

        let offset = extend_register(cpu, self.rm, extend_type, shift, 64);
        let mut address = cpu.address_for_rn(self.rn);
        address = address.wrapping_add(offset);

        let val = if datasize == 128 {
            let (_, d) = cpu.read_memory_128bit(address as usize);
            if cpu.mmu.faulted {
                return;
            }
            d
        } else {
            let (_, d) = cpu.read_memory(address as usize, (datasize / 8) as usize);
            if cpu.mmu.faulted {
                return;
            }
            d as u128
        };

        cpu.v_write(self.rt.into(), datasize, val);
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let s = get_bits_ct!(word, 12, 1) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrSimdRegOffset(Self { size, opc, rm, option, s, rn, rt })
    }

    pub const LDR_SIMD_REG_OFFSET: InstDesc = InstDesc {
        mask: 0b0011_1111_0110_0000_0000_1100_0000_0000,
        value: 0b0011_1100_0110_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}
