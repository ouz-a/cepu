use crate::{
    bus::Bus,
    cpu::ExceptionLevel,
    memory::PhyMemStatus,
    utils::{BitUtils, bits_get, bits_get_in_place},
};

pub const ENTRY_IDX_WIDTH: u8 = 9;

/// Descriptor is 64 bits
/// 64 / 8 == 8
pub const ENTRY_SIZE: u8 = 8;
pub const PAGE_OFFSET_WIDTH: u8 = 12;
pub const DESCR_ADDR_WIDTH: u8 = 36;

pub const TLB_SIZE: usize = 64;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TlbEntry {
    pub valid: bool = false,
    pub global: bool,
    pub virtual_page: u64,
    pub physical_page: u64,
    pub ap: u8,
    pub asid: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tlb {
    pub entries: [TlbEntry; TLB_SIZE],
}

impl Tlb {
    pub fn insert_new_entry(&mut self, va: u64, pa: u64, ap: u8, asid: u16, global: bool) {
        let vpage = va >> 12;
        let index = (vpage & (TLB_SIZE as u64 - 1)) as usize;
        let tlb_entry =
            TlbEntry { valid: true, global, virtual_page: vpage, physical_page: pa >> 12, ap, asid };
        self.entries[index] = tlb_entry;
    }

    pub fn index(va: u64) -> usize {
        ((va >> 12) & (TLB_SIZE as u64 - 1)) as usize
    }
}

impl Default for Tlb {
    fn default() -> Self {
        Tlb { entries: [TlbEntry::default(); 64] }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultType {
    #[default]
    Translation,
    Permission,
}

#[derive(Default, Debug)]
pub struct Mmu {
    pub bus: Bus,
    pub enabled: bool,
    pub tlb: Tlb,

    pub tcr_el1: u64,

    pub ttbr0_el1: u64,
    pub ttbr1_el1: u64,

    // Fault state
    pub faulted: bool,
    pub fault_va: usize,
    pub fault_level: u8,
    pub fault_type: FaultType,
    /// Fault Status Code
    pub fsc: u8,
    /// Was it a write that caused the fault (for WnR bit)
    pub fault_is_write: bool,
}

impl Mmu {
    pub(crate) fn mmu_read(
        &mut self,
        address: usize,
        size: usize,
        current_el: ExceptionLevel,
    ) -> (PhyMemStatus, u64) {
        if self.enabled {
            let pa = if let Some(pa) = self.tlb_lookup(address, AccessType::Read, current_el) {
                if self.faulted {
                    return (PhyMemStatus::default(), 0);
                }
                pa
            } else {
                let pa = self.page_walk(address, AccessType::Read, current_el);
                if self.faulted {
                    return (PhyMemStatus::default(), 0);
                }
                pa
            };
            return self.bus.read_memory(pa, size);
        }
        return self.bus.read_memory(address, size);
    }

    pub(crate) fn mmu_write(
        &mut self,
        address: usize,
        size: usize,
        value: u64,
        current_el: ExceptionLevel,
    ) -> PhyMemStatus {
        if self.enabled {
            let pa = if let Some(pa) = self.tlb_lookup(address, AccessType::Write, current_el) {
                if self.faulted {
                    return PhyMemStatus::default();
                }
                pa
            } else {
                let pa = self.page_walk(address, AccessType::Write, current_el);
                if self.faulted {
                    return PhyMemStatus::default();
                }
                pa
            };
            self.bus.write_memory(pa, size, value)
        } else {
            self.bus.write_memory(address, size, value)
        }
    }

    pub(crate) fn mmu_write_128bit(
        &mut self,
        address: usize,
        value: u128,
        current_el: ExceptionLevel,
    ) -> PhyMemStatus {
        let lo = value;
        let high = (value >> 64) as u64;
        self.mmu_write(address, 8, lo as u64, current_el);
        self.mmu_write(address + 8, 8, high, current_el)
    }

    pub(crate) fn mmu_read_128bit(
        &mut self,
        address: usize,
        current_el: ExceptionLevel,
    ) -> (PhyMemStatus, u128) {
        let (_, lo) = self.mmu_read(address, 8, current_el);
        if self.faulted {
            return (PhyMemStatus::default(), 0);
        }
        let (status, hi) = self.mmu_read(address + 8, 8, current_el);
        (status, lo as u128 | ((hi as u128) << 64))
    }

    /// Returns base_add,globalness and asid
    fn which_base(&self, va: usize) -> (usize, bool, u16) {
        let global = bits_get(va as u64, 48, 16) != 0;

        let base_add = if global {
            bits_get_in_place(self.ttbr1_el1, 12, 36)
        } else {
            bits_get_in_place(self.ttbr0_el1, 12, 36)
        };
        let asid = self.ttbr1_el1.bits_get(48, 16) as u16;
        (base_add as usize, global, asid)
    }

    pub fn handle_fault(
        &mut self,
        va: usize,
        entry_idx: u8,
        fault_type: FaultType,
        access_type: AccessType,
    ) -> usize {
        self.faulted = true;
        self.fault_va = va;
        self.fault_level = (39 - entry_idx) / ENTRY_IDX_WIDTH;
        self.fault_type = fault_type;
        self.fault_is_write = matches!(access_type, AccessType::Write);
        if fault_type == FaultType::Translation {
            self.fsc = 0b000100 | self.fault_level;
        } else {
            self.fsc = 0b001100 | self.fault_level;
        }
        0
    }

    pub fn check_access(
        &self,
        ap: u8,
        access_type: AccessType,
        current_el: ExceptionLevel,
    ) -> bool {
        let unpriv = current_el.is_el0();
        let is_write = matches!(access_type, AccessType::Write);

        match ap {
            0b0 => {
                if !unpriv {
                    return true;
                }
            }
            0b01 => {
                return true;
            }
            0b10 => {
                if !unpriv && !is_write {
                    return true;
                }
            }
            0b11 => {
                if !is_write {
                    return true;
                }
            }
            _ => {
                panic!("Wrong ap");
            }
        }
        false
    }

    pub fn has_access(
        &mut self,
        descriptor: u64,
        access_type: AccessType,
        current_el: ExceptionLevel,
    ) -> bool {
        let ap: u8 = descriptor.bits_get(6, 2) as u8;

        self.check_access(ap, access_type, current_el)
    }

    pub fn page_walk(
        &mut self,
        va: usize,
        access_type: AccessType,
        current_el: ExceptionLevel,
    ) -> usize {
        let first_level = 39;
        let levels = [30, 21, 12];
        let (base_add, global, asid) = self.which_base(va);
        let mut block_width = 18;

        let entry = base_add + get_entry_offset(va, first_level);
        let mut descriptor = self.bus.read_memory(entry, ENTRY_SIZE as usize).1;

        if DescriptorKind::from(descriptor) == DescriptorKind::Invalid {
            return self.handle_fault(va, first_level as u8, FaultType::Translation, access_type);
        }

        for level in levels.into_iter() {
            let page_table_address =
                bits_get_in_place(descriptor, PAGE_OFFSET_WIDTH, DESCR_ADDR_WIDTH) as usize;

            let entry_address = page_table_address + get_entry_offset(va, level);
            descriptor = self.bus.read_memory(entry_address, ENTRY_SIZE.into()).1;

            match DescriptorKind::from(descriptor) {
                DescriptorKind::Block => {
                    if !self.has_access(descriptor, access_type, current_el) {
                        return self.handle_fault(
                            va,
                            level as u8,
                            FaultType::Permission,
                            access_type,
                        );
                    }
                    let block_base_address =
                        bits_get_in_place(descriptor, level as u8, block_width);
                    let offset = bits_get(va.try_into().unwrap(), 0, level as u8);
                    let pa = (block_base_address + offset) as usize;
                    let ap: u8 = descriptor.bits_get(6, 2) as u8;
                    self.tlb.insert_new_entry(
                        va as u64,
                        pa as u64,
                        ap,
                        asid,
                        global,
                    );
                    return pa;
                }
                DescriptorKind::TablePage => {
                    if level == 12 {
                        if !self.has_access(descriptor, access_type, current_el) {
                            return self.handle_fault(
                                va,
                                level as u8,
                                FaultType::Permission,
                                access_type,
                            );
                        }
                        let block_base_address =
                            bits_get_in_place(descriptor, level as u8, block_width);
                        let offset = bits_get(va.try_into().unwrap(), 0, level as u8);
                        let pa = (block_base_address + offset) as usize;
                        let ap: u8 = descriptor.bits_get(6, 2) as u8;
                        self.tlb.insert_new_entry(
                            va as u64,
                            pa as u64,
                            ap,
                            asid,
                            global,
                        );
                        return pa;
                    }
                }
                DescriptorKind::Invalid => {
                    return self.handle_fault(va, level as u8, FaultType::Translation, access_type);
                }
                DescriptorKind::Reserved => panic!("Invalid Memory"),
            }

            block_width += 9;
        }

        0
    }

    pub fn tlb_lookup(
        &mut self,
        address: usize,
        access_type: AccessType,
        current_el: ExceptionLevel,
    ) -> Option<usize> {
        let (_, _, asid) = self.which_base(address);
        let va = address as u64;
        let vpage = va >> 12;
        let index = Tlb::index(va);
        let entry = self.tlb.entries[index];

        if entry.valid && entry.virtual_page == vpage && (entry.global || entry.asid == asid) {
            if !self.check_access(entry.ap, access_type, current_el) {
                self.handle_fault(address, 12, FaultType::Permission, access_type);
                return Some(0);
            }
            let pa = (entry.physical_page << 12) | va.bits_get(0, 12);
            return Some(pa as usize);
        }

        None
    }

    pub fn tlb_flush_all(&mut self) {
        for entry in self.tlb.entries.iter_mut() {
            entry.valid = false;
        }
    }

    pub fn tlb_invalidate_va(&mut self, vpn: u64, asid: u16) {
        for entry in self.tlb.entries.iter_mut() {
            if entry.valid && entry.virtual_page == vpn && entry.asid == asid {
                entry.valid = false;
            }
        }
    }

    pub fn tlb_invalidate_asid(&mut self, asid: u16) {
        for entry in self.tlb.entries.iter_mut() {
            if entry.valid && entry.asid == asid {
                entry.valid = false;
            }
        }
    }

    pub fn tlb_invalidate_va_all_asids(&mut self, vpn: u64) {
        for entry in self.tlb.entries.iter_mut() {
            if entry.valid && entry.virtual_page == vpn {
                entry.valid = false;
            }
        }
    }
}

pub fn get_entry_offset(va: usize, entry_idx: u64) -> usize {
    va.bits_get(entry_idx as u8, ENTRY_IDX_WIDTH) * ENTRY_SIZE as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessType {
    Read,
    Write,
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
