use std::{fs::File, io::Read, path::PathBuf};

use crate::{cpu::Cpu, sys::MEMORY_SIZE};

// ARM64 boot protocol: kernel must be at (2MB-aligned base) + text_offset
// Image header shows text_offset = 0, so kernel must be at 2MB boundary
pub const KERNEL_LOAD_ADD: usize = 0x200000;

pub const DTB_LOAD_ADD: usize = 0x80000; // 512KB - safe distance from kernel
pub const INITRD_LOAD_ADD: usize = 0x0800_0000; // 128MB

pub fn load_initramfs(cpu: &mut Cpu, initrd_path: &PathBuf) -> usize {
    let mut buf: Vec<u8> = Vec::new();
    let mut initrd = File::open(initrd_path).expect("Failed to open initramfs");
    initrd.read_to_end(&mut buf).expect("Failed to read initramfs");
    let size = buf.len();
    assert!(INITRD_LOAD_ADD + size <= MEMORY_SIZE, "initramfs doesn't fit in RAM");

    cpu.mmu.bus.write_slice(&buf, INITRD_LOAD_ADD);
    println!(
        "Loaded initramfs: {} bytes at 0x{:08x}-0x{:08x}",
        size,
        INITRD_LOAD_ADD,
        INITRD_LOAD_ADD + size
    );
    size
}

pub fn load_device_blob(cpu: &mut Cpu, dtb_path: &PathBuf) {
    let mut buf = Vec::new();
    let mut dtb = File::open(dtb_path).expect("Failed to open DTB");
    dtb.read_to_end(&mut buf).expect("Failed to read DTB file to buffer");
    assert!(DTB_LOAD_ADD + buf.len() <= MEMORY_SIZE, "device blob doesn't fit in RAM");
    cpu.mmu.bus.write_slice(&buf, DTB_LOAD_ADD);
    cpu.x_write(0, DTB_LOAD_ADD as u64, false);
}

pub fn load_kernel_image(cpu: &mut Cpu, kernel_image_path: &PathBuf) {
    let mut buf = Vec::new();
    let mut kernel = File::open(kernel_image_path).expect("Failed to open Kernel IMAGE");
    kernel.read_to_end(&mut buf).expect("Failed to read Kernel IMAGE file to buffer");
    assert!(KERNEL_LOAD_ADD + buf.len() <= MEMORY_SIZE, "device blob doesn't fit in RAM");
    cpu.mmu.bus.write_slice(&buf, KERNEL_LOAD_ADD);
    cpu.pc = KERNEL_LOAD_ADD as u64;
}
