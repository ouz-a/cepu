use std::sync::OnceLock;

use crate::{
    branch_exc_sys_instr::{
        Bcond, Bl, Branch, Bti, Cbnz, Cbz, Ccmpi, Dmb, Eret, Mrs, MsrImm, MsrReg, Ret, Tbnz, Tbz,
        Wfi,
    },
    cpu::Cpu,
    imm_instr::{AddImmediate, Movk, Movz, SubImmediate, Subs},
    load_store_instr::{
        LdrImmPostIdx, LdrImmPreIdx, LdrImmUnOffset, LdrLit, LdrReg, StpSignedOffset,
        StrImmUnOffset, StrRegister, StrbImmUnOffset,
    },
    register_instr::{
        AddShiftedReg, AddsShiftedReg, Adrp, AndImmediate, AndShiftedRegister, AndsImmediate,
        AndsShiftedReg, Bfm, BicShiftedReg, Csel, Dc, Dsb, Isb, Lslv, Lsrv, Madd, Nop,
        OrShiftedRegister, OrrImmediate, SubShiftedRegister, SubsShiftedReg, Ubfx, Udf, Udiv,
    },
};

const PRIME_SIZE: usize = 1 << 12;

static TABLES: OnceLock<Tables> = OnceLock::new();

type DecodeFn = fn(u32) -> Instruction;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Udf(Udf),
    Isb(Isb),
    Nop(Nop),
    Dc(Dc),
    Dsb(Dsb),
    Madd(Madd),
    AddsShiftedReg(AddsShiftedReg),
    AddShiftedReg(AddShiftedReg),
    SubShiftedRegister(SubShiftedRegister),
    SubsShiftedReg(SubsShiftedReg),
    AndShiftedRegister(AndShiftedRegister),
    AndsImmediate(AndsImmediate),
    OrrImmediate(OrrImmediate),
    AndsShiftedReg(AndsShiftedReg),
    AndImmediate(AndImmediate),
    Lslv(Lslv),
    Lsrv(Lsrv),
    Csel(Csel),
    Adrp(Adrp),
    Ubfx(Ubfx),
    Bfm(Bfm),
    OrShiftedRegister(OrShiftedRegister),
    BicShiftedReg(BicShiftedReg),
    Udiv(Udiv),
    AddImmediate(AddImmediate),
    Movz(Movz),
    Movk(Movk),
    Subs(Subs),
    SubImmediate(SubImmediate),
    Ret(Ret),
    Eret(Eret),
    Bcond(Bcond),
    Tbnz(Tbnz),
    Tbz(Tbz),
    Cbnz(Cbnz),
    Cbz(Cbz),
    Ccmpi(Ccmpi),
    Bl(Bl),
    Branch(Branch),
    MsrImm(MsrImm),
    MsrReg(MsrReg),
    Mrs(Mrs),
    StrImmUnOffset(StrImmUnOffset),
    StrRegister(StrRegister),
    StrbImmUnOffset(StrbImmUnOffset),
    LdrImmUnOffset(LdrImmUnOffset),
    LdrImmPostIdx(LdrImmPostIdx),
    LdrImmPreIdx(LdrImmPreIdx),
    LdrReg(LdrReg),
    LdrLit(LdrLit),
    StpSignedOffset(StpSignedOffset),
    Wfi(Wfi),
    Dmb(Dmb),
    Bti(Bti),
}

impl Instruction {
    pub fn exec(&self, cpu: &mut Cpu, old_pc: u64) {
        match self {
            Self::Udf(i) => i.exec(cpu, old_pc),
            Self::Nop(i) => i.exec(cpu, old_pc),
            Self::Isb(i) => i.exec(cpu, old_pc),
            Self::Dc(i) => i.exec(cpu, old_pc),
            Self::Dsb(i) => i.exec(cpu, old_pc),
            Self::Madd(i) => i.exec(cpu, old_pc),
            Self::AddsShiftedReg(i) => i.exec(cpu, old_pc),
            Self::AddShiftedReg(i) => i.exec(cpu, old_pc),
            Self::SubShiftedRegister(i) => i.exec(cpu, old_pc),
            Self::SubsShiftedReg(i) => i.exec(cpu, old_pc),
            Self::AndShiftedRegister(i) => i.exec(cpu, old_pc),
            Self::AndsImmediate(i) => i.exec(cpu, old_pc),
            Self::OrrImmediate(i) => i.exec(cpu, old_pc),
            Self::AndsShiftedReg(i) => i.exec(cpu, old_pc),
            Self::AndImmediate(i) => i.exec(cpu, old_pc),
            Self::Csel(i) => i.exec(cpu, old_pc),
            Self::Adrp(i) => i.exec(cpu, old_pc),
            Self::Ubfx(i) => i.exec(cpu, old_pc),
            Self::Bfm(i) => i.exec(cpu, old_pc),
            Self::Lslv(i) => i.exec(cpu, old_pc),
            Self::Lsrv(i) => i.exec(cpu, old_pc),
            Self::OrShiftedRegister(i) => i.exec(cpu, old_pc),
            Self::BicShiftedReg(i) => i.exec(cpu, old_pc),
            Self::Udiv(i) => i.exec(cpu, old_pc),
            Self::AddImmediate(i) => i.exec(cpu, old_pc),
            Self::Movz(i) => i.exec(cpu, old_pc),
            Self::Movk(i) => i.exec(cpu, old_pc),
            Self::Subs(i) => i.exec(cpu, old_pc),
            Self::SubImmediate(i) => i.exec(cpu, old_pc),
            Self::Ret(i) => i.exec(cpu, old_pc),
            Self::Eret(i) => i.exec(cpu, old_pc),
            Self::Bcond(i) => i.exec(cpu, old_pc),
            Self::Cbnz(i) => i.exec(cpu, old_pc),
            Self::Ccmpi(i) => i.exec(cpu, old_pc),
            Self::Tbnz(i) => i.exec(cpu, old_pc),
            Self::Tbz(i) => i.exec(cpu, old_pc),
            Self::Cbz(i) => i.exec(cpu, old_pc),
            Self::Bl(i) => i.exec(cpu, old_pc),
            Self::Branch(i) => i.exec(cpu, old_pc),
            Self::MsrImm(i) => i.exec(cpu, old_pc),
            Self::MsrReg(i) => i.exec(cpu, old_pc),
            Self::Mrs(i) => i.exec(cpu, old_pc),
            Self::StrImmUnOffset(i) => i.exec(cpu, old_pc),
            Self::StrRegister(i) => i.exec(cpu, old_pc),
            Self::StrbImmUnOffset(i) => i.exec(cpu, old_pc),
            Self::LdrImmUnOffset(i) => i.exec(cpu, old_pc),
            Self::LdrImmPostIdx(i) => i.exec(cpu, old_pc),
            Self::LdrImmPreIdx(i) => i.exec(cpu, old_pc),
            Self::LdrReg(i) => i.exec(cpu, old_pc),
            Self::LdrLit(i) => i.exec(cpu, old_pc),
            Self::StpSignedOffset(i) => i.exec(cpu, old_pc),
            Self::Wfi(i) => i.exec(cpu, old_pc),
            Self::Dmb(i) => i.exec(cpu, old_pc),
            Self::Bti(i) => i.exec(cpu, old_pc),
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
    Udf::UDF,
    Isb::ISB,
    Nop::NOP,
    Dc::DC,
    Dsb::DSB,
    Madd::MADD,
    AddsShiftedReg::ADDS_SHIFTED_REG,
    AddShiftedReg::ADD_SHIFTED_REG,
    SubShiftedRegister::SUB_SHIFTED_REGISTER,
    SubsShiftedReg::SUBS_SHIFTED_REG,
    AndShiftedRegister::AND_SHIFTED_REGISTER,
    AndsImmediate::ANDS_IMMEDIATE,
    OrrImmediate::ORR_IMMEDIATE,
    AndsShiftedReg::ANDS_SHIFTED_REG,
    AndImmediate::AND_IMMEDIATE,
    Csel::CSEL,
    Adrp::ADRP,
    OrShiftedRegister::OR_SHIFTED_REGISTER,
    BicShiftedReg::BIC_SHIFTED_REG,
    Udiv::UDIV,
    AddImmediate::ADD_IMMEDIATE,
    Movz::MOVZ,
    Movk::MOVK,
    Subs::SUBS,
    SubImmediate::SUB_IMMEDIATE,
    Ret::RET,
    Eret::ERET,
    Bcond::B_COND,
    Tbnz::TBNZ,
    Tbz::TBZ,
    Cbnz::CBNZ,
    Cbz::CBZ,
    Ccmpi::CCMPI,
    Bl::BL,
    Branch::BRANCH,
    MsrImm::MSR_IMM,
    MsrReg::MSR_REG,
    Mrs::MRS,
    StrImmUnOffset::STR_IMM_UN_OFFSET,
    StrRegister::STR_REGISTER,
    StrbImmUnOffset::STRB_IMM_UN_OFFSET,
    LdrImmUnOffset::LDR_IMM_UN_OFFSET,
    LdrImmPostIdx::LDR_IMM_POST_IDX,
    LdrImmPreIdx::LDR_IMM_PRE_IDX,
    LdrReg::LDR_REG,
    LdrLit::LDR_LIT,
    StpSignedOffset::STP_SIGNED_OFFSET,
    Wfi::WFI,
    Dmb::DMB,
    Bti::BTI,
    Ubfx::UBFX,
    Bfm::BFM,
    Lslv::LSLV,
    Lsrv::LSRV,
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
Binary form: {:032b}",
        word.to_be(),
        word
    );
    panic!("{formatted}");
}

pub fn decode_undef(_: u32) -> Instruction {
    panic!("Undefined instruction")
}

pub const UNDEF_DESC: InstDesc = InstDesc { mask: 0, value: 0, decode: decode_undef };
