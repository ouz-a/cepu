use crate::{
    cpu::Cpu,
    data_processing::shift_lsl,
    get_bits_ct,
    instruction::{InstDesc, Instruction},
    simd_fp_instr::{dup_general_instruction, str_imd_fp_instruction, str_pair_fp_instruction},
    utils::{BitUtils, bits_get, sign_extend},
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
        let size = bits_get(self.imm5.into(), 0, 3).trailing_zeros();

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
        let opc = get_bits_ct!(word, 22, 1) as u8;
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
        let opc = get_bits_ct!(word, 22, 1) as u8;
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
        let wback = false;
        let postindex = false;
        let offset = shift_lsl(sign_extend(self.imm12.into(), 12), scale);
        let datasize = 8 << scale;

        str_imd_fp_instruction(cpu, self.rn, self.rt, datasize, postindex, wback, offset);
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let opc = get_bits_ct!(word, 22, 1) as u8;
        let imm12 = get_bits_ct!(word, 12, 9) as u16;
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

        str_pair_fp_instruction(
            cpu, self.rn, self.rt, self.rt2, offset, datasize, true, true,
        );
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

        str_pair_fp_instruction(
            cpu, self.rn, self.rt, self.rt2, offset, datasize, false, true,
        );
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

        str_pair_fp_instruction(
            cpu, self.rn, self.rt, self.rt2, offset, datasize, false, false,
        );
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

