use crate::{
    branch::{
        instruction_bl, instruction_branch, instruction_bunc, instruction_cbnz, instruction_cbz,
        instruction_eret, instruction_msr_imm, instruction_ret, instruction_tbnz,
    },
    cpu::{Cpu, ExceptionLevel, INSTRUCTION_SIZE, PstateField},
    get_bits_ct,
    instruction::{InstDesc, Instruction},
    utils::sign_extend,
};

#[derive(Debug, Clone, Copy)]
pub struct Ret {
    pub rn: u8,
}

impl Ret {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        instruction_ret(cpu, self.rn, old_pc);
    }

    pub const fn decode(word: u32) -> Instruction {
        Instruction::Ret(Self { rn: get_bits_ct!(word, 5, 5) as u8 })
    }

    pub const RET: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1100_0001_1111,
        value: 0b1101_0110_0101_1111_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Eret {}

impl Eret {
    pub fn exec(self, cpu: &mut Cpu, _: u64) {
        instruction_eret(cpu);
    }

    pub const fn decode(_: u32) -> Instruction {
        Instruction::Eret(Self {})
    }

    pub const ERET: InstDesc = InstDesc {
        mask: 0b1101_0110_1001_1111_0000_0011_1110_0000,
        value: 0b1101_0110_1001_1111_0000_0011_1110_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Bcond {
    pub imm19: u32,
    pub cond: u8,
}

impl Bcond {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let off = crate::utils::sign_extend(self.imm19 as u64, 19) << 2;
        instruction_branch(cpu, self.cond, off, old_pc);
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm19 = get_bits_ct!(word, 5, 19);
        let cond = get_bits_ct!(word, 0, 4) as u8;
        Instruction::Bcond(Self { imm19, cond })
    }

    pub const B_COND: InstDesc = InstDesc {
        mask: 0b1111_1111_0000_0000_0000_0000_0001_0000,
        value: 0b0101_0100_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Test bit and branch if nonzero
#[derive(Debug, Clone, Copy)]
pub struct Tbnz {
    pub b5: bool,
    pub b40: u8,
    pub imm14: u16,
    pub rt: u8,
}

impl Tbnz {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let datasize = if self.b5 { 64 } else { 32 };
        let bit_pos = ((self.b5 as u8) << 5) | self.b40;
        let offset = sign_extend(self.imm14 as u64, 14) << 2;
        instruction_tbnz(cpu, datasize, self.rt, bit_pos, offset, old_pc);
    }

    pub const fn decode(word: u32) -> Instruction {
        let b5 = get_bits_ct!(word, 31, 1) as u8 == 1;
        let b40 = get_bits_ct!(word, 19, 4) as u8;
        let imm14 = get_bits_ct!(word, 5, 14) as u16;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Tbnz(Self { b5, b40, imm14, rt })
    }
    pub const TBNZ: InstDesc = InstDesc {
        mask: 0b0111_1111_0000_0000_0000_0000_0000_0000,
        value: 0b0011_0111_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Test bit and branch if zero
#[derive(Debug, Clone, Copy)]
pub struct Tbz {
    pub b5: bool,
    pub b40: u8,
    pub imm14: u16,
    pub rt: u8,
}

impl Tbz {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let datasize = if self.b5 { 64 } else { 32 };
        let bit_pos = ((self.b5 as u8) << 5) | self.b40;
        let offset = sign_extend(self.imm14 as u64, 14) << 2;
        instruction_tbnz(cpu, datasize, self.rt, bit_pos, offset, old_pc);
    }

    pub const fn decode(word: u32) -> Instruction {
        let b5 = get_bits_ct!(word, 31, 1) as u8 == 1;
        let b40 = get_bits_ct!(word, 19, 4) as u8;
        let imm14 = get_bits_ct!(word, 5, 14) as u16;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Tbz(Self { b5, b40, imm14, rt })
    }
    pub const TBZ: InstDesc = InstDesc {
        mask: 0b0111_1111_0000_0000_0000_0000_0000_0000,
        value: 0b0011_0110_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Compare and branch on nonzero
#[derive(Debug, Clone, Copy)]
pub struct Cbnz {
    pub sf: bool,
    pub imm19: u32,
    pub rt: u8,
}

impl Cbnz {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let off = sign_extend(self.imm19 as u64, 19) << 2;
        let datasize = if self.sf { 64 } else { 32 };
        instruction_cbnz(cpu, datasize, self.rt, off, old_pc);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) as u8 == 1;
        let imm19 = get_bits_ct!(word, 5, 19);
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Cbnz(Self { sf, imm19, rt })
    }

    pub const CBNZ: InstDesc = InstDesc {
        mask: 0b0111_1111_0000_0000_0000_0000_0000_0000,
        value: 0b0011_0101_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Compare and branch on zero
#[derive(Debug, Clone, Copy)]
pub struct Cbz {
    pub sf: bool,
    pub imm19: u32,
    pub rt: u8,
}

impl Cbz {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let off = crate::utils::sign_extend(self.imm19 as u64, 19) << 2;
        let datasize = if self.sf { 64 } else { 32 };
        instruction_cbz(cpu, datasize, self.rt, off, old_pc);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) as u8 == 1;
        let imm19 = get_bits_ct!(word, 5, 19);
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Cbz(Self { sf, imm19, rt })
    }

    pub const CBZ: InstDesc = InstDesc {
        mask: 0b0111_1111_0000_0000_0000_0000_0000_0000,
        value: 0b0011_0100_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Branch with a link
#[derive(Debug, Clone, Copy)]
pub struct Bl {
    pub imm26: u32,
}

impl Bl {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        instruction_bl(cpu, (self.imm26 << 2).into(), old_pc);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm26 = get_bits_ct!(word, 0, 26);
        Instruction::Bl(Self { imm26 })
    }

    pub const BL: InstDesc = InstDesc {
        mask: 0b1111_1110_0000_0000_0000_0000_0000_0000,
        value: 0b1001_0100_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Branch {
    pub imm26: u32,
}

impl Branch {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        instruction_bunc(cpu, (self.imm26 as u64) * INSTRUCTION_SIZE, old_pc);
    }
    pub fn decode(word: u32) -> Instruction {
        let imm26 = get_bits_ct!(word, 0, 26);
        Instruction::Branch(Self { imm26 })
    }

    pub const BRANCH: InstDesc = InstDesc {
        mask: 0b1111_1100_0000_0000_0000_0000_0000_0000,
        value: 0b0001_0100_0000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct MsrImm {
    pub op1: u8,
    pub crm: u8,
    pub op2: u8,
}
impl MsrImm {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let min_el = match self.op1 & 0b00000111 {
            0b011 => ExceptionLevel::EL0,
            0b110 => ExceptionLevel::EL3,
            0b100 | 0b101 => ExceptionLevel::EL2,
            0b000 | 0b001 => ExceptionLevel::EL1,
            0b111 => ExceptionLevel::EL1,
            _ => panic!("Value not convered {}", self.op1 & 0b00000111),
        };
        let op1_op2 = (self.op1 << 3) | self.op2;
        let field = match op1_op2 {
            5 => PstateField::Sp,
            30 => PstateField::Daifset,
            31 => PstateField::Daifclr,
            _ => panic!("Value not covered {op1_op2}"),
        };
        instruction_msr_imm(cpu, self.crm, self.op1, self.crm, min_el, field);
    }

    pub const fn decode(word: u32) -> Instruction {
        let op1 = get_bits_ct!(word, 16, 3) as u8;
        let crm = get_bits_ct!(word, 8, 4) as u8;
        let op2 = get_bits_ct!(word, 5, 3) as u8;
        Instruction::MsrImm(Self { op1, crm, op2 })
    }

    pub const MSR_IMM: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1000_1111_0000_0001_1111,
        value: 0b1101_0101_0000_0000_0100_0000_0001_1111,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct MsrReg {
    pub op0: u8,
    pub op1: u8,
    pub crn: u8,
    pub crm: u8,
    pub op2: u8,
    pub rt: u8,
}

impl MsrReg {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        cpu.sys_reg_write(self.op0, self.op1, self.crn, self.crm, self.op2, self.rt);
    }
    pub const fn decode(word: u32) -> Instruction {
        let op0 = 2 + get_bits_ct!(word, 19, 1) as u8;
        let op1 = get_bits_ct!(word, 16, 3) as u8;
        let crn = get_bits_ct!(word, 12, 4) as u8;
        let crm = get_bits_ct!(word, 8, 4) as u8;
        let op2 = get_bits_ct!(word, 5, 3) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::MsrReg(Self { op0, op1, crn, crm, op2, rt })
    }
    pub const MSR_REG: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_0000_0000_0000_0000_0000,
        value: 0b1101_0101_0001_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Mrs {
    pub op0: u8,
    pub op1: u8,
    pub crn: u8,
    pub crm: u8,
    pub op2: u8,
    pub rt: u8,
}
impl Mrs {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        cpu.sys_reg_read(self.op0, self.op1, self.crn, self.crm, self.op2, self.rt);
    }
    pub const fn decode(word: u32) -> Instruction {
        let op0 = 2 + get_bits_ct!(word, 19, 1) as u8;
        let op1 = get_bits_ct!(word, 16, 3) as u8;
        let crn = get_bits_ct!(word, 12, 4) as u8;
        let crm = get_bits_ct!(word, 8, 4) as u8;
        let op2 = get_bits_ct!(word, 5, 3) as u8;
        let rt = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Mrs(Self { op0, op1, crn, crm, op2, rt })
    }

    pub const MRS: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_0000_0000_0000_0000_0000,
        value: 0b1101_0101_0011_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Wfi;

impl Wfi {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        cpu.sleeping.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Wfi(Self)
    }
    pub const WFI: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1111_1111_1111,
        value: 0b1101_0101_0000_0011_0010_0000_0111_1111,
        decode: Self::decode,
    };
}
