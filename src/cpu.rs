#[derive(Debug, Clone, Copy)]
struct Cpu {
    /// 64-Bit General Purpose Register
    x: [u64; 32],

    /// Program Counter
    pc: u64,

    /// Stack Pointer EL0
    sp_el0: u64,
    /// Stack Pointer EL1
    sp_el1: u64,
    /// Stack Pointer EL2
    sp_el2: u64,
    /// Stack Pointer EL3
    sp_el3: u64,

    /// System Control Register EL1
    sctlr_el1: u64,
    /// System Control Register EL2
    sctlr_el2: u64,
    /// System Control Register EL3
    sctlr_el3: u64,

    /// Secure Configuration Register EL3
    scr_el3: u64,

    /// Saved Program Status Register EL3
    spsr_el3: u64,
    /// Saved Program Status Register EL2
    spsr_el2: u64,
    /// Saved Program Status Register EL1
    spsr_el1: u64,

    /// Exception Link Register 3
    elr_el3: u64,
    /// Exception Link Register 2
    elr_el2: u64,
    /// Exception Link Register 1
    elr_el1: u64,
}

#[derive(Debug, Clone, Copy)]
struct PState {
    /// Negative
    n: bool,
    /// Zero
    z: bool,
    /// Carry
    c: bool,
    /// Overflow
    v: bool,

    /// Stack pointer
    sp: u8,
    current_el: ExceptionLevel,

    /// Debug
    d: bool,
    /// Async
    a: bool,
    /// IRQ
    i: bool,
    /// FIQ
    f: bool,

    /// Execution state
    /// 0 === AArch64
    nrw: bool,
    /// Software-step
    /// Until we support MDSCR this will be always 0
    ss: bool,
    /// Illegal
    il: bool,
}

#[derive(Debug, Clone, Copy)]
enum ExceptionLevel {
    EL0,
    EL1,
    EL2,
    EL3,
}
