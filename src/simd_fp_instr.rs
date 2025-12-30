use crate::
    cpu::Cpu
;

pub fn dup_general_instruction(
    cpu: &mut Cpu,
    rn: u8,
    rd: u8,
    esize: u8,
    datasize: u8,
) {
    let element = cpu.x_read(rn.into(), esize);
    let elements = datasize / esize;
    let mut result: u128 = 0;
    for e in 0..elements {
        result |= (element as u128) << (e * esize);
    }
    cpu.v_write(rd.into(), datasize as u8, result);
}
