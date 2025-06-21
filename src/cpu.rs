const GPRS: usize = 32;
const ZERO_REG: usize = 31;

const HIGH_32_MASK: u64 = 0xFFFF_FFFF_0000_0000;
const LOW_32_MASK: u64 = 0xFFFF_FFFFu64;

#[derive(Default, Debug, Clone, Copy)]
struct Cpu {
    /// 64-Bit General Purpose Register
    x: [u64; GPRS],

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

    pstate: PState,
}

impl Cpu {
    fn init() -> Self {
        let mut cpu = Cpu::default();
        cpu.pstate.sp = 1;
        cpu.sp_el0 = 1024;
        cpu.sp_el1 = 1024 * 2;
        cpu.sp_el2 = 1024 * 3;
        cpu.sp_el3 = 1024 * 4;

        cpu.x[31] = cpu.sp_el0;
        // TODO: Use bitflags crate(?)
        cpu.sctlr_el1 |= 1 << 1; // SCTLR_A
        cpu.sctlr_el1 |= 1 << 2; // SCTLR_C
        cpu.sctlr_el1 |= 1 << 12; // SCTLR_I 
        cpu
    }
    fn sp_read(&self) -> u64 {
        if self.pstate.sp == 0 {
            return self.sp_el0;
        }
        match self.pstate.el() {
            ExceptionLevel::EL0 => self.sp_el0,
            ExceptionLevel::EL1 => self.sp_el1,
            ExceptionLevel::EL2 => self.sp_el2,
            ExceptionLevel::EL3 => self.sp_el3,
        }
    }
    fn sp_write(&mut self, value: u64) {
        if self.pstate.sp == 0 {
            self.sp_el0 = value;
        } else {
            match self.pstate.el() {
                ExceptionLevel::EL0 => self.sp_el0 = value,
                ExceptionLevel::EL1 => self.sp_el1 = value,
                ExceptionLevel::EL2 => self.sp_el2 = value,
                ExceptionLevel::EL3 => self.sp_el3 = value,
            }
        }
    }

    /// When n == 31 we return 0
    fn x_read(&self, n: usize, width: u8) -> u64 {
        assert!(n < GPRS);
        assert!(width <= 64 && width % 8 == 0);
        if n != ZERO_REG {
            let mask = if width == 64 { !0u64 } else { (1 << width) - 1 };
            return self.x[n] & mask;
        }
        0
    }

    fn x_write(&mut self, n: usize, value: u64, is_32b: bool) {
        assert!(n < GPRS);
        if is_32b {
            // We want lower 32 bits when value is 32bit
            self.x[n] = (self.x[n] & HIGH_32_MASK) | (value & LOW_32_MASK);
        } else {
            self.x[n] = value;
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
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

impl PState {
    fn el(&self) -> ExceptionLevel {
        self.current_el
    }
}

#[derive(Default, Debug, Clone, Copy)]
enum ExceptionLevel {
    EL0,
    EL1,
    EL2,
    #[default]
    EL3,
}
