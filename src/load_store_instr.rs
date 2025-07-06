use crate::{
    cpu::Cpu,
    data_processing::shift_lsl,
    get_bits_ct,
    instruction::{InstDesc, Instruction},
    load_and_store::{
        ExtendType, instruction_ldr_imm_base, instruction_ldr_literal, instruction_ldr_register,
        instruction_str_imm_un_off,
    },
    utils::sign_extend_xor,
};

#[derive(Debug, Clone, Copy)]
pub struct StrImmUnOffset {
    pub size: u8,
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrImmUnOffset {
    pub fn exec(self, cpu: &mut Cpu) {
        let offset = if (self.size & 1) == 1 {
            shift_lsl(self.imm12 as u64, self.size)
        } else {
            self.imm12 as u64
        };
        let datasize = (8 << (self.size)) as u64;
        let tag_checked = self.rn != 31;
        if self.rn == self.rt && self.rn != 31 {
            panic!("Unpredictable");
        }
        instruction_str_imm_un_off(
            cpu,
            self.rn,
            self.rt,
            datasize as usize,
            offset,
            false,
            false,
            false,
            tag_checked,
            false,
        );
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrImmUnOffset(StrImmUnOffset { size, imm12, rn, rt })
    }
    pub const STR_IMM_UN_OFFSET: InstDesc = InstDesc {
        mask: 0b1011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b1011_1001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LdrImmUnOffset {
    pub size: u8,
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrImmUnOffset {
    pub fn exec(self, cpu: &mut Cpu) {
        let offset = if (self.size & 1) == 1 {
            shift_lsl(self.imm12 as u64, self.size)
        } else {
            self.imm12 as u64
        };
        let datasize = (8 << (self.size)) as u64;
        let tag_checked = self.rn != 31;
        if self.rn == self.rt && self.rn != 31 {
            panic!("Unpredictable");
        }
        instruction_ldr_imm_base(
            cpu,
            self.rn,
            self.rt,
            datasize as usize,
            offset,
            false,
            false,
            false,
            tag_checked,
            false,
        );
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrImmUnOffset(LdrImmUnOffset { size, imm12, rn, rt })
    }
    pub const LDR_IMM_UN_OFFSET: InstDesc = InstDesc {
        mask: 0b1011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b1011_1001_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LdrImmPostIdx {
    pub size: u8,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrImmPostIdx {
    pub fn exec(self, cpu: &mut Cpu) {
        let offset = sign_extend_xor(self.imm9 as u64, 9);
        let datasize = (8 << (self.size)) as u64;
        let tagchecked = self.rn != 31;

        if self.rn == self.rt && self.rn != 31 {
            panic!("Unpredictable");
        }
        instruction_ldr_imm_base(
            cpu,
            self.rn,
            self.rt,
            datasize as usize,
            offset,
            true,
            true,
            false,
            tagchecked,
            false,
        );
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrImmPostIdx(LdrImmPostIdx { size, imm9, rn, rt })
    }
    pub const LDR_IMM_POST_IDX: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0100_0000_0000_0100_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LdrImmPreIdx {
    pub size: u8,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrImmPreIdx {
    pub fn exec(self, cpu: &mut Cpu) {
        let offset = sign_extend_xor(self.imm9 as u64, 9);
        let datasize = (8 << (self.size)) as u64;
        let tagchecked = self.rn != 31;

        if self.rn == self.rt && self.rn != 31 {
            panic!("Unpredictable");
        }
        instruction_ldr_imm_base(
            cpu,
            self.rn,
            self.rt,
            datasize as usize,
            offset,
            false,
            true,
            false,
            tagchecked,
            false,
        );
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrImmPreIdx(LdrImmPreIdx { size, imm9, rn, rt })
    }
    pub const LDR_IMM_PRE_IDX: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0100_0000_0000_1100_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LdrReg {
    pub size: u8,
    pub rm: u8,
    pub option: u8,
    pub s: u8,
    pub rn: u8,
    pub rt: u8,
}

impl LdrReg {
    pub fn exec(self, cpu: &mut Cpu) {
        let datasize = 8 << (self.size as u64);
        let is_64 = (self.size & 1) == 1;
        if is_64 {
            if self.option == 0 {
                panic!("ldr_register undefined");
            }
            let extend_type = ExtendType::from_u8(self.option);
            let shift = if self.s == 1 { self.size } else { 0 };
            instruction_ldr_register(
                cpu,
                self.rt,
                self.rn,
                self.rm,
                datasize,
                if datasize == 64 { 64 } else { 32 },
                shift as u64,
                extend_type,
                false,
                true,
            );
        }
        instruction_ldr_register(
            cpu,
            self.rt,
            self.rn,
            self.rm,
            datasize,
            if datasize == 64 { 64 } else { 32 },
            0,
            ExtendType::UxTx,
            false,
            true,
        );
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let s = get_bits_ct!(word, 12, 1) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrReg(LdrReg { size, rm, option, s, rn, rt })
    }
    pub const LDR_REG: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0110_0000_0000_1000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LdrLit {
    pub opc: u8,
    pub imm19: u32,
    pub rt: u8,
}

impl LdrLit {
    pub fn exec(self, cpu: &mut Cpu) {
        instruction_ldr_literal(cpu, self.rt, 4 << self.opc, (self.imm19 as u64) << 2)
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 30, 1) as u8;
        let imm19 = get_bits_ct!(word, 5, 19);
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrLit(LdrLit { opc, imm19, rt })
    }

    pub const LDR_LIT: InstDesc = InstDesc {
        mask: 0b1011_1111_0000_0000_0000_0000_0000_0000,
        value: 0b0001_1000_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}
