use std::{
    io::{Write, stdout},
    sync::{
        Arc, Condvar, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    get_bits_ct, gic::InterruptState, instruction::UNDEF_PANIC, memory::PhyMemStatus, mmu::Mmu,
    utils::*,
};
pub const MEM_TOP: usize = crate::memory::MEMORY_SIZE;

static START: OnceLock<Instant> = OnceLock::new();
pub const MAX_SLEEP_NS: u64 = 80 * 1000 * 1000;
pub const CNTFRQ: u64 = 24_000_000;
pub const BATCH: u32 = 100;

pub const INSTRUCTION_SIZE: u64 = 4;

const GPRS: usize = 32;
const VPRS: usize = 32;
const ZERO_REG: usize = 31;
pub const SP_REGISTER: usize = 31;

const HAVE_AARCH64: bool = true;
const HAVE_EL: bool = true;
const HAVE_AARCH32: bool = false;
const _USING_AARCH32: bool = false;
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

#[derive(Default, Debug)]
pub struct Timer {
    /// Physical Timer Compare Value (CNTP_CVAL_EL0)
    pub cntp_cval_el0: u64,
    /// Physical Timer Control (CNTP_CTL_EL0)
    /// Bit 0 —> ENABLE
    /// Bit 1 —> IMASK
    /// Bit 2 —> ISTATUS
    pub cntp_ctl_el0: u32,

    /// Virtual Timer Compare Value (CNTV_CVAL_EL0) (in ticks)
    pub ctnv_cval_el0: u64,
    /// Virtual Timer Control (CNTV_CTL_EL0) (in ticks)
    pub cntv_ctl_el0: u64,

    // TODO: Handle this, it
    /// Counter-timer Kernel Control Register
    /// Bit 0 —> Allow EL0 to read physical_counter
    /// Bit 1 —> Allow EL0 to read virtual_counter
    pub cntkctl_el1: u64,

    /// Physical Counter (CNTPCT_EL0)
    /// Instead of increasing this we just call
    /// cntp_now()
    pub cntp_ct_el0: u64,

    /// Physical Timer Expiry Time (nanoseconds)
    /// Translation of CVAL value in host time
    /// This doesn't exist in hardware.
    /// It's emulator trick
    pub cntp_expiry_ns: AtomicU64,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ExclusiveMonitor {
    pub address_size: Option<(u64, u8)>,
}

impl ExclusiveMonitor {
    pub fn off(&mut self) {
        self.address_size = None;
    }
    pub fn set(&mut self, address: u64, size: u8) {
        self.address_size = Some((address, size));
    }
    pub fn safe(self, address: u64, size: u8) -> bool {
        self.address_size.is_some_and(|a| address == a.0 && size == a.1)
    }
}

#[derive(Default, Debug)]
pub struct Cpu {
    // ========================================================================
    // CORE EXECUTION STATE
    // ========================================================================
    /// General Purpose Registers X0-X30 (X31 is XZR when read, SP when used as
    /// base)
    pub x: [u64; GPRS],
    /// Vector Registers V0-V31
    pub v: [u128; VPRS],
    /// Program Counter (address of currently executing instruction)
    pub pc: u64,
    /// Process State (condition flags, exception level, interrupt masks)
    pub pstate: PState,
    /// Provides floating-point system status information.
    pub fpsr: u64,
    /// Controls floating-point behavior.
    pub fpcr: u64,

    /// Branch taken flag (set by branch instructions)
    pub branch_taken: bool,
    /// Branch target address (where to jump if branch_taken is true)
    pub branch_target: u64,
    /// Event Register (set by SEV, cleared by WFE)
    pub event_register: bool,

    // ========================================================================
    // EXCEPTION LEVEL BANKED REGISTERS
    // ========================================================================

    // --- Stack Pointers (one per exception level) ---
    sp_el0: u64,
    sp_el1: u64,
    sp_el2: u64,
    sp_el3: u64,

    // --- Exception Link Registers (return address after exception) ---
    pub elr_el1: u64,
    pub elr_el2: u64,
    pub elr_el3: u64,

    // --- Saved Program Status Registers (saved PSTATE on exception entry) ---
    pub spsr_el1: u64,
    pub spsr_el2: u64,
    pub spsr_el3: u64,

    // ========================================================================
    // SYSTEM CONTROL & CONFIGURATION
    // ========================================================================
    /// System Control Register EL1 (MMU enable, cache enable, alignment
    /// checks)
    pub sctlr_el1: u64,
    /// System Control Register EL2 (unused, reserved for future EL2 support)
    _sctlr_el2: u64,
    /// System Control Register EL3 (unused, reserved for future EL3 support)
    _sctlr_el3: u64,
    /// Secure Configuration Register EL3
    scr_el3: u64,
    /// Memory Attribute Indirection Register EL1 (memory type encoding)
    mair_el1: u64,
    /// Architectural Feature Access Control Register EL1
    cpacr_el1: u64,
    /// Monitor Debug System Control Register EL1
    mdscr_el1: u64,
    /// Cache Level ID Register EL1
    pub clidr_el1: u64,
    /// Used to lock or unlock the OS Lock.
    pub oslar_el1: u64,

    // ========================================================================
    // EXCEPTION & INTERRUPT HANDLING
    // ========================================================================
    /// Vector Base Address Register EL1 (base address of exception vector
    /// table)
    pub vbar_el1: u64,
    /// Exception Syndrome Register EL1 (exception class and details)
    pub esr_el1: u64,
    /// Fault Address Register EL1 (virtual address that caused fault)
    pub far_el1: u64,
    /// Physical Address Register EL1 (result of AT address translation
    /// instruction)
    pub par_el1: u64,

    /// Pending interrupt lines (bitfield, bit N = IRQ line N pending)
    pub pending_irq: AtomicU32,
    /// CPU sleeping state (true when WFI executed, cleared by interrupt/event)
    pub sleeping: AtomicBool,
    /// Condition variable for waking sleeping CPU thread
    pub condvar: Arc<(Mutex<bool>, Condvar)>,

    // ========================================================================
    // THREAD & CONTEXT IDENTIFICATION
    // ========================================================================
    /// Thread Pointer ID Register EL1 (kernel-managed thread pointer)
    tpidr_el1: u64,
    /// Thread Pointer ID Register EL0 (user-space thread pointer, read-write)
    tpidr_el0: u64,
    /// Thread Pointer ID Register EL0 Read-Only (user-space TLS, read-only
    /// from EL0)
    tpidrro_el0: u64,

    // ========================================================================
    // CPU IDENTIFICATION REGISTERS
    // ========================================================================
    /// Main ID Register (implementer, variant, architecture, part number,
    /// revision)
    pub midr_el1: u64,
    /// Multiprocessor Affinity Register (CPU topology)
    pub mpidr_el1: u64,
    /// Revision ID Register (implementation-specific revision info)
    pub revidr_el1: u64,
    /// Auxiliary ID Register (implementation-specific auxiliary info)
    pub aidr_el1: u64,

    /// Cache Type Register EL0 (cache line sizes, cache organization)
    ctr_el0: u64,
    /// Data Cache Zero ID Register (DC ZVA block size)
    dczid_el0: u64,

    // ========================================================================
    // FEATURE IDENTIFICATION REGISTERS
    // ========================================================================

    // --- Processor Feature Registers (general CPU features) ---
    id_aa64pfr0_el1: u64,
    id_aa64pfr1_el1: u64,
    id_aa64pfr2_el1: u64,

    // --- Debug Feature Registers ---
    id_aa64dfr0_el1: u64,
    id_aa64dfr1_el1: u64,
    id_dfr0_el1: u64, // AArch32
    id_dfr1_el1: u64, // AArch32

    // --- Instruction Set Attribute Registers ---
    id_aa64isar0_el1: u64,
    id_aa64isar1_el1: u64,
    id_aa64isar2_el1: u64,
    id_aa64isar3_el1: u64,
    id_isar0_el1: u64, // AArch32

    // --- Memory Model Feature Registers ---
    id_aa64mmfr0_el1: u64,
    id_aa64mmfr1_el1: u64,
    pub id_aa64mmfr2_el1: u64,
    id_aa64mmfr3_el1: u64,
    pub id_aa64mmfr4_el1: u64,

    // --- SVE/SME/FP Feature Registers ---
    id_aa64zfr0_el1: u64,  // SVE features
    id_aa64smfr0_el1: u64, // SME features
    id_aa64fpfr0_el1: u64, // Floating-point features

    // ========================================================================
    // TIMER
    // ========================================================================
    /// Physical and Virtual Timer state (CNTP_*, CNTV_*)
    pub timer: Timer,

    // ========================================================================
    // MEMORY SYSTEM
    // ========================================================================
    /// MMU and page table walker (handles address translation)
    pub mmu: Mmu,
    /// Exclusive access monitor (for LDXR/STXR synchronization)
    pub monitor: ExclusiveMonitor,

    // ========================================================================
    // DEBUG
    // ========================================================================
    pub uart_debug: String,
}

impl Cpu {
    pub fn init() -> Self {
        let mut cpu = Self { uart_debug: String::new(), ..Default::default() };
        cpu.pstate.sp = 1;

        cpu.sp_el0 = 0x100000; // 1MB
        cpu.sp_el1 = 0x10000000 - 0x10000; // 256MB - 64KB
        cpu.sp_el2 = 0x10000000 - 0x20000; // 256MB - 128KB
        cpu.sp_el3 = 0x10000000 - 0x30000; // 256MB - 192KB

        // System identification registers
        // DebugVer=9 (Debugv8p4), but BRPs=0, WRPs=0, CTX_CMPs=0 (no debug
        // registers)
        cpu.id_aa64dfr0_el1 = 0x000f00f000000009;
        cpu.id_aa64pfr0_el1 = 0x11;
        cpu.id_aa64dfr1_el1 = 0;
        cpu.midr_el1 = 0x410F0510;
        // We don't have CACHE
        cpu.clidr_el1 = 0;
        cpu.revidr_el1 = 0;
        cpu.aidr_el1 = 0;
        cpu.id_aa64mmfr0_el1 = 0x000000000F000020;
        cpu.id_aa64mmfr1_el1 = 0;
        cpu.id_aa64isar0_el1 = 0;
        cpu.id_aa64mmfr3_el1 = 0;
        cpu.id_aa64isar1_el1 = 0;
        cpu.id_aa64isar2_el1 = 0;
        cpu.id_aa64isar3_el1 = 0;
        cpu.id_aa64mmfr2_el1 = 0;
        cpu.id_aa64mmfr4_el1 = 0x000000000F000000;
        cpu.id_aa64pfr1_el1 = 0;
        cpu.id_aa64pfr2_el1 = 0;
        cpu.id_aa64zfr0_el1 = 0;
        cpu.id_aa64smfr0_el1 = 0;
        cpu.id_aa64fpfr0_el1 = 0;
        cpu.id_dfr0_el1 = 0;
        cpu.id_dfr1_el1 = 0;
        cpu.id_isar0_el1 = 0;
        cpu.dczid_el0 = 0x14;
        cpu.ctr_el0 = 0x34448004;
        cpu.mpidr_el1 = 0x8000_0000;

        // Boot at EL1 (kernel mode)
        cpu.pstate.current_el = ExceptionLevel::EL1;

        // System Control Register - MMU off, caches configured for boot
        cpu.sctlr_el1 = 0x00C50838;

        cpu.monitor.off();
        cpu
    }

    pub fn address_for_rn(&self, rn: u8) -> u64 {
        if rn == 31 { self.sp_read() } else { self.x_read(rn.into(), 64) }
    }

    pub fn ret_sp_or_reg(&self, rn: u8) -> u64 {
        if rn == 31 { self.sp_read() } else { self.x_read(rn.into(), 64) }
    }

    pub fn wback_address_write(&mut self, rn: u8, size: u8, address: u64) {
        if rn == 31 {
            self.sp_write(address);
        } else {
            self.x_write(rn.into(), address, size == 32);
        }
    }

    pub fn handle_wback_postindex(
        &mut self,
        postindex: bool,
        address: u64,
        offset: u64,
        size: u8,
        rn: u8,
    ) {
        let mut address = address;
        if postindex {
            address = address.wrapping_add(offset);
        }
        self.wback_address_write(rn, size, address);
    }

    pub fn memory_op_faulted(&self) -> bool {
        self.mmu.faulted
    }

    pub fn read_memory(&mut self, address: usize, size: usize) -> (PhyMemStatus, u64) {
        self.mmu.mmu_read(address, size, self.pstate.current_el)
    }

    pub fn write_memory(&mut self, address: usize, size: usize, value: u64) -> PhyMemStatus {
        self.mmu.mmu_write(address, size, value, self.pstate.current_el)
    }

    pub fn write_memory_128bit(&mut self, address: usize, value: u128) -> PhyMemStatus {
        self.mmu.mmu_write_128bit(address, value, self.pstate.current_el)
    }

    pub fn read_memory_128bit(&mut self, address: usize) -> (PhyMemStatus, u128) {
        self.mmu.mmu_read_128bit(address, self.pstate.current_el)
    }

    pub fn memory_op_safe(&self, address: u64, dbytes: u8) -> bool {
        self.monitor.safe(address, dbytes)
    }

    pub fn handle_devices(&mut self) {
        // TX: drain written byte to stdout
        if self.mmu.bus.uart.dr != 0 {
            let buf = &[self.mmu.bus.uart.dr];
            stdout().write_all(buf).unwrap();
            self.uart_debug.push_str(str::from_utf8(buf).unwrap());
            stdout().flush().unwrap();
            self.mmu.bus.uart.dr = 0;
            self.mmu.bus.uart.ris.bit_set(5);
            if (self.mmu.bus.uart.ris & self.mmu.bus.uart.imsc.to_bits()) != 0 {
                self.mmu.bus.gic.set_state(33, InterruptState::Pending);
            }
            if self.uart_debug.contains("end Kernel panic") {
                UNDEF_PANIC.store(true, Ordering::Relaxed);
            }
        }

    }

    pub fn poll_uart_rx(&mut self) {
        let uart = &self.mmu.bus.uart;
        if uart.cr.rxe && uart.imsc.rxim && !uart.rx_fifo.is_full() {
            if let Some(byte) = crate::terminal::try_read_byte() {
                let uart = &mut self.mmu.bus.uart;
                uart.rx_fifo.push_back(byte);
                uart.fr.rxfe = false;
                uart.fr.rxff = uart.rx_fifo.is_full();
                uart.ris = uart.ris.bit_set(4);
                if (uart.ris & uart.imsc.to_bits()) != 0 {
                    self.mmu.bus.gic.set_state(33, InterruptState::Pending);
                }
            }
        }
    }

    pub fn handle_data_abort(&mut self, old_pc: &mut u64) {
        self.elr_el1 = *old_pc;
        self.spsr_el1 = self.spsr_from_pstate();
        self.far_el1 = self.mmu.fault_va.try_into().unwrap();
        let ec = if self.pstate.current_el == ExceptionLevel::EL0 { 0x24 } else { 0x25 };
        let wnr = if self.mmu.fault_is_write { 1u64 << 6 } else { 0 };
        self.esr_el1 = (ec << 26) | (1u64 << 25) | wnr | (self.mmu.fsc as u64);
        self.pstate.daif_disable();
        let offset = if self.pstate.current_el == ExceptionLevel::EL0 { 0x400 } else { 0x200 };
        self.pstate.set_to_exception();
        *old_pc = self.vbar_el1 + offset;
        self.mmu.faulted = false;
        self.pstate.current_el = ExceptionLevel::EL1;
    }

    pub fn handle_instruction_abort(&mut self, old_pc: &mut u64) {
        self.elr_el1 = *old_pc;
        self.spsr_el1 = self.spsr_from_pstate();
        self.far_el1 = self.mmu.fault_va.try_into().unwrap();
        let ec = if self.pstate.current_el == ExceptionLevel::EL0 { 0x20 } else { 0x21 };
        self.esr_el1 = (ec << 26) | (1u64 << 25) | (self.mmu.fsc as u64);
        self.pstate.daif_disable();
        let offset = if self.pstate.current_el == ExceptionLevel::EL0 { 0x400 } else { 0x200 };
        self.pstate.set_to_exception();
        *old_pc = self.vbar_el1 + offset;
        self.mmu.faulted = false;
        self.pstate.current_el = ExceptionLevel::EL1;
    }

    pub fn handle_interrupts(&mut self, next_pc: &mut u64) {
        let pending = self.mmu.bus.gic.has_active_and_pending_interrupt();
        if !pending || self.pstate.irq_masked() {
            return;
        }
        //UNDEF_PANIC.store(true, Ordering::SeqCst);

        // We return to current exception level
        let _cur_el = self.pstate.current_el;
        // We are only handling IRQ now
        let target_el = ExceptionLevel::EL1;
        match target_el {
            ExceptionLevel::EL0 | ExceptionLevel::EL1 => {
                self.elr_el1 = *next_pc;
                self.spsr_el1 = self.spsr_from_pstate();
            }
            ExceptionLevel::EL3 => {
                self.elr_el3 = *next_pc;
                self.spsr_el3 = self.spsr_from_pstate();
            }
            ExceptionLevel::EL2 => panic!("EL2 Is not implemented!"),
        }

        let offset = if self.pstate.current_el == ExceptionLevel::EL0 { 0x480 } else { 0x280 };
        self.pstate.il = false;
        self.pstate.current_el = target_el;
        self.pstate.set_to_exception();

        *next_pc = self.vbar_el1 + offset;
    }

    pub fn timer_device_tick(&mut self) {
        let expire = self.timer.cntp_expiry_ns.load(Ordering::Relaxed);
        if expire != 0 && monotonic_ns() >= expire {
            self.mmu.bus.gic.set_state(27, InterruptState::Pending);
            self.timer.cntp_expiry_ns.store(0, Ordering::Relaxed);
        }
    }

    /// Translate hardware ticks into host clock
    /// for compare value so we can actually check
    /// if some amount of time has passed.
    pub fn timer_rearm(&mut self) {
        // Enable is 0 && IMASK is 0 == Interrupt surpressed
        if (self.timer.cntp_ctl_el0 & 1) == 0 || (self.timer.cntp_ctl_el0 & 2) != 0 {
            self.timer.cntp_expiry_ns.store(0, Ordering::Relaxed);
            return;
        }

        let now = cntpct_now();
        let cval = self.timer.cntp_cval_el0;

        // Kernel might have wanted to trigger interrupt as soon as possible
        if now >= cval {
            self.timer.cntp_expiry_ns.store(monotonic_ns(), Ordering::Relaxed);
            return;
        }

        // Kernel wants to trigger interrupt in the future
        let delta_ticks = cval.saturating_sub(now);
        let delta_ns = ((delta_ticks as u128 * 1_000_000_000u128) / CNTFRQ as u128) as u64;
        let host_expiry = monotonic_ns().saturating_add(delta_ns);
        self.timer.cntp_expiry_ns.store(host_expiry, Ordering::Relaxed);
    }

    pub fn should_wake(&self) -> bool {
        self.mmu.bus.gic.has_active_and_pending_interrupt()
    }

    pub fn get_elr_elx(&self) -> u64 {
        if self.pstate.current_el.is_el0() || self.pstate.current_el.is_el1() {
            self.elr_el1
        } else if self.pstate.current_el.is_el2() {
            self.elr_el2
        } else {
            self.elr_el3
        }
    }
    pub fn get_spsr_elx(&self) -> u64 {
        if self.pstate.current_el.is_el0() || self.pstate.current_el.is_el1() {
            self.spsr_el1
        } else if self.pstate.current_el.is_el2() {
            self.spsr_el2
        } else {
            self.spsr_el3
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
        let stack_align_check = if self.pstate.current_el.is_el0() {
            (self.sctlr_el1 & SCTLR_SA0) != 0
        } else {
            (self.sctlr_el1 & SCTLR_A) != 0
        };
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

    pub fn v_read(&self, n: usize, width: u8) -> u128 {
        assert!(n < VPRS);
        assert!(width <= 128 && width.is_multiple_of(8));
        let mask = if width == 128 { !0u128 } else { (1u128 << width) - 1 };
        self.v[n] & mask
    }

    pub fn v_write(&mut self, destination_register: usize, width: u8, value: u128) {
        assert!(destination_register < VPRS);
        assert!(width <= 128 && width.is_multiple_of(8));
        let mask = if width == 128 { !0u128 } else { (1u128 << width) - 1 };
        self.v[destination_register] = value & mask;
    }

    pub fn v_part_write(&mut self, destination: usize, part: u8, width: u8, value: u128) {
        assert!(part == 1 || part == 0);
        if part == 0 {
            assert!(width < 128);
            self.v_write(destination, width, value);
        } else {
            assert_eq!(width, 64);
            let vreg = self.v_read(destination, 64);
            let value_half = value.bits_get(0, 64);
            self.v_write(destination, 128, (value_half << 64) | vreg);
        }
    }

    pub fn v_part_read(&mut self, destination: usize, part: u8, width: u8) -> u128 {
        assert!(part == 1 || part == 0);
        if part == 0 {
            assert!(width < 128);
            self.v_read(destination, width)
        } else {
            assert!(width == 32 || width == 64);
            let vreg = self.v_read(destination, 128);
            vreg.bits_get(64, width)
        }
    }

    /// When n == 31 we return 0
    pub fn x_read(&self, n: usize, width: u8) -> u64 {
        assert!(n < GPRS);
        assert!(width <= 64 && width.is_multiple_of(8));
        if n != ZERO_REG {
            let mask = if width == 64 { !0u64 } else { (1 << width) - 1 };
            return self.x[n] & mask;
        }
        0
    }

    pub fn x_write(&mut self, n: usize, value: u64, is_32b: bool) {
        assert!(n < GPRS);
        if n == 31 {
            return;
        }
        if is_32b {
            self.x[n] = value & 0xFFFF_FFFF;
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

        let el0_accessible = matches!(
            register,
            MsrRegisters::TpidrEl0
                | MsrRegisters::TpidrroEl0
                | MsrRegisters::CntpCvalEl0
                | MsrRegisters::CntpCtlEl0
                | MsrRegisters::Fpcr
                | MsrRegisters::Fpsr
        );

        if !self.pstate.current_el.is_el1() && !el0_accessible {
            println!(
                "\r\n[MSR] EL0 trying to write privileged register: {:?} (comp={:#x})",
                register, comp
            );
            UNDEF_PANIC.store(true, Ordering::SeqCst);
        }
        match register {
            MsrRegisters::Unknown => {
                UNDEF_PANIC.store(true, Ordering::Relaxed);
                println!("\r\nValue {comp} not convered, please check the ARM docs!")
            }
            MsrRegisters::ElrEl3 => {
                self.elr_el3 = self.x_read(t.into(), 64);
            }
            MsrRegisters::SpsrEl3 => {
                self.spsr_el3 = self.x_read(t.into(), 64);
            }
            MsrRegisters::ScrEl3 => {
                self.scr_el3 = self.x_read(t.into(), 64);
            }
            MsrRegisters::SpEl1 => {
                self.sp_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::CntpCvalEl0 => {
                self.timer.cntp_cval_el0 = self.x_read(t.into(), 64);
                self.timer_rearm();
            }
            MsrRegisters::CntpCtlEl0 => {
                self.timer.cntp_ctl_el0 = self.x_read(t.into(), 32) as u32;
                self.timer_rearm();
            }
            MsrRegisters::VbarEl1 => {
                let val = self.x_read(t.into(), 64);
                self.vbar_el1 = val & !0x7FFu64;
            }
            MsrRegisters::SctlrEl1 => {
                self.sctlr_el1 = self.x_read(t.into(), 64);
                self.mmu.enabled = (self.sctlr_el1 & SCTLR_M) != 0;
            }
            MsrRegisters::SpsrEl1 => {
                self.spsr_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::ElrEl1 => {
                self.elr_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::CpacrEl1 => {
                self.cpacr_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::MdscrEl1 => {
                self.mdscr_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::MairEl1 => {
                self.mair_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::TcrEl1 => {
                self.mmu.tcr_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::Ttbr0El1 => {
                self.mmu.ttbr0_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::Ttbr1El1 => {
                self.mmu.ttbr1_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::SpEl0 => {
                self.sp_el0 = self.x_read(t.into(), 64);
            }
            MsrRegisters::TpidrEl1 => {
                self.tpidr_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::TpidrEl0 => {
                self.tpidr_el0 = self.x_read(t.into(), 64);
            }
            MsrRegisters::TpidrroEl0 => self.tpidrro_el0 = self.x_read(t.into(), 64),
            MsrRegisters::Daif => {
                self.pstate.daif_from_spsr(self.x_read(t.into(), 64));
            }
            MsrRegisters::FarEl1 => {
                // AArch64-far_el1.xml: allow software writes
                self.far_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::ParEl1 => {
                // AArch64-par_el1.xml: allow software writes
                self.par_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::CntvCtlEl0 => {
                self.timer.cntp_ctl_el0 = self.x_read(t.into(), 64) as u32;
                self.timer_rearm();
            }
            MsrRegisters::CntvCvalEl0 => {
                self.timer.cntp_cval_el0 = self.x_read(t.into(), 64);
                self.timer_rearm();
            }
            MsrRegisters::CntkctlEl1 => self.timer.cntkctl_el1 = self.x_read(t.into(), 64),
            MsrRegisters::ClidrEl1 => {
                self.clidr_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::OslarEl1 => {
                self.oslar_el1 = self.x_read(t.into(), 64);
            }
            MsrRegisters::Fpsr => self.fpsr = self.x_read(t.into(), 64),
            MsrRegisters::Fpcr => self.fpcr = self.x_read(t.into(), 64),
            MsrRegisters::OsdlrEl1
            | MsrRegisters::Dbgbvr0El1
            | MsrRegisters::Dbgbcr0El1
            | MsrRegisters::Dbgwvr0El1
            | MsrRegisters::Dbgwcr0El1 => {
                // RAZ/WI
            }
        }
    }

    pub fn sys_reg_read(
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

        let register: MrsRegisters = comp.into();
        match register {
            MrsRegisters::Unknown => {
                UNDEF_PANIC.store(true, Ordering::Relaxed);
                println!("\r\nValue {comp} not convered, please check the ARM docs!")
            }
            MrsRegisters::CntfrqEl0 => {
                if !self.pstate.current_el.is_el0() {
                    self.x_write(t.into(), CNTFRQ, false);
                } else {
                    panic!("Please implement CntfrqEl0 access for EL0")
                }
            }
            // Counter value
            MrsRegisters::CntpctEl0 => {
                if !self.pstate.current_el.is_el0() {
                    self.x_write(t.into(), cntpct_now(), false);
                } else {
                    panic!("Please implement CntpctEl0 access for EL0");
                }
            }
            // Timer physical control
            MrsRegisters::CntpCtlEl0 => {
                if !self.pstate.current_el.is_el0() && !self.pstate.current_el.is_el2() {
                    let mut ctl = self.timer.cntp_ctl_el0 as u64;
                    if (ctl & 1) != 0 && cntpct_now() >= self.timer.cntp_cval_el0 {
                        ctl |= 1 << 2; // ISTATUS only when ENABLE=1
                    }
                    self.x_write(t.into(), ctl, false);
                } else {
                    panic!("Please implement CntpctEl0 access for EL0");
                }
            }
            MrsRegisters::CurrentEL => {
                if self.pstate.current_el.is_el0() {
                    panic!("Undefined")
                }
                if self.pstate.current_el.is_el1() {
                    self.x_write(t.into(), self.pstate.current_el.to_currentel_value(), false);
                }
            }
            MrsRegisters::SctlrEl1 => {
                if self.pstate.current_el.is_el0() {
                    panic!("Undefined")
                }
                if self.pstate.current_el.is_el1() {
                    self.x_write(t.into(), self.sctlr_el1, false);
                }
            }
            MrsRegisters::CtrEl0 => {
                self.x_write(t.into(), self.ctr_el0, false);
            }
            MrsRegisters::IdAa64dfr0El1 => {
                self.x_write(t.into(), self.id_aa64dfr0_el1, false);
            }
            MrsRegisters::IdAa64pfr0El1 => {
                self.x_write(t.into(), self.id_aa64pfr0_el1, false);
            }
            MrsRegisters::MidrEl1 => {
                if self.pstate.current_el.is_el1() {
                    self.x_write(t.into(), self.midr_el1, false);
                }
            }
            MrsRegisters::IdAa64mmfr0El1 => {
                if self.pstate.current_el.is_el1() {
                    self.x_write(t.into(), self.id_aa64mmfr0_el1, false);
                }
            }
            MrsRegisters::IdAa64mmfr1El1 => {
                if self.pstate.current_el.is_el1() {
                    self.x_write(t.into(), self.id_aa64mmfr1_el1, false);
                }
            }
            MrsRegisters::IdAa64mmfr3El1 => {
                if self.pstate.current_el.is_el1() {
                    self.x_write(t.into(), self.id_aa64mmfr3_el1, false);
                }
            }
            MrsRegisters::IdAa64isar0El1 => {
                let v = self.id_aa64isar0_el1;
                self.x_write(t.into(), v, false);
            }
            MrsRegisters::DczidEl0 => {
                self.x_write(t.into(), self.dczid_el0, false);
            }
            MrsRegisters::TcrEl1 => {
                self.x_write(t.into(), self.mmu.tcr_el1, false);
            }
            MrsRegisters::IdAa64isar1El1 => {
                self.x_write(t.into(), self.id_aa64isar1_el1, false);
            }
            MrsRegisters::IdAa64isar2El1 => {
                self.x_write(t.into(), self.id_aa64isar2_el1, false);
            }
            MrsRegisters::IdAa64pfr1El1 => {
                self.x_write(t.into(), self.id_aa64pfr1_el1, false);
            }
            MrsRegisters::MpidrEl1 => {
                self.x_write(t.into(), self.mpidr_el1, false);
            }
            MrsRegisters::SpEl0 => {
                self.x_write(t.into(), self.sp_el0, false);
            }
            MrsRegisters::Daif => {
                self.x_write(t.into(), self.pstate.daif_to_64bit(), false);
            }
            MrsRegisters::TpidrEl1 => {
                self.x_write(t.into(), self.tpidr_el1, false);
            }
            MrsRegisters::TpidrEl0 => {
                self.x_write(t.into(), self.tpidr_el0, false);
            }
            MrsRegisters::ElrEl1 => {
                self.x_write(t.into(), self.elr_el1, false);
            }
            MrsRegisters::SpsrEl1 => {
                self.x_write(t.into(), self.spsr_el1, false);
            }
            MrsRegisters::EsrEl1 => {
                self.x_write(t.into(), self.esr_el1, false);
            }
            MrsRegisters::FarEl1 => {
                self.x_write(t.into(), self.far_el1, false);
            }
            MrsRegisters::ParEl1 => {
                self.x_write(t.into(), self.par_el1, false);
            }
            MrsRegisters::CntvctEl0 => {
                self.x_write(t.into(), cntpct_now(), false);
            }
            MrsRegisters::TpidrroEl0 => {
                self.x_write(t.into(), self.tpidrro_el0, false);
            }
            MrsRegisters::RevidrEl1 => {
                self.x_write(t.into(), self.revidr_el1, false);
            }
            MrsRegisters::AidrEl1 => {
                self.x_write(t.into(), self.aidr_el1, false);
            }
            MrsRegisters::IdAa64dfr1El1 => {
                self.x_write(t.into(), self.id_aa64dfr1_el1, false);
            }
            MrsRegisters::IdAa64isar3El1 => {
                self.x_write(t.into(), self.id_aa64isar3_el1, false);
            }
            MrsRegisters::IdAa64mmfr2El1 => {
                self.x_write(t.into(), self.id_aa64mmfr2_el1, false);
            }
            MrsRegisters::IdAa64mmfr4El1 => {
                self.x_write(t.into(), self.id_aa64mmfr4_el1, false);
            }
            MrsRegisters::IdAa64pfr2El1 => {
                self.x_write(t.into(), self.id_aa64pfr2_el1, false);
            }
            MrsRegisters::IdAa64zfr0El1 => {
                self.x_write(t.into(), self.id_aa64zfr0_el1, false);
            }
            MrsRegisters::IdAa64smfr0El1 => {
                self.x_write(t.into(), self.id_aa64smfr0_el1, false);
            }
            MrsRegisters::IdAa64fpfr0El1 => {
                self.x_write(t.into(), self.id_aa64fpfr0_el1, false);
            }
            MrsRegisters::IdDfr0El1 => {
                self.x_write(t.into(), self.id_dfr0_el1, false);
            }
            MrsRegisters::IdDfr1El1 => {
                self.x_write(t.into(), self.id_dfr1_el1, false);
            }
            MrsRegisters::IdIsar0El1 => {
                self.x_write(t.into(), self.id_isar0_el1, false);
            }
            MrsRegisters::CntvCtlEl0 => {
                // Since virtual timer is alias for hardware in our emulator
                let mut ctl = self.timer.cntp_ctl_el0 as u64;
                if (ctl & 1) != 0 && cntpct_now() >= self.timer.cntp_cval_el0 {
                    ctl |= 1 << 2; // ISTATUS only when ENABLE=1
                }
                self.x_write(t.into(), ctl, false);
            }
            MrsRegisters::CntvCvalEl0 => {
                self.x_write(t.into(), self.timer.cntp_cval_el0, false);
            }
            MrsRegisters::CntkctlEl1 => {
                self.x_write(t.into(), self.timer.cntkctl_el1, false);
            }
            MrsRegisters::ClidrEl1 => {
                self.x_write(t.into(), self.clidr_el1, false);
            }
            MrsRegisters::CpacrEl1 => {
                self.x_write(t.into(), self.cpacr_el1, false);
            }
            MrsRegisters::Ttbr1El1 => {
                self.x_write(t.into(), self.mmu.ttbr1_el1, false);
            }
            MrsRegisters::OslarEl1 => {
                self.x_write(t.into(), self.oslar_el1, false);
            }
            MrsRegisters::Fpsr => {
                self.x_write(t.into(), self.fpsr, false);
            }
            MrsRegisters::Fpcr => {
                self.x_write(t.into(), self.fpcr, false);
            }
            MrsRegisters::OsdlrEl1
            | MrsRegisters::Dbgbvr0El1
            | MrsRegisters::Dbgbcr0El1
            | MrsRegisters::Dbgwvr0El1
            | MrsRegisters::Dbgwcr0El1 => {
                // RAZ/WI
                self.x_write(t.into(), 0, false);
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
                self.pstate.sp = get_bits_ct!(spsr, 0, 1) as u8;
            }
        }

        if self.pstate.il && self.pstate.nrw {
            panic!("We haven't implemented AArch32 support!");
        }
        self.pstate.nzcv_from_spsr(spsr);
        if self.pstate.nrw {
            panic!("We haven't implemented AArch32 support!");
        } else {
            self.pstate.daif_from_spsr(spsr);
        }
    }

    pub fn spsr_from_pstate(&mut self) -> u64 {
        let mut spsr = 0;
        if self.pstate.n {
            spsr |= 1 << 31
        };
        if self.pstate.z {
            spsr |= 1 << 30;
        }
        if self.pstate.c {
            spsr |= 1 << 29;
        }
        if self.pstate.v {
            spsr |= 1 << 28;
        }
        if self.pstate.masked_d {
            spsr |= 1 << 9;
        }
        if self.pstate.masked_a {
            spsr |= 1 << 8;
        }
        if self.pstate.masked_i {
            spsr |= 1 << 7;
        }
        if self.pstate.masked_f {
            spsr |= 1 << 6;
        }

        let m = match (self.pstate.current_el, self.pstate.sp) {
            (ExceptionLevel::EL0, 0) => 0b0000,
            (ExceptionLevel::EL1, 0) => 0b0100,
            (ExceptionLevel::EL1, 1) => 0b0101,
            _ => panic!("Pstate: ({:?}-{:?}) not covered", self.pstate.current_el, self.pstate.sp),
        };
        spsr |= m;

        spsr
    }

    #[allow(clippy::if_same_then_else)]
    fn el_from_spsr(&mut self, spsr: u64, valid: &mut bool, bits: &mut u8) {
        let spsr_4 = (spsr >> 4) & 1;
        if spsr_4 == 0 {
            let el = get_bits_ct!(spsr, 2, 2) as u8;
            *bits = el;
            if !HAVE_AARCH64 {
                *valid = false;
            } else if !HAVE_EL {
                *valid = false;
            } else if ((spsr >> 1) & 1) == 1 {
                *valid = false;
            } else if (self.pstate.current_el == ExceptionLevel::EL0) && ((spsr & 1) == 1) {
                *valid = false;
            } else {
                *valid = !((self.pstate.current_el == ExceptionLevel::EL2)
                    && HAVE_EL
                    && !IS_SECURE_EL2_ENABLED
                    && ((self.scr_el3 & 1) == 0));
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
        self.branch_taken = true;
        self.branch_target = new_pc_in;
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
        false
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
    pub masked_d: bool,
    /// Async
    pub masked_a: bool,
    /// IRQ
    pub masked_i: bool,
    /// FIQ
    pub masked_f: bool,

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
        self.masked_d = get_bits_ct!(spsr, 9, 1) != 0;
        self.masked_a = get_bits_ct!(spsr, 8, 1) != 0;
        self.masked_i = get_bits_ct!(spsr, 7, 1) != 0;
        self.masked_f = get_bits_ct!(spsr, 6, 1) != 0;
    }

    pub fn set_flags_from_bits(&mut self, bits: u8) {
        self.n = (bits & (1 << 3)) != 0;
        self.z = (bits & (1 << 2)) != 0;
        self.c = (bits & (1 << 1)) != 0;
        self.v = (bits & 1) != 0;
    }

    pub fn daif_to_bits(&self) -> u8 {
        let mut bits: u8 = 0;
        if self.masked_d {
            bits |= 1 << 3;
        }
        if self.masked_a {
            bits |= 1 << 2;
        }
        if self.masked_i {
            bits |= 1 << 1;
        }
        if self.masked_f {
            bits |= 1;
        }
        bits
    }

    pub fn daif_to_64bit(&self) -> u64 {
        let bits = self.daif_to_bits() as u64;
        bits << 6
    }

    #[inline]
    fn irq_masked(&self) -> bool {
        self.masked_i
    }

    pub fn daif_disable(&mut self) {
        self.masked_d = true;
        self.masked_a = true;
        self.masked_i = true;
        self.masked_f = true;
    }

    pub fn set_to_exception(&mut self) {
        self.daif_disable();
        self.il = false;
        self.sp = 1;
    }
}

#[repr(u8)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExceptionLevel {
    EL0 = 0b00,
    EL1 = 0b01,
    EL2 = 0b10,
    #[default]
    EL3 = 0b11,
}

impl ExceptionLevel {
    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => Self::EL0,
            0b01 => Self::EL1,
            0b10 => Self::EL2,
            0b11 => Self::EL3,
            _ => panic!("Invalid bits"),
        }
    }

    pub const fn bits2(self) -> u8 {
        self as u8 & 0b11
    }

    pub const fn from_currentel_value(currentel: u64) -> Self {
        Self::from_bits(((currentel >> 2) & 0b11) as u8)
    }

    #[inline]
    pub const fn to_currentel_value(self) -> u64 {
        (self.bits2() as u64) << 2
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
    // Debug registers (RAZ/WI - n=0 only since BRPs=0, WRPs=0)
    Dbgbvr0El1 = 8589934596,
    Dbgbcr0El1 = 8589934597,
    Dbgwvr0El1 = 8589934598,
    Dbgwcr0El1 = 8589934599,
    Fpsr = 12935496705,
    Fpcr = 12935496704,
    OsdlrEl1 = 8590000900,
    OslarEl1 = 8590000132,
    Daif = 12935496193,
    ElrEl3  = 12985827329,
    SpsrEl3 = 12985827328,
    ScrEl3  = 12985630976,
    SpEl1   = 12952273152,
    CntpCvalEl0 = 12936151554,
    CntpCtlEl0 =  12936151553,
    VbarEl1 = 12885688320,
    SctlrEl1 = 12884967424,
    SpsrEl1 = 12885164032,
    ElrEl1 = 12885164033,
    CpacrEl1 = 12884967426,
    MdscrEl1 = 8589935106,
    MairEl1 = 12885557760,
    TcrEl1 = 12885032962,
    Ttbr0El1 = 12885032960,
    Ttbr1El1 = 12885032961,
    SpEl0 = 12885164288,
    TpidrEl1 = 12885753860,
    TpidrEl0 = 12936085506,
    TpidrroEl0 = 12936085507,
    FarEl1   = 12885295104,
    ParEl1   = 12885361664,
    CntvCtlEl0 = 12936151809,
    CntvCvalEl0 = 12936151810,
    CntkctlEl1 = 12885819648,
    ClidrEl1 = 12901679105,
}

macro_rules! mrs_enum {
    ($($variant:ident = $value:expr),* $(,)?) => {
        #[repr(u64)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum MrsRegisters {
            $($variant = $value,)*
            Unknown = 99999999999,
        }

        impl From<u64> for MrsRegisters {
            #[inline(always)]
            fn from(v: u64) -> Self {
                match v {
                    $($value => Self::$variant,)*
                    _ => Self::Unknown,
                }
            }
        }

        impl From<MrsRegisters> for u64 {
            #[inline(always)]
            fn from(r: MrsRegisters) -> u64 { r as u64 }
        }
    };
}

mrs_enum! {
    // Debug registers (RAZ/WI - n=0 only since BRPs=0, WRPs=0)
    Dbgbvr0El1 = 8589934596,
    Dbgbcr0El1 = 8589934597,
    Dbgwvr0El1 = 8589934598,
    Dbgwcr0El1 = 8589934599,

    Fpcr = 12935496704,
    Fpsr = 12935496705,
    TpidrEl1 = 12885753860,
    TpidrEl0 = 12936085506,
    TpidrroEl0 = 12936085507,
    Daif = 12935496193,
    SpEl0 = 12885164288,
    SpsrEl1 = 12885164032,
    ElrEl1 = 12885164033,
    EsrEl1 = 12885230080,
    MpidrEl1 = 12884901893,
    RevidrEl1 = 12884901894,
    AidrEl1 = 12901679111,
    CntfrqEl0 = 12936151040,
    CntpctEl0 = 12936151041,
    CntpCtlEl0 = 12936151553,
    CurrentEL = 12885164546,
    SctlrEl1 = 12884967424,
    CtrEl0 = 12935233537,
    IdAa64dfr0El1 = 12884903168,
    IdAa64pfr0El1 = 12884902912,
    IdAa64dfr1El1 =12884903169,
    MidrEl1 = 12884901888,
    DczidEl0 = 12935233543,
    TcrEl1 = 12885032962,
    CntvctEl0 = 12936151042,
    CntvCtlEl0 = 12936151809,
    CntvCvalEl0 = 12936151810,
    CntkctlEl1 = 12885819648,
    ClidrEl1 = 12901679105,
    CpacrEl1 = 12884967426,
    Ttbr1El1 = 12885032961,
    OsdlrEl1 = 8590000900,
    OslarEl1 = 8590000132,

    IdAa64mmfr0El1   = 12884903680,
    IdAa64mmfr1El1   = 12884903681,
    IdAa64mmfr3El1   = 12884903683,
    IdAa64isar0El1   = 12884903424,
    IdAa64isar1El1   = 12884903425,
    IdAa64isar2El1   = 12884903426,
    IdAa64pfr1El1    = 12884902913,
    IdAa64isar3El1   = 12884903427,
    IdAa64mmfr2El1   = 12884903682,
    IdAa64mmfr4El1   = 12884903684,
    IdAa64pfr2El1    = 12884902914,
    IdAa64zfr0El1    = 12884902916,
    IdAa64smfr0El1   = 12884902917,
    IdAa64fpfr0El1   = 12884902919,
    IdDfr0El1        = 12884902146,
    IdDfr1El1        = 12884902661,
    IdIsar0El1       = 12884902400,
    FarEl1 = 12885295104,
    ParEl1 = 12885361664,
}

pub fn sleep_ns(ns: u64) {
    if ns > 0 {
        let ns = ns.min(MAX_SLEEP_NS);
        sleep(Duration::from_nanos(ns));
    }
}

pub fn monotonic_ns() -> u64 {
    let epoch = *START.get_or_init(Instant::now);
    let dur = epoch.elapsed();

    dur.as_nanos().try_into().expect("duration exceeded u64 max")
}

fn cntpct_now() -> u64 {
    ((monotonic_ns() as u128 * CNTFRQ as u128) / 1_000_000_000u128) as u64
}
