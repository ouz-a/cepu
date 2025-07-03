use crate::{
    cpu::Cpu,
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

pub const DESCR: &[InstDesc] = &[Madd::MADD];

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
