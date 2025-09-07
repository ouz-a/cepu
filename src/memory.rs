use core::ffi::c_void;

pub const MEMORY_SIZE: usize = 64 << 22;

pub static mut MEMORY: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];

/*
MEMORY {
  ROM (rx)  : ORIGIN = 0x00000000, LENGTH = 0x8000
  RAM (rwx) : ORIGIN = 0x00008000, LENGTH = 0x100000
}
*/

pub fn read_32(address: usize) -> u32 {
    assert!(address + 3 < MEMORY_SIZE);
    let instruction: u32 = unsafe {
        u32::from_le_bytes([
            MEMORY[address],
            MEMORY[address + 1],
            MEMORY[address + 2],
            MEMORY[address + 3],
        ])
    };
    instruction
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultStatus {
    #[default]
    None,
    AccessFlag,
    Alignment,
    Background,
    Domain,
    Permission,
    Translation,
    AddressSize,
    SyncExternal,
    SyncExternalOnWalk,
    SyncParity,
    SyncParityOnWalk,
    GpcfOnWalk,
    GpcfOnOutput,
    AsyncParity,
    AsyncExternal,
    TagCheck,
    Debug,
    TlbConflict,
    BranchTarget,
    HwUpdateAccessFlag,
    Lockdown,
    Exclusive,
    ICacheMaint,
}

#[repr(u8)]
#[derive(Clone, Default, Copy, Debug, PartialEq, Eq)]
pub enum ErrorState {
    #[default]
    Uc, // Uncontainable
    Ueu, // Unrecoverable state
    Ueo, // Restartable state
    Uer, // Recoverable state
    Ce,  // Correctable error
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaRange {
    #[default]
    Lower,
    Upper,
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemAtomicOp {
    #[default]
    Gcsss1,
    Add,
    Bic,
    Eor,
    Orr,
    Smax,
    Smin,
    Umax,
    Umin,
    Swp,
    Cas,
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemOp {
    #[default]
    Load,
    Store,
    Prefetch,
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheOp {
    #[default]
    Clean,
    Invalidate,
    CleanInvalidate,
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheOpScope {
    #[default]
    SetWay,
    PoU,
    PoC,
    PoE,
    PoP,
    PoDp,
    PoPa,
    Allu,
    Alluis,
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheType {
    #[default]
    Data,
    Tag,
    DataTag,
    Instruction,
}

#[repr(u8)]
#[derive(Clone, Default, Copy, Debug, PartialEq, Eq)]
pub enum AccessType {
    /// Instruction fetch
    #[default]
    IFetch,
    /// Software load/store to a General-Purpose Register
    Gpr,
    /// ASIMD extension load/store instructions
    Asimd,
    /// SVE load/store instructions
    Sve,
    /// SME load/store instructions
    Sme,
    /// Sys-op IC cache maintenance
    Ic,
    /// Sys-op DC cache maintenance (not DC {Z,G,GZ}VA)
    Dc,
    /// Sys-op DC {Z,G,GZ}VA (“DC Zero”)
    DcZero,
    /// Sys-op AT (Address Translate) operation
    At,
    /// NV2 memory-redirected access
    Nv2,
    /// Statistical Profiling buffer access
    Spe,
    /// Guarded Control Stack access
    Gcs,
    /// Trace Buffer access
    Trbe,
    /// Granule Protection Table Walk
    Gptw,
    /// Access to the HACDBS structure
    Hacdbs,
    /// Access to entries in HDBSS
    Hdbss,
    /// Translation Table Walk
    Ttw,
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SecurityState {
    #[default]
    NonSecure,
    Root,
    Realm,
    Secure,
}

#[derive(Default, Clone, Debug)]
pub struct AccessDescriptor {
    pub acc_type: AccessType,
    pub el: u8,
    pub ss: SecurityState,

    /* memory ordering & exclusives */
    pub accqsc: bool,
    pub acqpc: bool,
    pub relsc: bool,
    pub limited_ordered: bool,
    pub exclusive: bool,
    pub atomicop: bool,
    pub mod_op: MemAtomicOp,

    /* data-access attributes */
    pub non_temporal: bool,
    pub read: bool,
    pub write: bool,

    /* cache maintenance */
    pub cache_op: CacheOp,
    pub cache_op_scope: CacheOpScope,
    pub cache_type: CacheType,

    /* fault / permission controls */
    pub pan: bool,
    pub non_fault: bool,
    pub first_fault: bool,
    pub first: bool,

    /* miscellaneous attributes */
    pub contiguous: bool,
    pub streaming_sve: bool,
    pub ls_64: bool,
    pub mops: bool,
    pub rcw: bool,
    pub rcws: bool,
    pub top_level: bool,
    pub va_range: VaRange,
    pub a321_smd: bool,
    pub tag_checked: bool,
    pub tag_access: bool,
    pub devs_toreunpred: bool,
    pub transactional: bool,

    /* pair-load/store hints */
    pub is_pair: bool,
    pub highest_address_first: bool,

    /* MPAM pointer — still present for layout parity, but discouraged */
    #[deprecated(note = "Access to MPAM is forbidden")]
    pub mpam: *mut c_void,
}
impl AccessDescriptor {
    fn new(acc_type: AccessType) -> Self {
        Self { acc_type, ..Default::default() }
    }
    pub fn create_acc_descr_gpr(
        memop: MemOp,
        non_temporal: bool,
        priveleged: bool,
        tag_checked: bool,
    ) -> Self {
        let mut acc_descr = Self::new(AccessType::Gpr);
        acc_descr.el = if priveleged { 3 } else { 0 };
        acc_descr.non_temporal = non_temporal;
        acc_descr.read = memop == MemOp::Load;
        acc_descr.write = memop == MemOp::Store;
        acc_descr.pan = true;
        acc_descr.tag_checked = tag_checked;
        acc_descr
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PhyMemStatus {
    pub fault_status: FaultStatus,
    pub _error_state: ErrorState,
}
