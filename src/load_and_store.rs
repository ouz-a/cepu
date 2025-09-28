#![allow(clippy::too_many_arguments)]

use crate::{
    cpu::{Cpu, SP_REGISTER},
    memory::AccessDescriptor,
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
        address = address.wrapping_add(offset);
    }
    let data = if wb_unknown {
        panic!("Unpredictable");
    } else {
        cpu.bus.read_memory(address as usize, datasize / 8)
    };
    cpu.x_write(t as usize, data.1, datasize == 32);

    if wback {
        if postindex {
            address = address.wrapping_add(offset);
        }
        if n == SP_REGISTER as u8 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(n as usize, address, false);
        }
    }
}

pub fn instruction_ldp(
    cpu: &mut Cpu,
    t: u8,
    t2: u8,
    n: u8,
    datasize: u8,
    offset: u64,
    wback: bool,
    postindex: bool,
) {
    let dbytes = datasize / 8;
    let post_index = postindex;
    let non_temporal = false;
    let tag_checked = false;
    //    let rt_unknown = false;

    let privileged = !cpu.pstate.current_el.is_el0();
    let _acc_descr = AccessDescriptor::create_acc_descr_gpr(
        crate::memory::MemOp::Store,
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

    if !post_index {
        address = address.wrapping_add(offset);
    }

    let address2 = address + dbytes as u64;
    let data1 = cpu.bus.read_memory(address as usize, dbytes.into());
    let data2 = cpu.bus.read_memory(address2 as usize, dbytes.into());
    cpu.x_write(t.into(), data1.1, datasize == 32);
    cpu.x_write(t2.into(), data2.1, datasize == 32);

    if wback {
        if post_index {
            address = address.wrapping_add(offset);
        }
        if n == 31 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(n.into(), address, false);
        }
    }
}

pub fn instruction_stp(
    cpu: &mut Cpu,
    t: u8,
    t2: u8,
    n: u8,
    datasize: u8,
    offset: u64,
    wback: bool,
    postindex: bool,
) {
    let dbytes = datasize / 8;
    let post_index = postindex;
    let non_temporal = false;
    let tag_checked = false;
    let rt_unknown = false;

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

    if !post_index {
        address = address.wrapping_add(offset);
    }

    let data1 = if rt_unknown && t == n {
        //  data1 = bits(datasize) UNKNOWN;
        panic!("Figure out what does line above means")
    } else {
        cpu.x_read(t.into(), datasize)
    };

    let data2 = if rt_unknown && t == n {
        //  data2 = bits(datasize) UNKNOWN;
        panic!("Figure out what does line above means")
    } else {
        cpu.x_read(t2.into(), datasize)
    };

    let address2 = address + dbytes as u64;
    cpu.bus.write_memory(address as usize, dbytes.into(), data1);
    cpu.bus.write_memory(address2 as usize, dbytes.into(), data2);

    if wback {
        if post_index {
            address = address.wrapping_add(offset);
        }
        if n == 31 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(n.into(), address, false);
        }
    }
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

    address = address.wrapping_add(offset);

    let (_, data) = cpu.bus.read_memory(address as usize, (datasize / 8) as usize);
    let is_32b = reg_size == 32;
    cpu.x_write(t as usize, data, is_32b);
}

pub fn instruction_ldr_literal(cpu: &mut Cpu, t: u8, size: u8, offset: u64, _old_pc: u64) {
    let address = offset;
    let privileged = !cpu.pstate.current_el.is_el0();
    let _access_descrip = AccessDescriptor::create_acc_descr_gpr(
        crate::memory::MemOp::Load,
        false,
        privileged,
        false,
    );

    let is_64b = size * 8 >= 64;
    let (_, word) = cpu.bus.read_memory(address as usize, size as usize);
    cpu.x_write(t as usize, word, !is_64b);
}
pub fn instruction_str_halfword_imm(
    cpu: &mut Cpu,
    n: u8,
    t: u8,
    offset: u64,
    postindex: bool,
    wback: bool,
) {
    let mut address;
    if n == SP_REGISTER as u8 {
        address = cpu.sp_read();
    } else {
        address = cpu.x_read(n.into(), 64);
    };
    if !postindex {
        address = address.wrapping_add(offset);
    }

    let data = cpu.x_read(t.into(), 16);
    cpu.bus.write_memory(address as usize, 2, data);
    if wback {
        if postindex {
            address = address.wrapping_add(offset);
        }
        if n == SP_REGISTER as u8 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(n.into(), address, false);
        }
    }
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
        crate::memory::MemOp::Store,
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
        address = address.wrapping_add(offset);
    }

    let data =
        if rt_unknown { panic!("Unpredictable") } else { cpu.x_read(t as usize, datasize as u8) };
    cpu.bus.write_memory(address as usize, datasize / 8, data);
    if wback {
        if postindex {
            address = address.wrapping_add(offset);
        }
        if n == SP_REGISTER as u8 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(n as usize, address, false);
        }
    }
}
pub fn instruction_strb_imm_un_off(
    cpu: &mut Cpu,
    n: u8,
    t: u8,
    offset: u64,
    postindex: bool,
    wback: bool,
    non_temporal: bool,
    tag_checked: bool,
    rt_unknown: bool,
) {
    let privileged = !cpu.pstate.current_el.is_el0();
    let _acc_descr = AccessDescriptor::create_acc_descr_gpr(
        crate::memory::MemOp::Store,
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
        address = address.wrapping_add(offset);
    }

    let data = if rt_unknown { panic!("Unpredictable") } else { cpu.x_read(t as usize, 8) };
    cpu.bus.write_memory(address as usize, 1, data);
    if wback {
        if postindex {
            address = address.wrapping_add(offset);
        }
        if n == SP_REGISTER as u8 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(n as usize, address, false);
        }
    }
}

pub fn instruction_str_register(
    cpu: &mut Cpu,
    n: u8,
    t: u8,
    m: u8,
    exttype: ExtendType,
    shift: u8,
    datasize: u8,
) {
    let privileged = !cpu.pstate.current_el.is_el0();
    let _acc_descr = AccessDescriptor::create_acc_descr_gpr(
        crate::memory::MemOp::Store,
        false,
        privileged,
        true,
    );
    let offset = extend_register(cpu, m, exttype, shift, 64);
    let mut address;
    if n == SP_REGISTER as u8 {
        cpu.check_space_alignment();
        address = cpu.sp_read();
    } else {
        address = cpu.x_read(n as usize, 64);
    }
    address = address.wrapping_add(offset);

    cpu.bus.write_memory(address as usize, (datasize / 8).into(), cpu.x_read(t.into(), datasize));
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
