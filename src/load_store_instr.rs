use crate::{
    cpu::Cpu,
    data_processing::shift_lsl,
    get_bits_ct,
    instruction::{InstDesc, Instruction},
    load_and_store::{
        ExtendType, instruction_ldp, instruction_ldr_imm_base, instruction_ldr_literal,
        instruction_ldr_register, instruction_stp, instruction_str_halfword_imm,
        instruction_str_imm_un_off, instruction_str_register, instruction_strb_imm_un_off,
    },
    utils::{sign_extend, sign_extend_xor, zero_extend},
};

/// Store register halfword
#[derive(Debug, Clone, Copy)]
pub struct StrhUnsigned {
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrhUnsigned {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = shift_lsl(self.imm12.into(), 1);
        instruction_str_halfword_imm(cpu, self.rn, self.rt, offset, false, false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrhUnsigned(Self { imm12, rn, rt })
    }
    pub const STRH_UNSIGNED: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0111_1001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct StrImmUnOffset {
    pub size: u8,
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrImmUnOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = shift_lsl(self.imm12 as u64, self.size);
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
        Instruction::StrImmUnOffset(Self { size, imm12, rn, rt })
    }
    pub const STR_IMM_UN_OFFSET: InstDesc = InstDesc {
        mask: 0b1011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b1011_1001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store register byte (immediate)
#[derive(Debug, Clone, Copy)]
pub struct StrbImmUnOffset {
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrbImmUnOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        // Not sure about this ?
        // TODO: Re-read the spec
        let offset = shift_lsl(self.imm12 as u64, 0);
        let tag_checked = self.rn != 31;
        if self.rn == self.rt && self.rn != 31 {
            panic!("Unpredictable");
        }
        instruction_strb_imm_un_off(
            cpu,
            self.rn,
            self.rt,
            offset,
            false,
            false,
            false,
            tag_checked,
            false,
        );
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrbImmUnOffset(Self { imm12, rn, rt })
    }

    pub const STRB_IMM_UN_OFFSET: InstDesc = InstDesc {
        mask: 0b1111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0011_1001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store register (register)
#[derive(Clone, Copy, Debug)]
pub struct StrRegister {
    pub size: u8,
    pub rm: u8,
    pub option: u8,
    pub s: bool,
    pub rn: u8,
    pub rt: u8,
}

impl StrRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let extend_type = ExtendType::from_u8(self.option);
        let shift = if self.s { self.size } else { 0 };
        let datasize = 8 << self.size;
        instruction_str_register(cpu, self.rn, self.rt, self.rm, extend_type, shift, datasize);
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let s = get_bits_ct!(word, 12, 1) == 1;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        if get_bits_ct!(option, 0, 1) == 0 {
            panic!("Undef, sub-word index")
        }
        Instruction::StrRegister(Self { size, rm, option, s, rn, rt })
    }

    pub const STR_REGISTER: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0010_0000_0000_1000_0000_0000,
        decode: Self::decode,
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
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = shift_lsl(zero_extend(self.imm12 as u64, 64), self.size);
        let datasize = (8 << (self.size)) as u64;
        let tag_checked = self.rn != 31;
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
        Instruction::LdrImmUnOffset(Self { size, imm12, rn, rt })
    }
    pub const LDR_IMM_UN_OFFSET: InstDesc = InstDesc {
        mask: 0b1011_1111_1100_0000_0000_0000_0000_0000,
        value: 0b1011_1001_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
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
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
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
        Instruction::LdrImmPostIdx(Self { size, imm9, rn, rt })
    }
    pub const LDR_IMM_POST_IDX: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0100_0000_0000_0100_0000_0000,
        decode: Self::decode,
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
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
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
        Instruction::LdrImmPreIdx(Self { size, imm9, rn, rt })
    }
    pub const LDR_IMM_PRE_IDX: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0100_0000_0000_1100_0000_0000,
        decode: Self::decode,
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
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 8 << (self.size as u64);
        let is_64 = self.size == 3;
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
        } else {
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
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let s = get_bits_ct!(word, 12, 1) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrReg(Self { size, rm, option, s, rn, rt })
    }
    pub const LDR_REG: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0110_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LdrLit {
    pub opc: u8,
    pub imm19: u32,
    pub rt: u8,
}

impl LdrLit {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let offset = old_pc.wrapping_add(crate::utils::sign_extend(self.imm19 as u64, 19) << 2);
        instruction_ldr_literal(cpu, self.rt, 4 << self.opc, offset, old_pc)
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 30, 1) as u8;
        let imm19 = get_bits_ct!(word, 5, 19);
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrLit(Self { opc, imm19, rt })
    }

    pub const LDR_LIT: InstDesc = InstDesc {
        mask: 0b1011_1111_0000_0000_0000_0000_0000_0000,
        value: 0b0001_1000_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store pair of registers
#[derive(Debug, Clone, Copy)]
pub struct StpSignedOffset {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl StpSignedOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_stp(cpu, self.rt, self.rt2, self.rn, datasize, offset, false, false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 31, 1) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StpSignedOffset(Self { opc, imm7, rt2, rn, rt })
    }

    pub const STP_SIGNED_OFFSET: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct StpPreIndex {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl StpPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_stp(cpu, self.rt, self.rt2, self.rn, datasize, offset, true, false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 31, 1) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StpPreIndex(Self { opc, imm7, rt2, rn, rt })
    }

    pub const STP_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1001_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load pair of registers
#[derive(Debug, Clone, Copy)]
pub struct LdpPostIndex {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl LdpPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_ldp(cpu, self.rt, self.rt2, self.rn, datasize, offset, true, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 31, 1) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdpPostIndex(Self { opc, imm7, rt2, rn, rt })
    }

    pub const LDP_POST_INDEX: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1000_1100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load pair of registers
#[derive(Debug, Clone, Copy)]
pub struct LdpSignedOffset {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl LdpSignedOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_ldp(cpu, self.rt, self.rt2, self.rn, datasize, offset, false, false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 31, 1) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdpSignedOffset(Self { opc, imm7, rt2, rn, rt })
    }

    pub const LDP_SIGNED_OFFSET: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1001_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}
