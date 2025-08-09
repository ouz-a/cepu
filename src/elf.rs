use std::{fs, mem::MaybeUninit, path::Path, slice};

use crate::{
    cpu::Cpu,
    memory::{MEMORY, MEMORY_SIZE},
};

pub const EI_NIDENT: usize = 16;

pub const EI_MAG0: usize = 0;
pub const EI_MAG1: usize = 1;
pub const EI_MAG2: usize = 2;
pub const EI_MAG3: usize = 3;
pub const EI_CLASS: usize = 4;
pub const EI_DATA: usize = 5;
pub const EI_VERSION: usize = 6;

pub const ELFMAG0: u8 = 0x7F;
pub const ELFMAG1: u8 = b'E';
pub const ELFMAG2: u8 = b'L';
pub const ELFMAG3: u8 = b'F';

pub const ELFMAG: &[u8; 4] = b"\x7FELF";
pub const S_ELFMAG: usize = 4;

pub const ELFCLASSNONE: u8 = 0;
pub const ELFCLASS32: u8 = 1;
pub const ELFCLASS64: u8 = 2;

pub const ELFDATA2LSB: u8 = 1;
pub const ELFDATA2MSB: u8 = 2;

pub const EV_CURRENT: u32 = 1;

pub const EM_AARCH64: u16 = 183;

pub const ET_NONE: u16 = 0;
pub const ET_REL: u16 = 1;
pub const ET_EXEC: u16 = 2;
pub const ET_DYN: u16 = 3;
pub const ET_CORE: u16 = 4;

pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_NOTE: u32 = 4;
pub const PT_SHLIB: u32 = 5;
pub const PT_PHDR: u32 = 6;
pub const PT_TLS: u32 = 7;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf64Ehdr {
    pub e_ident: [u8; EI_NIDENT],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shtrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf64Shdr {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

fn validate_elf_header(elf_header: &Elf64Ehdr) {
    if size_of_val(elf_header) != size_of::<Elf64Ehdr>() {
        panic!("Size of ELF header doesn't match defined one");
    }

    if elf_header.e_ident[..S_ELFMAG] != ELFMAG[..] {
        panic!("Invalid magic number!Not a valid ELF file");
    }

    if elf_header.e_ident[EI_CLASS] != ELFCLASS64 {
        panic!("ELF file is not 64-bit");
    }

    if elf_header.e_ident[EI_DATA] != ELFDATA2LSB {
        panic!("ELF data encoding is not little endian");
    }

    if elf_header.e_type != ET_EXEC {
        panic!("ELF file is not an executable type");
    }
}

fn read_elf64_header(buf: &[u8]) -> Option<Elf64Ehdr> {
    let need = size_of::<Elf64Ehdr>();

    let head_bytes = buf.get(..need)?;

    let mut tmp = MaybeUninit::<Elf64Ehdr>::uninit();
    unsafe {
        core::ptr::copy_nonoverlapping(head_bytes.as_ptr(), tmp.as_mut_ptr() as *mut u8, need);
        Some(tmp.assume_init())
    }
}

pub fn validate_and_load_elf_header(cpu: &mut Cpu, path: &Path) {
    let buf = match fs::read(path) {
        Ok(dat) => dat,
        Err(err) => panic!("Failed to read ELF path {path:?} because {err}"),
    };
    let elf = match read_elf64_header(buf.as_slice()) {
        Some(elf) => elf,
        None => panic!("Data is not valid ELF file"),
    };
    validate_elf_header(&elf);
    load_elf_header(cpu, &elf, &buf);
}

fn create_program_headers<'a>(elf: &Elf64Ehdr, file: &[u8]) -> &'a [Elf64Phdr] {
    let phdr_size = size_of::<Elf64Phdr>();
    let start = elf.e_phoff as usize;
    let count = elf.e_phnum as usize;
    let needed = phdr_size * count;
    let raw_bytes = match file.get(start..start + needed) {
        Some(raw_b) => raw_b,
        None => panic!("File doesn't contain enough program headers"),
    };
    let ptr = raw_bytes.as_ptr();
    if (ptr as *const Elf64Phdr).is_aligned() {
        unsafe { slice::from_raw_parts(ptr.cast(), count) }
    } else {
        panic!("Program headers are not aligned")
    }
}

fn load_elf_header(cpu: &mut Cpu, elf: &Elf64Ehdr, file: &[u8]) {
    let headers = create_program_headers(elf, file);

    for p_header in headers {
        if p_header.p_type != PT_LOAD {
            continue;
        }
        if (p_header.p_vaddr + p_header.p_memsz) > MEMORY_SIZE as u64 {
            panic!("Segment exceeds emulator's memory size")
        }
        let start = p_header.p_offset as usize;
        let end = (p_header.p_offset + p_header.p_filesz) as usize;
        let file_data = match file.get(start..end) {
            Some(dat) => dat,
            None => panic!("File doesn't contain program segment"),
        };
        let start = p_header.p_vaddr as usize;
        let end = start + p_header.p_filesz as usize;

        let dest = unsafe { &mut MEMORY[start..end] };
        dest.copy_from_slice(file_data);
        if p_header.p_memsz > p_header.p_filesz {
            let zero_end = start + p_header.p_memsz as usize;
            let zeros = unsafe { &mut MEMORY[end..zero_end] };
            zeros.fill(0);
        }
    }
    cpu.pc = elf.e_entry;
    println!("Starting pc is: {}", cpu.pc);
}
