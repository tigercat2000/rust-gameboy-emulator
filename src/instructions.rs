use nom::{
    number::complete::{le_u16, u8},
    IResult,
};

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,                   // 0x00
    LoadBC_Immediate(u16), // 0x01
    LoadBC_A,              // 0x02
    IncrementBC,           // 0x03
    IncrementB,            // 0x04
    DecrementB,            // 0x05
    LoadB_Immediate(u8),   // 0x06
    RotateLeftA,           // 0x07
    LoadMem_SP(u16),       // 0x08
    AddHL_BC,              // 0x09
    LoadA_BC,              // 0x0A
    DecrementBC,           // 0x0B
    IncrementC,            // 0x0C
    DecrementC,            // 0x0D
    LoadC_Immediate(u8),   // 0x0E
    RotateRightA,          // 0x0F
}

impl Instruction {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (rest, opcode) = u8(input)?;
        match opcode {
            0x00 => Ok((rest, Instruction::Nop)),
            0x01 => {
                let (rest, imm) = le_u16(rest)?;
                Ok((rest, Instruction::LoadBC_Immediate(imm)))
            }
            0x02 => Ok((rest, Instruction::LoadBC_A)),
            0x03 => Ok((rest, Instruction::IncrementBC)),
            0x04 => Ok((rest, Instruction::IncrementB)),
            0x05 => Ok((rest, Instruction::DecrementB)),
            0x06 => {
                let (rest, imm) = u8(rest)?;
                Ok((rest, Instruction::LoadB_Immediate(imm)))
            }
            0x07 => Ok((rest, Instruction::RotateLeftA)),
            0x08 => {
                let (rest, imm) = le_u16(rest)?;
                Ok((rest, Instruction::LoadMem_SP(imm)))
            }
            0x09 => Ok((rest, Instruction::AddHL_BC)),
            0x0A => Ok((rest, Instruction::LoadA_BC)),
            0x0B => Ok((rest, Instruction::DecrementBC)),
            0x0C => Ok((rest, Instruction::IncrementC)),
            0x0D => Ok((rest, Instruction::DecrementC)),
            0x0E => {
                let (rest, imm) = u8(rest)?;
                Ok((rest, Instruction::LoadC_Immediate(imm)))
            }
            0x0F => Ok((rest, Instruction::RotateRightA)),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_success {
        ($name:ident, $input:expr => $output:expr) => {
            #[test]
            fn $name() {
                let (rest, instr) = Instruction::parse(&$input)
                    .expect("Instruction parse failed when it shouldn't have");
                assert_eq!(rest.len(), 0);
                assert_eq!(instr, $output);
            }
        };
    }

    test_success!(nop, [0x00] => Instruction::Nop);
    test_success!(
        load_bc_imm,
        [0x01, 0xAD, 0xDE] => Instruction::LoadBC_Immediate(0xDEAD)
    );
    test_success!(load_bc_a, [0x02] => Instruction::LoadBC_A);
    test_success!(inc_bc, [0x03] => Instruction::IncrementBC);
    test_success!(inc_b, [0x04] => Instruction::IncrementB);
    test_success!(dec_b, [0x05] => Instruction::DecrementB);
    test_success!(load_b_imm, [0x06, 0xDE] => Instruction::LoadB_Immediate(0xDE));
    test_success!(rotate_left_a, [0x07] => Instruction::RotateLeftA);
    test_success!(load_mem_sp, [0x08, 0xAD, 0xDE] => Instruction::LoadMem_SP(0xDEAD));
    test_success!(add_hl_bc, [0x09] => Instruction::AddHL_BC);
    test_success!(load_a_bc, [0x0A] => Instruction::LoadA_BC);
    test_success!(decrement_bc, [0x0B] => Instruction::DecrementBC);
    test_success!(increment_c, [0x0C] => Instruction::IncrementC);
    test_success!(decrement_c, [0x0D] => Instruction::DecrementC);
    test_success!(load_c_imm, [0x0E, 0xDE] => Instruction::LoadC_Immediate(0xDE));
    test_success!(rotate_right_a, [0x0F] => Instruction::RotateRightA);
}
