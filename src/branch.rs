use crate::{
    cpu::{Cpu, ExceptionLevel, PstateField},
    get_bits_ct,
    utils::{bits_get, sign_extend, zero_extend},
};

pub fn condition_holds(cpu: &Cpu, cond: u8) -> bool {
    let mut result = match cond >> 1 {
        0b000 => cpu.pstate.z,
        0b001 => cpu.pstate.c,
        0b010 => cpu.pstate.n,
        0b011 => cpu.pstate.v,
        0b100 => cpu.pstate.c && !cpu.pstate.z,
        0b101 => cpu.pstate.n == cpu.pstate.v,
        0b110 => cpu.pstate.n == cpu.pstate.v && !cpu.pstate.z,
        0b111 => true,
        _ => panic!("Unknown condition"),
    };

    if (cond & 0b0001) == 0b1 && cond != 0b1111 {
        result = !result;
    }
    result
}

// TODO: Properly implement this
#[inline(always)]
pub fn effective_tbi(el: u8) -> bool {
    el == 0
}

#[inline(always)]
pub fn addr_top(el: u8) -> u8 {
    if effective_tbi(el) { 55 } else { 63 }
}

#[inline(always)]
pub fn branch_addr(vaddress: u64, el: u8) -> u64 {
    let msbit = addr_top(el);
    if msbit == 63 {
        return vaddress;
    }

    let load = bits_get(vaddress, 0, msbit + 1);
    let sign_bit_set = bits_get(vaddress, msbit, 1) != 0;
    if (el == 0 || el == 1) && sign_bit_set {
        sign_extend(load, msbit)
    } else {
        zero_extend(load, msbit)
    }
}

pub fn branch_to(cpu: &mut Cpu, target: u64, is_32bit: bool, old_pc: u64) {
    cpu.branch_taken = true;
    if is_32bit {
        cpu.branch_target = old_pc.wrapping_add(target.wrapping_mul(4))
    } else {
        let tgt = branch_addr(target, cpu.pstate.current_el as u8);
        cpu.branch_target = tgt;
    }
}

pub fn instruction_branch(cpu: &mut Cpu, cond: u8, offset: u64, old_pc: u64) {
    if condition_holds(cpu, cond) {
        branch_to(cpu, old_pc.wrapping_add(offset), false, old_pc);
    }
}

pub fn instruction_tbnz(
    cpu: &mut Cpu,
    datasize: u8,
    rt: u8,
    bit_pos: u8,
    offset: u64,
    old_pc: u64,
) {
    let op1 = cpu.x_read(rt as usize, datasize);
    if bits_get(op1, bit_pos, 1) != 0 {
        let is_32 = datasize == 32;
        branch_to(cpu, old_pc.wrapping_add(offset), is_32, old_pc);
    }
}

pub fn instruction_cbnz(cpu: &mut Cpu, datasize: u8, rt: u8, offset: u64, old_pc: u64) {
    let op1 = cpu.x_read(rt as usize, datasize);
    if op1 != 0 {
        let is_32 = datasize == 32;
        branch_to(cpu, old_pc.wrapping_add(offset), is_32, old_pc);
    }
}

pub fn instruction_cbz(cpu: &mut Cpu, datasize: u8, rt: u8, offset: u64, old_pc: u64) {
    let op1 = cpu.x_read(rt as usize, datasize);
    if op1 == 0 {
        let is_32 = datasize == 32;
        branch_to(cpu, old_pc.wrapping_add(offset), is_32, old_pc);
    }
}

pub fn instruction_bl(cpu: &mut Cpu, offset: u64, old_pc: u64) {
    cpu.x_write(30, old_pc.wrapping_add(4), false);
    branch_to(cpu, old_pc.wrapping_add(offset), false, old_pc);
}

pub fn instruction_ret(cpu: &mut Cpu, n: u8, old_pc: u64) {
    let target = cpu.x_read(n as usize, 64);
    branch_to(cpu, target, false, old_pc);
}

pub fn instruction_eret(cpu: &mut Cpu) {
    let _pac = false;
    let _use_key_a = true;
    //check_for_eret_trap(cpu, pac, use_key_a);
    let target = cpu.get_elr_elx();
    let spsr = cpu.get_spsr_elx();
    cpu.aarch64_exception_return(target, spsr);
}

pub fn instruction_bunc(cpu: &mut Cpu, offset: u64, old_pc: u64) {
    branch_to(cpu, old_pc.wrapping_add(offset), false, old_pc);
}

pub fn instruction_msr_imm(
    cpu: &mut Cpu,
    operand: u8,
    _op1: u8,
    _crm: u8,
    min_el: ExceptionLevel,
    field: PstateField,
) {
    // TODO: Implement security features
    if cpu.pstate.current_el < min_el {
        panic!("\r\nCurrent exception level is below required");
    }

    match field {
        PstateField::Sp => {
            cpu.pstate.sp = get_bits_ct!(operand, 0, 1);
        }
        PstateField::Daifset => {
            let m = operand & 0xF;
            if (m & 0b1000) != 0 {
                cpu.pstate.masked_d = true;
            }
            if (m & 0b0100) != 0 {
                cpu.pstate.masked_a = true;
            }
            if (m & 0b0010) != 0 {
                cpu.pstate.masked_i = true;
            }
            if (m & 0b0001) != 0 {
                cpu.pstate.masked_f = true;
            }
        }
        PstateField::Daifclr => {
            let m = operand & 0xF;
            if (m & 0b1000) != 0 {
                cpu.pstate.masked_d = false;
            }
            if (m & 0b0100) != 0 {
                cpu.pstate.masked_a = false;
            }
            if (m & 0b0010) != 0 {
                cpu.pstate.masked_i = false;
            }
            if (m & 0b0001) != 0 {
                cpu.pstate.masked_f = false;
            }
        }
        _ => {}
    }
}
