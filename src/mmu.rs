use crate::{
    bus::Bus,
    memory::PhyMemStatus,
    utils::{bits_get, bits_get_in_place},
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mmu {
    pub bus: Bus,
    pub enabled: bool,

    pub tcr_el1: u64,

    pub ttbr0_el1: u64,
    pub ttbr1_el1: u64,

    // Fault state
    pub faulted: bool,
    pub fault_va: usize,
    pub fault_level: u8,
}

impl Mmu {
    pub fn read_memory(&mut self, address: usize, size: usize) -> (PhyMemStatus, u64) {
        if self.enabled {
            let pa = self.page_walk(address);
            if self.faulted {
                return (PhyMemStatus::default(), 0);
            }
            self.bus.read_memory(pa, size)
        } else {
            self.bus.read_memory(address, size)
        }
    }
    pub fn write_memory(&mut self, address: usize, size: usize, value: u64) -> PhyMemStatus {
        if self.enabled {
            let pa = self.page_walk(address);
            if self.faulted {
                return PhyMemStatus::default();
            }
            self.bus.write_memory(pa, size, value)
        } else {
            self.bus.write_memory(address, size, value)
        }
    }
    pub fn write_memory_128bit(&mut self, address: usize, value: u128) -> PhyMemStatus {
        let lo = value;
        let high = (value >> 64) as u64;
        self.write_memory(address, 8, lo as u64);
        self.write_memory(address + 8, 8, high)
    }
    pub fn read_memory_128bit(&mut self, address: usize) -> (PhyMemStatus, u128) {
        let (_, lo) = self.read_memory(address, 8);
        if self.faulted {
            return (PhyMemStatus::default(), 0);
        }
        let (status, hi) = self.read_memory(address + 8, 8);
        (status, lo as u128 | ((hi as u128) << 64))
    }
    fn which_base(&self, va: usize) -> usize {
        let top_bits = bits_get(va as u64, 48, 16);

        let base_add = if top_bits != 0 {
            bits_get_in_place(self.ttbr1_el1, 12, 36)
        } else {
            bits_get_in_place(self.ttbr0_el1, 12, 36)
        };
        base_add as usize
    }

    pub fn page_walk(&mut self, va: usize) -> usize {
        let base_add = self.which_base(va);
        let mut entry_idx = 39;
        let mut descript_bits = 18;
        let entry = base_add + (bits_get(va as u64, 39, 9) * 8) as usize;
        let mut descriptor = self.bus.read_memory(entry, 8).1;
        if DescriptorKind::from(descriptor) == DescriptorKind::Invalid {
            self.faulted = true;
            self.fault_va = va;
            self.fault_level = (entry_idx - 12) / 9;
            return 0;
        }

        loop {
            let table = bits_get_in_place(descriptor, 12, 36) as usize;
            entry_idx -= 9;

            let entry = table + (bits_get(va as u64, entry_idx, 9) * 8) as usize;
            descriptor = self.bus.read_memory(entry, 8).1;
            if entry_idx == 12 {
                if DescriptorKind::from(descriptor) == DescriptorKind::Invalid {
                    self.faulted = true;
                    self.fault_va = va;
                    self.fault_level = 3;
                    return 0;
                }
                let page_addr = bits_get_in_place(descriptor, 12, 36);

                let page_offset = bits_get(va.try_into().unwrap(), 0, entry_idx);
                let final_physical_addr = page_addr + page_offset;
                return final_physical_addr as usize;
            }
            match DescriptorKind::from(descriptor) {
                DescriptorKind::Invalid => {
                    self.faulted = true;
                    self.fault_va = va;
                    self.fault_level = (entry_idx - 12) / 9;
                    return 0;
                }
                DescriptorKind::Block => {
                    let block_base = bits_get_in_place(descriptor, entry_idx, descript_bits);
                    let offset = bits_get(va.try_into().unwrap(), 0, entry_idx);
                    let pa = (block_base + offset) as usize;
                    return pa;
                }
                DescriptorKind::Reserved => panic!("Invalid memory"),
                DescriptorKind::TablePage => {}
            }
            descript_bits += 9;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DescriptorKind {
    Invalid = 0b00,
    Block = 0b01,
    Reserved = 0b10,
    TablePage = 0b11,
}

impl From<u8> for DescriptorKind {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => DescriptorKind::Invalid,
            0b01 => DescriptorKind::Block,
            0b10 => DescriptorKind::Reserved,
            0b11 => DescriptorKind::TablePage,
            _ => DescriptorKind::Invalid,
        }
    }
}
impl From<usize> for DescriptorKind {
    fn from(value: usize) -> Self {
        match value & 0b11 {
            0b00 => DescriptorKind::Invalid,
            0b01 => DescriptorKind::Block,
            0b10 => DescriptorKind::Reserved,
            0b11 => DescriptorKind::TablePage,
            _ => DescriptorKind::Invalid,
        }
    }
}
impl From<u64> for DescriptorKind {
    fn from(value: u64) -> Self {
        match value & 0b11 {
            0b00 => DescriptorKind::Invalid,
            0b01 => DescriptorKind::Block,
            0b10 => DescriptorKind::Reserved,
            0b11 => DescriptorKind::TablePage,
            _ => DescriptorKind::Invalid,
        }
    }
}
