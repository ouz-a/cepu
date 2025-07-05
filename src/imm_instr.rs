use crate::{
    cpu::Cpu,
    data_processing::{
        instruction_addd_immediate, instruction_imm_sub, instruction_imm_subs, instruction_movz,
    },
    get_bits_ct,
    instruction::*,
};

#[derive(Debug, Clone, Copy)]
pub struct AddImmediate {
    pub sf: bool,
    pub sh: bool,
    pub imm12: u16,
    pub rn: u8,
    pub rd: u8,
}

impl AddImmediate {
    pub fn exec(self, cpu: &mut Cpu) {
        let imm: u32 = if self.sh { (self.imm12 as u32) << 12 } else { self.imm12 as u32 };
        instruction_addd_immediate(cpu, self.rd, self.rn, imm, !self.sf);
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let sh = get_bits_ct!(word, 22, 1) == 1;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AddImmediate(AddImmediate { sf, sh, imm12, rn, rd })
    }

    pub const ADD_IMMEDIATE: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0001_0001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Movz {
    pub sf: bool,
    pub hw: u8,
    pub imm16: u16,
    pub rd: u8,
}

impl Movz {
    pub fn execute(self, cpu: &mut Cpu) {
        if !self.sf && (self.hw & 1) == 1 {
            panic!("Undefined")
        }
        instruction_movz(cpu, self.rd, self.imm16, 0, !self.sf);
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let hw = get_bits_ct!(word, 21, 2) as u8;
        let imm16 = get_bits_ct!(word, 5, 16) as u16;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Movz(Movz { sf, hw, imm16, rd })
    }
    pub const MOVZ: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0101_0010_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Subs {
    pub sf: bool,
    pub sh: bool,
    pub imm12: u16,
    pub rn: u8,
    pub rd: u8,
}

impl Subs {
    pub fn exec(self, cpu: &mut Cpu) {
        let imm = if self.sh { (self.imm12 as u32) << 12 } else { self.imm12 as u32 };
        instruction_imm_subs(cpu, self.rn, self.rd, imm, if self.sf { 64 } else { 32 });
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let sh = get_bits_ct!(word, 22, 1) == 1;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Subs(Subs { sf, sh, imm12, rn, rd })
    }
    pub const SUBS: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0111_0001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}

#[derive(Debug, Clone, Copy)]
pub struct SubImmediate {
    pub sf: bool,
    pub sh: bool,
    pub imm12: u16,
    pub rn: u8,
    pub rd: u8,
}

impl SubImmediate {
    pub fn exec(self, cpu: &mut Cpu) {
        let imm = if self.sh { (self.imm12 as u32) << 12 } else { self.imm12 as u32 };
        instruction_imm_sub(cpu, self.rn, self.rd, imm, if self.sf { 64 } else { 32 });
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let sh = get_bits_ct!(word, 22, 1) == 1;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::SubImmediate(SubImmediate { sf, sh, imm12, rn, rd })
    }
    pub const SUB_IMMEDIATE: InstDesc = InstDesc {
        mask: 0b0111_1111_1100_0000_0000_0000_0000_0000,
        value: 0b0101_0001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
        exec: |c, d| d.exec(c),
    };
}
