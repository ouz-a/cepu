use crate::{
    cpu::Cpu,
    data_processing::shift_lsl,
    get_bits_ct,
    instruction::{InstDesc, Instruction},
    load_and_store::{ExtendType, extend_register},
    simd_fp_instr::{
        dup_general_instruction, instruction_ldp_simd_fp, instruction_ldr_simd_fp,
        str_imd_fp_instruction, str_pair_fp_instruction,
    },
    utils::{BitUtils, bits_get, elem_set, sign_extend},
};

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
        let q = get_bits_ct!(word, 31, 1) == 1;
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

/// Move immediate (vector)
#[derive(Debug, Clone, Copy)]
pub struct Movi {
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

impl Movi {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 64 << self.q;
        let imm8 = (self.a as u8) << 7
            | (self.b as u8) << 6
            | (self.c as u8) << 5
            | (self.d as u8) << 4
            | (self.e as u8) << 3
            | (self.f as u8) << 2
            | (self.g as u8) << 1
            | (self.h as u8);
        let imm64 = adv_simd_expand_imm(self.op, self.cmode, imm8);
        if datasize == 128 {
            let imm: u128 = imm64.replicate(2);
            cpu.v_write(self.rd.into(), 128, imm);
        } else {
            cpu.v_write(self.rd.into(), 64, imm64 as u128);
        }
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
        Instruction::Movi(Self { q, op, a, b, c, cmode, d, e, f, g, h, rd })
    }

    pub const MOVI: InstDesc = InstDesc {
        mask: 0b1001_1111_1111_1000_0000_1100_0000_0000,
        value: 0b0000_1111_0000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

pub fn adv_simd_expand_imm(op: u8, cmode: u8, imm8: u8) -> u64 {
    let mut imm64: u64 = 0;

    match cmode.bits_get(1, 3) {
        0b000 => {
            imm64 = (imm8 as u32).replicate::<u64>(2);
        }
        0b001 => {
            let imm8 = imm8 as u32;
            let imm8 = imm8 << 8;
            imm64 = imm8.replicate::<u64>(2);
        }
        0b010 => {
            let imm8 = imm8 as u32;
            let imm8 = imm8 << 16;
            imm64 = imm8.replicate::<u64>(2);
        }
        0b011 => {
            let imm8 = imm8 as u32;
            let imm8 = imm8 << 24;
            imm64 = imm8.replicate::<u64>(2);
        }
        0b100 => {
            let imm8 = imm8 as u16;
            imm64 = imm8.replicate::<u64>(4);
        }
        0b101 => {
            let imm8 = imm8 as u16;
            let imm8 = imm8 << 8;
            imm64 = imm8.replicate::<u64>(4);
        }
        0b110 => {
            if cmode.single_bit(0) == 0 {
                let imm8 = imm8 as u32;
                let ones: u32 = 0b1.replicate(8);
                let imm8 = (imm8 << 8) | ones;
                imm64 = imm8.replicate::<u64>(2);
            } else {
                let imm8 = imm8 as u32;
                let ones: u32 = 0b1.replicate(16);
                let imm8 = (imm8 << 16) | ones;
                imm64 = imm8.replicate::<u64>(2);
            }
        }
        0b111 => {
            if cmode.single_bit(0) == 0 && op == 0 {
                imm64 = imm8.replicate::<u64>(8);
            }
            if cmode.single_bit(0) == 0 && op == 1 {
                let imm8a: u8 = imm8.single_bit(7).replicate(8);
                let imm8b: u8 = imm8.single_bit(6).replicate(8);
                let imm8c: u8 = imm8.single_bit(5).replicate(8);
                let imm8d: u8 = imm8.single_bit(4).replicate(8);
                let imm8e: u8 = imm8.single_bit(3).replicate(8);
                let imm8f: u8 = imm8.single_bit(2).replicate(8);
                let imm8g: u8 = imm8.single_bit(1).replicate(8);
                let imm8h: u8 = imm8.single_bit(0).replicate(8);
                imm64 =
                    u64::from_be_bytes([imm8a, imm8b, imm8c, imm8d, imm8e, imm8f, imm8g, imm8h]);
            }
            if cmode.single_bit(0) == 1 && op == 0 {
                let bit7 = imm8.single_bit(7) as u32;
                let bit6 = imm8.single_bit(6) as u32;
                let bits5_0 = imm8.bits_get(0, 5) as u32;
                let imm32: u32 = (bit7 << 31)
                    | ((bit6 ^ 1) << 30)
                    | (bit6.replicate::<u32>(5) << 25)
                    | (bits5_0 << 19);
                imm64 = imm32.replicate(2);
            }
            if cmode.single_bit(0) == 1 && op == 1 {
                let bit7 = imm8.single_bit(7) as u64;
                let bit6 = imm8.single_bit(6) as u64;
                let bits5_0 = imm8.bits_get(0, 5) as u64;
                imm64 = (bit7 << 63)
                    | ((bit6 ^ 1) << 62)
                    | (bit6.replicate::<u64>(8) << 54)
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
        cpu.mmu
            .write_memory_128bit(address.try_into().unwrap(), cpu.v_read(self.rt.into(), datasize));
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
            cpu.mmu.write_memory_128bit(
                address.try_into().unwrap(),
                cpu.v_read(self.rt.into(), datasize),
            );
        } else {
            cpu.mmu.write_memory(
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
        let rpt;
        match self.opcode {
            0b0010 => rpt = 4,
            0b0110 => rpt = 3,
            0b1010 => rpt = 2,
            0b0111 => rpt = 1,
            _ => panic!("Undefined"),
        }

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
        let rpt;
        match self.opcode {
            0b0010 => rpt = 4,
            0b0110 => rpt = 3,
            0b1010 => rpt = 2,
            0b0111 => rpt = 1,
            _ => panic!("Undefined"),
        }

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

pub fn instruction_ld1(
    cpu: &mut Cpu,
    rn: u8,
    rm: u8,
    rt: u8,
    elements: u8,
    esize: u8,
    datasize: u8,
    rpt: u8,
    wback: bool,
) {
    let ebytes = esize / 8;

    let mut address = cpu.address_for_rn(rn);

    let mut offs: u64 = 0;

    for r in 0..rpt {
        for e in 0..elements {
            let mut tt = (rt + r) % 32;
            // selem
            for _ in 0..1 {
                let rval = cpu.v_read(tt.into(), datasize);
                let eaddr = address.wrapping_add(offs);
                let val = cpu.mmu.read_memory(eaddr as usize, ebytes.into()).1;
                if cpu.mmu.faulted {
                    return;
                }
                let rval = elem_set(rval, e.try_into().unwrap(), esize.try_into().unwrap(), val);
                cpu.v_write(tt.into(), datasize, rval);
                offs = offs.wrapping_add(ebytes.into());
                tt = (tt + 1) % 32;
            }
        }
    }
    if wback {
        if rm != 31 {
            offs = cpu.x_read(rm.into(), 64);
        }
        address = address.wrapping_add(offs);
        if rn == 31 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(rn.into(), address, false);
        }
    }
}
