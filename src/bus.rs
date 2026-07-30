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

use crate::{CepuCel, sys::*};

#[derive(Default, Debug)]
pub struct AddressSpace {
    pub base: *mut u8,
    pub len: usize,
    pub dirty: Vec<u8>,
}

impl AddressSpace {
    pub fn read(&self, address: usize, size: usize) -> (PhyMemStatus, u64) {
        assert!(matches!(size, 1 | 2 | 4 | 8));
        assert!(address + size <= MEMORY_SIZE);

        let mut status = PhyMemStatus::default();
        let ret_val;
        unsafe {
            let src = self.base.add(address);
            let mut value = [0u8; 8];
            core::ptr::copy_nonoverlapping(src, value.as_mut_ptr(), size);
            ret_val = u64::from_le_bytes(value);
        }
        status.fault_status = FaultStatus::None;
        (status, ret_val)
    }

    pub fn write(&self, address: usize, size: usize, value: u64) -> PhyMemStatus {
        assert!(matches!(size, 1 | 2 | 4 | 8));
        assert!(address + size <= MEMORY_SIZE);

        let mut status = PhyMemStatus::default();
        unsafe {
            let dst = self.base.add(address);
            let bytes = value.to_le_bytes();
            let src = bytes.as_ptr();

            core::ptr::copy_nonoverlapping(src, dst, size);
        }
        status.fault_status = FaultStatus::None;
        status
    }

    pub fn write_slice(&self, slice: &[u8], offset: usize) {
        unsafe {
            std::ptr::copy_nonoverlapping(slice.as_ptr(), self.base.add(offset), slice.len());
        }
    }

    pub fn read_struct<T>(&self, address: usize) -> T {
        assert!(address + core::mem::size_of::<T>() <= self.len);
        unsafe { core::ptr::read_unaligned(self.base.add(address) as *const T) }
    }
}

unsafe impl Send for AddressSpace {}
unsafe impl Sync for AddressSpace {}

#[derive(Default, Debug)]
pub struct Bus {
    pub uart: Uart,
    pub gic: Gic,
    pub cepu_cel: Arc<(Mutex<CepuCel>, Condvar)>,
    pub address_space: Arc<AddressSpace>,
}

impl Bus {
    pub fn read_memory(&mut self, address: usize, size: usize) -> (PhyMemStatus, u64) {
        match address {
            RAM_RANGE_BEG..=RAM_RANGE_END => self.address_space.read(address, size),
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
            RAM_RANGE_BEG..=RAM_RANGE_END => self.address_space.write(address, size, value),
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

    pub fn write_slice(&mut self, slice: &[u8], offset: usize) {
        self.address_space.write_slice(slice, offset);
    }
}
