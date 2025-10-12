use std::{fs::File, io::Read, path::PathBuf};

use crate::{cpu::Cpu, memory::MEMORY};

// ARM64 boot protocol: kernel must be at (2MB-aligned base) + text_offset
// Image header shows text_offset = 0, so kernel must be at 2MB boundary
pub const KERNEL_LOAD_ADD: usize = 0x200000;

pub const DTB_LOAD_ADD: usize = 0x80000; // 512KB - safe distance from kernel

pub fn load_device_blob(cpu: &mut Cpu, dtb_path: &PathBuf) {
    let mut buf = Vec::new();
    let mut dtb = File::open(dtb_path).expect("Failed to open DTB");
    dtb.read_to_end(&mut buf).expect("Failed to read DTB file to buffer");
    unsafe {
        MEMORY[DTB_LOAD_ADD..(DTB_LOAD_ADD + buf.len())].copy_from_slice(&buf);
    }
    cpu.x_write(0, DTB_LOAD_ADD as u64, false);
}

pub fn load_kernel_image(cpu: &mut Cpu, kernel_image_path: &PathBuf) {
    let mut buf = Vec::new();
    let mut kernel = File::open(kernel_image_path).expect("Failed to open Kernel IMAGE");
    kernel.read_to_end(&mut buf).expect("Failed to read Kernel IMAGE file to buffer");
    unsafe {
        MEMORY[KERNEL_LOAD_ADD..(KERNEL_LOAD_ADD + buf.len())].copy_from_slice(&buf);
    }
    cpu.pc = KERNEL_LOAD_ADD as u64;
}
