use std::sync::atomic::{AtomicBool, Ordering};

use crate::{memory::MEMORY, utils::BitUtils};

// Register offsets (bytes from base)
pub const REG_QUEUE_BASE: u32 = 0x00;
pub const REG_HEAD: u32 = 0x04;
pub const REG_TAIL: u32 = 0x08;
pub const REG_CONTROL: u32 = 0x0C;
pub const REG_STATUS: u32 = 0x10;
pub const REG_INT_STATE: u32 = 0x14;
pub const REG_DOOR_BELL: u32 = 0x18;
pub const REG_BUFFER_SIZE: u32 = 0x1C;

pub static CEPU_CEL_INTERRUPT_FLAG: AtomicBool = AtomicBool::new(false);

#[derive(Default, Debug, Clone, Copy)]
pub enum DataType {
    F8,
    F16,
    #[default]
    F32,
    F64,
}

#[repr(C)]
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

impl MatMul {
    pub fn get_index(&self, addr: u64, row: u32, number_of_columns: u32, col: u32) -> usize {
        let row = row as usize;
        let k = number_of_columns as usize;
        let col = col as usize;
        let addr = addr as usize;
        addr + (row * k + col) * 4
    }

    // Size of f32 is 4
    pub fn work(&mut self) {
        let a_addr = self.a_addr;
        let b_addr = self.b_addr;
        for row in 0..self.m {
            for col in 0..self.n {
                let mut sum = 0.0;
                for i in 0..self.k {
                    let index_a = self.get_index(a_addr, row, self.k, i);
                    let index_b = self.get_index(b_addr, i, self.n, col);
                    let a = unsafe {
                        f32::from_le_bytes(MEMORY[index_a..index_a + 4].try_into().unwrap())
                    };
                    let b = unsafe {
                        f32::from_le_bytes(MEMORY[index_b..index_b + 4].try_into().unwrap())
                    };
                    sum += a * b;
                }

                let result_index = self.get_index(self.ret_addr, row, self.n, col);

                unsafe {
                    let dst = (core::ptr::addr_of_mut!(MEMORY) as *mut u8).add(result_index);
                    let bytes = sum.to_le_bytes();
                    let src = bytes.as_ptr();

                    core::ptr::copy_nonoverlapping(src, dst, 4);
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Commands {
    MatMul(MatMul),
}

#[derive(Default, Debug, Clone, Copy)]
pub struct InterruptFlags {
    pub completed: bool,
    pub error: bool,
}

impl InterruptFlags {
    pub fn to_bits(&self) -> u8 {
        let mut bits: u8 = 0;
        if self.completed {
            bits = bits.bit_set(0);
        }
        if self.error {
            bits = bits.bit_set(1);
        }
        bits
    }

    pub fn from_bits(bits: u8) -> Self {
        Self { completed: bits.single_bit(0) == 1, error: bits.single_bit(1) == 1 }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ControlFlags {
    pub device_enabled: bool,
    pub interrupts_enabled: bool,
}

impl ControlFlags {
    pub fn to_bits(&self) -> u8 {
        let mut bits: u8 = 0;
        if self.device_enabled {
            bits = bits.bit_set(0);
        }
        if self.interrupts_enabled {
            bits = bits.bit_set(1);
        }
        bits
    }

    pub fn from_bits(bits: u8) -> Self {
        Self {
            device_enabled: bits.single_bit(0) == 1,
            interrupts_enabled: bits.single_bit(1) == 1,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum DeviceStatus {
    #[default]
    Idle,
    Busy,
    Error,
}

impl DeviceStatus {
    pub fn to_bits(&self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Busy => 1,
            Self::Error => 2,
        }
    }

    pub fn from_bits(bits: u8) -> Self {
        match bits {
            0 => Self::Idle,
            1 => Self::Busy,
            2 => Self::Error,
            _ => panic!("Invalid DeviceStatus bits: {bits}"),
        }
    }
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
    pub buffer_size: u32,
}

impl CepuCel {
    pub fn get_base_head(&self) -> (usize, usize) {
        (self.base as usize, self.head as usize)
    }

    pub fn advance_then_set(&mut self) {
        assert!(self.buffer_size > 0, "CepuCel queue buffer size was not configured");

        assert!(
            self.buffer_size <= u16::MAX as u32 + 1,
            "CepuCel queue buffer size does not fit the head register"
        );

        let next_head = (u32::from(self.head) + 1) % self.buffer_size;

        self.head = next_head as u16;
        self.interrupt_state.completed = true;
        self.interrupt_state.error = false;
        self.status = DeviceStatus::Idle;

        CEPU_CEL_INTERRUPT_FLAG.store(true, Ordering::SeqCst);
    }

    pub fn get_command(base: usize, head: usize) -> Commands {
        let size_of_command = std::mem::size_of::<Commands>();
        let index_position = size_of_command * head;
        let start = base + index_position;
        let command =
            unsafe { &*(MEMORY[start..(start + size_of_command)].as_ptr() as *const Commands) };
        *command
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self, address: usize, _size: usize) -> u64 {
        let address = address as u32;
        match address {
            // REG_QUEUE_BASE
            REG_QUEUE_BASE..REG_HEAD => self.base as u64,
            // REG_HEAD
            REG_HEAD..REG_TAIL => self.head as u64,
            // REG_TAIL
            REG_TAIL..REG_CONTROL => self.tail as u64,
            // REG_CONTROL
            REG_CONTROL..REG_STATUS => self.control.to_bits() as u64,
            // REG_STATUS
            REG_STATUS..REG_INT_STATE => self.status.to_bits() as u64,
            // REG_INT_STATE
            REG_INT_STATE..REG_DOOR_BELL => self.interrupt_state.to_bits() as u64,
            // REG_DOOR_BELL
            REG_DOOR_BELL..REG_BUFFER_SIZE => self.door_bell as u64,
            REG_BUFFER_SIZE..0x20 => self.buffer_size as u64,
            0x20.. => {
                panic!("Trying to read at address {address}");
            }
        }
    }
    pub fn write(&mut self, address: usize, _size: usize, value: u64) {
        let address = address as u32;
        match address {
            // REG_QUEUE_BASE
            REG_QUEUE_BASE..REG_HEAD => self.base = value as u32,
            // REG_HEAD
            REG_HEAD..REG_TAIL => {
                panic!("CPU is trying to write to HEAD");
            }
            // REG_TAIL
            REG_TAIL..REG_CONTROL => self.tail = value as u16,
            // REG_CONTROL
            REG_CONTROL..REG_STATUS => self.control = ControlFlags::from_bits(value as u8),
            // REG_STATUS
            REG_STATUS..REG_INT_STATE => {
                panic!("CPU is trying to change STATUS");
            }
            // REG_INT_STATE
            REG_INT_STATE..REG_DOOR_BELL => {
                self.interrupt_state = InterruptFlags::from_bits(value as u8)
            }
            // REG_DOOR_BELL
            REG_DOOR_BELL..REG_BUFFER_SIZE => self.door_bell = value != 0,
            // REG_BUFFER_SIZE
            REG_BUFFER_SIZE..0x20 => self.buffer_size = value as u32,
            0x20.. => {
                panic!("Trying to write at address {address}");
            }
        }
    }
}
