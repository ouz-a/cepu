#![allow(clippy::too_many_arguments)]

use std::ops::Not;

use crate::{
    cpu::{Cpu, SP_REGISTER},
    utils::{bits_get, insert_16bit_field, zero_extend},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ShiftTypes {
    StLsl = 0b00,
    StLsr = 0b01,
    StAsr = 0b10,
    StRor = 0b11,
}

pub const fn decode_shift(bits: u8) -> ShiftTypes {
    match bits & 0b11 {
        0b00 => ShiftTypes::StLsl,
        0b01 => ShiftTypes::StLsr,
        0b10 => ShiftTypes::StAsr,
        0b11 => ShiftTypes::StRor,
        _ => unreachable!(),
    }
}

#[derive(Debug)]
pub struct AddResult {
    pub result: u64,
    pub n: bool,
    pub z: bool,
    pub c: bool,
    pub v: bool,
}

impl AddResult {
    #[inline]
    pub fn flag_to_bits(&self) -> u8 {
        let mut bits: u8 = 0;
        if self.n {
            bits |= 1 << 3;
        }
        if self.z {
            bits |= 1 << 2;
        }
        if self.c {
            bits |= 1 << 1;
        }
        if self.v {
            bits |= 1;
        }
        bits
    }

    #[inline]
    pub fn set_flags_from_bits(&mut self, bits: u8) {
        self.n = (bits & (1 << 3)) != 0;
        self.z = (bits & (1 << 2)) != 0;
        self.c = (bits & (1 << 1)) != 0;
        self.v = (bits & 1) != 0;
    }
}

pub fn add_with_carry(x: u64, y: u64, carry_in: u64) -> AddResult {
    assert!(carry_in <= 1, "carry_in must be 0 or 1");

    let wide_u = x as u128 + y as u128 + carry_in as u128;
    let result = wide_u as u64;
    let c = wide_u > u64::MAX as u128;

    let wide_s = (x as i64) as i128 + (y as i64) as i128 + carry_in as i128;
    let v = wide_s < i64::MIN as i128 || wide_s > i64::MAX as i128;

    let n = (result as i64) < 0;
    let z = result == 0;

    AddResult { result, n, z, c, v }
}

pub fn instruction_add_immediate(cpu: &mut Cpu, d: u8, n: u8, imm12: u32, is_32b: bool) {
    let operand1 = if d == SP_REGISTER as u8 {
        cpu.sp_read()
    } else {
        cpu.x_read(n as usize, if is_32b { 32 } else { 64 })
    };
    let result = add_with_carry(operand1, imm12 as u64, 0);

    if d == SP_REGISTER as u8 {
        cpu.sp_write(result.result);
    } else {
        cpu.x_write(d as usize, result.result, is_32b);
    }
}

pub fn instruction_imm_sub(cpu: &mut Cpu, d: u8, n: u8, imm24: u32, datasize: u8) {
    let op1 = if n == SP_REGISTER as u8 { cpu.sp_read() } else { cpu.x_read(n as usize, datasize) };
    let op2 = imm24 as u64;

    let res = add_with_carry(op1, !op2, 1);
    if d == SP_REGISTER as u8 {
        cpu.sp_write(res.result);
    } else {
        cpu.x_write(d as usize, res.result, datasize == 32);
    }
}

pub fn instruction_imm_subs(cpu: &mut Cpu, d: u8, n: u8, imm24: u32, datasize: u8) {
    let op1 = if n == SP_REGISTER as u8 { cpu.sp_read() } else { cpu.x_read(n as usize, datasize) };
    let op2 = zero_extend(imm24 as u64, datasize);

    let res = add_with_carry(op1, !op2, 1);
    cpu.pstate.c = res.c;
    cpu.pstate.n = res.n;
    cpu.pstate.z = res.z;
    cpu.pstate.v = res.v;

    let is_32b = datasize == 32;
    // ITS COMP INSTRUCTION
    if d == SP_REGISTER as u8 {
        return;
    }
    cpu.x_write(d as usize, res.result, is_32b);
}

pub fn instruction_multiply_add(cpu: &mut Cpu, d: u8, n: u8, m: u8, a: u8, datasize: u8) {
    let op1 = cpu.x_read(n as usize, datasize);
    let op2 = cpu.x_read(m as usize, datasize);
    let op3 = cpu.x_read(a as usize, datasize);

    let res = op3 + (op1 * op2);
    let is_32b = datasize == 32;
    cpu.x_write(d as usize, res, is_32b);
}

pub fn instruction_udiv(cpu: &mut Cpu, d: u8, n: u8, m: u8, datasize: u8) {
    let op1 = cpu.x_read(n as usize, datasize);
    let op2 = cpu.x_read(m as usize, datasize);

    let is_32b = datasize == 32;
    if op2 == 0 {
        cpu.x_write(d as usize, 0, is_32b);
    } else {
        let result: u64 = op1 / op2;
        cpu.x_write(d as usize, result, is_32b);
    }
}

pub fn instruction_ands_imm(cpu: &mut Cpu, rn: u8, rd: u8, datasize: u8, imm: u64) {
    let op1 = cpu.x_read(rn.into(), datasize);
    let op2 = imm;

    let result = op1 & op2;
    let is_32b = datasize == 32;

    cpu.x_write(rd.into(), result, is_32b);

    let is_zero = if result == 0 { 1 } else { 0 };
    let state = (bits_get(result, datasize - 1, 1) << 3) | (is_zero << 2);
    cpu.pstate.set_flags_from_bits(state.try_into().unwrap());
}

pub fn instruction_orr_imm(cpu: &mut Cpu, rn: u8, rd: u8, datasize: u8, imm: u64) {
    let op1 = cpu.x_read(rn.into(), datasize);
    let op2 = imm;

    let result = op1 | op2;
    let is_32b = datasize == 32;

    if rd == 31 {
        cpu.sp_write(result);
    } else {
        cpu.x_write(rd.into(), result, is_32b);
    }
}

pub fn instruction_ands_shifted_reg(
    cpu: &mut Cpu,
    rm: u8,
    rn: u8,
    rd: u8,
    datasize: u8,
    shift_amount: u8,
    shift_type: ShiftTypes,
) {
    let op1 = cpu.x_read(rn.into(), datasize);
    let op2 = shift_reg(cpu, rm, shift_type, shift_amount, datasize);

    let result = op1 & op2;
    let is_32b = datasize == 32;

    cpu.x_write(rd.into(), result, is_32b);

    let is_zero = if result == 0 { 1 } else { 0 };
    let state = (bits_get(result, datasize - 1, 1) << 3) | (is_zero << 2);
    cpu.pstate.set_flags_from_bits(state.try_into().unwrap());
}

pub fn instruction_adds_shifted_register(
    cpu: &mut Cpu,
    n: u8,
    m: u8,
    d: u8,
    shift_amount: u8,
    s_type: ShiftTypes,
    datasize: u8,
    is_32b: bool,
) {
    let op1 = cpu.x_read(n as usize, datasize);
    let op2 = shift_reg(cpu, m, s_type, shift_amount, datasize);
    let res = add_with_carry(op1, op2, 0);
    cpu.pstate.set_flags_from_bits(res.flag_to_bits());
    cpu.x_write(d as usize, res.result, is_32b);
}

pub fn instruction_subs_shifted_reg(
    cpu: &mut Cpu,
    n: u8,
    m: u8,
    d: u8,
    shift_amount: u8,
    s_type: ShiftTypes,
    datasize: u8,
    is_32b: bool,
) {
    let op1 = cpu.x_read(n as usize, datasize);
    let op2 = shift_reg(cpu, m, s_type, shift_amount, datasize).not();
    let res = add_with_carry(op1, op2, 1);
    println!("res {res:?} flags to bits {:b}", res.flag_to_bits());
    if d != 31 {
        cpu.x_write(d as usize, res.result, is_32b);
    }
    cpu.pstate.set_flags_from_bits(res.flag_to_bits());
    println!("cpu pstate {:?}", cpu.pstate);
}

pub fn instruction_add_shifted_register(
    cpu: &mut Cpu,
    n: u8,
    m: u8,
    d: u8,
    shift_amount: u8,
    s_type: ShiftTypes,
    datasize: u8,
    is_32b: bool,
) {
    let op1 = cpu.x_read(n as usize, datasize);
    let op2 = shift_reg(cpu, m, s_type, shift_amount, datasize);
    let res = add_with_carry(op1, op2, 0);
    cpu.x_write(d as usize, res.result, is_32b);
}

pub fn instruction_sub_shifted_register(
    cpu: &mut Cpu,
    n: u8,
    m: u8,
    d: u8,
    shift_amount: u8,
    s_type: ShiftTypes,
    datasize: u8,
    is_32b: bool,
) {
    let op1 = cpu.x_read(n as usize, datasize);
    let op2 = !shift_reg(cpu, m, s_type, shift_amount, datasize);
    let res = add_with_carry(op1, op2, 1);
    cpu.x_write(d as usize, res.result, is_32b);
}

pub fn instruction_movz(cpu: &mut Cpu, d: u8, imm16: u16, position: u8, is_32b: bool) {
    let result = insert_16bit_field(0, imm16, position as u32);
    cpu.x_write(d as usize, result, is_32b);
}

pub fn instruction_mov(cpu: &mut Cpu, d: u64, n: u64, is_32b: bool) {
    let dat = cpu.x_read(n as usize, if is_32b { 32 } else { 64 });
    cpu.x_write(d as usize, dat, is_32b);
}

pub fn instruction_movn(cpu: &mut Cpu, reg_num: u64, imm16: u16, shift: u8, is_32b: bool) {
    let mut result: u64;
    result = (imm16 as u64) << shift;
    result = !(result);
    cpu.x_write(reg_num as usize, result, is_32b);
}

pub fn instruction_movk(cpu: &mut Cpu, reg_num: usize, imm16: u16, shift: u8, is_32b: bool) {
    let datasize = if is_32b { 32 } else { 64 };

    assert!(shift.is_multiple_of(16) && shift < datasize, "invalid MOVK shift");

    let mut result: u64 = cpu.x_read(reg_num, datasize);

    let field_mask: u64 = 0xFFFFu64.checked_shl(shift.into()).unwrap();
    result = (result & !field_mask) | (((imm16 as u64) << shift) & field_mask);

    if is_32b {
        result &= 0xFFFF_FFFF;
    }

    cpu.x_write(reg_num, result, is_32b);
}

#[inline]
pub fn shift_lsl(x: u64, amount: u8) -> u64 {
    if amount == 0 { x } else { x << (amount as u32) }
}

/// Logical shift-right (LSR)
#[inline]
pub fn shift_lsr(x: u64, amount: u8) -> u64 {
    if amount == 0 { x } else { x >> (amount as u32) }
}

/// Arithmetic shift-right (ASR) – sign-extending
#[inline]
pub fn shift_asr(x: u64, amount: u8) -> u64 {
    if amount == 0 { x } else { ((x as i64) >> (amount as u32)) as u64 }
}

/// Rotate-right (ROR)
#[inline]
pub fn shift_ror(x: u64, amount: u8) -> u64 {
    if amount == 0 { x } else { x.rotate_right(amount as u32) }
}

pub fn shift_reg(cpu: &Cpu, m: u8, s_type: ShiftTypes, shift_amount: u8, datasize: u8) -> u64 {
    let val = cpu.x_read(m as usize, datasize);
    match s_type {
        ShiftTypes::StLsl => shift_lsl(val, shift_amount),
        ShiftTypes::StLsr => shift_lsr(val, shift_amount),
        ShiftTypes::StAsr => shift_asr(val, shift_amount),
        ShiftTypes::StRor => shift_ror(val, shift_amount),
    }
}
