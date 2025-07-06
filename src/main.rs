use crate::{
    cpu::Cpu,
    instruction::{DESCR, InstructionEntry, UNDEFINED},
};

pub mod branch;
pub mod branch_exc_sys_instr;
pub mod cpu;
pub mod data_processing;
pub mod imm_instr;
pub mod instruction;
pub mod load_and_store;
pub mod load_store_instr;
pub mod memory;
pub mod register_instr;
pub mod utils;

pub static PRIMARY: [InstructionEntry; 2048] = build_primary();

const fn build_primary() -> [InstructionEntry; 2048] {
    let mut tbl = [UNDEFINED; 2048];
    let mut sig = [0u32; 2048];
    let mut init = [false; 2048];

    let mut i = 0;
    while i < DESCR.len() {
        let d = DESCR[i];
        let mut key = 0;
        while key < 2048 {
            let candidate = (key as u32) << 21;
            if (candidate & d.mask) == d.value {
                let spec = d.mask.count_ones();
                if spec > tbl[key].specificity {
                    tbl[key] =
                        InstructionEntry { exec: d.exec, decode: d.decode, specificity: spec };
                    sig[key] = candidate & d.mask;
                    init[key] = true;
                } else if spec == tbl[key].specificity && init[key] {
                    let new_sig = candidate & d.mask;
                    if new_sig != sig[key] {
                        panic!("Descriptor overlap in primary table");
                    }
                }
            }
            key += 1;
        }
        i += 1;
    }
    tbl
}

fn main() {
    let _cpu = Cpu::init();
}
