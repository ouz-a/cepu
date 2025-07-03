use crate::{
    cpu::Cpu,
    data_processing::{
        ShiftTypes, decode_shift, instruction_add_shifted_register, instruction_multiply_add,
        instruction_sub_shifted_register, instruction_udiv,
    },
    get_bits_ct,
    instruction::*,
};

#[derive(Clone, Copy, Debug)]
pub struct Madd {
    pub sf: bool,
    pub rd: u8,
    pub rn: u8,
    pub ra: u8,
    pub rm: u8,
}
impl Madd {
    pub fn exec(self, cpu: &mut Cpu) {
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
        exec: |c, d| d.exec(c),
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
    pub fn exec(self, cpu: &mut Cpu) {
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
        Instruction::AddShiftedReg(AddShiftedReg { sf, shift, rm, imm6, rn, rd })
    }

    pub const ADD_SHIFTED_REG: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0010_1011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
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
    pub fn exec(self, cpu: &mut Cpu) {
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
        Instruction::SubShiftedRegister(SubShiftedRegister { sf, shift, rm, imm6, rn, rd })
    }
    pub const SUB_SHIFTED_REGISTER: InstDesc = InstDesc {
        mask: 0b0111_1111_0010_0000_0000_0000_0000_0000,
        value: 0b0110_1011_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
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
    pub fn execute(self, cpu: &mut Cpu) {
        instruction_udiv(cpu, self.rd, self.rn, self.rm, if self.sf { 64 } else { 32 });
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Udiv(Udiv { sf, rm, rn, rd })
    }
    pub const UDIV: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0001_1010_1100_0000_0000_1000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}
