#![feature(int_lowest_highest_one)]
#![feature(default_field_values)]
#![allow(clippy::too_many_arguments)]

use std::{
    collections::VecDeque,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Condvar, Mutex, atomic::Ordering},
    time::Duration,
};

use crate::{
    accelerator::{CepuCel, Commands, DeviceStatus},
    branch::branch_addr,
    cpu::{BATCH, Cpu, INSTRUCTION_SIZE},
    image::{load_device_blob, load_initramfs, load_kernel_image},
    instruction::{
        CURRENT_PC, DESCR, UNDEF_PANIC, capture_trace, decode, print_undefined_trace,
        validate_tables,
    },
    memory::{MEMORY_SIZE, read_32},
};
// Core
pub mod bus;
pub mod cpu;
pub mod memory;
pub mod mmu;

// Instruction decode/execute
pub mod branch;
pub mod branch_exc_sys_instr;
pub mod data_processing;
pub mod imm_instr;
pub mod instruction;
pub mod load_and_store;
pub mod load_store_instr;
pub mod register_instr;
pub mod simd_fp;
pub mod simd_fp_instr;

// Devices
pub mod accelerator;
pub mod devices;
pub mod gic;
pub mod terminal;

// Boot/image
pub mod elf;
pub mod image;

// Utilities
pub mod utils;
pub fn run_block(cpu: &mut Cpu) {
    let mut batch_left = BATCH;

    let mut pc = cpu.pc;
    let limit = MEMORY_SIZE;
    let mut how_many: u64 = 0;
    let mut trace = VecDeque::new();
    loop {
        if !cpu.sleeping.load(Ordering::Relaxed) {
            cpu.handle_devices();
            cpu.handle_interrupts(&mut pc);
            let old_pc = pc;
            let word = if cpu.mmu.enabled {
                cpu.read_memory(old_pc as usize, 4).1 as u32
            } else {
                read_32(old_pc as usize)
            };
            if cpu.mmu.faulted {
                cpu.handle_instruction_abort(&mut pc);
                continue;
            }
            pc = pc.wrapping_add(INSTRUCTION_SIZE);
            let dec = decode(word);
            capture_trace(&mut trace, pc, word);
            if UNDEF_PANIC.load(Ordering::Acquire) {
                print_undefined_trace(how_many, &mut trace);
                std::process::exit(1);
            }
            CURRENT_PC.store(old_pc, Ordering::Relaxed);
            dec.exec(cpu, old_pc);
            if cpu.mmu.faulted {
                pc = old_pc;
                cpu.handle_data_abort(&mut pc);
                continue;
            }
            how_many += 1;

            // TODO: This is a hack
            if UNDEF_PANIC.load(Ordering::Acquire) {
                print_undefined_trace(how_many, &mut trace);
                std::process::exit(1);
            }

            if cpu.branch_taken {
                pc = cpu.branch_target;
                cpu.branch_taken = false;
            }

            if !cpu.mmu.enabled && pc >= limit.try_into().unwrap() {
                break;
            }

            batch_left -= 1;
            if batch_left == 0 {
                batch_left = BATCH;
                cpu.timer_device_tick();
                cpu.poll_uart_rx();
            }
        } else {
            loop {
                cpu.timer_device_tick();
                cpu.poll_uart_rx();
                if cpu.should_wake() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            cpu.sleeping.store(false, Ordering::Relaxed);
        }
    }
    cpu.pc = branch_addr(pc, cpu.pstate.current_el as u8) & 0x00FF_FFFF_FFFF_FFFF;
}

fn main() {
    validate_tables(DESCR);
    let cepu_cel = CepuCel::new();
    let cepu_cel: Arc<(Mutex<CepuCel>, Condvar)> = Arc::new((Mutex::new(cepu_cel), Condvar::new()));
    let cepu_cel_clone = cepu_cel.clone();
    let mut cpu = Cpu::init();
    cpu.mmu.bus.cepu_cel = cepu_cel.clone();

    //validate_and_load_elf_header(&mut cpu,
    // Path::new("./.executables/boot.elf"));
    load_device_blob(&mut cpu, &PathBuf::from_str("/Users/ouz/code/cepu_now/cepu.dtb").unwrap());
    load_kernel_image(&mut cpu, &PathBuf::from_str("/Users/ouz/code/cepu_now/Image").unwrap());
    load_initramfs(&PathBuf::from_str("/Users/ouz/code/cepu_now/initramfs.cpio").unwrap());

    let _ = std::thread::spawn(move || {
        let (cepu_cel, cvar) = &*cepu_cel_clone;
        let mut guard = cepu_cel.lock().unwrap();
        loop {
            while guard.head == guard.tail {
                guard = cvar.wait(guard).unwrap();
            }
            let (base, head) = guard.get_base_head();
            guard.status = DeviceStatus::Busy;
            drop(guard);
            let command = CepuCel::get_command(base, head);
            let Commands::MatMul(mut command) = command;
            command.work();
            guard = cepu_cel.lock().unwrap();
            guard.advance_then_set();
        }
    });
    cepu_cel.1.notify_one();
    let _term = terminal::RawTerminal::enable();
    run_block(&mut cpu);
}
