use std::{path::Path, sync::atomic::Ordering, time::Duration};

use crate::{
    branch::branch_addr,
    cpu::{
        BATCH, Cpu, DRIFT_LIMIT, INSTRUCTION_SIZE, NS_PER_CYCLE, SYS_COUNTER, monotonic_ns,
        sleep_ns,
    },
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
    let wall_start = monotonic_ns();
    let mut vtime = 0;
    let mut batch_left = BATCH;

    let mut pc = cpu.pc;
    let limit = MEMORY_SIZE;
    loop {
        if !cpu.sleeping.load(Ordering::Relaxed) {
            let old_pc = pc;
            let word = read_32(old_pc as usize);
            pc = pc.wrapping_add(INSTRUCTION_SIZE);
            let dec = decode(word);
            println!("Instruction is {dec:?} raw: {:08X}", word.to_be());
            vtime += 1;
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
                unsafe { SYS_COUNTER = vtime }
                batch_left = BATCH;

                let ideal_ns = vtime as f32 * NS_PER_CYCLE;
                let real_ns = monotonic_ns() - wall_start;
                let drift_ns = ideal_ns - real_ns as f32;
                cpu.timer_device_tick();

                if drift_ns > DRIFT_LIMIT as f32 {
                    sleep_ns(drift_ns as u64);
                }
            }
        } else {
            let condvar_pair = cpu.condvar.clone();
            let (lock, cvar) = &*condvar_pair;
            let mut guard = lock.lock().unwrap();

            while !cpu.should_wake() {
                cpu.timer_device_tick();
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
                cpu.sleeping.store(false, Ordering::Relaxed);
            }
        }
    }
    cpu.pc = branch_addr(pc, cpu.pstate.current_el as u8) & 0x00FF_FFFF_FFFF_FFFF;
}

fn main() {
    let mut cpu = Cpu::init();
    create_and_validate_elf_header(&mut cpu, Path::new("boot.elf"));
    run_block(&mut cpu);
}
