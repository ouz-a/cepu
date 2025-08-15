// src/instruction.rs

use std::sync::OnceLock;

use crate::{
    branch_exc_sys_instr::{Bcond, Bl, Branch, Cbnz, Cbz, Eret, Mrs, MsrImm, MsrReg, Ret, Wfi},
    cpu::Cpu,
    imm_instr::{AddImmediate, Movk, Movz, SubImmediate, Subs},
    load_store_instr::{
        LdrImmPostIdx, LdrImmPreIdx, LdrImmUnOffset, LdrLit, LdrReg, StrImmUnOffset,
    },
    register_instr::{
        AddShiftedReg, AddsShiftedReg, AndShiftedRegister, Madd, OrShiftedRegister,
        SubShiftedRegister, Udiv,
    },
};

const PRIME_SIZE: usize = 1 << 12;

static TABLES: OnceLock<Tables> = OnceLock::new();

type DecodeFn = fn(u32) -> Instruction;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Madd(Madd),
    AddsShiftedReg(AddsShiftedReg),
    AddShiftedReg(AddShiftedReg),
    SubShiftedRegister(SubShiftedRegister),
    AndShiftedRegister(AndShiftedRegister),
    OrShiftedRegister(OrShiftedRegister),
    Udiv(Udiv),
    AddImmediate(AddImmediate),
    Movz(Movz),
    Movk(Movk),
    Subs(Subs),
    SubImmediate(SubImmediate),
    Ret(Ret),
    Eret(Eret),
    Bcond(Bcond),
    Cbnz(Cbnz),
    Cbz(Cbz),
    Bl(Bl),
    Branch(Branch),
    MsrImm(MsrImm),
    MsrReg(MsrReg),
    Mrs(Mrs),
    StrImmUnOffset(StrImmUnOffset),
    LdrImmUnOffset(LdrImmUnOffset),
    LdrImmPostIdx(LdrImmPostIdx),
    LdrImmPreIdx(LdrImmPreIdx),
    LdrReg(LdrReg),
    LdrLit(LdrLit),
    Wfi(Wfi),
}

impl Instruction {
    pub fn exec(&self, cpu: &mut Cpu, old_pc: u64) {
        match self {
            Instruction::Madd(i) => i.exec(cpu, old_pc),
            Instruction::AddsShiftedReg(i) => i.exec(cpu, old_pc),
            Instruction::AddShiftedReg(i) => i.exec(cpu, old_pc),
            Instruction::SubShiftedRegister(i) => i.exec(cpu, old_pc),
            Instruction::AndShiftedRegister(i) => i.exec(cpu, old_pc),
            Instruction::OrShiftedRegister(i) => i.exec(cpu, old_pc),
            Instruction::Udiv(i) => i.exec(cpu, old_pc),
            Instruction::AddImmediate(i) => i.exec(cpu, old_pc),
            Instruction::Movz(i) => i.exec(cpu, old_pc),
            Instruction::Movk(i) => i.exec(cpu, old_pc),
            Instruction::Subs(i) => i.exec(cpu, old_pc),
            Instruction::SubImmediate(i) => i.exec(cpu, old_pc),
            Instruction::Ret(i) => i.exec(cpu, old_pc),
            Instruction::Eret(i) => i.exec(cpu, old_pc),
            Instruction::Bcond(i) => i.exec(cpu, old_pc),
            Instruction::Cbnz(i) => i.exec(cpu, old_pc),
            Instruction::Cbz(i) => i.exec(cpu, old_pc),
            Instruction::Bl(i) => i.exec(cpu, old_pc),
            Instruction::Branch(i) => i.exec(cpu, old_pc),
            Instruction::MsrImm(i) => i.exec(cpu, old_pc),
            Instruction::MsrReg(i) => i.exec(cpu, old_pc),
            Instruction::Mrs(i) => i.exec(cpu, old_pc),
            Instruction::StrImmUnOffset(i) => i.exec(cpu, old_pc),
            Instruction::LdrImmUnOffset(i) => i.exec(cpu, old_pc),
            Instruction::LdrImmPostIdx(i) => i.exec(cpu, old_pc),
            Instruction::LdrImmPreIdx(i) => i.exec(cpu, old_pc),
            Instruction::LdrReg(i) => i.exec(cpu, old_pc),
            Instruction::LdrLit(i) => i.exec(cpu, old_pc),
            Instruction::Wfi(i) => i.exec(cpu, old_pc),
        }
    }
}

#[derive(Clone, Copy)]
pub struct InstDesc {
    pub mask: u32,
    pub value: u32,
    pub decode: DecodeFn,
}

pub const DESCR: &[InstDesc] = &sort_by_specificity([
    Madd::MADD,
    AddsShiftedReg::ADDS_SHIFTED_REG,
    AddShiftedReg::ADD_SHIFTED_REG,
    SubShiftedRegister::SUB_SHIFTED_REGISTER,
    AndShiftedRegister::AND_SHIFTED_REGISTER,
    OrShiftedRegister::OR_SHIFTED_REGISTER,
    Udiv::UDIV,
    AddImmediate::ADD_IMMEDIATE,
    Movz::MOVZ,
    Movk::MOVK,
    Subs::SUBS,
    SubImmediate::SUB_IMMEDIATE,
    Ret::RET,
    Eret::ERET,
    Bcond::B_COND,
    Cbnz::CBNZ,
    Cbz::CBZ,
    Bl::BL,
    Branch::BRANCH,
    MsrImm::MSR_IMM,
    MsrReg::MSR_REG,
    Mrs::MRS,
    StrImmUnOffset::STR_IMM_UN_OFFSET,
    LdrImmUnOffset::LDR_IMM_UN_OFFSET,
    LdrImmPostIdx::LDR_IMM_POST_IDX,
    LdrImmPreIdx::LDR_IMM_PRE_IDX,
    LdrReg::LDR_REG,
    LdrLit::LDR_LIT,
    Wfi::WFI,
]);

const fn sort_by_specificity<const N: usize>(mut arr: [InstDesc; N]) -> [InstDesc; N] {
    let mut i = 1;
    while i < N {
        let key = arr[i];
        let bits = key.mask.count_ones();
        let mut j = i;
        while j > 0 && arr[j - 1].mask.count_ones() < bits {
            arr[j] = arr[j - 1];
            j -= 1;
        }
        arr[j] = key;
        i += 1;
    }
    arr
}

#[derive(Clone, Copy)]
struct Bucket {
    first: u32,
    count: u16,
}

#[derive(Clone)]
pub struct Tables {
    primary: [Bucket; PRIME_SIZE],
    dst: Vec<InstDesc>,
}

fn build_tables_runtime() -> Tables {
    let mut primary = [Bucket { first: 0, count: 0 }; PRIME_SIZE];
    let mut dst = Vec::new();

    for (key, item) in primary.iter_mut().enumerate().take(PRIME_SIZE) {
        item.first = dst.len() as u32;
        for &d in DESCR.iter() {
            let v12 = (d.value >> 20) as u16;
            let m12 = (d.mask >> 20) as u16;
            if ((key as u16) & m12) == v12 {
                dst.push(d);
                item.count += 1;
            }
        }
    }

    Tables { primary, dst }
}

#[inline(always)]
pub fn decode(word: u32) -> Instruction {
    let tables = TABLES.get_or_init(build_tables_runtime);
    let key = (word >> 20) as usize;
    let b = tables.primary[key];
    let mut i = b.first as usize;
    let end = i + b.count as usize;

    while i < end {
        let d = tables.dst[i];
        if (word & d.mask) == d.value {
            return (d.decode)(word);
        }
        i += 1;
    }
    let formatted = format!(
        "Undefined instruction: {:08X}
Binary form: {:b}",
        word.to_be(),
        word
    );
    panic!("{formatted}");
}

pub fn decode_undef(_: u32) -> Instruction {
    panic!("Undefined instruction")
}

pub const UNDEF_DESC: InstDesc = InstDesc { mask: 0, value: 0, decode: decode_undef };
