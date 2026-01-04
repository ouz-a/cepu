use std::ops::Not;

use crate::{
    cpu::Cpu,
    data_processing::{
        add_with_carry, instruction_add_immediate, instruction_imm_sub, instruction_imm_subs,
        instruction_movk, instruction_movn, instruction_movz,
    },
    get_bits_ct,
    instruction::*,
    utils::{Utils, bits_get},
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
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let imm: u32 = if self.sh { (self.imm12 as u32) << 12 } else { self.imm12 as u32 };
        instruction_add_immediate(cpu, self.rd, self.rn, imm, !self.sf);
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let sh = get_bits_ct!(word, 22, 1) == 1;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::AddImmediate(Self { sf, sh, imm12, rn, rd })
    }

    pub const ADD_IMMEDIATE: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0001_0001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
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
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if !self.sf && get_bits_ct!(self.hw, 1, 1) == 1 {
            panic!("Undefined")
        }
        instruction_movz(cpu, self.rd, self.imm16, self.hw << 4, !self.sf);
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let hw = get_bits_ct!(word, 21, 2) as u8;
        let imm16 = get_bits_ct!(word, 5, 16) as u16;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Movz(Self { sf, hw, imm16, rd })
    }
    pub const MOVZ: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0101_0010_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Movn {
    pub sf: bool,
    pub hw: u8,
    pub imm16: u16,
    pub rd: u8,
}

impl Movn {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if !self.sf && (self.hw & 0b10) != 0 {
            panic!("Undefined");
        }
        instruction_movn(cpu, self.rd.into(), self.imm16, self.hw << 4, !self.sf);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let hw = get_bits_ct!(word, 21, 2) as u8;
        let imm16 = get_bits_ct!(word, 5, 16) as u16;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Movn(Self { sf, hw, imm16, rd })
    }

    pub const MOVN: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0001_0010_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Movk {
    pub sf: bool,
    pub hw: u8,
    pub imm16: u16,
    pub rd: u8,
}

impl Movk {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if !self.sf && (self.hw & 0b10) != 0 {
            panic!("Undefined");
        }
        instruction_movk(cpu, self.rd.into(), self.imm16, self.hw << 4, !self.sf);
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let hw = get_bits_ct!(word, 21, 2) as u8;
        let imm16 = get_bits_ct!(word, 5, 16) as u16;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Movk(Self { sf, hw, imm16, rd })
    }
    pub const MOVK: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0111_0010_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
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
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let imm = if self.sh { (self.imm12 as u32) << 12 } else { self.imm12 as u32 };
        instruction_imm_subs(cpu, self.rd, self.rn, imm, if self.sf { 64 } else { 32 });
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let sh = get_bits_ct!(word, 22, 1) == 1;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Subs(Self { sf, sh, imm12, rn, rd })
    }
    pub const SUBS: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0111_0001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Subtract with carry
#[derive(Debug, Clone, Copy)]
pub struct Sbc {
    pub sf: bool,
    pub rm: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Sbc {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = 32 << self.sf as u8;

        if datasize == 32 {
            let op1 = cpu.x_read(self.rn.into(), datasize);
            let op2 = (cpu.x_read(self.rm.into(), datasize) as u32).not();
            let op2 = op2 as u64;
            let res = add_with_carry(op1, op2, cpu.pstate.c as u64, 32);
            cpu.x_write(self.rd.into(), res.result, true);
        } else {
            let op1 = cpu.x_read(self.rn.into(), datasize);
            let op2 = (cpu.x_read(self.rm.into(), datasize)).not();
            let res = add_with_carry(op1, op2, cpu.pstate.c as u64, 64);
            cpu.x_write(self.rd.into(), res.result, true);
        }
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Sbc(Self { sf, rm, rn, rd })
    }
    pub const SBC: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0101_1010_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
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
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let imm = if self.sh { (self.imm12 as u32) << 12 } else { self.imm12 as u32 };
        instruction_imm_sub(cpu, self.rd, self.rn, imm, if self.sf { 64 } else { 32 });
    }
    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let sh = get_bits_ct!(word, 22, 1) == 1;
        let imm12 = get_bits_ct!(word, 10, 12) as u16;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::SubImmediate(Self { sf, sh, imm12, rn, rd })
    }
    pub const SUB_IMMEDIATE: InstDesc = InstDesc {
        mask: 0b0111_1111_1000_0000_0000_0000_0000_0000,
        value: 0b0101_0001_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Extract register
#[derive(Debug, Clone, Copy)]
pub struct Extr {
    pub sf: bool,
    pub n: bool,
    pub rm: u8,
    pub imms: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Extr {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if self.n != self.sf {
            panic!("Undefined")
        }
        if !self.sf && bits_get(self.imms.into(), 5, 1) == 1 {
            panic!("Undefined")
        }
        let lsb = self.imms as u64;
        let datasize = self.sf.datasize_sf();
        let op1 = cpu.x_read(self.rn.into(), datasize);
        let op2 = cpu.x_read(self.rm.into(), datasize);

        let concat: u128 = ((op1 as u128) << datasize) | (op2 as u128);

        let result = bits_get((concat >> lsb) as u64, 0, datasize);

        cpu.x_write(self.rd.into(), result, datasize == 32);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let n = get_bits_ct!(word, 22, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let imms = get_bits_ct!(word, 10, 6) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Extr(Self { sf, n, rm, imms, rn, rd })
    }

    pub const EXTR: InstDesc = InstDesc {
        mask: 0b0111_1111_1010_0000_0000_0000_0000_0000,
        value: 0b0001_0011_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}
