use crate::{devices::Uart, memory::*};

pub const RAM_RANGE_BEG: usize = 0;
pub const RAM_SIZE: usize = 0x10000000;
pub const RAM_RANGE_END: usize = RAM_RANGE_BEG + RAM_SIZE - 1;

pub const UART_RANGE_BEG: usize = 0x9000_0000;
pub const UART_RANGE_END: usize = UART_RANGE_BEG + 4096;

#[derive(Default, Clone, Debug)]
pub struct Bus {
    pub uart: Uart,
}

impl Bus {
    pub fn read_memory(&mut self, address: usize, size: usize) -> (PhyMemStatus, u64) {
        match address {
            RAM_RANGE_BEG..=RAM_RANGE_END => Bus::read_memory_impl(address, size),
            UART_RANGE_BEG..=UART_RANGE_END => self.uart.read((address - UART_RANGE_BEG) as u8),
            _ => {
                panic!("Out of bounds memory access! Range {address:x}")
            }
        }
    }

    /// Size as in bytes not bits     
    pub fn write_memory(&mut self, address: usize, size: usize, value: u64) -> PhyMemStatus {
        match address {
            RAM_RANGE_BEG..=RAM_RANGE_END => Bus::write_memory_impl(address, size, value),
            UART_RANGE_BEG..=UART_RANGE_END => {
                self.uart.write((address - UART_RANGE_BEG) as u8, value, size)
            }
            _ => {
                panic!("Out of bounds memory access! Range {address:x}")
            }
        }
    }

    fn read_memory_impl(address: usize, size: usize) -> (PhyMemStatus, u64) {
        assert!(matches!(size, 1 | 2 | 4 | 8));
        assert!(address + size <= MEMORY_SIZE);

        let mut status = PhyMemStatus::default();
        let ret_val;
        unsafe {
            let src = (core::ptr::addr_of!(MEMORY) as *const u8).add(address);
            let mut value = [0u8; 8];
            core::ptr::copy_nonoverlapping(src, value.as_mut_ptr(), size);
            ret_val = u64::from_le_bytes(value);
        }
        status.fault_status = FaultStatus::None;
        (status, ret_val)
    }

    fn write_memory_impl(address: usize, size: usize, value: u64) -> PhyMemStatus {
        assert!(matches!(size, 1 | 2 | 4 | 8));
        assert!(address + size <= MEMORY_SIZE);

        let mut status = PhyMemStatus::default();
        unsafe {
            let dst = (core::ptr::addr_of_mut!(MEMORY) as *mut u8).add(address);
            let bytes = value.to_le_bytes();
            let src = bytes.as_ptr();

            core::ptr::copy_nonoverlapping(src, dst, size);
        }
        status.fault_status = FaultStatus::None;
        status
    }
}
