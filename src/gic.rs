#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterruptState {
    #[default]
    Inactive,
    Pending,
    Active,
    /// A processor is servicing the interrupt and the GIC has a pending
    /// interrupt from the same source.
    ActiveAndPending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Interrupt {
    pub enabled: bool,
    pub state: InterruptState,

    //  GICD_ICFGRn
    /// Default = false
    /// Default = level-sensitive
    pub edge_triggered: bool,
    pub is_1n: bool,

    pub priority: u8,
}

impl Default for Interrupt {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            state: Default::default(),
            edge_triggered: false,
            is_1n: true,
            priority: 0,
        }
    }
}

// We use 16 priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Gic {
    interrupts: [Interrupt; 1024],

    // ————— DISTRIBUTOR REGISTERS —————
    /// Offset 0x00 — GICD @ 0x80000000
    /// Master switch for entire distributor
    pub gicd_ctlr: u32, // Distributor control

    /// Offset 0x04 — GICD @ 0x80000004
    pub gicd_typer: u32,

    // ————— CPU INTERFACE REGISTERS —————
    /// Offset 0x00 — GICC @ 0x80010000
    gicc_ctlr: u32, // CPU interface control

    /// Offset 0x04 — GICC @ 0x80010004
    /// Provides an interrupt priority filter. Only interrupts with higher
    /// priority than the value in this  register are signaled to the processor.
    gicc_pmr: u32, // Priority mask

    /// Offset 0x0C — GICC @ 0x8001000C
    gicc_iar: u32, // Read -> get interrupt ID

    /// Offset 0x10 — GICC @ 0x80010010
    gicc_eoir: u32, // Write interrupt ID -> signal "I'm done"

    /// Offset 0xD0..=0xDC - GICC @ 0x800100D0
    /// Active Priorities Registers
    gicc_apr: [u32; 4],
}

impl Default for Gic {
    fn default() -> Self {
        let interrupt = Interrupt {
            enabled: false,
            state: InterruptState::Inactive,
            edge_triggered: false,
            is_1n: true,
            priority: 0,
        };
        Self {
            interrupts: [interrupt; 1024],
            gicd_ctlr: Default::default(),
            // 1 cpu
            gicd_typer: 0b0000_0000_0000_0000_0000_0000_0001_1111,
            gicc_ctlr: Default::default(),
            gicc_pmr: Default::default(),
            gicc_iar: Default::default(),
            gicc_eoir: Default::default(),
            gicc_apr: Default::default(),
        }
    }
}

impl Gic {
    /// Address is 0x80000000-0x80010FFF (GICD) or 0x80010000-0x80011FFF (GICC)
    /// Enforced by bus. We mask to get offset within GIC space.
    pub fn read(&self, addr: u64) -> u32 {
        let addr = addr & 0x1FFFF;
        match addr {
            // ----------GICD REGISTERS----------
            // GICD registers
            0x000 => self.gicd_ctlr,
            0x004 => self.gicd_typer,
            // GICD_ISENABLER
            0x100..=0x17C => {
                let req_index = (addr - 0x100) / 4; // which reg
                let base_irq = req_index * 32;

                let mut result: u32 = 0;
                for i in 0..32_usize {
                    if self.interrupts[base_irq as usize + i].enabled {
                        result |= 1 << i;
                    }
                }
                result
            }
            // GICD_ICENABLER
            0x180..=0x1FC => {
                // each reg is 4 bytes == 32 bits
                let req_index = (addr - 0x180) / 4;
                let base_irq = req_index * 32;

                let mut result: u32 = 0;
                for i in 0..32_usize {
                    if self.interrupts[base_irq as usize + i].enabled {
                        result |= 1 << i;
                    }
                }
                result
            }

            // GICD_ITARGETSR
            // "In a uniprocessor implementation, all interrupts target the one processor, and
            // the GICD_ITARGETSRs are RAZ/WI"
            0x800..=0xBFC => 0,
            // GICD_ICFGRn
            0xC00..=0xCFC => {
                // each reg is 4 bytes == 32 bits
                let req_index = (addr - 0xc00) / 4;
                let base_irq = req_index * 16;

                let mut result: u32 = 0;
                for i in 0..16_usize {
                    if self.interrupts[base_irq as usize + i].is_1n {
                        result |= 1 << (i * 2);
                    }
                    if self.interrupts[base_irq as usize + i].edge_triggered {
                        result |= 2 << (i * 2);
                    }
                }
                result
            }
            // ----------GICC REGISTERS----------
            0x10000 => self.gicc_ctlr,
            0x10004 => self.gicc_pmr,
            0x10010 => self.gicc_eoir,
            // GICC_IIDR
            0x100FC => 0x0002_043B,
            _ => panic!("Attempt to read unmapped GIC memory, addr {:x}", addr),
        }
    }

    pub fn write(&mut self, addr: u64, val: u32) {
        let addr = addr & 0x1FFFF;
        match addr {
            // ----------GICD REGISTERS----------
            // GICD_CTLR
            0x000 => {
                self.gicd_ctlr = val;
            }
            // GICD_ISENABLER - writing 1 SETS (enables) the interrupt
            0x100..=0x17C => {
                let req_index = (addr - 0x100) / 4;
                let base_irq = (req_index * 32) as usize;

                for i in 0..32 {
                    if (val >> i) & 1 == 1 {
                        self.interrupts[base_irq + i].enabled = true;
                    }
                }
            }
            // GICD_ICENABLER - writing 1 CLEARS (disables) the interrupt
            0x180..=0x1FC => {
                let req_index = (addr - 0x180) / 4;
                let base_irq = (req_index * 32) as usize;

                for i in 0..32 {
                    if (val >> i) & 1 == 1 {
                        self.interrupts[base_irq + i].enabled = false;
                    }
                }
            }
            // GICD_ICACTIVERn
            0x380..=0x3FC => {
                let req_index = (addr - 0x380) / 4;
                let base_irq = (req_index * 32) as usize;

                for i in 0..32 {
                    if (val >> i) & 1 == 1 {
                        self.interrupts[base_irq + i].state = InterruptState::Inactive;
                    }
                }
            }
            // GICD_IPRIORITYRn
            0x400..=0x7FC => {
                let mut req_index = (addr - 0x400) as usize;
                let four_pieces: [u8; 4] = val.to_le_bytes();

                for i in four_pieces.into_iter() {
                    self.interrupts[req_index].priority = i & 0b1111_0000;
                    req_index += 1;
                }
            }
            // GICD_ITARGETSRn - RAZ/WI on uniprocessor
            0x800..=0xBFC => { /* Write Ignored */ }
            // GICD_ICFGRn
            0xC00..=0xCFC => {
                let req_index = (addr - 0xc00) / 4;
                let base_irq = (req_index * 16) as usize;

                for i in 0..16 {
                    // This is ignored by Linux, but we write it anyway
                    // Linux always does N-N Model (All targeted CPUs receive the interrupt)
                    self.interrupts[base_irq + i].is_1n = (val >> (i * 2)) & 1 != 0;
                    self.interrupts[base_irq + i].edge_triggered = (val >> (i * 2 + 1)) & 1 != 0;
                }
            }
            // ----------GICC REGISTERS----------
            // GICD_CTLR
            0x10000 => {
                self.gicc_ctlr = val;
            }
            // GICC_PMR
            0x10004 => {
                self.gicc_pmr = val & 0xF0;
            }
            // GICC_APRn
            0x100D0..=0x100DC => {
                let req_index = (addr - 0x100D0) / 4;

                self.gicc_apr[req_index as usize] = val;
            }
            _ => panic!("Attempt to write unmapped GIC memory, addr {addr:x}, val {val:b}"),
        }
    }
}
