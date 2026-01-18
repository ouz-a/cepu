use std::ops::Not;

use crate::{
    branch::{
        branch_to, condition_holds, instruction_bl, instruction_branch, instruction_bunc,
        instruction_cbnz, instruction_cbz, instruction_ccmpi, instruction_eret,
        instruction_msr_imm, instruction_ret, instruction_smc, instruction_tbnz, instruction_tbz,
    },
    cpu::{Cpu, ExceptionLevel, PstateField},
    get_bits_ct,
    instruction::{InstDesc, Instruction},
    mmu::AccessType,
    utils::{bits_get, sign_extend, zero_extend},
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
        mask: 0b1111_1111_1111_1111_1111_1111_1111_1111,
        value: 0b1101_0110_1001_1111_0000_0011_1110_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Smc {
    pub imm: u16,
}

impl Smc {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        instruction_smc(cpu);
    }

    pub const fn decode(word: u32) -> Instruction {
        Instruction::Smc(Self { imm: get_bits_ct!(word, 5, 16) as u16 })
    }

    pub const SMC: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_0000_0001_1111,
        value: 0b1101_0100_0000_0000_0000_0000_0000_0011,
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
        let b40 = get_bits_ct!(word, 19, 5) as u8;
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
        instruction_tbz(cpu, datasize, self.rt, bit_pos, offset, old_pc);
    }

    pub const fn decode(word: u32) -> Instruction {
        let b5 = get_bits_ct!(word, 31, 1) as u8 == 1;
        let b40 = get_bits_ct!(word, 19, 5) as u8;
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

/// Conditional compare (immediate)
#[derive(Debug, Clone, Copy)]
pub struct Ccmpi {
    pub sf: bool,
    pub imm5: u8,
    pub cond: u8,
    pub rn: u8,
    pub nzcv: u8,
}

impl Ccmpi {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let datasize = if self.sf { 64 } else { 32 };
        let imm = zero_extend(self.imm5 as u64, datasize);
        instruction_ccmpi(cpu, datasize, self.rn, imm, self.cond, self.nzcv);
    }

    pub fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) as u8 == 1;
        let imm5 = get_bits_ct!(word, 16, 5) as u8;
        let cond = get_bits_ct!(word, 12, 4) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let nzcv = get_bits_ct!(word, 0, 4) as u8;
        Instruction::Ccmpi(Self { sf, imm5, cond, rn, nzcv })
    }

    pub const CCMPI: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_0000_1100_0001_0000,
        value: 0b0111_1010_0100_0000_0000_1000_0000_0000,
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
        let off = crate::utils::sign_extend(self.imm26 as u64, 26) << 2;
        instruction_bl(cpu, off, old_pc);
    }
    pub const fn decode(word: u32) -> Instruction {
        let imm26 = get_bits_ct!(word, 0, 26);
        Instruction::Bl(Self { imm26 })
    }

    pub const BL: InstDesc = InstDesc {
        mask: 0b1111_1100_0000_0000_0000_0000_0000_0000,
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
        let off = crate::utils::sign_extend(self.imm26 as u64, 26) << 2;
        instruction_bunc(cpu, off, old_pc);
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
pub struct Br {
    pub rn: u8,
}

impl Br {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let target = cpu.x_read(self.rn.into(), 64);
        branch_to(cpu, target, false, old_pc);
    }
    pub fn decode(word: u32) -> Instruction {
        let rn = get_bits_ct!(word, 5, 5) as u8;
        Instruction::Br(Self { rn })
    }

    pub const BR: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1100_0001_1111,
        value: 0b1101_0110_0001_1111_0000_0000_0000_0000,
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
pub struct Sevl;
impl Sevl {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        cpu.event_register = true;
    }
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Sevl(Self)
    }
    pub const SEVL: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1111_1111_1111,
        value: 0b1101_0101_0000_0011_0010_0000_1011_1111,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Wfe;
impl Wfe {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        cpu.event_register = true;
    }
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Wfe(Wfe)
    }
    pub const WFE: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1111_1111_1111,
        value: 0b1101_0101_0000_0011_0010_0000_0101_1111,
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

/// YIELD - Yield hint
#[derive(Debug, Clone, Copy)]
pub struct Yield;

impl Yield {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Yield(Self)
    }
    pub const YIELD: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1111_1111_1111,
        value: 0b1101_0101_0000_0011_0010_0000_0011_1111,
        decode: Self::decode,
    };
}

/// XPACLRI - Strip Pointer Authentication Code
#[derive(Debug, Clone, Copy)]
pub struct Xpaclri;

impl Xpaclri {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}
    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Xpaclri(Self)
    }
    pub const XPACLRI: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1111_1111_1111,
        value: 0b1101_0101_0000_0011_0010_0000_1111_1111,
        decode: Self::decode,
    };
}

/// Data memory barrier
#[derive(Debug, Clone, Copy)]
pub struct Dmb;

impl Dmb {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}

    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Dmb(Self)
    }

    pub const DMB: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_0000_1111_1111,
        value: 0b1101_0101_0000_0011_0011_0000_1011_1111,
        decode: Self::decode,
    };
}

/// Data memory barrier
#[derive(Debug, Clone, Copy)]
pub struct Csdb;

impl Csdb {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}

    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Csdb(Self)
    }

    pub const CSDB: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1111_1111_1111,
        value: 0b1101_0101_0000_0011_0010_0010_1001_1111,
        decode: Self::decode,
    };
}

/// Branch target identification
#[derive(Debug, Clone, Copy)]
pub struct Bti;

impl Bti {
    pub fn exec(self, _cpu: &mut Cpu, _old_pc: u64) {}

    pub const fn decode(_word: u32) -> Instruction {
        Instruction::Bti(Self)
    }

    pub const BTI: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1111_0011_1111,
        value: 0b1101_0101_0000_0011_0010_0100_0001_1111,
        decode: Self::decode,
    };
}

/// Conditional select invert
#[derive(Debug, Clone, Copy)]
pub struct Csinv {
    pub sf: bool,
    pub rm: u8,
    pub cond: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Csinv {
    pub fn exec(self, cpu: &mut Cpu, _: u64) {
        let datasize = if self.sf { 64 } else { 32 };
        let result = if condition_holds(cpu, self.cond) {
            cpu.x_read(self.rn.into(), datasize)
        } else {
            let not = cpu.x_read(self.rm.into(), datasize).not();
            bits_get(not, 0, datasize)
        };
        cpu.x_write(self.rd.into(), result, !self.sf);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let cond = get_bits_ct!(word, 12, 4) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Csinv(Self { sf, rm, cond, rn, rd })
    }
    pub const CSINV: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0101_1010_1000_0000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

/// Conditional select increment
#[derive(Debug, Clone, Copy)]
pub struct Csinc {
    pub sf: bool,
    pub rm: u8,
    pub cond: u8,
    pub rn: u8,
    pub rd: u8,
}

impl Csinc {
    pub fn exec(self, cpu: &mut Cpu, _: u64) {
        let datasize = if self.sf { 64 } else { 32 };
        let result = if condition_holds(cpu, self.cond) {
            cpu.x_read(self.rn.into(), datasize)
        } else {
            cpu.x_read(self.rm.into(), datasize) + 1
        };
        cpu.x_write(self.rd.into(), result, !self.sf);
    }

    pub const fn decode(word: u32) -> Instruction {
        let sf = get_bits_ct!(word, 31, 1) == 1;
        let rm = get_bits_ct!(word, 16, 5) as u8;
        let cond = get_bits_ct!(word, 12, 4) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::Csinc(Self { sf, rm, cond, rn, rd })
    }
    pub const CSINC: InstDesc = InstDesc {
        mask: 0b0111_1111_1110_0000_0000_1100_0000_0000,
        value: 0b0001_1010_1000_0000_0000_0100_0000_0000,
        decode: Self::decode,
    };
}

/// Branch with link to register
#[derive(Debug, Clone, Copy)]
pub struct Blr {
    rn: u8,
}

impl Blr {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        let target = cpu.x_read(self.rn.into(), 64);
        cpu.x_write(30, old_pc.wrapping_add(4), false);
        branch_to(cpu, target, false, old_pc);
    }

    pub const fn decode(word: u32) -> Instruction {
        let rn = get_bits_ct!(word, 5, 5) as u8;
        Instruction::Blr(Self { rn })
    }

    pub const BLR: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1111_1111_1100_0001_1111,
        value: 0b1101_0110_0011_1111_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

// TODO: SYS requires organization
#[derive(Clone, Copy, Debug)]
pub struct Sys {
    pub word: u32,
}

impl Sys {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        let word = self.word;
        let dc_mask = 0b1111_1111_1111_1000_1111_0000_0000_0000;
        let dc_value = 0b1101_0101_0000_1000_0111_0000_0000_0000;
        let at_mask = 0b1111_1111_1111_1000_1111_1110_0000_0000;
        let at_value = 0b1101_0101_0000_1000_0111_1000_0000_0000;
        let tlbi_mask: u32 = 0b1111_1111_1111_1000_1110_0000_0000_0000;
        let tlbi_value: u32 = 0b1101_0101_0000_1000_1000_0000_0000_0000;

        if (word & tlbi_mask) == tlbi_value {
        } else if (word & at_mask) == at_value {
            let op1 = get_bits_ct!(word, 16, 3);
            let crn = get_bits_ct!(word, 12, 4);
            let crm = get_bits_ct!(word, 8, 4);
            let op2 = get_bits_ct!(word, 5, 3);
            let pattern = ((op1 << 11) | (crn << 7) | (crm << 3) | op2) as u16;

            let at_op = compute_sys_op_at(pattern);

            let rt = get_bits_ct!(word, 0, 5) as u8;
            let va = cpu.x_read(rt.into(), 64);

            if matches!(at_op.stage, TranslationStage::Stage12) {
                panic!("Stage 2 translation not implemented");
            }

            let prev_faulted = cpu.mmu.faulted;
            let prev_fault_va = cpu.mmu.fault_va;
            let prev_fault_level = cpu.mmu.fault_level;

            cpu.mmu.faulted = false;
            // TODO: This is totally wrong please fix it
            let pa = cpu.mmu.page_walk(va as usize, AccessType::Read, cpu.pstate.current_el);
            let at_faulted = cpu.mmu.faulted;
            let at_fault_level = cpu.mmu.fault_level;

            if at_faulted {
                let level = match at_fault_level {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    _ => 3,
                } as u64;
                let fst = 0b00_0100_u64 | level;
                cpu.par_el1 = 0;
                cpu.par_el1 |= 1; // F
                cpu.par_el1 |= fst << 1; // FS
                cpu.par_el1 |= 1u64 << 11; // RES1 for 64-bit PAR
            } else {
                let mut par: u64 = 0;
                par |= (pa as u64) & !0xFFFu64;
                par |= 1u64 << 11;
                cpu.par_el1 = par;
            }

            cpu.mmu.faulted = prev_faulted;
            cpu.mmu.fault_va = prev_fault_va;
            cpu.mmu.fault_level = prev_fault_level;
        } else if (word & dc_mask) == dc_value {
            // NO OP
        } else {
            panic!("Sys instruction not covered")
        }
    }

    pub const fn decode(word: u32) -> Instruction {
        Instruction::Sys(Self { word })
    }

    pub const SYS: InstDesc = InstDesc {
        mask: 0b1111_1111_1111_1000_0000_0000_0000_0000,
        value: 0b1101_0101_0000_1000_0000_0000_0000_0000,
        decode: Self::decode,
    };
}

#[derive(Debug, Clone, Copy)]
pub enum AtaAccess {
    Read,
    Write,
    Any,
    ReadPan,
    WritePan,
}

#[derive(Debug, Clone, Copy)]
pub enum TranslationStage {
    Stage1,
    Stage12,
}

#[derive(Debug, Clone, Copy)]
pub struct SysOpAt {
    pub access: AtaAccess,
    pub stage: TranslationStage,
    pub el: ExceptionLevel,
}

fn compute_sys_op_at(pat: u16) -> SysOpAt {
    let access;
    let stage;
    let el;
    match pat {
        // S1E1R
        960 => {
            access = AtaAccess::Read;
            el = ExceptionLevel::EL1;
            stage = TranslationStage::Stage1;
        }
        // S1E1W
        961 => {
            access = AtaAccess::Write;
            el = ExceptionLevel::EL1;
            stage = TranslationStage::Stage1;
        }
        // S1E0R
        962 => {
            access = AtaAccess::Read;
            el = ExceptionLevel::EL0;
            stage = TranslationStage::Stage1;
        }
        // S1E0W
        963 => {
            access = AtaAccess::Write;
            el = ExceptionLevel::EL0;
            stage = TranslationStage::Stage1;
        }
        // S1E1RP
        968 => {
            access = AtaAccess::ReadPan;
            el = ExceptionLevel::EL1;
            stage = TranslationStage::Stage1;
        }
        // S1E1WP
        969 => {
            access = AtaAccess::WritePan;
            el = ExceptionLevel::EL1;
            stage = TranslationStage::Stage1;
        }
        // S1E1A
        970 => {
            access = AtaAccess::Any;
            el = ExceptionLevel::EL1;
            stage = TranslationStage::Stage1;
        }
        // S1E2R
        9152 => {
            access = AtaAccess::Read;
            el = ExceptionLevel::EL2;
            stage = TranslationStage::Stage1;
        }
        // S1E2W
        9153 => {
            access = AtaAccess::Write;
            el = ExceptionLevel::EL2;
            stage = TranslationStage::Stage1;
        }
        // S1E2A
        9162 => {
            access = AtaAccess::Any;
            el = ExceptionLevel::EL2;
            stage = TranslationStage::Stage1;
        }
        // S12E1R
        9156 => {
            access = AtaAccess::Read;
            el = ExceptionLevel::EL1;
            stage = TranslationStage::Stage12;
        }
        // S12E1W
        9157 => {
            access = AtaAccess::Write;
            el = ExceptionLevel::EL1;
            stage = TranslationStage::Stage12;
        }
        // S12E0R
        9158 => {
            access = AtaAccess::Read;
            el = ExceptionLevel::EL0;
            stage = TranslationStage::Stage12;
        }
        // S12E0W
        9159 => {
            access = AtaAccess::Write;
            el = ExceptionLevel::EL0;
            stage = TranslationStage::Stage12;
        }
        // S1E3R
        13248 => {
            access = AtaAccess::Read;
            el = ExceptionLevel::EL3;
            stage = TranslationStage::Stage1;
        }
        // S1E3W
        13249 => {
            access = AtaAccess::Write;
            el = ExceptionLevel::EL3;
            stage = TranslationStage::Stage1;
        }
        // S1E3A
        13258 => {
            access = AtaAccess::Any;
            el = ExceptionLevel::EL3;
            stage = TranslationStage::Stage1;
        }
        _ => panic!("Wrong SysOP for AT: {pat}"),
    }
    SysOpAt { access, stage, el }
}

/// Supervisor call
#[derive(Debug, Clone, Copy)]
pub struct Svc {
    pub imm16: u16,
}

impl Svc {
    pub fn exec(self, cpu: &mut Cpu, old_pc: u64) {
        // Clear exclusive monitor on exception entry.
        cpu.monitor.off();
        cpu.elr_el1 = old_pc.wrapping_add(4);

        cpu.spsr_el1 = cpu.spsr_from_pstate();

        // 0x15 -> SVC
        cpu.esr_el1 = (0x15 << 26) | (1 << 25) | (self.imm16 as u64);

        let offset = if cpu.pstate.current_el == ExceptionLevel::EL0 {
            0x400
        } else if cpu.pstate.sp == 1 {
            0x200
        } else {
            0x000
        };

        cpu.pstate.daif_disable();

        cpu.pstate.current_el = ExceptionLevel::EL1;
        cpu.pstate.sp = 1;
        cpu.branch_taken = true;
        cpu.branch_target = cpu.vbar_el1 + offset;
    }

    pub const fn decode(word: u32) -> Instruction {
        let imm16 = get_bits_ct!(word, 5, 16) as u16;
        Instruction::Svc(Self { imm16 })
    }

    pub const SVC: InstDesc = InstDesc {
        mask: 0b1111_1111_1110_0000_0000_0000_0001_1111,
        value: 0b1101_0100_0000_0000_0000_0000_0000_0001,
        decode: Self::decode,
    };
}
