use crate::{
    cpu::Cpu, get_bits_ct, instruction::{InstDesc, Instruction}, simd_fp_instr::dup_general_instruction, utils::bits_get
};

/// Duplicate general-purpose register to vector
#[derive(Clone, Copy, Debug)]
pub struct DupGeneral {
    pub q: bool,
    pub imm5: u8,
    pub rn: u8,
    pub rd: u8,
}

impl DupGeneral {
    pub fn exec(self, cpu: &mut Cpu, _old_pc: u64) {
        if bits_get(self.imm5.into(), 0, 4) == 0 {
            panic!("Undefined")
        }
        if bits_get(self.imm5.into(), 0, 4) == 0b1000 && !self.q {
            panic!("Undefined")
        }
        let size = bits_get(self.imm5.into(), 0, 3).trailing_zeros();

        let esize = 8 << size;
        let datasize = 64 << self.q as u8;
        dup_general_instruction(cpu, self.rn, self.rd, esize, datasize);
    }
    pub const fn decode(word: u32) -> Instruction {
        let q = get_bits_ct!(word, 31, 1) == 1;
        let imm5 = get_bits_ct!(word, 16, 5) as u8;
        let rn = get_bits_ct!(word, 5, 5) as u8;
        let rd = get_bits_ct!(word, 0, 5) as u8;
        Instruction::DupGeneral(Self { q, imm5, rn, rd })
    }
    pub const DUP_GENERAL: InstDesc = InstDesc {
        mask: 0b1011_1111_1110_0000_1111_1100_0000_0000,
        value: 0b0000_1110_0000_0000_0000_1100_0000_0000,
        decode: Self::decode,
    };
}
