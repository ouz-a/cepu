use std::{fs::File, io::Read, path::PathBuf};

use crate::{cpu::Cpu, memory::MEMORY};

pub const KERNEL_LOAD_ADD: usize = 0x80000;
pub const DTB_LOAD_ADD: usize = KERNEL_LOAD_ADD - 0x2000;

pub fn load_device_blob(cpu: &mut Cpu, dtb_path: &PathBuf) {
    let mut buf = Vec::new();
    let mut dtb = File::open(dtb_path).expect("Failed to open DTB");
    dtb.read_to_end(&mut buf).expect("Faield to write DTB file to buffer");
    unsafe {
        MEMORY[DTB_LOAD_ADD..(DTB_LOAD_ADD + buf.len())].copy_from_slice(&buf);
    }
    cpu.sp_write(DTB_LOAD_ADD as u64);
}

pub fn load_kernel_image(cpu: &mut Cpu, kernel_image_path: &PathBuf) {
    let mut buf = Vec::new();
    let mut dtb = File::open(kernel_image_path).expect("Failed to open Kernel IMAGE");
    dtb.read_to_end(&mut buf).expect("Faield to write Kernel IMAGE file to buffer");
    unsafe {
        MEMORY[KERNEL_LOAD_ADD..(KERNEL_LOAD_ADD + buf.len())].copy_from_slice(&buf);
    }
    cpu.pc = KERNEL_LOAD_ADD as u64;
}
