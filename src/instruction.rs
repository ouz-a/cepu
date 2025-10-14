use std::sync::OnceLock;

use crate::{
    branch_exc_sys_instr::{
        Bcond, Bl, Blr, Branch, Bti, Cbnz, Cbz, Ccmpi, Csinc, Csinv, Dmb, Eret, Mrs, MsrImm,
        MsrReg, Ret, Tbnz, Tbz, Wfi,
    },
    cpu::Cpu,
    imm_instr::{AddImmediate, Movk, Movn, Movz, SubImmediate, Subs},
    load_store_instr::{
        LdpPostIndex, LdpSignedOffset, LdrImmPostIdx, LdrImmPreIdx, LdrImmUnOffset, LdrLit, LdrReg,
        LdrbUnsignedOff, Ldur, StpPreIndex, StpSignedOffset, StrImmUnOffset, StrRegister,
        StrbImmUnOffset, StrhUnsigned, Stur,
    },
    register_instr::{
        AddExtendedRegister, AddShiftedReg, AddsShiftedReg, Adrp, AndImmediate, AndShiftedRegister,
        AndsImmediate, AndsShiftedReg, Bfm, BicShiftedReg, BicShiftedRegSet, Clz, Csel, Dc, Dsb,
        Isb, Lslv, Lsrv, Madd, Nop, OrShiftedRegister, OrrImmediate, Rev, Sbfm, SubShiftedRegister,
        SubsShiftedReg, Tlbi, Ubfx, Udiv,
    },
};

const PRIME_SIZE: usize = 1 << 12;

macro_rules! define_instructions {
    ($($variant:ident($inner:ty)),* $(,)?) => {
        #[derive(Debug, Clone, Copy)]
        pub enum Instruction {
            $($variant($inner),)*
        }

        impl Instruction {
            pub fn exec(&self, cpu: &mut Cpu, old_pc: u64) {
                match self {
                    $(Self::$variant(i) => i.exec(cpu, old_pc),)*
                }
            }
        }
    };
}

define_instructions!(
    Tlbi(Tlbi),
    Isb(Isb),
    Nop(Nop),
    Dc(Dc),
    Dsb(Dsb),
    Madd(Madd),
    AddsShiftedReg(AddsShiftedReg),
    AddShiftedReg(AddShiftedReg),
    AddExtendedRegister(AddExtendedRegister),
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
    BicShiftedRegSet(BicShiftedRegSet),
    Rev(Rev),
    Udiv(Udiv),
    AddImmediate(AddImmediate),
    Movz(Movz),
    Movk(Movk),
    Movn(Movn),
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
    StrhUnsigned(StrhUnsigned),
    StrbImmUnOffset(StrbImmUnOffset),
    LdrImmUnOffset(LdrImmUnOffset),
    LdrImmPostIdx(LdrImmPostIdx),
    LdrImmPreIdx(LdrImmPreIdx),
    LdrReg(LdrReg),
    LdrLit(LdrLit),
    Ldur(Ldur),
    Stur(Stur),
    StpSignedOffset(StpSignedOffset),
    StpPreIndex(StpPreIndex),
    LdpPostIndex(LdpPostIndex),
    Wfi(Wfi),
    Dmb(Dmb),
    Bti(Bti),
    Sbfm(Sbfm),
    Clz(Clz),
    LdpSignedOffset(LdpSignedOffset),
    LdrbUnsignedOff(LdrbUnsignedOff),
    Csinv(Csinv),
    Csinc(Csinc),
    Blr(Blr),
);

type DecodeFn = fn(u32) -> Instruction;

#[derive(Clone, Copy)]
pub struct InstDesc {
    pub mask: u32,
    pub value: u32,
    pub decode: DecodeFn,
}

static TABLES: OnceLock<Tables> = OnceLock::new();

pub const DESCR: &[InstDesc] = &sort_by_specificity([
    Tlbi::TLBI,
    Isb::ISB,
    Nop::NOP,
    Dc::DC,
    Dsb::DSB,
    Madd::MADD,
    AddsShiftedReg::ADDS_SHIFTED_REG,
    AddShiftedReg::ADD_SHIFTED_REG,
    AddExtendedRegister::ADD_EXTENDED_REGISTER,
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
    BicShiftedRegSet::BIC_SHIFTED_REG_SET,
    Udiv::UDIV,
    AddImmediate::ADD_IMMEDIATE,
    Movz::MOVZ,
    Movk::MOVK,
    Movn::MOVN,
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
    StrhUnsigned::STRH_UNSIGNED,
    StrbImmUnOffset::STRB_IMM_UN_OFFSET,
    LdrImmUnOffset::LDR_IMM_UN_OFFSET,
    LdrImmPostIdx::LDR_IMM_POST_IDX,
    LdrImmPreIdx::LDR_IMM_PRE_IDX,
    LdrReg::LDR_REG,
    LdrLit::LDR_LIT,
    StpSignedOffset::STP_SIGNED_OFFSET,
    StpPreIndex::STP_PRE_INDEX,
    LdpPostIndex::LDP_POST_INDEX,
    LdpSignedOffset::LDP_SIGNED_OFFSET,
    Wfi::WFI,
    Dmb::DMB,
    Bti::BTI,
    Ubfx::UBFX,
    Bfm::BFM,
    Sbfm::SBFM,
    Lslv::LSLV,
    Lsrv::LSRV,
    Clz::CLZ,
    Rev::REV,
    Ldur::LDUR,
    Stur::STUR,
    LdrbUnsignedOff::LDRB_UNSIGNED_OFF,
    Csinv::CSINV,
    Csinc::CSINC,
    Blr::BLR,
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
