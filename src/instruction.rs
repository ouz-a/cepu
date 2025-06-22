use crate::{cpu::Cpu, get_bits_ct, utils::get_bits_u32};

#[derive(Debug, Clone, Copy)]
enum RegisterFieldName {}

#[derive(Debug, Clone, Copy)]
enum InstructionName {}

#[derive(Debug, Clone, Copy)]
struct InstructionField {
    pub name: RegisterFieldName,
    pub pos: u8,
    pub len: u8,
}

struct DecodedInstruction {
    def: Instruction,
    fields: Vec<u32>,
    raw_instruction: u32,
}

type InstructionHandler = fn(&mut Cpu, &DecodedInstruction);
#[derive(Debug, Clone, Copy)]
struct Instruction {
    mask: u32,
    value: u32,
    name: InstructionName,
    handler: InstructionHandler,
    fields: &'static [InstructionField],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstructionGroup {
    GroupSme,
    GroupUnallocated,
    GroupSve,
    GroupDpImmediate,
    GroupBranchExcSys,
    GroupDpRegister,
    GroupDpScalarSimd,
    GroupLoadStore,
}

impl InstructionGroup {
    fn match_instruction_group(op1: u8) -> InstructionGroup {
        let sub_group: InstructionSubGroup = op1.into();
        sub_group.to_main_group()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum InstructionSubGroup {
    Sme = 0b0000,
    Unallocated0 = 0b0001,
    Sve = 0b0010,
    Unallocated1 = 0b0011,
    DpImmediate0 = 0b1001,
    DpImmediate1 = 0b1000,
    BranchExcSys0 = 0b1010,
    BranchExcSys1 = 0b1011,
    DpRegister0 = 0b0101,
    DpRegister1 = 0b1101,
    DpScalarSimd0 = 0b0111,
    DpScalarSimd1 = 0b1111,
    LoadStore0 = 0b0100,
    LoadStore1 = 0b0110,
    LoadStore2 = 0b1110,
    LoadStore3 = 0b1100,
}

impl InstructionSubGroup {
    fn to_main_group(&self) -> InstructionGroup {
        match self {
            InstructionSubGroup::Sme => InstructionGroup::GroupSme,
            InstructionSubGroup::Sve => InstructionGroup::GroupSve,
            InstructionSubGroup::Unallocated0 | InstructionSubGroup::Unallocated1 => {
                InstructionGroup::GroupUnallocated
            }
            InstructionSubGroup::DpImmediate0 | InstructionSubGroup::DpImmediate1 => {
                InstructionGroup::GroupDpImmediate
            }
            InstructionSubGroup::BranchExcSys0 | InstructionSubGroup::BranchExcSys1 => {
                InstructionGroup::GroupBranchExcSys
            }
            InstructionSubGroup::DpRegister0 | InstructionSubGroup::DpRegister1 => {
                InstructionGroup::GroupDpRegister
            }
            InstructionSubGroup::DpScalarSimd0 | InstructionSubGroup::DpScalarSimd1 => {
                InstructionGroup::GroupDpScalarSimd
            }
            InstructionSubGroup::LoadStore0
            | InstructionSubGroup::LoadStore1
            | InstructionSubGroup::LoadStore2
            | InstructionSubGroup::LoadStore3 => InstructionGroup::GroupLoadStore,
        }
    }
}

impl From<u8> for InstructionSubGroup {
    fn from(value: u8) -> Self {
        match value {
            0b0000 => Self::Sme,
            0b0001 => Self::Unallocated0,
            0b0010 => Self::Sve,
            0b0011 => Self::Unallocated1,
            0b1001 => Self::DpImmediate0,
            0b1000 => Self::DpImmediate1,
            0b1010 => Self::BranchExcSys0,
            0b1011 => Self::BranchExcSys1,
            0b0101 => Self::DpRegister0,
            0b1101 => Self::DpRegister1,
            0b0111 => Self::DpScalarSimd0,
            0b1111 => Self::DpScalarSimd1,
            0b0100 => Self::LoadStore0,
            0b0110 => Self::LoadStore1,
            0b1110 => Self::LoadStore2,
            0b1100 => Self::LoadStore3,
            _ => panic!("Unexpected InstructionSubGroup value: {value}"),
        }
    }
}
const DP_IMMEDIATE_INSTRUCTIONS: [Instruction; 0] = { [] };
const BRANCH_EXC_SYS_INSTRUCTIONS: [Instruction; 0] = { [] };
const DP_REGISTER_INSTRUCTIONS: [Instruction; 0] = { [] };
const LOAD_AND_STORE_INSTRUCTIONS: [Instruction; 0] = { [] };

fn decode_instruction(raw_instruction: u32, decoded: &mut DecodedInstruction) {
    let op1 = get_bits_ct!(raw_instruction, 25, 4) as u8;
    let group = InstructionGroup::match_instruction_group(op1);

    let table;

    match group {
        InstructionGroup::GroupDpImmediate => {
            table = DP_IMMEDIATE_INSTRUCTIONS;
        }
        InstructionGroup::GroupBranchExcSys => {
            table = BRANCH_EXC_SYS_INSTRUCTIONS;
        }
        InstructionGroup::GroupDpRegister => {
            table = DP_REGISTER_INSTRUCTIONS;
        }
        InstructionGroup::GroupLoadStore => {
            table = LOAD_AND_STORE_INSTRUCTIONS;
        }
        _ => panic!("Unimplemented instruction group {:?}", group),
    }
    for instruction in table.iter() {
        if (raw_instruction & instruction.mask) == instruction.value {
            decoded.def = *instruction;
            decoded.raw_instruction = raw_instruction;
            for field in instruction.fields.iter() {
                decoded.fields.push(get_bits_u32(
                    raw_instruction,
                    field.pos.into(),
                    field.len.into(),
                ));
            }
        }
    }
    if decoded.raw_instruction == 0 {
        panic!("Unimplemented instruction: Dec {raw_instruction} Hex {raw_instruction:X}")
    }
}
