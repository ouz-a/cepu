use crate::{get_bits_ct, utils::align};

const GPRS: usize = 32;

const ZERO_REG: usize = 31;
pub const SP_REGISTER: usize = 31;

const HIGH_32_MASK: u64 = 0xFFFF_FFFF_0000_0000;
const LOW_32_MASK: u64 = 0xFFFF_FFFFu64;

const HAVE_AARCH64: bool = false;
const HAVE_EL: bool = true;
const HAVE_AARCH32: bool = false;
const USING_AARCH32: bool = false;
const IS_SECURE_EL2_ENABLED: bool = false;

/// MMU enable
pub const SCTLR_M: u64 = 1u64 << 0;

/// Alignment check enable
pub const SCTLR_A: u64 = 1u64 << 1;

/// Data cache enable
pub const SCTLR_C: u64 = 1u64 << 2;

/// Stack-alignment check enable
pub const SCTLR_SA: u64 = 1u64 << 3;

/// Stack-alignment check enable for EL0
pub const SCTLR_SA0: u64 = 1u64 << 4;

/// Instruction cache enable
pub const SCTLR_I: u64 = 1u64 << 12;

/// Write-permission implies XN
pub const SCTLR_WXN: u64 = 1u64 << 19;

/// Exception endianness
pub const SCTLR_EE: u64 = 1u64 << 25;

#[derive(Default, Debug, Clone, Copy)]
pub struct Cpu {
    /// 64-Bit General Purpose Register
    pub x: [u64; GPRS],

    /// Program Counter
    pub pc: u64,

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

    event_register: bool,
    pub pstate: PState,
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

    pub fn get_elr_elx(&self) -> u64 {
        if self.pstate.current_el.is_el0() || self.pstate.current_el.is_el1() {
            return self.elr_el1;
        } else if self.pstate.current_el.is_el2() {
            return self.elr_el2;
        } else {
            return self.elr_el3;
        }
    }
    pub fn get_spsr_elx(&self) -> u64 {
        if self.pstate.current_el.is_el0() || self.pstate.current_el.is_el1() {
            return self.spsr_el1;
        } else if self.pstate.current_el.is_el2() {
            return self.spsr_el2;
        } else {
            return self.spsr_el3;
        }
    }

    pub fn sp_read(&self) -> u64 {
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

    pub fn check_space_alignment(&self) {
        let sp = self.sp_read();
        let stack_align_check;
        if self.pstate.current_el.is_el0() {
            stack_align_check = (self.sctlr_el1 & SCTLR_SA0) != 0;
        } else {
            stack_align_check = (self.sctlr_el1 & SCTLR_A) != 0;
        }
        if stack_align_check && (sp != align(sp, 16)) {
            panic!("Alignment fault");
        }
    }

    pub fn sp_write(&mut self, value: u64) {
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
    pub fn x_read(&self, n: usize, width: u8) -> u64 {
        assert!(n < GPRS);
        assert!(width <= 64 && width % 8 == 0);
        if n != ZERO_REG {
            let mask = if width == 64 { !0u64 } else { (1 << width) - 1 };
            return self.x[n] & mask;
        }
        0
    }

    pub fn x_write(&mut self, n: usize, value: u64, is_32b: bool) {
        assert!(n < GPRS);
        if is_32b {
            // We want lower 32 bits when value is 32bit
            self.x[n] = (self.x[n] & HIGH_32_MASK) | (value & LOW_32_MASK);
        } else {
            self.x[n] = value;
        }
    }

    pub fn sys_reg_write(
        &mut self,
        sys_op0: u8,
        sys_op1: u8,
        sys_crn: u8,
        sys_crm: u8,
        sys_op2: u8,
        t: u8,
    ) {
        let comp: u64 = ((sys_op0 as u64) << 32)
            | ((sys_op1 as u64) << 24)
            | ((sys_crn as u64) << 16)
            | ((sys_crm as u64) << 8)
            | (sys_op2 as u64);

        let register: MsrRegisters = comp.into();
        match register {
            MsrRegisters::Unknown => {
                panic!("Value {comp} not convered, please check the ARM docs!")
            }
            MsrRegisters::ElrEl3 => {
                if self.pstate.current_el.is_el3() {
                    self.elr_el3 = self.x_read(t.into(), 64);
                } else {
                    panic!("Can't modify ELR_EL3, at current exception level");
                }
            }
            MsrRegisters::SpsrEl3 => {
                if self.pstate.current_el.is_el3() {
                    self.spsr_el3 = self.x_read(t.into(), 64);
                }
                panic!("Can't modify SPSR_EL3, at current exception level");
            }
            MsrRegisters::ScrEl3 => {
                if self.pstate.current_el.is_el3() {
                    self.scr_el3 = self.x_read(t.into(), 64);
                } else {
                    panic!("Can't modify SCR_EL3, at current exception level");
                }
            }
            MsrRegisters::SpEl1 => {
                if self.pstate.current_el.is_el2() || self.pstate.current_el.is_el3() {
                    self.sp_el1 = self.x_read(t.into(), 64);
                } else {
                    panic!("Can't modify SP_EL1, at current exception level");
                }
            }
        }
    }

    pub fn pstate_from_spsr(&mut self, spsr: u64, illegal_spsr_state: bool) {
        self.pstate.ss = false;
        if illegal_spsr_state {
            self.pstate.il = false;
        } else {
            self.pstate.il = get_bits_ct!(spsr, 20, 1) != 0;
            if get_bits_ct!(spsr, 4, 1) == 1 {
                panic!("We haven't implemented AArch32 support!");
            } else {
                self.pstate.nrw = false;
                self.pstate.current_el = ExceptionLevel::from_bits(get_bits_ct!(spsr, 2, 2) as u8);
                self.sp_write(get_bits_ct!(spsr, 0, 1));
            }
        }
        if self.pstate.il == true && self.pstate.nrw == true {
            panic!("We haven't implemented AArch32 support!");
        }
        self.pstate.nzcv_from_spsr(spsr);
        if self.pstate.nrw {
            panic!("We haven't implemented AArch32 support!");
        } else {
            self.pstate.daif_from_spsr(spsr);
        }
    }

    fn el_from_spsr(&mut self, spsr: u64, valid: &mut bool, bits: &mut u8) {
        let spsr_4 = (spsr >> 4) & 1;
        if spsr_4 == 0 {
            let el = get_bits_ct!(spsr, 2, 1) as u8;
            *bits = el;
            if !HAVE_AARCH64 {
                *valid = false;
            } else if !HAVE_EL {
                *valid = false;
            } else if ((spsr >> 1) & 1) == 1 {
                *valid = false;
            } else if (self.pstate.current_el == ExceptionLevel::EL0) && ((spsr & 1) == 1) {
                *valid = false;
            } else if (self.pstate.current_el == ExceptionLevel::EL2)
                && HAVE_EL
                && !IS_SECURE_EL2_ENABLED
                && ((self.scr_el3 & 1) == 0)
            {
                *valid = false;
            }
        } else if HAVE_AARCH32 {
            panic!("We haven't implemented AArch32 support!");
        } else {
            *valid = false;
        }
        if !*valid {
            panic!("Not a valid exception!");
        }
    }

    pub fn aarch64_exception_return(&mut self, new_pc_in: u64, spsr: u64) {
        let illegal_psr_state = self.illegal_exception_return(spsr);
        self.pstate_from_spsr(spsr, illegal_psr_state);

        self.event_register = true;
        self.pc = new_pc_in;
    }

    fn illegal_exception_return(&mut self, spsr: u64) -> bool {
        let mut valid = false;
        let mut target = 0;
        self.el_from_spsr(spsr, &mut valid, &mut target);
        if !valid {
            return true;
        }
        if ExceptionLevel::from_bits(target) > self.pstate.current_el {
            return true;
        }
        return false;
    }
}

pub enum PstateField {
    Daifset,
    Daifclr,
    Pan,
    Uao,
    Dit,
    Ssbs,
    Tco,
    Svcrsm,
    Svcrza,
    Svcrsmza,
    Allint,
    Pm,
    Sp,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PState {
    /// Negative
    pub n: bool,
    /// Zero
    pub z: bool,
    /// Carry
    pub c: bool,
    /// Overflow
    pub v: bool,

    /// Stack pointer
    pub sp: u8,
    pub current_el: ExceptionLevel,

    /// Debug
    pub d: bool,
    /// Async
    pub a: bool,
    /// IRQ
    pub i: bool,
    /// FIQ
    pub f: bool,

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
    fn nzcv_from_spsr(&mut self, spsr: u64) {
        self.n = get_bits_ct!(spsr, 31, 1) != 0;
        self.z = get_bits_ct!(spsr, 30, 1) != 0;
        self.c = get_bits_ct!(spsr, 29, 1) != 0;
        self.v = get_bits_ct!(spsr, 28, 1) != 0;
    }
    fn daif_from_spsr(&mut self, spsr: u64) {
        self.d = get_bits_ct!(spsr, 9, 1) != 0;
        self.a = get_bits_ct!(spsr, 8, 1) != 0;
        self.i = get_bits_ct!(spsr, 7, 1) != 0;
        self.f = get_bits_ct!(spsr, 6, 1) != 0;
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExceptionLevel {
    EL0,
    EL1,
    EL2,
    #[default]
    EL3,
}

impl ExceptionLevel {
    fn from_bits(bits: u8) -> Self {
        match bits {
            0b0 => Self::EL0,
            0b01 => Self::EL1,
            0b10 => Self::EL2,
            0b11 => Self::EL3,
            _ => panic!("Invalid bits"),
        }
    }
    #[inline]
    pub const fn is_el0(self) -> bool {
        matches!(self, Self::EL0)
    }
    #[inline]
    pub const fn is_el1(self) -> bool {
        matches!(self, Self::EL1)
    }
    #[inline]
    pub const fn is_el2(self) -> bool {
        matches!(self, Self::EL2)
    }
    #[inline]
    pub const fn is_el3(self) -> bool {
        matches!(self, Self::EL3)
    }
}

macro_rules! msr_enum {
    ($($variant:ident = $value:expr),* $(,)?) => {
        #[repr(u64)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum MsrRegisters {
            $($variant = $value,)*
            Unknown = 99999999999,
        }

        impl From<u64> for MsrRegisters {
            #[inline(always)]
            fn from(v: u64) -> Self {
                match v {
                    $($value => Self::$variant,)*
                    _ => Self::Unknown,
                }
            }
        }

        impl From<MsrRegisters> for u64 {
            #[inline(always)]
            fn from(r: MsrRegisters) -> u64 { r as u64 }
        }
    };
}

msr_enum! {
    ElrEl3  = 12985827329,
    SpsrEl3 = 12985827328,
    ScrEl3  = 12985630976,
    SpEl1   = 12952273152,
}
