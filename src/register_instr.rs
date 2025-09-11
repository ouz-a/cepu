use std::ops::Not;

use crate::{
    branch::condition_holds,
    cpu::Cpu,
    data_processing::*,
    get_bits_ct,
    instruction::*,
    utils::{bits_get, decode_bit_mask, sign_extend, zero_extend},
};

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
        if !sf && !n {
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
            cpu.x_write(self.rd.into(), result, self.sf);
        }
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
        if !sf && get_bits_ct!(imm6, 4, 1) == 1 {
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
        cpu.x_write(self.rd.into(), result, self.sf);
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

/// Form PC-relative address to 4KB page
#[derive(Clone, Copy, Debug)]
pub struct Adrp {
    pub immlo: u8,
    pub immhi: u32,
    pub rd: u8,
}

impl Adrp {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let imm: u32 = (self.immhi << 14) | ((self.immlo as u32) << 12);
        let imm = sign_extend(imm.into(), 64);
        let base = bits_get(old_pc, 12, 64 - 12);
        cpu.x_write(self.rd.into(), base + imm, false);
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
        let (wmask, tmask) = decode_bit_mask(self.n, self.imms, self.immr, true, width);
        let src = cpu.x_read(self.rn.into(), width);
        let bot = shift_ror(src, self.immr.into()) & wmask;

        cpu.x_write(self.rd.into(), bot & tmask, false);
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
        Instruction::Ubfx(Self { sf, n, immr, imms, rn, rd })
    }

    pub const UBFX: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0101_0011_0000_0000_0000_0000_0000_0000,
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
