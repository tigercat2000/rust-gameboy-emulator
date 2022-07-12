use nom::{combinator::all_consuming, multi::many1};

use crate::instructions::Instruction;

#[derive(Debug, PartialEq, Eq)]
pub struct ROM {
    instructions: Vec<Instruction>,
}

impl ROM {
    pub fn parse(input: &[u8]) -> Result<Self, nom::Err<nom::error::Error<&[u8]>>> {
        let (_, instructions) = all_consuming(many1(Instruction::parse))(input)?;
        Ok(ROM { instructions })
    }
}

#[cfg(test)]
mod test {
    use crate::instructions::Instruction;

    use super::*;

    #[test]
    fn comprehensive() {
        // NOP, NOP, LD (BC) 0xDEAD, NOP
        let code = [0x00, 0x00, 0x01, 0xAD, 0xDE, 0x00];

        let rom = ROM::parse(&code).expect("ROM should parse fine");

        assert_eq!(
            rom,
            ROM {
                instructions: vec![
                    Instruction::Nop,
                    Instruction::Nop,
                    Instruction::LoadBC_Immediate(0xDEAD),
                    Instruction::Nop,
                ]
            }
        );
    }
}
