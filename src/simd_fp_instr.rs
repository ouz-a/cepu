use crate::cpu::Cpu;

pub fn dup_general_instruction(cpu: &mut Cpu, rn: u8, rd: u8, esize: u8, datasize: u8) {
    let element = cpu.x_read(rn.into(), esize);
    let elements = datasize / esize;
    let mut result: u128 = 0;
    for e in 0..elements {
        result |= (element as u128) << (e * esize);
    }
    cpu.v_write(rd.into(), datasize as u8, result);
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
        cpu.mmu.write_memory_128bit(address as usize, cpu.v_read(rt.into(), datasize / 8));
        if cpu.mmu.faulted {
            return;
        }
    }
    if wback {
        cpu.handle_wback_postindex(postindex, address, offset, datasize, rn);
    }
}
