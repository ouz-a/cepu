#![feature(int_lowest_highest_one)]

use std::{path::PathBuf, str::FromStr, sync::atomic::Ordering, time::Duration};

use crate::{
    branch::branch_addr,
    cpu::{BATCH, Cpu, INSTRUCTION_SIZE, monotonic_ns},
    image::{load_device_blob, load_kernel_image},
    instruction::decode,
    memory::{MEMORY_SIZE, read_32},
};

pub mod branch;
pub mod branch_exc_sys_instr;
pub mod bus;
pub mod cpu;
pub mod data_processing;
pub mod devices;
pub mod elf;
pub mod image;
pub mod imm_instr;
pub mod instruction;
pub mod load_and_store;
pub mod load_store_instr;
pub mod memory;
pub mod register_instr;
pub mod utils;

pub fn run_block(cpu: &mut Cpu) {
    let mut batch_left = BATCH;

    let mut pc = cpu.pc;
    let limit = MEMORY_SIZE;
    loop {
        if !cpu.sleeping.load(Ordering::Relaxed) {
            cpu.handle_devices();
            cpu.handle_interrupts(&mut pc);
            let old_pc = pc;
            let word = read_32(old_pc as usize);
            pc = pc.wrapping_add(INSTRUCTION_SIZE);
            let dec = decode(word);
            println!("Instruction is {dec:?} raw: {:08X}", word.to_be());
            dec.exec(cpu, old_pc);

            if cpu.branch_taken {
                pc = cpu.branch_target;
                cpu.branch_taken = false;
            }

            if pc >= limit.try_into().unwrap() {
                break;
            }

            batch_left -= 1;
            if batch_left == 0 {
                batch_left = BATCH;
                cpu.timer_device_tick();
            }
        } else {
            let condvar_pair = cpu.condvar.clone();
            let (lock, cvar) = &*condvar_pair;
            let mut guard = lock.lock().unwrap();

            loop {
                cpu.timer_device_tick();

                if cpu.should_wake() {
                    break;
                }

                let now = monotonic_ns();
                let deadline = &cpu.timer.cntp_expiry_ns;
                let deadline = deadline.load(Ordering::Relaxed).saturating_sub(now);
                let duration = Duration::from_nanos(deadline);
                if deadline > 0 {
                    let g = cvar
                        .wait_timeout(guard, duration)
                        .expect("Condvar failed to wait for a timeout");
                    guard = g.0;
                } else {
                    guard = cvar.wait(guard).expect("Condvar failed to wait");
                }
            }
            cpu.sleeping.store(false, Ordering::Relaxed);
        }
    }
    cpu.pc = branch_addr(pc, cpu.pstate.current_el as u8) & 0x00FF_FFFF_FFFF_FFFF;
}

fn main() {
    let mut cpu = Cpu::init();
    //validate_and_load_elf_header(&mut cpu, Path::new("./.executables/boot.elf"));
    load_device_blob(&mut cpu, &PathBuf::from_str("/Users/ouz/code/cepu_now/cepu.dtb").unwrap());
    load_kernel_image(&mut cpu, &PathBuf::from_str("/Users/ouz/code/cepu_now/Image").unwrap());
    run_block(&mut cpu);
}
