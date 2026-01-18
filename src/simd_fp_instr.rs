use crate::cpu::Cpu;

pub fn dup_general_instruction(cpu: &mut Cpu, rn: u8, rd: u8, esize: u8, datasize: u8) {
    let element = cpu.x_read(rn.into(), esize);
    let elements = datasize / esize;
    let mut result: u128 = 0;
    for e in 0..elements {
        result |= (element as u128) << (e * esize);
    }
    cpu.v_write(rd.into(), datasize, result);
}

pub fn str_imd_fp_instruction(
    cpu: &mut Cpu,
    rn: u8,
    rt: u8,
    datasize: u8,
    postindex: bool,
    wback: bool,
    offset: u64,
) {
    let mut address = cpu.ret_sp_or_reg(rn);

    if !postindex {
        address = address.wrapping_add(offset);
    }

    if datasize == 128 {
        cpu.write_memory_128bit(address as usize, cpu.v_read(rt.into(), datasize));
        if cpu.mmu.faulted {
            return;
        }
    } else {
        cpu.write_memory(
            address as usize,
            (datasize / 8).into(),
            cpu.v_read(rt.into(), datasize) as u64,
        );
    }
    if wback {
        cpu.handle_wback_postindex(postindex, address, offset, datasize, rn);
    }
}

pub fn str_pair_fp_instruction(
    cpu: &mut Cpu,
    rn: u8,
    rt: u8,
    rt2: u8,
    offset: u64,
    datasize: u8,
    postindex: bool,
    wback: bool,
) {
    let dbytes = datasize / 8;
    let mut address = cpu.address_for_rn(rn);

    if !postindex {
        address = address.wrapping_add(offset);
    }

    let address2 = address.wrapping_add(dbytes.into());
    if datasize == 128 {
        cpu.write_memory_128bit(address as usize, cpu.v_read(rt.into(), 128));
        if cpu.mmu.faulted {
            return;
        }
        cpu.write_memory_128bit(address2 as usize, cpu.v_read(rt2.into(), 128));
        if cpu.mmu.faulted {
            return;
        }
    } else {
        cpu.write_memory(
            address as usize,
            (datasize / 8).into(),
            cpu.v_read(rt.into(), datasize) as u64,
        );
        if cpu.mmu.faulted {
            return;
        }
        cpu.write_memory(
            address2 as usize,
            (datasize / 8).into(),
            cpu.v_read(rt2.into(), datasize) as u64,
        );
        if cpu.mmu.faulted {
            return;
        }
    }

    if wback {
        cpu.handle_wback_postindex(postindex, address, offset, datasize, rn);
    }
}

pub fn instruction_ldp_simd_fp(
    cpu: &mut Cpu,
    rt: u8,
    rt2: u8,
    rn: u8,
    datasize: u8,
    offset: u64,
    postindex: bool,
    wback: bool,
) {
    let dbytes: u8 = datasize / 8;
    let mut address = if rn == 31 {
        cpu.check_space_alignment();
        cpu.sp_read()
    } else {
        cpu.x_read(rn.into(), 64)
    };

    if !postindex {
        address = address.wrapping_add(offset);
    }

    let address_2 = address.wrapping_add(dbytes.into());

    let (data1, data2): (u128, u128) = if datasize == 128 {
        // 128-bit loads require special handling
        let (_, d1) = cpu.read_memory_128bit(address.try_into().unwrap());
        if cpu.mmu.faulted {
            return;
        }
        let (_, d2) = cpu.read_memory_128bit(address_2.try_into().unwrap());
        if cpu.mmu.faulted {
            return;
        }
        (d1, d2)
    } else {
        // 32-bit or 64-bit loads
        let (_, d1) = cpu.read_memory(address.try_into().unwrap(), dbytes.into());
        if cpu.mmu.faulted {
            return;
        }
        let (_, d2) = cpu.read_memory(address_2.try_into().unwrap(), dbytes.into());
        if cpu.mmu.faulted {
            return;
        }
        (d1.into(), d2.into())
    };

    cpu.v_write(rt.into(), datasize, data1);
    cpu.v_write(rt2.into(), datasize, data2);

    if wback {
        if postindex {
            address = address.wrapping_add(offset);
        }
        if rn == 31 {
            cpu.sp_write(address);
        } else {
            cpu.x_write(rn.into(), address, false);
        }
    }
}

pub fn instruction_ldr_simd_fp(
    cpu: &mut Cpu,
    rt: u8,
    rn: u8,
    datasize: u8,
    offset: u64,
    postindex: bool,
    wback: bool,
) {
    let dbytes: u8 = datasize / 8;
    let mut address = cpu.address_for_rn(rn);

    if !postindex {
        address = address.wrapping_add(offset);
    }

    let data1 = if datasize == 128 {
        let (_, d1) = cpu.read_memory_128bit(address.try_into().unwrap());
        if cpu.mmu.faulted {
            return;
        }
        if cpu.mmu.faulted {
            return;
        }
        d1
    } else {
        let (_, d1) = cpu.read_memory(address.try_into().unwrap(), dbytes.into());
        if cpu.mmu.faulted {
            return;
        }
        d1.into()
    };

    cpu.v_write(rt.into(), datasize, data1);

    if wback {
        cpu.handle_wback_postindex(postindex, address, offset, datasize, rn);
    }
}
