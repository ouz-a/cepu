// Register offsets (bytes from base)
pub const REG_QUEUE_BASE: u32 = 0x00;
pub const REG_HEAD: u32 = 0x04;
pub const REG_TAIL: u32 = 0x08;
pub const REG_CONTROL: u32 = 0x0C;
pub const REG_STATUS: u32 = 0x10;
pub const REG_INT_STATE: u32 = 0x14;
pub const REG_DOOR_BELL: u32 = 0x18;
#[derive(Default, Debug, Clone, Copy)]
pub enum DataType {
    F8,
    F16,
    #[default]
    F32,
    F64,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct MatMul {
    pub a_addr: u64,
    pub b_addr: u64,
    pub ret_addr: u64,
    /// rows of A and result
    pub m: u32,
    /// cols of A, rows of B
    pub k: u32,
    /// cols of B and result
    pub n: u32,
    pub data_type: DataType,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct InterruptFlags {
    pub completed: bool,
    pub error: bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ControlFlags {
    pub device_enabled: bool,
    pub interrupts_enabled: bool,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum DeviceStatus {
    #[default]
    Idle,
    Busy,
    Error,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CepuCel {
    pub base: u32,
    pub head: u16,
    pub tail: u16,
    pub door_bell: bool,
    pub control: ControlFlags,
    pub status: DeviceStatus,
    pub interrupt_state: InterruptFlags,
}

impl CepuCel {
    pub fn new() -> Self {
        Self::default()
    }
}
