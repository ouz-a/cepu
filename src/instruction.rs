use std::sync::OnceLock;

use crate::{
    branch_exc_sys_instr::{
        Bcond, Bl, Blr, Br, Branch, Bti, Cbnz, Cbz, Ccmpi, Csinc, Csinv, Dmb, Eret, Mrs, MsrImm,
        MsrReg, Ret, Sys, Tbnz, Tbz, Wfi, Xpaclri, Yield,
    },
    cpu::Cpu,
    imm_instr::{AddImmediate, Movk, Movn, Movz, SubImmediate, Subs},
    load_store_instr::{
        Ldar, Ldaxr, LdpPostIndex, LdpPreIndex, LdpSignedOffset, LdrImmPostIdx, LdrImmPreIdx,
        LdrImmUnOffset, LdrLit, LdrReg, LdrbPostIndex, LdrbPreIndex, LdrbRegister, LdrbUnsignedOff,
        LdrhImmPostIndex, LdrhImmPreIndex, LdrhImmUnOffset, LdrhRegister, LdrsbImmPostIndex,
        LdrsbImmPreIndex, LdrsbImmUnsignedOffset, LdrshImmPostIndex, LdrshImmPreIndex,
        LdrshImmUnsignedOffset, LdrswImmPostIndex, LdrswImmPreIndex, LdrswImmUnOffset,
        LdrswRegister, Ldur, Ldurb, Ldurh, Ldursw, Ldxr, Prfm, StlrNoOffset, Stlrb, Stlxr,
        StpPostIndex, StpPreIndex, StpSignedOffset, StrImmPostIndex, StrImmPreIndex,
        StrImmUnOffset, StrRegister, StrbImmUnOffset, StrbPostIndex, StrbPreIndex, StrbRegister,
        StrhUnsigned, Stur, Sturb, Sturh, Stxr,
    },
    register_instr::{
        AddExtendedRegister, AddShiftedReg, AddsExtendedRegister, AddsImmediate, AddsShiftedReg, Adr, Adrp, AndImmediate, AndShiftedRegister, AndsImmediate, AndsShiftedReg, Asrv, AutiaspSystem, Bfm, BicShiftedReg, BicShiftedRegSet, CcmpRegister, Clz, Csel, Csneg, Dsb, EorImmediate, EorShiftedReg, Isb, Lslv, Lsrv, Madd, Msub, Nop, OrNotShiftedRegister, OrShiftedRegister, OrrImmediate, PaciaSystem, Rbit, Rev, Sbfm, Smaddl, SubExtendedReg, SubShiftedRegister, SubsExtendedReg, SubsShiftedReg, Ubfx, Udiv, Umaddl, Umulh
    },
};

const PRIME_SIZE: usize = 1 << 12;
type DecodeFn = fn(u32) -> Instruction;

#[derive(Clone, Copy)]
pub struct InstDesc {
    pub mask: u32,
    pub value: u32,
    pub decode: DecodeFn,
}

static TABLES: OnceLock<Tables> = OnceLock::new();

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
    // ============================================================
    // BRANCH
    // ============================================================
    // ----- Return -----
    Ret(Ret),
    // ----- Conditional -----
    Bcond(Bcond),
    // ----- Test Branch -----
    Cbnz(Cbnz),
    Cbz(Cbz),
    Tbnz(Tbnz),
    Tbz(Tbz),
    // ----- Unconditional -----
    Bl(Bl),
    Blr(Blr),
    Br(Br),
    Branch(Branch),
    // ============================================================
    // DATA PROCESSING
    // ============================================================
    // ----- Add Subtract -----
    AddsExtendedRegister(AddsExtendedRegister),
    AddsImmediate(AddsImmediate),
    AddsShiftedReg(AddsShiftedReg),
    AddExtendedRegister(AddExtendedRegister),
    AddImmediate(AddImmediate),
    AddShiftedReg(AddShiftedReg),
    Ccmpi(Ccmpi),
    CcmpRegister(CcmpRegister),
    Subs(Subs),
    SubsExtendedReg(SubsExtendedReg),
    SubExtendedReg(SubExtendedReg),
    SubsShiftedReg(SubsShiftedReg),
    SubImmediate(SubImmediate),
    SubShiftedRegister(SubShiftedRegister),
    // ----- Conditional Select -----
    Csel(Csel),
    Csinc(Csinc),
    Csinv(Csinv),
    Csneg(Csneg),
    // ----- Move -----
    Movk(Movk),
    Movn(Movn),
    Movz(Movz),
    // ----- Multiply -----
    Msub(Msub),
    Umulh(Umulh),
    Madd(Madd),
    Smaddl(Smaddl),
    Umaddl(Umaddl),
    // ----- Divide -----
    Udiv(Udiv),
    // ----- Logical And -----
    AndsImmediate(AndsImmediate),
    AndsShiftedReg(AndsShiftedReg),
    AndImmediate(AndImmediate),
    AndShiftedRegister(AndShiftedRegister),
    BicShiftedReg(BicShiftedReg),
    BicShiftedRegSet(BicShiftedRegSet),
    // ----- Logical Or -----
    OrrImmediate(OrrImmediate),
    OrNotShiftedRegister(OrNotShiftedRegister),
    OrShiftedRegister(OrShiftedRegister),
    // ----- Address Calculation -----
    Adr(Adr),
    Adrp(Adrp),
    // ----- Bitfield -----
    Bfm(Bfm),
    Sbfm(Sbfm),
    Ubfx(Ubfx),
    // ----- Shift -----
    Asrv(Asrv),
    Lslv(Lslv),
    Lsrv(Lsrv),
    // ----- Count -----
    Clz(Clz),
    // ----- Bit Reversal -----
    Rev(Rev),
    Rbit(Rbit),
    // ----- Logical Xor -----
    EorImmediate(EorImmediate),
    EorShiftedReg(EorShiftedReg),
    // ============================================================
    // SYSTEM
    // ============================================================
    // ----- Exceptions -----
    Eret(Eret),
    // ----- Flag Manipulation -----
    MsrImm(MsrImm),
    // ----- Registers -----
    Mrs(Mrs),
    MsrReg(MsrReg),
    // ----- Hints -----
    Bti(Bti),
    Nop(Nop),
    Wfi(Wfi),
    Xpaclri(Xpaclri),
    Yield(Yield),
    // ----- Barriers -----
    Dmb(Dmb),
    Dsb(Dsb),
    Isb(Isb),
    // ----- Cache Tlb -----
    Sys(Sys),
    // ----- Pointer Authentication -----
    AutiaspSystem(AutiaspSystem),
    PaciaSystem(PaciaSystem),
    // ============================================================
    // LOAD STORE
    // ============================================================
    // ----- Store Single -----
    Prfm(Prfm),
    StlrNoOffset(StlrNoOffset),
    StrbPostIndex(StrbPostIndex),
    StrbPreIndex(StrbPreIndex),
    StrbImmUnOffset(StrbImmUnOffset),
    StrbRegister(StrbRegister),
    StrhUnsigned(StrhUnsigned),
    StrImmPostIndex(StrImmPostIndex),
    StrImmPreIndex(StrImmPreIndex),
    StrImmUnOffset(StrImmUnOffset),
    StrRegister(StrRegister),
    Stlrb(Stlrb),
    Stur(Stur),
    Sturh(Sturh),
    Sturb(Sturb),
    // ----- Load Single -----
    Ldar(Ldar),
    LdrbPostIndex(LdrbPostIndex),
    LdrbPreIndex(LdrbPreIndex),
    LdrbRegister(LdrbRegister),
    LdrbUnsignedOff(LdrbUnsignedOff),
    LdrhRegister(LdrhRegister),
    LdrhImmPostIndex(LdrhImmPostIndex),
    LdrhImmPreIndex(LdrhImmPreIndex),
    LdrhImmUnOffset(LdrhImmUnOffset),
    LdrsbImmPostIndex(LdrsbImmPostIndex),
    LdrsbImmPreIndex(LdrsbImmPreIndex),
    LdrsbImmUnsignedOffset(LdrsbImmUnsignedOffset),
    LdrshImmPostIndex(LdrshImmPostIndex),
    LdrshImmPreIndex(LdrshImmPreIndex),
    LdrshImmUnsignedOffset(LdrshImmUnsignedOffset),
    LdrswImmPostIndex(LdrswImmPostIndex),
    LdrswImmPreIndex(LdrswImmPreIndex),
    LdrswImmUnOffset(LdrswImmUnOffset),
    LdrswRegister(LdrswRegister),
    LdrImmPostIdx(LdrImmPostIdx),
    LdrImmPreIdx(LdrImmPreIdx),
    LdrImmUnOffset(LdrImmUnOffset),
    LdrLit(LdrLit),
    LdrReg(LdrReg),
    Ldur(Ldur),
    Ldursw(Ldursw),
    Ldurh(Ldurh),
    Ldurb(Ldurb),
    // ----- Store Pair -----
    StpPostIndex(StpPostIndex),
    StpPreIndex(StpPreIndex),
    StpSignedOffset(StpSignedOffset),
    // ----- Load Pair -----
    LdpPostIndex(LdpPostIndex),
    LdpPreIndex(LdpPreIndex),
    LdpSignedOffset(LdpSignedOffset),
    // ----- Load Exclusive -----
    Ldaxr(Ldaxr),
    Ldxr(Ldxr),
    // ----- Store Exclusive -----
    Stlxr(Stlxr),
    Stxr(Stxr),
);

pub const DESCR: &[InstDesc] = &sort_by_specificity([
    // ============================================================
    // BRANCH
    // ============================================================
    // ----- Return -----
    Ret::RET,
    // ----- Conditional -----
    Bcond::B_COND,
    // ----- Test Branch -----
    Cbnz::CBNZ,
    Cbz::CBZ,
    Tbnz::TBNZ,
    Tbz::TBZ,
    // ----- Unconditional -----
    Bl::BL,
    Blr::BLR,
    Br::BR,
    Branch::BRANCH,
    // ============================================================
    // DATA PROCESSING
    // ============================================================
    // ----- Add Subtract -----
    AddsExtendedRegister::ADDS_EXTENDED_REGISTER,
    AddsImmediate::ADDS_IMMEDIATE,
    AddsShiftedReg::ADDS_SHIFTED_REG,
    AddExtendedRegister::ADD_EXTENDED_REGISTER,
    AddImmediate::ADD_IMMEDIATE,
    AddShiftedReg::ADD_SHIFTED_REG,
    Ccmpi::CCMPI,
    CcmpRegister::CCMP_REGISTER,
    Subs::SUBS,
    SubsExtendedReg::SUBS_EXTENDED_REG,
    SubExtendedReg::SUB_EXTENDED_REG,
    SubsShiftedReg::SUBS_SHIFTED_REG,
    SubImmediate::SUB_IMMEDIATE,
    SubShiftedRegister::SUB_SHIFTED_REGISTER,
    // ----- Conditional Select -----
    Csel::CSEL,
    Csinc::CSINC,
    Csinv::CSINV,
    Csneg::CSNEG,
    // ----- Move -----
    Movk::MOVK,
    Movn::MOVN,
    Movz::MOVZ,
    // ----- Multiply -----
    Msub::MSUB,
    Umulh::UMULH,
    Madd::MADD,
    Smaddl::SMADDL,
    Umaddl::UMADDL,
    // ----- Divide -----
    Udiv::UDIV,
    // ----- Logical And -----
    AndsImmediate::ANDS_IMMEDIATE,
    AndsShiftedReg::ANDS_SHIFTED_REG,
    AndImmediate::AND_IMMEDIATE,
    AndShiftedRegister::AND_SHIFTED_REGISTER,
    BicShiftedReg::BIC_SHIFTED_REG,
    BicShiftedRegSet::BIC_SHIFTED_REG_SET,
    // ----- Logical Or -----
    OrrImmediate::ORR_IMMEDIATE,
    OrNotShiftedRegister::OR_NOT_SHIFTED_REGISTER,
    OrShiftedRegister::OR_SHIFTED_REGISTER,
    // ----- Address Calculation -----
    Adr::ADR,
    Adrp::ADRP,
    // ----- Bitfield -----
    Bfm::BFM,
    Sbfm::SBFM,
    Ubfx::UBFX,
    // ----- Shift -----
    Asrv::ASRV,
    Lslv::LSLV,
    Lsrv::LSRV,
    // ----- Count -----
    Clz::CLZ,
    // ----- Bit Reversal -----
    Rev::REV,
    Rbit::RBIT,
    // ----- Logical Xor -----
    EorShiftedReg::EOR_SHIFTED_REG,
    EorImmediate::EOR_IMMEDIATE,
    // ============================================================
    // SYSTEM
    // ============================================================
    // ----- Exceptions -----
    Eret::ERET,
    // ----- Flag Manipulation -----
    MsrImm::MSR_IMM,
    // ----- Registers -----
    Mrs::MRS,
    MsrReg::MSR_REG,
    // ----- Hints -----
    Bti::BTI,
    Nop::NOP,
    Wfi::WFI,
    Xpaclri::XPACLRI,
    Yield::YIELD,
    // ----- Barriers -----
    Dmb::DMB,
    Dsb::DSB,
    Isb::ISB,
    // ----- Cache Tlb -----
    Sys::SYS,
    // ----- Pointer Authentication -----
    AutiaspSystem::AUTIASP_SYSTEM,
    PaciaSystem::PACIA_SYSTEM,
    // ============================================================
    // LOAD STORE
    // ============================================================
    // ----- Store Single -----
    Prfm::PRFM,
    StlrNoOffset::STLR_NO_OFFSET,
    StrbPostIndex::STRB_POST_INDEX,
    StrbPreIndex::STRB_PRE_INDEX,
    StrbImmUnOffset::STRB_IMM_UN_OFFSET,
    StrbRegister::STRB_REGISTER,
    StrhUnsigned::STRH_UNSIGNED,
    StrImmPostIndex::STR_IMM_POST_INDEX,
    StrImmPreIndex::STR_IMM_PRE_INDEX,
    StrImmUnOffset::STR_IMM_UN_OFFSET,
    StrRegister::STR_REGISTER,
    Stur::STUR,
    Sturh::STURH,
    Sturb::STURB,
    Stlrb::STLRB,
    // ----- Load Single -----
    Ldar::LDAR,
    LdrbPostIndex::LDRB_POST_INDEX,
    LdrbPreIndex::LDRB_PRE_INDEX,
    LdrbRegister::LDRB_REGISTER,
    LdrbUnsignedOff::LDRB_UNSIGNED_OFF,
    LdrhRegister::LDRH_REGISTER,
    LdrhImmPostIndex::LDRH_IMM_POST_INDEX,
    LdrhImmPreIndex::LDRH_IMM_PRE_INDEX,
    LdrsbImmPostIndex::LDRSB_IMM_POST_INDEX,
    LdrsbImmPreIndex::LDRSB_IMM_PRE_INDEX,
    LdrsbImmUnsignedOffset::LDRSB_IMM_UNSIGNED_OFFSET,
    LdrhImmUnOffset::LDRH_IMM_UN_OFFSET,
    LdrshImmPostIndex::LDRSH_IMM_POST_INDEX,
    LdrshImmPreIndex::LDRSH_IMM_PRE_INDEX,
    LdrshImmUnsignedOffset::LDRSH_IMM_UNSIGNED_OFFSET,
    LdrswImmPostIndex::LDRSW_IMM_POST_INDEX,
    LdrswImmPreIndex::LDRSW_IMM_PRE_INDEX,
    LdrswImmUnOffset::LDRSW_IMM_UN_OFFSET,
    LdrswRegister::LDRSW_REGISTER,
    LdrImmPostIdx::LDR_IMM_POST_IDX,
    LdrImmPreIdx::LDR_IMM_PRE_IDX,
    LdrImmUnOffset::LDR_IMM_UN_OFFSET,
    LdrLit::LDR_LIT,
    LdrReg::LDR_REG,
    Ldurh::LDURH,
    Ldur::LDUR,
    Ldursw::LDURSW,
    Ldurb::LDURB,
    // ----- Store Pair -----
    StpPostIndex::STP_POST_INDEX,
    StpPreIndex::STP_PRE_INDEX,
    StpSignedOffset::STP_SIGNED_OFFSET,
    // ----- Load Pair -----
    LdpPostIndex::LDP_POST_INDEX,
    LdpPreIndex::LDP_PRE_INDEX,
    LdpSignedOffset::LDP_SIGNED_OFFSET,
    // ----- Load Exclusive -----
    Ldaxr::LDAXR,
    Ldxr::LDXR,
    // ----- Store Exclusive -----
    Stlxr::STLXR,
    Stxr::STXR,
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

pub fn validate_tables(table: &[InstDesc]) {
    let mut clash = Vec::new();

    let mut index = 1;
    for start in table.iter() {
        for end in table[index..].iter().rev() {
            let overlap = start.mask & end.mask;
            if (start.value & overlap) == (end.value & overlap) {
                clash.push((start, end));
            }
        }
        index += 1;
    }

    if !clash.is_empty() {
        println!("FATAL ERROR!");
        println!("THERE ARE COLLIDING ENTRIES IN INSTRUCTION TABLE");
        clash.iter().for_each(|e| {
            println!("These two are colliding.\r\n-> {:032b}\r\n-> {:032b}", e.0.value, e.1.value)
        });
        println!("Aborting the execution");
        std::process::exit(0);
    }
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
