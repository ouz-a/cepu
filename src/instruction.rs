use std::{
    collections::VecDeque,
    sync::{OnceLock, atomic::AtomicBool},
};

use crate::{
    branch_exc_sys_instr::{
        Bcond, Bl, Blr, Br, Branch, Bti, Cbnz, Cbz, Ccmpi, Chkfeat, Csdb, Csinc, Csinv, Dmb, Eret,
        Mrs, MsrImm, MsrReg, Ret, Sevl, Smc, Svc, Sys, Tbnz, Tbz, Wfe, Wfi, Xpaclri, Yield,
    },
    cpu::Cpu,
    imm_instr::{AddImmediate, Extr, Movk, Movn, Movz, Sbc, SubImmediate, Subs},
    load_store_instr::{
        Ldar, Ldaxr, LdpPostIndex, LdpPreIndex, LdpSignedOffset, LdpswPostIndex, LdpswPreIndex,
        LdpswSignedOffset, LdrImmPostIdx, LdrImmPreIdx, LdrImmUnOffset, LdrLit, LdrReg, LdrUnpriv,
        LdrbPostIndex, LdrbPreIndex, LdrbRegister, LdrbUnsignedOff, LdrhImmPostIndex,
        LdrhImmPreIndex, LdrhImmUnOffset, LdrhRegister, LdrsbImmPostIndex, LdrsbImmPreIndex,
        LdrsbImmUnsignedOffset, LdrsbReg, LdrshImmPostIndex, LdrshImmPreIndex,
        LdrshImmUnsignedOffset, LdrswImmPostIndex, LdrswImmPreIndex, LdrswImmUnOffset,
        LdrswRegister, Ldur, Ldurb, Ldurh, Ldursw, Ldxp, Ldxr, Ldxrb, Prfm, StlrNoOffset, Stlrb,
        Stlrh, Stlxp, Stlxr, Stnp, StpPostIndex, StpPreIndex, StpSignedOffset, StrImmPostIndex,
        StrImmPreIndex, StrImmUnOffset, StrRegister, StrUnpriv, StrbImmUnOffset, StrbPostIndex,
        StrbPreIndex, StrbRegister, Strh, StrhPostIndex, StrhPreIndex, StrhUnpriv, StrhUnsigned,
        Stur, Sturb, Sturh, Stxp, Stxr, Stxrb,
    },
    register_instr::{
        AddExtendedRegister, AddShiftedReg, AddsExtendedRegister, AddsImmediate, AddsShiftedReg,
        Adr, Adrp, AndImmediate, AndShiftedRegister, AndsImmediate, AndsShiftedReg, Asrv,
        AutiaspSystem, Bfm, BicShiftedReg, BicShiftedRegSet, CcmpRegister, Clz, Csel, Csneg, Dsb,
        EorImmediate, EorShiftedReg, Isb, Lslv, Lsrv, Madd, Msub, Nop, OrNotShiftedRegister,
        OrShiftedRegister, OrrImmediate, PaciaSystem, Rbit, Rev, Rev16, Sbfm, Sdiv, Smaddl, Smulh,
        SubExtendedReg, SubShiftedRegister, SubsExtendedReg, SubsShiftedReg, Ubfx, Udiv, Umaddl,
        Umulh,
    },
    simd_fp::{
        AddpScalar, AddpVector, AsimdModImm, BitwiseInsert, CmeqRegScalar, CmeqRegVector,
        CmeqScalar, CmeqVector, CmhsRegisterScalar, CmhsRegisterVector, DupGeneral, Ext,
        FmovGeneral, Ld1NoOffset, Ld1PostIndex, LdpSimdFpPostIndex, LdpSimdFpPreIndex,
        LdpSimdFpSignedOffset, LdrSimdFpPostIndex, LdrSimdFpPreIndex, LdrSimdFpUnsignedOffset,
        LdurSimd, Shrn, StrImdFpPostIndex, StrImdFpPreIndex, StrImdFpUnsignedOffset,
        StrPairFpPostIndex, StrPairFpPreIndex, StrPairFpSignedOffset, StrSimdRegOffset,
        SturSimdUnscaledOffset, Umaxp,
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
    Sbc(Sbc),
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
    Smulh(Smulh),
    Madd(Madd),
    Smaddl(Smaddl),
    Umaddl(Umaddl),
    // ----- Divide -----
    Udiv(Udiv),
    Sdiv(Sdiv),
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
    Rev16(Rev16),
    Rbit(Rbit),
    // ----- Logical Xor -----
    EorImmediate(EorImmediate),
    EorShiftedReg(EorShiftedReg),
    // ============================================================
    // SYSTEM
    // ============================================================
    // ----- Exceptions -----
    Eret(Eret),
    Smc(Smc),
    Svc(Svc),
    // ----- Flag Manipulation -----
    MsrImm(MsrImm),
    // ----- Registers -----
    Extr(Extr),
    Mrs(Mrs),
    MsrReg(MsrReg),
    // ----- Hints -----
    Bti(Bti),
    Chkfeat(Chkfeat),
    Nop(Nop),
    Wfi(Wfi),
    Wfe(Wfe),
    Sevl(Sevl),
    Xpaclri(Xpaclri),
    Yield(Yield),
    // ----- Barriers -----
    Csdb(Csdb),
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
    StrhPostIndex(StrhPostIndex),
    StrhPreIndex(StrhPreIndex),
    StrImmPostIndex(StrImmPostIndex),
    StrImmPreIndex(StrImmPreIndex),
    StrImmUnOffset(StrImmUnOffset),
    StrRegister(StrRegister),
    Stlrb(Stlrb),
    Stur(Stur),
    Sturh(Sturh),
    Stlrh(Stlrh),
    Strh(Strh),
    Sturb(Sturb),
    StrUnpriv(StrUnpriv),
    StrhUnpriv(StrhUnpriv),
    // ----- Load Single -----
    LdrsbReg(LdrsbReg),
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
    LdrUnpriv(LdrUnpriv),
    LdpSimdFpPostIndex(LdpSimdFpPostIndex),
    LdpSimdFpPreIndex(LdpSimdFpPreIndex),
    LdpSimdFpSignedOffset(LdpSimdFpSignedOffset),
    // ----- Store Pair -----
    Stnp(Stnp),
    StpPostIndex(StpPostIndex),
    StpPreIndex(StpPreIndex),
    StpSignedOffset(StpSignedOffset),
    // ----- Load Pair -----
    Ldxp(Ldxp),
    LdpPostIndex(LdpPostIndex),
    LdpPreIndex(LdpPreIndex),
    LdpSignedOffset(LdpSignedOffset),
    LdpswPostIndex(LdpswPostIndex),
    LdpswPreIndex(LdpswPreIndex),
    LdpswSignedOffset(LdpswSignedOffset),
    // ----- Load Exclusive -----
    Ldaxr(Ldaxr),
    Ldxr(Ldxr),
    Ldxrb(Ldxrb),
    // ----- Store Exclusive -----
    Stxp(Stxp),
    Stlxp(Stlxp),
    Stlxr(Stlxr),
    Stxr(Stxr),
    Stxrb(Stxrb),
    // ----- Simd, FP -----
    AddpVector(AddpVector),
    AddpScalar(AddpScalar),
    AsimdModImm(AsimdModImm),
    LdurSimd(LdurSimd),
    Umaxp(Umaxp),
    CmhsRegisterScalar(CmhsRegisterScalar),
    CmhsRegisterVector(CmhsRegisterVector),
    FmovGeneral(FmovGeneral),
    Shrn(Shrn),
    BitwiseInsert(BitwiseInsert),
    CmeqVector(CmeqVector),
    CmeqScalar(CmeqScalar),
    CmeqRegVector(CmeqRegVector),
    CmeqRegScalar(CmeqRegScalar),
    DupGeneral(DupGeneral),
    StrSimdRegOffset(StrSimdRegOffset),
    StrImdFpPostIndex(StrImdFpPostIndex),
    StrImdFpPreIndex(StrImdFpPreIndex),
    StrImdFpUnsignedOffset(StrImdFpUnsignedOffset),
    StrPairFpPostIndex(StrPairFpPostIndex),
    StrPairFpPreIndex(StrPairFpPreIndex),
    StrPairFpSignedOffset(StrPairFpSignedOffset),
    SturSimdUnscaledOffset(SturSimdUnscaledOffset),
    LdrSimdFpPostIndex(LdrSimdFpPostIndex),
    LdrSimdFpPreIndex(LdrSimdFpPreIndex),
    LdrSimdFpUnsignedOffset(LdrSimdFpUnsignedOffset),
    Ld1NoOffset(Ld1NoOffset),
    Ld1PostIndex(Ld1PostIndex),
    Ext(Ext),
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
    Sbc::SBC,
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
    Smulh::SMULH,
    Madd::MADD,
    Smaddl::SMADDL,
    Umaddl::UMADDL,
    // ----- Divide -----
    Sdiv::SDIV,
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
    Rev16::REV16,
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
    Smc::SMC,
    Svc::SVC,
    // ----- Flag Manipulation -----
    MsrImm::MSR_IMM,
    // ----- Registers -----
    Extr::EXTR,
    Mrs::MRS,
    MsrReg::MSR_REG,
    // ----- Hints -----
    Bti::BTI,
    Chkfeat::CHKFEAT,
    Nop::NOP,
    Wfi::WFI,
    Wfe::WFE,
    Sevl::SEVL,
    Xpaclri::XPACLRI,
    Yield::YIELD,
    // ----- Barriers -----
    Csdb::CSDB,
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
    StrhPostIndex::STRH_POST_INDEX,
    StrhPreIndex::STRH_PRE_INDEX,
    StrImmPostIndex::STR_IMM_POST_INDEX,
    StrImmPreIndex::STR_IMM_PRE_INDEX,
    StrImmUnOffset::STR_IMM_UN_OFFSET,
    StrRegister::STR_REGISTER,
    Stur::STUR,
    Sturh::STURH,
    Stlrh::STLRH,
    Strh::STRH,
    Sturb::STURB,
    Stlrb::STLRB,
    StrUnpriv::STR_UNPRIV,
    StrhUnpriv::STRH_UNPRIV,
    // ----- Load Single -----
    Ldar::LDAR,
    LdrsbReg::LDRSB_REG,
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
    LdrUnpriv::LDR_UNPRIV,
    LdpSimdFpPostIndex::LDP_SIMD_FP_POST_INDEX,
    LdpSimdFpPreIndex::LDP_SIMD_FP_PRE_INDEX,
    LdpSimdFpSignedOffset::LDP_SIMD_FP_SIGNED_OFFSET,
    // ----- Store Pair -----
    Stnp::STNP,
    StpPostIndex::STP_POST_INDEX,
    StpPreIndex::STP_PRE_INDEX,
    StpSignedOffset::STP_SIGNED_OFFSET,
    // ----- Load Pair -----
    Ldxp::LDXP,
    LdpPostIndex::LDP_POST_INDEX,
    LdpPreIndex::LDP_PRE_INDEX,
    LdpSignedOffset::LDP_SIGNED_OFFSET,
    LdpswPostIndex::LDPSW_POST_INDEX,
    LdpswPreIndex::LDPSW_PRE_INDEX,
    LdpswSignedOffset::LDPSW_SIGNED_OFFSET,
    // ----- Load Exclusive -----
    Ldaxr::LDAXR,
    Ldxr::LDXR,
    Ldxrb::LDXRB,
    // ----- Store Exclusive -----
    Stxp::STXP,
    Stlxp::STLXP,
    Stlxr::STLXR,
    Stxr::STXR,
    Stxrb::STXRB,
    // ----- Simd, FP -----
    AddpVector::ADDP_VECTOR,
    AddpScalar::ADDP_SCALAR,
    AsimdModImm::ASIMD_MOD_IMM,
    LdurSimd::LDUR_SIMD,
    Umaxp::UMAXP,
    CmhsRegisterScalar::CMHS_REGISTER_SCALAR,
    CmhsRegisterVector::CMHS_REGISTER_VECTOR,
    FmovGeneral::FMOV_GENERAL,
    Shrn::SHRN_01XX,
    Shrn::SHRN_001X,
    Shrn::SHRN_0001,
    BitwiseInsert::BITWISE_INSERT,
    CmeqVector::CMEQ_VECTOR,
    CmeqScalar::CMEQ_SCALAR,
    CmeqRegScalar::CMEQ_REG_SCALAR,
    CmeqRegVector::CMEQ_REG_VECTOR,
    DupGeneral::DUP_GENERAL,
    StrSimdRegOffset::STR_SIMD_REG_OFFSET,
    StrImdFpPostIndex::STR_IMD_FP_POST_INDEX,
    StrImdFpPreIndex::STR_IMD_FP_PRE_INDEX,
    StrImdFpUnsignedOffset::STR_IMD_FP_UNSIGNED_OFFSET,
    StrPairFpPostIndex::STR_PAIR_FP_POST_INDEX,
    StrPairFpPreIndex::STR_PAIR_FP_PRE_INDEX,
    StrPairFpSignedOffset::STR_PAIR_FP_SIGNED_OFFSET,
    SturSimdUnscaledOffset::STUR_SIMD_UNSCALED_OFFSET,
    LdrSimdFpPostIndex::LDR_SIMD_FP_POST_INDEX,
    LdrSimdFpPreIndex::LDR_SIMD_FP_PRE_INDEX,
    LdrSimdFpUnsignedOffset::LDR_SIMD_FP_UNSIGNED_OFFSET,
    Ld1NoOffset::LD1_NO_OFFSET,
    Ld1PostIndex::LD1_POST_INDEX,
    Ext::EXT,
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

fn format_u32_binary(value: u32) -> String {
    let binary = format!("{:032b}", value);

    let formatted: String = binary
        .as_bytes()
        .chunks(4)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join("_");

    format!("0b{}", formatted)
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
            println!(
                "These two are colliding.\r\n-> {}\r\n-> {}",
                format_u32_binary(e.0.value),
                format_u32_binary(e.1.value)
            )
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
    UNDEF_PANIC.store(true, std::sync::atomic::Ordering::SeqCst);
    Instruction::Nop(Nop)
}

pub fn decode_undef(_: u32) -> Instruction {
    panic!("Undefined instruction")
}

pub const UNDEF_DESC: InstDesc = InstDesc { mask: 0, value: 0, decode: decode_undef };
pub static UNDEF_PANIC: AtomicBool = AtomicBool::new(false);
pub static CURRENT_PC: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn capture_trace(trace_holder: &mut VecDeque<(u64, u32)>, pc: u64, word: u32) {
    if trace_holder.len() >= 20 {
        trace_holder.pop_front();
    }
    trace_holder.push_back((pc, word));
}

const WIDTH: usize = 76;

pub fn print_undefined_trace(how_many: u64, trace_holder: &mut VecDeque<(u64, u32)>) {
    let (pc, word) = trace_holder.pop_back().unwrap();

    let line = "—".repeat(WIDTH);

    println!("\n{line}");
    println!("{:^WIDTH$}", "FATAL ERROR");
    println!("{line}");
    println!("\n  Undefined instruction encountered at PC: 0x{pc:016X}");
    println!("\n  Instruction encoding:");
    println!("    Hex:    0x{:08X}", word.to_be());
    println!("    Binary: 0b{:032b}", word);

    let formatted_count = how_many
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join("_");

    println!("\n  Successfully executed {} instructions before failure", formatted_count);
    println!("\n{line}");
    println!("Execution Trace (most recent last):\n");

    for (pc, word) in trace_holder.iter() {
        let decoded = decode(*word);
        println!("  PC: 0x{pc:016X}  │  {decoded:?}");
    }

    println!("\n{line}");
    println!("Cepu will quit now! Bye");
    println!("{line}\n");
}
