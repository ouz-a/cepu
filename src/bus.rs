use crate::memory::*;

pub const ROM_RANGE_BEG: usize = 0x0000;
pub const ROM_RANGE_END: usize = 0x7FFF;

pub const RAM_RANGE_BEG: usize = ROM_RANGE_END + 1;
pub const RAM_SIZE: usize = 0x100000;
pub const RAM_RANGE_END: usize = RAM_RANGE_BEG + RAM_SIZE - 1;

#[derive(Default, Clone, Debug)]
pub struct Bus {}

impl Bus {
    pub fn read_memory(&self, address: usize, size: usize) -> (PhyMemStatus, u64) {
        let address_range = address + size;
        match address_range {
            ROM_RANGE_BEG..=ROM_RANGE_END => Bus::read_memory_impl(address, size),
            RAM_RANGE_BEG..=RAM_RANGE_END => Bus::read_memory_impl(address, size),
            _ => {
                panic!("Out of bounds memory access!")
            }
        }
    }
    pub fn write_memory(&self, address: usize, size: usize, value: u64) -> PhyMemStatus {
        let address_range = address + size;
        match address_range {
            ROM_RANGE_BEG..=ROM_RANGE_END => {
                panic!("Trying to write to ROM!")
            }
            RAM_RANGE_BEG..=RAM_RANGE_END => Bus::write_memory_impl(address, size, value),
            _ => {
                panic!("Out of bounds memory access!")
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
