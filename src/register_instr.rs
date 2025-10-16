use std::ops::Not;

use crate::{
    branch::condition_holds,
    cpu::Cpu,
    data_processing::*,
    get_bits_ct,
    instruction::*,
    load_and_store::{ExtendType, extend_register},
    utils::{bits_get, decode_bit_mask, replicate_bits_u64, sign_extend, zero_extend},
};

// Temp NOOP instructions

// ------------------------

#[derive(Debug, Clone, Copy)]
pub struct Tlbi;
impl Tlbi {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Tlbi(Self)
    }

    pub const TLBI: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1000_1110_0000_0000_0000,
        value: 0b1101_0101_0000_1000_1000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Nop;
impl Nop {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Nop(Self)
    }

    pub const NOP: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1111_1111_1111,
        value: 0b1101_0101_0000_0011_0010_0000_0001_1111,
        decode: Self::decode,
    };
}

/// Instruction sync barrier
#[derive(Clone, Copy, Debug)]
pub struct Isb;
impl Isb {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Isb(Self)
    }

    pub const ISB: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_0000_1111_1111,
        value: 0b1101_0101_0000_0011_0011_0000_1101_1111,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Dc;

impl Dc {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Dc(Self)
    }
    pub const DC: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1000_1111_0000_0000_0000,
        value: 0b1101_0101_0000_1000_0111_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Dsb;

impl Dsb {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Dsb(Self)
    }
    pub const DSB: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_0000_1111_1111,
        value: 0b1101_0101_0000_0011_0011_0000_1001_1111,
        decode: Self::decode,
    };
}

// ------------------------

#[derive(Clone, Copy, Debug)]
pub struct Madd {
    pub sf: bool,
    pub rd: u8,
    pub rn: u8,
    pub ra: u8,
    pub rm: u8,
}
impl Madd {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        instruction_multiply_add(
            cpu,
            self.rn,
            self.rm,
            self.rd,
            self.ra,
            if self.sf { 64 } else { 32 },
        );
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let ra = get_bits_ct!(word, 10, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Madd(Self { sf, rd, rn, ra, rm })
    }

    pub const MADD: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_1000_0000_0000_0000,
        value: 0b0001_1011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct AddsShiftedReg {
    pub sf: bool,
    pub shift: ShiftTypes,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AddsShiftedReg {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        instruction_adds_shifted_register(
            cpu,
            self.rn,
            self.rm,
            self.rd,
            self.imm6,
            self.shift,
            if self.sf { 64 } else { 32 },
            !self.sf,
        );
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = decode_shift(get_bits_ct!(word, 22, 2) as u8);
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AddsShiftedReg(Self { sf, shift, rm, imm6, rn, rd })
    }

    pub const ADDS_SHIFTED_REG: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0010_1011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct AddsImmediate {
    pub sf: bool,
    pub sh: bool,
    pub imm12: u16,
    pub rn: u8,
    pub rd: u8,
}

impl AddsImmediate {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = if self.sf { 64 } else { 32 };
        let imm = if self.sh { (self.imm12 as u32) << 12 } else { self.imm12 as u32 };
        let op1 = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), datasize) };
        let op2 = zero_extend(imm.into(), datasize);

        let res = add_with_carry(op1, op2, 0);
        cpu.x_write(self.rd.into(), res.result, !self.sf);
        cpu.pstate.set_flags_from_bits(res.flag_to_bits());
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let sh = get_bits_ct!(word, 22, 1) == 1;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AddsImmediate(Self { sf, sh, imm12, rn, rd })
    }

    pub const ADDS_IMMEDIATE: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0011_0001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct AddShiftedReg {
    pub sf: bool,
    pub shift: ShiftTypes,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AddShiftedReg {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        instruction_add_shifted_register(
            cpu,
            self.rn,
            self.rm,
            self.rd,
            self.imm6,
            self.shift,
            if self.sf { 64 } else { 32 },
            !self.sf,
        );
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = decode_shift(get_bits_ct!(word, 22, 2) as u8);
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if let ShiftTypes::StRor = shift {
            panic!("Undefined");
        }
        if !sf && get_bits_ct!(imm6, 5, 1) == 1 {
            panic!("Undefined")
        }
        Instruction::AddShiftedReg(Self { sf, shift, rm, imm6, rn, rd })
    }

    pub const ADD_SHIFTED_REG: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0000_1011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct AddExtendedRegister {
    pub sf: bool,
    pub rm: u8,
    pub option: u8,
    pub imm3: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AddExtendedRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if let 0b101..=0b111 = self.imm3 {
            panic!("UNDEFINED")
        }
        let datasize = if self.sf { 64 } else { 32 };
        let extend_type = ExtendType::from_u8(self.option);
        let op1 = if self.rn == 31 { cpu.sp_read() } else { cpu.x_read(self.rn.into(), datasize) };
        let op2 = extend_register(cpu, self.rm, extend_type, self.imm3, datasize);

        let res = add_with_carry(op1, op2, 0).result;

        if self.rd == 31 {
            cpu.sp_write(zero_extend(res, datasize));
        } else {
            cpu.x_write(self.rd.into(), res, !self.sf);
        }
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let option = get_bits_ct!(word, 13, 3) as u8;
        let imm3 = get_bits_ct!(word, 10, 3) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AddExtendedRegister(Self { sf, rm, option, imm3, rn, rd })
    }

    pub const ADD_EXTENDED_REGISTER: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_0000_0000_0000_0000,
        value: 0b0000_1011_0010_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct SubShiftedRegister {
    pub sf: bool,
    pub shift: ShiftTypes,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}
impl SubShiftedRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        instruction_sub_shifted_register(
            cpu,
            self.rn,
            self.rm,
            self.rd,
            self.imm6,
            self.shift,
            if self.sf { 64 } else { 32 },
            !self.sf,
        );
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = decode_shift(get_bits_ct!(word, 22, 2) as u8);
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::SubShiftedRegister(Self { sf, shift, rm, imm6, rn, rd })
    }
    pub const SUB_SHIFTED_REGISTER: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0100_1011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Udiv {
    pub sf: bool,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Udiv {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        instruction_udiv(cpu, self.rd, self.rn, self.rm, if self.sf { 64 } else { 32 });
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Udiv(Self { sf, rm, rn, rd })
    }
    pub const UDIV: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0001_1010_1100_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

/// Bitwise AND (shifted register)
#[derive(Debug, Clone, Copy)]
pub struct AndShiftedRegister {
    pub sf: bool,
    pub shift: ShiftTypes,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AndShiftedRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let op1 = cpu.x_read(self.rn.into(), width);
        let op2 = shift_reg(cpu, self.rm, self.shift, self.imm6, width);

        cpu.x_write(self.rd.into(), op1 & op2, !self.sf);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = decode_shift(get_bits_ct!(word, 22, 2) as u8);
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && get_bits_ct!(imm6, 4, 1) == 1 {
            panic!("Undefined, end of decode");
        }
        Instruction::AndShiftedRegister(Self { sf, shift, rm, imm6, rn, rd })
    }

    pub const AND_SHIFTED_REGISTER: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0000_1010_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Bitwise AND (immediate), setting flags
#[derive(Debug, Clone, Copy)]
pub struct AndsImmediate {
    pub sf: bool,
    pub n: bool,
    pub immr: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AndsImmediate {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let (imms, _) = decode_bit_mask(self.n, self.imms, self.immr, true, width);
        instruction_ands_imm(cpu, self.rn, self.rd, width, imms);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let n = get_bits_ct!(word, 22, 1) == 1;
        let immr = get_bits_ct!(word, 16, 6) as u8;
        let imms = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && n {
            panic!("Undefined, end of decode");
        }
        Instruction::AndsImmediate(Self { sf, n, immr, imms, rn, rd })
    }

    pub const ANDS_IMMEDIATE: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0111_0010_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Bitwise OR (immediate)
#[derive(Debug, Clone, Copy)]
pub struct OrrImmediate {
    pub sf: bool,
    pub n: bool,
    pub immr: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

impl OrrImmediate {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let (imms, _) = decode_bit_mask(self.n, self.imms, self.immr, true, width);
        instruction_orr_imm(cpu, self.rn, self.rd, width, imms);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let n = get_bits_ct!(word, 22, 1) == 1;
        let immr = get_bits_ct!(word, 16, 6) as u8;
        let imms = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && n {
            panic!("Undefined, end of decode");
        }
        Instruction::OrrImmediate(Self { sf, n, immr, imms, rn, rd })
    }

    pub const ORR_IMMEDIATE: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0011_0010_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Bitwise AND (shifted register), setting flags
#[derive(Debug, Clone, Copy)]
pub struct AndsShiftedReg {
    pub sf: bool,
    pub shift: ShiftTypes,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AndsShiftedReg {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        instruction_ands_shifted_reg(cpu, self.rm, self.rn, self.rd, width, self.imm6, self.shift);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = decode_shift(get_bits_ct!(word, 22, 2) as u8);
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && get_bits_ct!(imm6, 4, 1) == 1 {
            panic!("Undefined, end of decode");
        }

        Instruction::AndsShiftedReg(Self { sf, shift, rm, imm6, rn, rd })
    }

    pub const ANDS_SHIFTED_REG: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0110_1010_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Bitwise AND
#[derive(Debug, Clone, Copy)]
pub struct AndImmediate {
    pub sf: bool,
    pub n: bool,
    pub immr: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

impl AndImmediate {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let (imms, _) = decode_bit_mask(self.n, self.imms, self.immr, true, width);

        let result = cpu.x_read(self.rn.into(), width) & imms;
        if self.rd == 31 {
            cpu.sp_write(zero_extend(result, 64));
        } else {
            cpu.x_write(self.rd.into(), result, !self.sf);
        }
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let n = get_bits_ct!(word, 22, 1) == 1;
        let immr = get_bits_ct!(word, 16, 6) as u8;
        let imms = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && n {
            panic!("Undefined, end of decode");
        }
        Instruction::AndImmediate(Self { sf, n, immr, imms, rn, rd })
    }

    pub const AND_IMMEDIATE: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0001_0010_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Bitwise OR
#[derive(Debug, Clone, Copy)]
pub struct OrShiftedRegister {
    pub sf: bool,
    pub shift: ShiftTypes,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}

impl OrShiftedRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let op1 = cpu.x_read(self.rn.into(), width);
        let op2 = shift_reg(cpu, self.rm, self.shift, self.imm6, width);

        cpu.x_write(self.rd.into(), op1 | op2, !self.sf);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = decode_shift(get_bits_ct!(word, 22, 2) as u8);
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && get_bits_ct!(imm6, 5, 1) == 1 {
            panic!("Undefined, end of decode");
        }

        Instruction::OrShiftedRegister(Self { sf, shift, rm, imm6, rn, rd })
    }

    pub const OR_SHIFTED_REGISTER: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0010_1010_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Bitwise bit clear (shifted register)
#[derive(Debug, Clone, Copy)]
pub struct BicShiftedReg {
    pub sf: bool,
    pub shift: ShiftTypes,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}

impl BicShiftedReg {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let op1 = cpu.x_read(self.rn.into(), width);
        let op2 = shift_reg(cpu, self.rm, self.shift, self.imm6, width);

        cpu.x_write(self.rd.into(), op1 & op2.not(), !self.sf);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = decode_shift(get_bits_ct!(word, 22, 2) as u8);
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && get_bits_ct!(imm6, 4, 1) == 1 {
            panic!("Undefined, end of decode");
        }

        Instruction::BicShiftedReg(Self { sf, shift, rm, imm6, rn, rd })
    }

    pub const BIC_SHIFTED_REG: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0000_1010_0010_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct BicShiftedRegSet {
    pub sf: bool,
    pub shift: ShiftTypes,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}

impl BicShiftedRegSet {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let op1 = cpu.x_read(self.rn.into(), width);
        let op2 = shift_reg(cpu, self.rm, self.shift, self.imm6, width);
        let result = op1 & op2.not();
        let is_zero = if result == 0 { 1 } else { 0 };
        let state = (bits_get(result, width - 1, 1) << 3) | (is_zero << 2);

        cpu.x_write(self.rd.into(), result, !self.sf);
        cpu.pstate.set_flags_from_bits(state.try_into().unwrap());
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = decode_shift(get_bits_ct!(word, 22, 2) as u8);
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && get_bits_ct!(imm6, 4, 1) == 1 {
            panic!("Undefined, end of decode");
        }

        Instruction::BicShiftedRegSet(Self { sf, shift, rm, imm6, rn, rd })
    }

    pub const BIC_SHIFTED_REG_SET: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0110_1010_0010_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Csel {
    pub sf: bool,
    pub rm: u8,
    pub cond: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Csel {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let result = if condition_holds(cpu, self.cond) {
            cpu.x_read(self.rn.into(), width)
        } else {
            cpu.x_read(self.rm.into(), width)
        };
        cpu.x_write(self.rd.into(), result, !self.sf);
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let cond = get_bits_ct!(word, 12, 4) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Csel(Self { sf, rm, cond, rn, rd })
    }
    pub const CSEL: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0001_1010_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Csneg {
    pub sf: bool,
    pub rm: u8,
    pub cond: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Csneg {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let result = if condition_holds(cpu, self.cond) {
            cpu.x_read(self.rn.into(), width)
        } else {
            let not = cpu.x_read(self.rm.into(), width).not();
            bits_get(not, 0, width) + 1
        };
        cpu.x_write(self.rd.into(), result, !self.sf);
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let cond = get_bits_ct!(word, 12, 4) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Csneg(Self { sf, rm, cond, rn, rd })
    }
    pub const CSNEG: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0101_1010_1000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Form PC-relative address to 4KB page
#[derive(Clone, Copy, Debug)]
pub struct Adrp {
    pub immlo: u8,
    pub immhi: u32,
    pub rd: u8,
}

impl Adrp {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let imm: u64 = ((self.immhi as u64) << 14) | ((self.immlo as u64) << 12);
        let imm = sign_extend(imm, 33);
        let base = old_pc & !0xfffu64;
        cpu.x_write(self.rd.into(), base.wrapping_add(imm), false);
    }

    pub const fn decode(word: u32) -> Instruction {
        let immlo = get_bits_ct!(word, 29, 2) as u8;
        let immhi = get_bits_ct!(word, 5, 19);
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Adrp(Self { immlo, immhi, rd })
    }

    pub const ADRP: InstDesc = InstDesc {
        mask: 0b1001_1111_0000_0000_0000_0000_0000_0000,
        value: 0b1001_0000_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Ubfx {
    pub sf: bool,
    pub n: bool,
    pub immr: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Ubfx {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let (wmask, tmask) = decode_bit_mask(self.n, self.imms, self.immr, false, width);
        let src = cpu.x_read(self.rn.into(), width);
        let bot = shift_ror(src, self.immr) & wmask;

        cpu.x_write(self.rd.into(), bot & tmask, !self.sf);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let n = get_bits_ct!(word, 22, 1) == 1;
        let immr = get_bits_ct!(word, 16, 6) as u8;
        let imms = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if sf && !n {
            panic!("Undefined, end of decode");
        }
        Instruction::Ubfx(Self { sf, n, immr, imms, rn, rd })
    }

    pub const UBFX: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0101_0011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Bitfield move
#[derive(Clone, Copy, Debug)]
pub struct Bfm {
    pub sf: bool,
    pub n: bool,
    pub immr: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Bfm {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let (wmask, tmask) = decode_bit_mask(self.n, self.imms, self.immr, false, width);
        let dst = cpu.x_read(self.rd.into(), width);
        let src = cpu.x_read(self.rn.into(), width);
        let bot = (dst & wmask.not()) | (shift_ror(src, self.immr) & wmask);
        let val = (dst & tmask.not()) | (bot & tmask);

        cpu.x_write(self.rd.into(), val, !self.sf);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let n = get_bits_ct!(word, 22, 1) == 1;
        let immr = get_bits_ct!(word, 16, 6) as u8;
        let imms = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && !n {
            panic!("Undefined, end of decode");
        }
        Instruction::Bfm(Self { sf, n, immr, imms, rn, rd })
    }

    pub const BFM: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0011_0011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Signed bitfield move
#[derive(Clone, Copy, Debug)]
pub struct Sbfm {
    pub sf: bool,
    pub n: bool,
    pub immr: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Sbfm {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let width = if self.sf { 64 } else { 32 };
        let (wmask, tmask) = decode_bit_mask(self.n, self.imms, self.immr, false, width);

        let src = cpu.x_read(self.rn.into(), width);
        let bot = shift_ror(src, self.immr) & wmask;
        let our_bit = bits_get(src, self.imms, 1);
        let top = replicate_bits_u64(our_bit, 1, width.into());
        let value = (top & tmask.not()) | (bot & tmask);

        cpu.x_write(self.rd.into(), value, !self.sf);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let n = get_bits_ct!(word, 22, 1) == 1;
        let immr = get_bits_ct!(word, 16, 6) as u8;
        let imms = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && !n {
            panic!("Undefined, end of decode");
        }
        Instruction::Sbfm(Self { sf, n, immr, imms, rn, rd })
    }

    pub const SBFM: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0001_0011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Logical shift left variable
#[derive(Clone, Copy, Debug)]
pub struct Lslv {
    pub sf: bool,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Lslv {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = if self.sf { 64 } else { 32 };
        let op2 = cpu.x_read(self.rm.into(), datasize);
        cpu.x_write(
            self.rd.into(),
            cpu.x_read(self.rn.into(), datasize) << (op2 % (datasize as u64)),
            !self.sf,
        );
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Lslv(Self { sf, rm, rn, rd })
    }
    pub const LSLV: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0001_1010_1100_0000_0010_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Logical shift right variable
#[derive(Clone, Copy, Debug)]
pub struct Lsrv {
    pub sf: bool,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Lsrv {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = if self.sf { 64 } else { 32 };
        let op2 = cpu.x_read(self.rm.into(), datasize);
        cpu.x_write(
            self.rd.into(),
            shift_reg(
                cpu,
                self.rn,
                ShiftTypes::StLsr,
                (op2 % datasize as u64).try_into().unwrap(),
                datasize,
            ),
            !self.sf,
        );
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Lsrv(Self { sf, rm, rn, rd })
    }
    pub const LSRV: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0001_1010_1100_0000_0010_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Subtract optionally-shifted register, setting flags
#[derive(Clone, Copy, Debug)]
pub struct SubsShiftedReg {
    pub sf: bool,
    pub shift: ShiftTypes,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}

impl SubsShiftedReg {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        instruction_subs_shifted_reg(
            cpu,
            self.rn,
            self.rm,
            self.rd,
            self.imm6,
            self.shift,
            if self.sf { 64 } else { 32 },
            !self.sf,
        );
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = decode_shift(get_bits_ct!(word, 22, 2) as u8);
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if !sf && get_bits_ct!(imm6, 4, 1) == 1 {
            panic!("Undefined, end of decode");
        }
        Instruction::SubsShiftedReg(Self { sf, shift, rm, imm6, rn, rd })
    }

    pub const SUBS_SHIFTED_REG: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0110_1011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Count leading zeros
#[derive(Debug, Clone, Copy)]
pub struct Clz {
    pub sf: bool,
    pub rn: u8,
    pub rd: u8,
}

impl Clz {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = if self.sf { 64 } else { 32 };

        let op1 = cpu.x_read(self.rn.into(), datasize);

        let count: u64 = if self.sf {
            op1.leading_zeros() as u64 // 0 → 64, etc.
        } else {
            (op1 as u32).leading_zeros() as u64 // 0 → 32, etc.
        };

        cpu.x_write(self.rd.into(), bits_get(count, 0, datasize), !self.sf);
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Clz(Self { sf, rn, rd })
    }

    pub const CLZ: InstDesc = InstDesc {
        mask: 0b0111_1111_1111_1111_1111_1100_0000_0000,
        value: 0b0101_1010_1100_0000_0001_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Rev {
    pub sf: bool,
    pub opc: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Rev {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = if self.sf { 64 } else { 32 };
        let op = cpu.x_read(self.rn.into(), datasize);
        let res = if datasize == 32 { (op as u32).swap_bytes() as u64 } else { op.swap_bytes() };
        cpu.x_write(self.rd.into(), res, datasize == 32);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let opc = get_bits_ct!(word, 10, 2) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        if opc == 0b11 && !sf {
            panic!("Undefined Rev")
        }
        Instruction::Rev(Self { sf, opc, rn, rd })
    }

    pub const REV: InstDesc = InstDesc {
        mask: 0b0111_1111_1111_1111_1111_1000_0000_0000,
        value: 0b0101_1010_1100_0000_0000_1000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct CcmpRegister {
    pub sf: bool,
    pub rm: u8,
    pub cond: u8,
    pub rn: u8,
    pub nzcv: u8,
}

impl CcmpRegister {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = if self.sf { 64 } else { 32 };
        let mut flags = self.nzcv;
        if condition_holds(cpu, self.cond) {
            let op1 = cpu.x_read(self.rn.into(), datasize);
            let op2 = cpu.x_read(self.rm.into(), datasize).not();
            flags = add_with_carry(op1, op2, 1).flag_to_bits();
        }
        cpu.pstate.set_flags_from_bits(flags);
    }

    pub fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let cond = get_bits_ct!(word, 12, 4) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let nzcv = get_bits_ct!(word, 0, 4) as u8;
        Instruction::CcmpRegister(Self { sf, rm, cond, rn, nzcv })
    }

    pub const CCMP_REGISTER: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_0000_1100_0001_0000,
        value: 0b0111_1010_0100_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct EorShiftedReg {
    pub sf: bool,
    pub shift: u8,
    pub rm: u8,
    pub imm6: u8,
    pub rn: u8,
    pub rd: u8,
}
impl EorShiftedReg {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if !self.sf && get_bits_ct!(self.imm6, 5, 1) == 1 {
            panic!("Undef");
        }
        let datasize = if self.sf { 64 } else { 32 };
        let shift = decode_shift(self.shift);
        let op1 = cpu.x_read(self.rn.into(), datasize);
        let op2 = shift_reg(cpu, self.rm, shift, self.imm6, datasize);

        cpu.x_write(self.rd.into(), op1 ^ op2, !self.sf);
    }

    pub fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let shift = get_bits_ct!(word, 22, 2) as u8;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imm6 = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::EorShiftedReg(Self { sf, shift, rm, imm6, rn, rd })
    }

    pub const EOR_SHIFTED_REG: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0100_1010_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}
