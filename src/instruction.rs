use crate::{
    branch_exc_sys_instr::{Bcond, Bl, Branch, Mrs, MsrImm, MsrReg, Ret},
    cpu::Cpu,
    imm_instr::{AddImmediate, Movz, SubImmediate, Subs},
    load_store_instr::{
        LdrImmPostIdx, LdrImmPreIdx, LdrImmUnOffset, LdrLit, LdrReg, StrImmUnOffset,
    },
    register_instr::{AddShiftedReg, Madd, SubShiftedRegister, Udiv},
};

type ExecFn = fn(&mut Cpu, Instruction);
type DecodeFn = fn(u32) -> Instruction;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Madd(Madd),
    AddShiftedReg(AddShiftedReg),
    SubShiftedRegister(SubShiftedRegister),
    Udiv(Udiv),
    AddImmediate(AddImmediate),
    Movz(Movz),
    Subs(Subs),
    SubImmediate(SubImmediate),
    Ret(Ret),
    Bcond(Bcond),
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
}

impl Instruction {
    pub fn exec(&self, cpu: &mut Cpu) {
        match self {
            Instruction::Madd(madd) => madd.exec(cpu),
            _ => (),
        }
    }
}

#[derive(Clone, Copy)]
pub struct InstDesc {
    pub mask: u32,
    pub value: u32,
    pub decode: DecodeFn,
    pub exec: ExecFn,
}

pub const DESCR: &[InstDesc] = &[
    Madd::MADD,
    AddShiftedReg::ADD_SHIFTED_REG,
    SubShiftedRegister::SUB_SHIFTED_REGISTER,
    Udiv::UDIV,
    AddImmediate::ADD_IMMEDIATE,
    Movz::MOVZ,
    Subs::SUBS,
    SubImmediate::SUB_IMMEDIATE,
    Ret::RET,
    Bcond::B_COND,
    Bl::BL,
    Branch::BRANCH,
    MsrImm::MRS_IMM,
    MsrReg::MRS_REG,
    Mrs::MRS,
    StrImmUnOffset::STR_IMM_UN_OFFSET,
    LdrImmUnOffset::LDR_IMM_UN_OFFSET,
    LdrImmPostIdx::LDR_IMM_POST_IDX,
    LdrImmPreIdx::LDR_IMM_PRE_IDX,
    LdrReg::LDR_REG,
    LdrLit::LDR_LIT,
];

#[derive(Clone, Copy)]
pub struct InstructionEntry {
    pub exec: ExecFn,
    pub decode: DecodeFn,
    pub specificity: u32,
}

pub fn exec_undef(_: &mut Cpu, _: Instruction) {
    panic!("Undefined instruction!");
}

pub const fn decode_undef(_: u32) -> Instruction {
    Instruction::Madd(Madd { sf: false, rd: 31, rn: 31, ra: 31, rm: 31 })
}

pub const UNDEFINED: InstructionEntry =
    InstructionEntry { exec: exec_undef, decode: decode_undef, specificity: 0 };
