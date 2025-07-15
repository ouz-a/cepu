use std::path::Path;

use crate::{
    branch::branch_addr,
    cpu::{Cpu, INSTRUCTION_SIZE},
    elf::create_and_validate_elf_header,
    instruction::decode,
    memory::{MEMORY_SIZE, read_32},
};

pub mod branch;
pub mod branch_exc_sys_instr;
pub mod cpu;
pub mod data_processing;
pub mod elf;
pub mod imm_instr;
pub mod instruction;
pub mod load_and_store;
pub mod load_store_instr;
pub mod memory;
pub mod register_instr;
pub mod utils;

pub fn run_block(cpu: &mut Cpu) {
    let mut pc = cpu.pc;
    let limit = MEMORY_SIZE;
    loop {
        let old_pc = pc;
        let word = read_32(old_pc as usize);
        pc = pc.wrapping_add(INSTRUCTION_SIZE);
        let dec = decode(word);
        println!("Instruction is {dec:?}");
        dec.exec(cpu, old_pc);

        if cpu.branch_taken {
            pc = cpu.branch_target;
            cpu.branch_taken = false;
        }
        if pc >= limit.try_into().unwrap() {
            break;
        }
    }
    cpu.pc = branch_addr(pc, cpu.pstate.current_el as u8) & 0x00FF_FFFF_FFFF_FFFF;
}

fn main() {
    let mut cpu = Cpu::init();
    create_and_validate_elf_header(&mut cpu, Path::new("boot.elf"));
    run_block(&mut cpu);
}
