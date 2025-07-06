#![allow(clippy::too_many_arguments)]

use crate::{
    cpu::{Cpu, SP_REGISTER},
    memory::{AccessDescriptor, read_memory},
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtendType {
    // Unsigned extensions (for indices/offsets)
    UxTb = 0b000, // zero-extend byte   (8 bits)  → 64 bits
    UxTh = 0b001, // zero-extend half   (16 bits) → 64 bits
    UxTw = 0b010, // zero-extend word   (32 bits) → 64 bits
    UxTx = 0b011, // zero-extend double (64 bits) → 64 bits

    // Signed extensions (for signed offsets)
    SxTb = 0b100, // sign-extend byte   (8 bits)  → 64 bits
    SxTh = 0b101, // sign-extend half   (16 bits) → 64 bits
    SxTw = 0b110, // sign-extend word   (32 bits) → 64 bits
    SxTx = 0b111, // sign-extend double (64 bits) → 64 bits
}

impl ExtendType {
    #[inline]
    pub const fn from_u8(bits: u8) -> Self {
        match bits {
            0b000 => Self::UxTb,
            0b001 => Self::UxTh,
            0b010 => Self::UxTw,
            0b011 => Self::UxTx,
            0b100 => Self::SxTb,
            0b101 => Self::SxTh,
            0b110 => Self::SxTw,
            0b111 => Self::SxTx,
            _ => panic!("invalid ExtendType bits"),
        }
    }
}
pub fn instruction_ldr_imm_base(
    cpu: &mut Cpu,
    n: u8,
    t: u8,
    datasize: usize,
    offset: u64,
    postindex: bool,
    wback: bool,
    non_temporal: bool,
    tag_checked: bool,
    wb_unknown: bool,
) {
    let privileged = !cpu.pstate.current_el.is_el0();
    let _acc_descr = AccessDescriptor::create_acc_descr_gpr(
        crate::memory::MemOp::Load,
        non_temporal,
        privileged,
        tag_checked,
    );
    let mut address;
    if n == SP_REGISTER as u8 {
        cpu.check_space_alignment();
        address = cpu.sp_read();
    } else {
        address = cpu.x_read(n as usize, 64);
    }
    if !postindex {
        address += offset;
    }
    let data = if wb_unknown {
        panic!("Unpredictable");
    } else {
        read_memory(address as usize, datasize / 8)
    };
    cpu.x_write(t as usize, data.1, datasize == 32);

    if wback {
        if postindex {
            address += offset;
        }
        if n == SP_REGISTER as u8 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(n as usize, address, true);
        }
    }
    cpu.pc += 4;
}

pub fn instruction_ldr_register(
    cpu: &mut Cpu,
    t: u8,
    n: u8,
    m: u8,
    datasize: u64,
    reg_size: u8,
    shift: u64,
    exttype: ExtendType,
    non_temporal: bool,
    tag_checked: bool,
) {
    let offset = extend_register(cpu, m, exttype, shift as u8, 64);
    let privileged = !cpu.pstate.current_el.is_el0();
    let _acc_desc = AccessDescriptor::create_acc_descr_gpr(
        crate::memory::MemOp::Load,
        non_temporal,
        privileged,
        tag_checked,
    );

    let mut address;
    if n == SP_REGISTER as u8 {
        cpu.check_space_alignment();
        address = cpu.sp_read();
    } else {
        address = cpu.x_read(n as usize, 64);
    }

    address += offset;

    let (_, data) = read_memory(address as usize, (datasize / 8) as usize);
    let is_32b = reg_size == 32;
    cpu.x_write(t as usize, data, is_32b);
    cpu.pc += 4;
}

pub fn instruction_ldr_literal(cpu: &mut Cpu, t: u8, size: u8, offset: u64) {
    let address = cpu.pc + offset;
    let privileged = !cpu.pstate.current_el.is_el0();
    let _access_descrip = AccessDescriptor::create_acc_descr_gpr(
        crate::memory::MemOp::Load,
        false,
        privileged,
        false,
    );

    let is_32b = size * 8 >= 64;
    let (_, word) = read_memory(address as usize, size as usize);
    cpu.x_write(t as usize, word, is_32b);
    cpu.pc += 4;
}

pub fn instruction_str_imm_un_off(
    cpu: &mut Cpu,
    n: u8,
    t: u8,
    datasize: usize,
    offset: u64,
    postindex: bool,
    wback: bool,
    non_temporal: bool,
    tag_checked: bool,
    rt_unknown: bool,
) {
    let privileged = !cpu.pstate.current_el.is_el0();
    let _acc_descr = AccessDescriptor::create_acc_descr_gpr(
        crate::memory::MemOp::Load,
        non_temporal,
        privileged,
        tag_checked,
    );
    let mut address;
    if n == SP_REGISTER as u8 {
        cpu.check_space_alignment();
        address = cpu.sp_read();
    } else {
        address = cpu.x_read(n as usize, 64);
    }
    if !postindex {
        address += offset;
    }

    let data =
        if rt_unknown { panic!("Unpredictable") } else { cpu.x_read(t as usize, datasize as u8) };
    cpu.x_write(t as usize, data, datasize == 32);

    if wback {
        if postindex {
            address += offset;
        }
        if n == SP_REGISTER as u8 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(n as usize, address, true);
        }
    }
    cpu.pc += 4;
}

#[inline(always)]
pub fn extend_register(cpu: &Cpu, reg: u8, exttype: ExtendType, shift: u8, n: u8) -> u64 {
    assert!(shift <= 4, "shift must be in the range 0..=4");

    let val: u64 = cpu.x_read(reg as usize, n);

    // (is_unsigned, source_len_bits)
    let (is_unsigned, len): (bool, u8) = match exttype {
        ExtendType::UxTb => (true, 8),
        ExtendType::UxTh => (true, 16),
        ExtendType::UxTw => (true, 32),
        ExtendType::UxTx => (true, 64),

        ExtendType::SxTb => (false, 8),
        ExtendType::SxTh => (false, 16),
        ExtendType::SxTw => (false, 32),
        ExtendType::SxTx => (false, 64),
    };

    let nbits = len.min(n);
    assert!(nbits > 0 && nbits <= 64);

    let mask: u64 = if nbits == 64 { u64::MAX } else { (1u64 << nbits) - 1 };
    let truncated_val = val & mask;

    let extval: u64 = if is_unsigned {
        truncated_val
    } else {
        let sign_bit = 1u64 << (nbits - 1);
        (truncated_val ^ sign_bit).wrapping_sub(sign_bit)
    };

    extval << shift
}
