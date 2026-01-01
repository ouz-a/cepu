use std::ops::Shl;

use crate::{
    cpu::Cpu,
    data_processing::shift_lsl,
    get_bits_ct,
    instruction::{InstDesc, Instruction},
    load_and_store::{
        ExtendType, extend_register, instruction_ldp, instruction_ldp_simd_fp, instruction_ldpsw,
        instruction_ldr_imm_base, instruction_ldr_literal, instruction_ldr_register,
        instruction_ldrh_imm, instruction_ldrsb_imm, instruction_ldrsh_imm, instruction_ldrsw_imm,
        instruction_stnp, instruction_stp, instruction_str_halfword_imm,
        instruction_str_imm_un_off, instruction_str_register, instruction_strb_imm_un_off,
    },
    utils::{bits_get, sign_extend, sign_extend_xor, zero_extend},
};

/// Store register halfword
#[derive(Debug, Clone, Copy)]
pub struct StrhPostIndex {
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrhPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        instruction_str_halfword_imm(cpu, self.rn, self.rt, offset, true, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrhPostIndex(Self { imm9, rn, rt })
    }
    pub const STRH_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0111_1000_0000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Store register halfword
#[derive(Debug, Clone, Copy)]
pub struct StrhPreIndex {
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrhPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        instruction_str_halfword_imm(cpu, self.rn, self.rt, offset, false, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrhPreIndex(Self { imm9, rn, rt })
    }
    pub const STRH_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0111_1000_0000_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

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
        mask: 0b1111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0111_1001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store register (immediate) - Unsigned offset
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

/// STTR
/// Store register (unprivileged)
#[derive(Debug, Clone, Copy)]
pub struct StrUnpriv {
    pub size: u8,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrUnpriv {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = (8 << (self.size)) as u8;
        let tag_checked = self.rn != 31;
        instruction_str_imm_un_off(
            cpu,
            self.rn,
            self.rt,
            datasize.into(),
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
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrUnpriv(Self { size, imm9, rn, rt })
    }

    pub const STR_UNPRIV: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0000_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Store register (immediate) Pre-index
#[derive(Debug, Clone, Copy)]
pub struct StrImmPreIndex {
    pub size: u8,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrImmPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = (8 << (self.size)) as u8;
        let tag_checked = self.rn != 31;
        instruction_str_imm_un_off(
            cpu,
            self.rn,
            self.rt,
            datasize.into(),
            offset,
            false,
            true,
            false,
            tag_checked,
            false,
        );
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrImmPreIndex(Self { size, imm9, rn, rt })
    }

    pub const STR_IMM_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0000_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Store register (immediate) Post-index
#[derive(Debug, Clone, Copy)]
pub struct StrImmPostIndex {
    pub size: u8,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrImmPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = (8 << (self.size)) as u8;
        let tag_checked = self.rn != 31;
        instruction_str_imm_un_off(
            cpu,
            self.rn,
            self.rt,
            datasize.into(),
            offset,
            true,
            true,
            false,
            tag_checked,
            false,
        );
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrImmPostIndex(Self { size, imm9, rn, rt })
    }

    pub const STR_IMM_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Store register byte (immediate)
#[derive(Debug, Clone, Copy)]
pub struct StrbPostIndex {
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrbPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        let tag_checked = self.rn != 31;
        if self.rn == self.rt && self.rn != 31 {
            panic!("Unpredictable");
        }
        instruction_strb_imm_un_off(
            cpu,
            self.rn,
            self.rt,
            offset,
            true,
            true,
            false,
            tag_checked,
            false,
        );
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrbPostIndex(Self { imm9, rn, rt })
    }

    pub const STRB_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0011_1000_0000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Store register byte (immediate)
#[derive(Debug, Clone, Copy)]
pub struct StrbPreIndex {
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl StrbPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
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
            true,
            false,
            tag_checked,
            false,
        );
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrbPreIndex(Self { imm9, rn, rt })
    }

    pub const STRB_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0011_1000_0000_0000_0000_1100_0000_0000,
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

/// Store register byte (register)
#[derive(Debug, Clone, Copy)]
pub struct StrbRegister {
    pub rm: u8,
    pub option: u8,
    pub rn: u8,
    pub rt: u8,
}

impl StrbRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if bits_get(self.option.into(), 1, 1) == 0 {
            panic!("Sub word index");
        }
        let extend_type = ExtendType::from_u8(self.option);
        let offset = extend_register(cpu, self.rm, extend_type, 0, 64);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        address = address.wrapping_add(offset);

        cpu.mmu.write_memory(address as usize, 1, cpu.x_read(self.rt.into(), 8));
    }

    pub const fn decode(word: u32) -> Instruction {
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StrbRegister(Self { rm, option, rn, rt })
    }

    pub const STRB_REGISTER: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0011_1000_0010_0000_0000_1000_0000_0000,
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
        if get_bits_ct!(option, 1, 1) == 0 {
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

/// Store register halfword (register)
#[derive(Debug, Clone, Copy)]
pub struct Strh {
    pub rm: u8,
    pub option: u8,
    pub s: bool,
    pub rn: u8,
    pub rt: u8,
}

impl Strh {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if bits_get(self.option.into(), 1, 1) == 0 {
            panic!("Sub word index");
        }
        let shift = if self.s { 1 } else { 0 };
        let extend_type = ExtendType::from_u8(self.option);
        let offset = extend_register(cpu, self.rm, extend_type, shift, 64);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        address = address.wrapping_add(offset);

        cpu.mmu.write_memory(address as usize, 2, cpu.x_read(self.rt.into(), 16));
        if cpu.mmu.faulted {}
    }

    pub const fn decode(word: u32) -> Instruction {
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let s = get_bits_ct!(word, 12, 1) == 1;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Strh(Self { rm, option, s, rn, rt })
    }

    pub const STRH: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0111_1000_0010_0000_0000_1000_0000_0000,
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

/// Store pair of registers with non-temporal hint
#[derive(Debug, Clone, Copy)]
pub struct Stnp {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl Stnp {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_stnp(cpu, self.rt, self.rt2, self.rn, datasize, offset);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 31, 1) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Stnp(Self { opc, imm7, rt2, rn, rt })
    }

    pub const STNP: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1000_0000_0000_0000_0000_0000_0000,
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

#[derive(Debug, Clone, Copy)]
pub struct StpPostIndex {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl StpPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_stp(cpu, self.rt, self.rt2, self.rn, datasize, offset, true, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 31, 1) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StpPostIndex(Self { opc, imm7, rt2, rn, rt })
    }

    pub const STP_POST_INDEX: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1000_1000_0000_0000_0000_0000_0000,
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
pub struct LdpPreIndex {
    opc: u8,
    imm7: u8,
    rt2: u8,
    rn: u8,
    rt: u8,
}

impl LdpPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let scale = 2 + self.opc;
        let datasize = 8 << scale;
        let offset = shift_lsl(sign_extend(self.imm7.into(), 7), scale);
        instruction_ldp(cpu, self.rt, self.rt2, self.rn, datasize, offset, true, false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 31, 1) as u8;
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdpPreIndex(Self { opc, imm7, rt2, rn, rt })
    }

    pub const LDP_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0010_1001_1100_0000_0000_0000_0000_0000,
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

/// LDTR
/// Load register (unprivileged)
#[derive(Debug, Clone, Copy)]
pub struct LdrUnpriv {
    size: u8,
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl LdrUnpriv {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = 8 << self.size;
        let regsize = if datasize == 64 { 64 } else { 32 };

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };
        address = address.wrapping_add(offset);
        let data = cpu.mmu.read_memory(address as usize, datasize / 8);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), zero_extend(data.1, regsize), datasize == 32);
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrUnpriv(Self { size, imm9, rn, rt })
    }
    pub const LDR_UNPRIV: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0100_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Load register (unscaled)
#[derive(Debug, Clone, Copy)]
pub struct Ldur {
    size: u8,
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl Ldur {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = 8 << self.size;
        let regsize = if datasize == 64 { 64 } else { 32 };

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };
        address = address.wrapping_add(offset);
        let data = cpu.mmu.read_memory(address as usize, datasize / 8);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), zero_extend(data.1, regsize), datasize == 32);
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ldur(Self { size, imm9, rn, rt })
    }
    pub const LDUR: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store register (unscaled)
#[derive(Debug, Clone, Copy)]
pub struct Stur {
    size: u8,
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl Stur {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = 8 << self.size;

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };
        address = address.wrapping_add(offset);
        let val = cpu.x_read(self.rt.into(), datasize);
        cpu.mmu.write_memory(address as usize, (datasize / 8).into(), val);
        if cpu.mmu.faulted {}
    }
    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Stur(Self { size, imm9, rn, rt })
    }
    pub const STUR: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store register halfword (unscaled)
#[derive(Debug, Clone, Copy)]
pub struct Sturh {
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl Sturh {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);
        let datasize = 16;

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        address = address.wrapping_add(offset);

        let val = cpu.x_read(self.rt.into(), datasize);

        cpu.mmu.write_memory(address as usize, (datasize / 8).into(), val);

        if cpu.mmu.faulted {}
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Sturh(Self { imm9, rn, rt })
    }
    pub const STURH: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0111_1000_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store-release register halfword
#[derive(Debug, Clone, Copy)]
pub struct Stlrh {
    rn: u8,
    rt: u8,
}

impl Stlrh {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 16;

        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        let val = cpu.x_read(self.rt.into(), datasize);

        cpu.mmu.write_memory(address as usize, (datasize / 8).into(), val);

        if cpu.mmu.faulted {}
    }
    pub const fn decode(word: u32) -> Instruction {
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Stlrh(Self { rn, rt })
    }
    pub const STLRH: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1100_0000_0000,
        value: 0b0100_1000_1001_1111_1111_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register byte (immediate)
#[derive(Debug, Clone, Copy)]
pub struct LdrbUnsignedOff {
    imm12: u16,
    rn: u8,
    rt: u8,
}

impl LdrbUnsignedOff {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = self.imm12 as u64;

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };
        address = address.wrapping_add(offset);
        let data = cpu.mmu.read_memory(address as usize, 1);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), zero_extend(data.1, 32), true);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrbUnsignedOff(Self { imm12, rn, rt })
    }
    pub const LDRB_UNSIGNED_OFF: InstDesc = InstDesc {
        mask: 0b1111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0011_1001_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load register byte (immediate)
#[derive(Debug, Clone, Copy)]
pub struct LdrbPostIndex {
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl LdrbPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };
        let data = cpu.mmu.read_memory(address as usize, 1);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), zero_extend(data.1, 32), true);

        address = address.wrapping_add(offset);
        if self.rn == 31 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(self.rn.into(), address, false);
        }
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrbPostIndex(Self { imm9, rn, rt })
    }
    pub const LDRB_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0011_1000_0100_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register byte (immediate)
#[derive(Debug, Clone, Copy)]
pub struct LdrbPreIndex {
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl LdrbPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };
        address = address.wrapping_add(offset);
        let data = cpu.mmu.read_memory(address as usize, 1);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), zero_extend(data.1, 32), true);

        // Write back the incremented address
        if self.rn == 31 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(self.rn.into(), address, false);
        }
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrbPreIndex(Self { imm9, rn, rt })
    }
    pub const LDRB_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0011_1000_0100_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed word (register)
#[derive(Debug, Clone, Copy)]
pub struct LdrswRegister {
    pub rm: u8,
    pub option: u8,
    pub s: bool,
    pub rn: u8,
    pub rt: u8,
}

impl LdrswRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if bits_get(self.option.into(), 1, 1) == 0 {
            panic!("Sub word index");
        }
        let extend_type = ExtendType::from_u8(self.option);
        let shift = if self.s { 2 } else { 0 };
        let offset = extend_register(cpu, self.rm, extend_type, shift, 64);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };
        address = address.wrapping_add(offset);

        let data = cpu.mmu.read_memory(address.try_into().unwrap(), 4);
        if cpu.mmu.faulted {
            return;
        }
        let extended = sign_extend(data.1, 32);
        cpu.x_write(self.rt.into(), extended, false);
    }
    pub fn decode(word: u32) -> Instruction {
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let s = get_bits_ct!(word, 12, 1) == 1;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrswRegister(Self { rm, option, s, rn, rt })
    }

    pub const LDRSW_REGISTER: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_1010_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LdrbRegister {
    pub rm: u8,
    pub option: u8,
    pub rn: u8,
    pub rt: u8,
}

impl LdrbRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if bits_get(self.option.into(), 1, 1) == 0 {
            panic!("Sub word index");
        }
        let extend_type = ExtendType::from_u8(self.option);
        let offset = extend_register(cpu, self.rm, extend_type, 0, 64);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };
        address = address.wrapping_add(offset);

        let data = cpu.mmu.read_memory(address.try_into().unwrap(), 1);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), zero_extend(data.1, 32), true);
    }
    pub fn decode(word: u32) -> Instruction {
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrbRegister(Self { rm, option, rn, rt })
    }

    pub const LDRB_REGISTER: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0011_1000_0110_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Load register halfword (register)
#[derive(Debug, Clone, Copy)]
pub struct LdrhRegister {
    pub rm: u8,
    pub option: u8,
    pub s: bool,
    pub rn: u8,
    pub rt: u8,
}

impl LdrhRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if bits_get(self.option.into(), 1, 1) == 0 {
            panic!("Sub word index");
        }
        let extend_type = ExtendType::from_u8(self.option);
        let shift = if self.s { 1 } else { 0 };
        let offset = extend_register(cpu, self.rm, extend_type, shift, 64);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };
        address = address.wrapping_add(offset);

        let data = cpu.mmu.read_memory(address.try_into().unwrap(), 2);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), zero_extend(data.1, 32), true);
    }
    pub fn decode(word: u32) -> Instruction {
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let s = get_bits_ct!(word, 12, 1) == 1;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrhRegister(Self { rm, option, s, rn, rt })
    }

    pub const LDRH_REGISTER: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0111_1000_0110_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Load register byte (unscaled)
#[derive(Debug, Clone, Copy)]
pub struct Ldurb {
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl Ldurb {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        address = address.wrapping_add(offset);

        let data = cpu.mmu.read_memory(address as usize, 1);
        if cpu.mmu.faulted {
            return;
        }

        cpu.x_write(self.rt.into(), zero_extend(data.1, 32), true);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ldurb(Self { imm9, rn, rt })
    }
    pub const LDURB: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0011_1000_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load register halfword (unscaled)
#[derive(Debug, Clone, Copy)]
pub struct Ldurh {
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl Ldurh {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        address = address.wrapping_add(offset);

        let data = cpu.mmu.read_memory(address as usize, 2);
        if cpu.mmu.faulted {
            return;
        }

        cpu.x_write(self.rt.into(), zero_extend(data.1, 32), true);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ldurh(Self { imm9, rn, rt })
    }
    pub const LDURH: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0111_1000_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed word (unscaled)
#[derive(Debug, Clone, Copy)]
pub struct Ldursw {
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl Ldursw {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        address = address.wrapping_add(offset);

        let data = cpu.mmu.read_memory(address as usize, 4);
        if cpu.mmu.faulted {
            return;
        }

        cpu.x_write(self.rt.into(), sign_extend(data.1, 32), false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ldursw(Self { imm9, rn, rt })
    }
    pub const LDURSW: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store register byte (unscaled)
#[derive(Debug, Clone, Copy)]
pub struct Sturb {
    imm9: u16,
    rn: u8,
    rt: u8,
}

impl Sturb {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let offset = sign_extend(self.imm9.into(), 9);

        let mut address =
            if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        address = address.wrapping_add(offset);

        let val = cpu.x_read(self.rt.into(), 8);
        cpu.mmu.write_memory(address.try_into().unwrap(), 1, val);
        if cpu.mmu.faulted {}
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Sturb(Self { imm9, rn, rt })
    }
    pub const STURB: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0011_1000_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Prefetch memory (immediate)
#[derive(Debug, Clone, Copy)]
pub struct Prfm {}
impl Prfm {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {
        // NOOP
    }
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Prfm(Self {})
    }
    pub const PRFM: InstDesc = InstDesc {
        mask: 0b1111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b1111_1001_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load-acquire exclusive register
#[derive(Debug, Clone, Copy)]
pub struct Ldaxr {
    pub size: u8,
    pub rn: u8,
    pub rt: u8,
}

impl Ldaxr {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let elsize = 8 << self.size;
        let regsize = if elsize == 64 { 64 } else { 32 };

        let dbytes = elsize / 8;

        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        cpu.monitor.set(address, dbytes as u8);
        let data = cpu.mmu.read_memory(address.try_into().unwrap(), dbytes);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), data.1, regsize == 32);
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ldaxr(Self { size, rn, rt })
    }

    pub const LDAXR: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_1111_1111_1100_0000_0000,
        value: 0b1000_1000_0101_1111_1111_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load exclusive register
#[derive(Debug, Clone, Copy)]
pub struct Ldxr {
    pub size: u8,
    pub rn: u8,
    pub rt: u8,
}

impl Ldxr {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let elsize = 8 << self.size;
        let regsize = if elsize == 64 { 64 } else { 32 };

        let dbytes = elsize / 8;

        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        cpu.monitor.set(address, dbytes as u8);
        let data = cpu.mmu.read_memory(address.try_into().unwrap(), dbytes);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), data.1, regsize == 32);
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ldxr(Self { size, rn, rt })
    }

    pub const LDXR: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_1111_1111_1100_0000_0000,
        value: 0b1000_1000_0101_1111_0111_1100_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Ldxrb {
    pub rn: u8,
    pub rt: u8,
}

impl Ldxrb {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        cpu.monitor.set(address, 1);
        let data = cpu.mmu.read_memory(address.try_into().unwrap(), 1);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), data.1, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ldxrb(Self { rn, rt })
    }

    pub const LDXRB: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1100_0000_0000,
        value: 0b0000_1000_0101_1111_0111_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load-acquire register
#[derive(Debug, Clone, Copy)]
pub struct Ldar {
    pub size: u8,
    pub rn: u8,
    pub rt: u8,
}

impl Ldar {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let elsize = 8 << self.size;
        let regsize = if elsize == 64 { 64 } else { 32 };

        let dbytes = elsize / 8;

        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        let data = cpu.mmu.read_memory(address.try_into().unwrap(), dbytes);
        if cpu.mmu.faulted {
            return;
        }
        cpu.x_write(self.rt.into(), data.1, regsize == 32);
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ldar(Self { size, rn, rt })
    }

    pub const LDAR: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_1111_1111_1100_0000_0000,
        value: 0b1000_1000_1101_1111_1111_1100_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Stxr {
    size: u8,
    rs: u8,
    rn: u8,
    rt: u8,
}

impl Stxr {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let elsize = 8 << self.size;
        let dbytes = elsize / 8;

        if self.rs == self.rt {
            panic!("Oh no");
        } else if self.rn == self.rs && self.rn != 31 {
            panic!("Oh no");
        }

        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        let data = cpu.x_read(self.rt.into(), elsize);

        let status = if cpu.monitor.safe(address, dbytes) {
            cpu.mmu.write_memory(address as usize, dbytes.into(), data);
            0u64
        } else {
            1u64
        };
        if cpu.mmu.faulted {
            return;
        }

        if status == 0 {
            cpu.monitor.off();
        }

        cpu.x_write(self.rs.into(), status, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let rs = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Stxr(Self { size, rs, rn, rt })
    }

    pub const STXR: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b1000_1000_0000_0000_0111_1100_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Stxrb {
    rs: u8,
    rn: u8,
    rt: u8,
}

impl Stxrb {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let elsize = 8;
        let dbytes = 1;

        if self.rs == self.rt {
            panic!("Oh no");
        } else if self.rn == self.rs && self.rn != 31 {
            panic!("Oh no");
        }

        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        let data = cpu.x_read(self.rt.into(), elsize);

        let status = if cpu.monitor.safe(address, dbytes) {
            cpu.mmu.write_memory(address as usize, dbytes.into(), data);
            0u64
        } else {
            1u64
        };
        if cpu.mmu.faulted {
            return;
        }

        if status == 0 {
            cpu.monitor.off();
        }

        cpu.x_write(self.rs.into(), status, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let rs = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Stxrb(Self { rs, rn, rt })
    }

    pub const STXRB: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0000_1000_0000_0000_0111_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed word (immediate) - Post Index
#[derive(Debug, Clone, Copy)]
pub struct LdrswImmPostIndex {
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}
impl LdrswImmPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = true;
        let wback = true;
        let offset = sign_extend(self.imm9.into(), 9);
        instruction_ldrsw_imm(cpu, self.rn, self.rt, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrswImmPostIndex(Self { imm9, rn, rt })
    }

    pub const LDRSW_IMM_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_1000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed word (immediate) - Pre Index
#[derive(Debug, Clone, Copy)]
pub struct LdrswImmPreIndex {
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrswImmPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = false;
        let wback = true;
        let offset = sign_extend(self.imm9.into(), 9);
        instruction_ldrsw_imm(cpu, self.rn, self.rt, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrswImmPreIndex(Self { imm9, rn, rt })
    }

    pub const LDRSW_IMM_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b1011_1000_1000_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed word (immediate) - Unsigned offset
#[derive(Debug, Clone, Copy)]
pub struct LdrswImmUnOffset {
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrswImmUnOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = false;
        let wback = false;
        let offset = zero_extend(self.imm12.into(), 64);
        let offset = offset << 2;
        instruction_ldrsw_imm(cpu, self.rn, self.rt, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrswImmUnOffset(Self { imm12, rn, rt })
    }

    pub const LDRSW_IMM_UN_OFFSET: InstDesc = InstDesc {
        mask: 0b1111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b1011_1001_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

// ---------------------------------------------

/// Load register halfword (immediate) - Post Index
#[derive(Debug, Clone, Copy)]
pub struct LdrhImmPostIndex {
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}
impl LdrhImmPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = true;
        let wback = true;
        let offset = sign_extend(self.imm9.into(), 9);
        instruction_ldrh_imm(cpu, self.rn, self.rt, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrhImmPostIndex(Self { imm9, rn, rt })
    }

    pub const LDRH_IMM_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0111_1000_0100_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register halfword (immediate) - Pre Index
#[derive(Debug, Clone, Copy)]
pub struct LdrhImmPreIndex {
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrhImmPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = false;
        let wback = true;
        let offset = sign_extend(self.imm9.into(), 9);
        instruction_ldrh_imm(cpu, self.rn, self.rt, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrhImmPreIndex(Self { imm9, rn, rt })
    }

    pub const LDRH_IMM_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0111_1000_0100_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register halfword (immediate) - Unsigned offset
#[derive(Debug, Clone, Copy)]
pub struct LdrhImmUnOffset {
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrhImmUnOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = false;
        let wback = false;
        let offset = zero_extend(self.imm12.into(), 64);
        let offset = offset << 1;
        instruction_ldrh_imm(cpu, self.rn, self.rt, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrhImmUnOffset(Self { imm12, rn, rt })
    }

    pub const LDRH_IMM_UN_OFFSET: InstDesc = InstDesc {
        mask: 0b1111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0111_1001_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store-release register - No offset
#[derive(Debug, Clone, Copy)]
pub struct StlrNoOffset {
    pub size: u8,
    pub rn: u8,
    pub rt: u8,
}

impl StlrNoOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let elsize = 8 << self.size;
        let dbytres = elsize / 8;

        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        let data = cpu.x_read(self.rt.into(), elsize);

        cpu.mmu.write_memory(address.try_into().unwrap(), dbytres.into(), data);
        if cpu.mmu.faulted {}
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::StlrNoOffset(Self { size, rn, rt })
    }

    pub const STLR_NO_OFFSET: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_1111_1111_1100_0000_0000,
        value: 0b1000_1000_1001_1111_1111_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Store-release register byte
#[derive(Debug, Clone, Copy)]
pub struct Stlrb {
    pub rn: u8,
    pub rt: u8,
}

impl Stlrb {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        let data = cpu.x_read(self.rt.into(), 8);

        cpu.mmu.write_memory(address.try_into().unwrap(), 1, data);
        if cpu.mmu.faulted {}
    }

    pub const fn decode(word: u32) -> Instruction {
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Stlrb(Self { rn, rt })
    }

    pub const STLRB: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1100_0000_0000,
        value: 0b0000_1000_1001_1111_1111_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Store-release exclusive register
#[derive(Debug, Clone, Copy)]
pub struct Stlxr {
    pub size: u8,
    pub rs: u8,
    pub rn: u8,
    pub rt: u8,
}

impl Stlxr {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let elsize = 8 << self.size;
        let dbytes = elsize / 8;

        if self.rs == self.rt {
            panic!("Oh no");
        } else if self.rn == self.rs && self.rn != 31 {
            panic!("Oh no");
        }

        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        let data = cpu.x_read(self.rt.into(), elsize);

        let status = if cpu.monitor.safe(address, dbytes) {
            cpu.mmu.write_memory(address as usize, dbytes.into(), data);
            0u64
        } else {
            1u64
        };
        if cpu.mmu.faulted {
            return;
        }

        if status == 0 {
            cpu.monitor.off();
        }

        cpu.x_write(self.rs.into(), status, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let size = get_bits_ct!(word, 30, 2) as u8;
        let rs = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Stlxr(Self { size, rs, rn, rt })
    }

    pub const STLXR: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b1000_1000_0000_0000_1111_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed halfword (immediate) - Post-index
#[derive(Debug, Clone, Copy)]
pub struct LdrshImmPostIndex {
    pub opc: bool,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrshImmPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = true;
        let wback = true;
        let offset = sign_extend(self.imm9.into(), 9);
        let regsize = 64 >> (self.opc as u8);
        instruction_ldrsh_imm(cpu, self.rn, self.rt, regsize, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 22, 1) == 1;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrshImmPostIndex(Self { opc, imm9, rn, rt })
    }

    pub const LDRSH_IMM_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1010_0000_0000_1100_0000_0000,
        value: 0b0111_1000_1000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed halfword (immediate) - Pre-index
#[derive(Debug, Clone, Copy)]
pub struct LdrshImmPreIndex {
    pub opc: bool,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrshImmPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = false;
        let wback = true;
        let offset = sign_extend(self.imm9.into(), 9);
        let regsize = 64 >> (self.opc as u8);
        instruction_ldrsh_imm(cpu, self.rn, self.rt, regsize, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 22, 1) == 1;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrshImmPreIndex(Self { opc, imm9, rn, rt })
    }

    pub const LDRSH_IMM_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1010_0000_0000_1100_0000_0000,
        value: 0b0111_1000_1000_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed halfword (immediate) - Pre-index
#[derive(Debug, Clone, Copy)]
pub struct LdrshImmUnsignedOffset {
    pub opc: bool,
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrshImmUnsignedOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = false;
        let wback = false;
        let offset = zero_extend(self.imm12.into(), 64).shl(1);
        let regsize = 64 >> (self.opc as u8);
        instruction_ldrsh_imm(cpu, self.rn, self.rt, regsize, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 22, 1) == 1;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrshImmUnsignedOffset(Self { opc, imm12, rn, rt })
    }

    pub const LDRSH_IMM_UNSIGNED_OFFSET: InstDesc = InstDesc {
        mask: 0b1111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0111_1001_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

// ------------------------------

/// Load register signed byte (immediate) - Post-index
#[derive(Debug, Clone, Copy)]
pub struct LdrsbImmPostIndex {
    pub opc: bool,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrsbImmPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = true;
        let wback = true;
        let offset = sign_extend(self.imm9.into(), 9);
        let regsize = 64 >> (self.opc as u8);
        instruction_ldrsb_imm(cpu, self.rn, self.rt, regsize, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 22, 1) == 1;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrsbImmPostIndex(Self { opc, imm9, rn, rt })
    }

    pub const LDRSB_IMM_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1010_0000_0000_1100_0000_0000,
        value: 0b0011_1000_1000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed byte (immediate) - Pre-index
#[derive(Debug, Clone, Copy)]
pub struct LdrsbImmPreIndex {
    pub opc: bool,
    pub imm9: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrsbImmPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = false;
        let wback = true;
        let offset = sign_extend(self.imm9.into(), 9);
        let regsize = 64 >> (self.opc as u8);
        instruction_ldrsb_imm(cpu, self.rn, self.rt, regsize, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 22, 1) == 1;
        let imm9 = get_bits_ct!(word, 12, 9) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrsbImmPreIndex(Self { opc, imm9, rn, rt })
    }

    pub const LDRSB_IMM_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1010_0000_0000_1100_0000_0000,
        value: 0b0011_1000_1000_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed byte (immediate) - Pre-index
#[derive(Debug, Clone, Copy)]
pub struct LdrsbImmUnsignedOffset {
    pub opc: bool,
    pub imm12: u16,
    pub rn: u8,
    pub rt: u8,
}

impl LdrsbImmUnsignedOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let post_index = false;
        let wback = false;
        let offset = zero_extend(self.imm12.into(), 64);
        let regsize = 64 >> (self.opc as u8);
        instruction_ldrsb_imm(cpu, self.rn, self.rt, regsize, offset, post_index, wback);
    }
    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 22, 1) == 1;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrsbImmUnsignedOffset(Self { opc, imm12, rn, rt })
    }

    pub const LDRSB_IMM_UNSIGNED_OFFSET: InstDesc = InstDesc {
        mask: 0b1111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0011_1001_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load pair of registers signed word
#[derive(Debug, Clone, Copy)]
pub struct LdpswPostIndex {
    pub imm7: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}

impl LdpswPostIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let wback = true;
        let postindex = true;
        instruction_ldpsw(cpu, self.imm7, self.rt2, self.rn, self.rt, postindex, wback);
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdpswPostIndex(Self { imm7, rt2, rn, rt })
    }

    pub const LDPSW_POST_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0110_1000_1100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LdpswPreIndex {
    pub imm7: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}

impl LdpswPreIndex {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let wback = true;
        let postindex = false;
        instruction_ldpsw(cpu, self.imm7, self.rt2, self.rn, self.rt, postindex, wback);
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdpswPreIndex(Self { imm7, rt2, rn, rt })
    }

    pub const LDPSW_PRE_INDEX: InstDesc = InstDesc {
        mask: 0b1111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0110_1001_1100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LdpswSignedOffset {
    pub imm7: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}

impl LdpswSignedOffset {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        instruction_ldpsw(cpu, self.imm7, self.rt2, self.rn, self.rt, false, false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm7 = get_bits_ct!(word, 15, 7) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdpswSignedOffset(Self { imm7, rt2, rn, rt })
    }

    pub const LDPSW_SIGNED_OFFSET: InstDesc = InstDesc {
        mask: 0b1111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0110_1001_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Load register signed byte (register)
#[derive(Debug, Clone, Copy)]
pub struct LdrsbReg {
    pub opc: u8,
    pub rm: u8,
    pub option: u8,
    pub s: bool,
    pub rn: u8,
    pub rt: u8,
}

impl LdrsbReg {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if bits_get(self.option.into(), 1, 1) == 0 {
            panic!("Sub word index")
        }
        let extend_type = ExtendType::from_u8(self.option);
        let regsize: u8 = 64 >> bits_get(self.opc.into(), 0, 1);

        let offset = extend_register(cpu, self.rm, extend_type, 0, 64);

        let address = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), 64) };

        let address = address.wrapping_add(offset) as usize;

        let data = cpu.mmu.read_memory(address, 1).1;

        if cpu.mmu.faulted {
            return;
        }

        let sign_extended = sign_extend(data, 8);
        cpu.x_write(self.rt.into(), sign_extended, regsize == 32);
    }

    pub const fn decode(word: u32) -> Instruction {
        let opc = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let s = get_bits_ct!(word, 12, 1) == 1;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::LdrsbReg(Self { opc, rm, option, s, rn, rt })
    }

    pub const LDRSB_REG: InstDesc = InstDesc {
        mask: 0b1111_1111_1010_0000_0000_1100_0000_0000,
        value: 0b0011_1000_1010_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Load exclusive pair of registers
#[derive(Debug, Clone, Copy)]
pub struct Ldxp {
    pub sz: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}

impl Ldxp {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let elsize: u8 = 32 << self.sz;
        let datasize = elsize * 2;
        let dbytes = datasize / 8;

        let address = cpu.address_for_rn(self.rn);

        cpu.monitor.set(address, dbytes);
        if elsize == 32 {
            let data = cpu.mmu.read_memory(address.try_into().unwrap(), dbytes.into()).1;
            if cpu.memory_op_faulted() {
                return;
            }

            let bot = bits_get(data, 0, elsize);
            let top = bits_get(data, elsize, elsize);
            cpu.x_write(self.rt.into(), bot, elsize == 32);
            cpu.x_write(self.rt2.into(), top, elsize == 32);
        } else {
            let address2 = address.wrapping_add(8);
            let data1 = cpu.mmu.read_memory(address.try_into().unwrap(), 8);
            if cpu.memory_op_faulted() {
                return;
            }
            let data2 = cpu.mmu.read_memory(address2.try_into().unwrap(), 8);
            if cpu.memory_op_faulted() {
                return;
            }
            cpu.x_write(self.rt.into(), data1.1, false);
            cpu.x_write(self.rt2.into(), data2.1, false);
        }
    }

    pub const fn decode(word: u32) -> Instruction {
        let sz = get_bits_ct!(word, 30, 1) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Ldxp(Self { sz, rt2, rn, rt })
    }

    pub const LDXP: InstDesc = InstDesc {
        mask: 0b1011_1111_1111_1111_1000_0000_0000_0000,
        value: 0b1000_1000_0111_1111_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Store exclusive pair of registers
#[derive(Debug, Clone, Copy)]
pub struct Stlxp {
    pub sz: u8,
    pub rs: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}

impl Stlxp {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let elsize: u8 = 32 << self.sz;
        let datasize = elsize * 2;
        let dbytes = datasize / 8;

        let address = cpu.address_for_rn(self.rn);

        let status = if cpu.monitor.safe(address, dbytes) {
            if elsize == 32 {
                let bot = cpu.x_read(self.rt.into(), elsize);
                let top = cpu.x_read(self.rt2.into(), elsize);

                cpu.mmu.write_memory(address.try_into().unwrap(), dbytes.into(), (top << 32) | bot);
                if cpu.memory_op_faulted() {
                    return;
                }
                0u64
            } else {
                let bot = cpu.x_read(self.rt.into(), elsize);
                let top = cpu.x_read(self.rt2.into(), elsize);

                cpu.mmu.write_memory(address.try_into().unwrap(), 8, bot);
                if cpu.memory_op_faulted() {
                    return;
                }

                cpu.mmu.write_memory((address + 8).try_into().unwrap(), 8, top);
                if cpu.memory_op_faulted() {
                    return;
                }
                0u64
            }
        } else {
            1u64
        };

        if status == 0 {
            cpu.monitor.off();
        }

        cpu.x_write(self.rs.into(), status, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sz = get_bits_ct!(word, 30, 1) as u8;
        let rs = get_bits_ct!(word, 16, 5) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Stlxp(Self { sz, rs, rt2, rn, rt })
    }

    pub const STLXP: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1000_0000_0000_0000,
        value: 0b1000_1000_0010_0000_1000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Stxp {
    pub sz: u8,
    pub rs: u8,
    pub rt2: u8,
    pub rn: u8,
    pub rt: u8,
}

impl Stxp {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.rs == self.rt || self.rs == self.rt2 {
            panic!("STXP: Rs overlaps with Rt/Rt2 (UNPREDICTABLE)");
        }
        if self.rs == self.rn && self.rn != 31 {
            panic!("STXP: Rs overlaps with Rn (UNPREDICTABLE)");
        }

        let elsize: u8 = 32 << self.sz;
        let datasize = elsize * 2;
        let dbytes = datasize / 8;

        let address = cpu.address_for_rn(self.rn);

        let status = if cpu.monitor.safe(address, dbytes) {
            if elsize == 32 {
                let bot = cpu.x_read(self.rt.into(), elsize);
                let top = cpu.x_read(self.rt2.into(), elsize);

                cpu.mmu.write_memory(address.try_into().unwrap(), dbytes.into(), (top << 32) | bot);
                if cpu.memory_op_faulted() {
                    return;
                }
                0u64
            } else {
                let bot = cpu.x_read(self.rt.into(), elsize);
                let top = cpu.x_read(self.rt2.into(), elsize);

                cpu.mmu.write_memory(address.try_into().unwrap(), 8, bot);
                if cpu.memory_op_faulted() {
                    return;
                }

                cpu.mmu.write_memory((address + 8).try_into().unwrap(), 8, top);
                if cpu.memory_op_faulted() {
                    return;
                }
                0u64
            }
        } else {
            1u64
        };

        if status == 0 {
            cpu.monitor.off();
        }

        cpu.x_write(self.rs.into(), status, true);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sz = get_bits_ct!(word, 30, 1) as u8;
        let rs = get_bits_ct!(word, 16, 5) as u8;
        let rt2 = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Stxp(Self { sz, rs, rt2, rn, rt })
    }

    pub const STXP: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1000_0000_0000_0000,
        value: 0b1000_1000_0010_0000_0000_0000_0000_0000,
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
