use crate::{accelerator::REG_DOOR_BELL, devices::Uart, gic::Gic, memory::*};

pub const RAM_RANGE_BEG: usize = 0;
pub const RAM_SIZE: usize = 0x10000000;
pub const RAM_RANGE_END: usize = RAM_RANGE_BEG + RAM_SIZE - 1;

pub const GIC_DIST_BEG: usize = 0x8000_0000;
pub const GIC_DIST_END: usize = 0x8000_0FFF;
pub const GIC_CPU_BEG: usize = 0x8001_0000;
pub const GIC_CPU_END: usize = 0x8001_1FFF;

pub const UART_RANGE_BEG: usize = 0x9000_0000;
pub const UART_RANGE_END: usize = UART_RANGE_BEG + 4096;

pub const CEL_BEG: usize = 0xA000_0000;
pub const CEL_END: usize = CEL_BEG + 4096;

use std::sync::{Arc, Condvar, Mutex};

use crate::CepuCel;

#[derive(Default, Debug)]
pub struct Bus {
    pub uart: Uart,
    pub gic: Gic,
    pub cepu_cel: Arc<(Mutex<CepuCel>, Condvar)>,
}

impl Bus {
    pub fn read_memory(&mut self, address: usize, size: usize) -> (PhyMemStatus, u64) {
        match address {
            RAM_RANGE_BEG..=RAM_RANGE_END => Bus::read_memory_impl(address, size),
            GIC_DIST_BEG..=GIC_DIST_END | GIC_CPU_BEG..=GIC_CPU_END => {
                let val = self.gic.read(address as u64);
                (PhyMemStatus::default(), val as u64)
            }
            UART_RANGE_BEG..=UART_RANGE_END => self.uart.read((address - UART_RANGE_BEG) as u16),
            CEL_BEG..=CEL_END => {
                let mut cepu_cel = self.cepu_cel.0.lock().expect("Failed to lock CepuCel");
                (PhyMemStatus::default(), cepu_cel.read(address - CEL_BEG, size))
            }
            _ => {
                panic!("Out of bounds memory read! Range {address:x}")
            }
        }
    }

    /// Size as in bytes not bits
    pub fn write_memory(&mut self, address: usize, size: usize, value: u64) -> PhyMemStatus {
        match address {
            RAM_RANGE_BEG..=RAM_RANGE_END => Bus::write_memory_impl(address, size, value),
            GIC_DIST_BEG..=GIC_DIST_END | GIC_CPU_BEG..=GIC_CPU_END => {
                self.gic.write(address as u64, value as u32);
                PhyMemStatus::default()
            }
            UART_RANGE_BEG..=UART_RANGE_END => {
                self.uart.write((address - UART_RANGE_BEG) as u16, value, size)
            }
            CEL_BEG..=CEL_END => {
                let mut cepu_cel = self.cepu_cel.0.lock().expect("Failed to lock CepuCel");
                let address = address - CEL_BEG;
                if address == (REG_DOOR_BELL as usize) {
                    cepu_cel.write(address, size, value);
                    self.cepu_cel.1.notify_one();
                } else {
                    cepu_cel.write(address, size, value);
                }
                PhyMemStatus::default()
            }

            _ => {
                panic!("Out of bounds memory write! Range {address:x}")
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
