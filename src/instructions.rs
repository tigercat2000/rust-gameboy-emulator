use bit_field::BitField;
use nom::{
    combinator::map,
    number::complete::{i8, le_u16, u8},
    IResult,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Condition {
    NZ,
    Z,
    NC,
    C,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Register16 {
    BC,
    DE,
    HL,
    SP,
}

/// Must only pass 2 bits to this
impl From<u8> for Register16 {
    fn from(register: u8) -> Self {
        match register {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            3 => Self::SP,
            _ => unreachable!(), // Only 2 bits are allowed to be passed here
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Register8 {
    B,
    C,
    D,
    E,
    H,
    L,
    IndirectHL,
    A,
}

/// Must only pass 3 bits to this
impl From<u8> for Register8 {
    fn from(register: u8) -> Self {
        match register {
            0 => Self::B,
            1 => Self::C,
            2 => Self::D,
            3 => Self::E,
            4 => Self::H,
            5 => Self::L,
            6 => Self::IndirectHL,
            7 => Self::A,
            _ => unreachable!(), // Only 3 bits are allowed to be passed here
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Register16Indirect {
    BC,
    DE,
    /// HL increment
    HLI,
    /// HL decrement
    HLD,
}

/// Must only pass 2 bits to this
impl From<u8> for Register16Indirect {
    fn from(register: u8) -> Self {
        match register {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HLI,
            3 => Self::HLD,
            _ => unreachable!(), // Only 2 bits are allowed to be passed here
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Nop,
    LoadSP(u16),
    JumpRelative(i8),
    JumpRelativeConditional(Condition, i8),
    LoadImmediate16(Register16, u16),
    AddHLRegister(Register16),
    LoadIndirectA(Register16Indirect),
    LoadAIndirect(Register16Indirect),
    Increment16(Register16),
    Decrement16(Register16),
    Increment(Register8),
    Decrement(Register8),
    LoadImmediate(Register8, u8),
}

impl Instruction {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (rest, opcode) = u8(input)?;

        let (p1, p2, p3) = (
            opcode.get_bits(0..3),
            opcode.get_bits(3..6),
            opcode.get_bits(6..8),
        );

        // We're checking bits and we should be explicit about 0 (false) or 1 (true).
        #[allow(clippy::bool_comparison)]
        match (p3, p2, p1) {
            (0b00, 0b000, 0b000) => Ok((rest, Instruction::Nop)),
            (0b00, 0b001, 0b000) => map(le_u16, Instruction::LoadSP)(rest),
            (0b00, 0b011, 0b000) => map(i8, Instruction::JumpRelative)(rest),
            (0b00, condition, 0b000) if condition.get_bit(2) == true => {
                let (rest, offset) = i8(rest)?;
                let condition = match condition.get_bits(0..2) {
                    0 => Condition::NZ,
                    1 => Condition::Z,
                    2 => Condition::NC,
                    3 => Condition::C,
                    _ => unreachable!(), // 2 bits can only represent 0..=3
                };
                Ok((
                    rest,
                    Instruction::JumpRelativeConditional(condition, offset),
                ))
            }
            (0b00, register, 0b001) if register.get_bit(0) == false => {
                let register = register.get_bits(1..=2).into();
                let (rest, immediate) = le_u16(rest)?;
                Ok((rest, Instruction::LoadImmediate16(register, immediate)))
            }
            (0b00, register, 0b001) if register.get_bit(0) == true => {
                let register = register.get_bits(1..=2).into();
                Ok((rest, Instruction::AddHLRegister(register)))
            }
            (0b00, register, 0b010) if register.get_bit(0) == false => {
                let register = register.get_bits(1..=2).into();
                Ok((rest, Instruction::LoadIndirectA(register)))
            }
            (0b00, register, 0b010) if register.get_bit(0) == true => {
                let register = register.get_bits(1..=2).into();
                Ok((rest, Instruction::LoadAIndirect(register)))
            }
            (0b00, register, 0b011) if register.get_bit(0) == false => {
                let register = register.get_bits(1..=2).into();
                Ok((rest, Instruction::Increment16(register)))
            }
            (0b00, register, 0b011) if register.get_bit(0) == true => {
                let register = register.get_bits(1..=2).into();
                Ok((rest, Instruction::Decrement16(register)))
            }
            (0b00, register, 0b100) => Ok((rest, Instruction::Increment(register.into()))),
            (0b00, register, 0b101) => Ok((rest, Instruction::Decrement(register.into()))),
            (0b00, register, 0b110) => {
                let register = register.into();
                let (rest, immediate) = u8(rest)?;
                Ok((rest, Instruction::LoadImmediate(register, immediate)))
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
/// Note: We deliberately use hex instead of binary to make our tests match a gb opcode table
/// instead of catering to our parser. The point of tests is to make sure the parser works correctly,
/// so carefully planning out bits to make it work is a flawed methodology.
mod test {
    #![allow(non_snake_case)]
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
    test_success!(load_sp, [0x08, 0xAD, 0xDE] => Instruction::LoadSP(0xDEAD));
    // JR
    test_success!(jr_unconditional, [0x18, 0xA] => Instruction::JumpRelative(0xA));
    test_success!(jr_Z, [0x28, 0xA] => Instruction::JumpRelativeConditional(Condition::Z, 0xA));
    test_success!(jr_C, [0x38, 0xA] => Instruction::JumpRelativeConditional(Condition::C, 0xA));
    test_success!(jr_NZ, [0x20, 0xA] => Instruction::JumpRelativeConditional(Condition::NZ, 0xA));
    test_success!(jr_NC, [0x30, 0xA] => Instruction::JumpRelativeConditional(Condition::NC, 0xA));
    // LD
    test_success!(ld_bc_imm, [0x01, 0xAD, 0xDE] => Instruction::LoadImmediate16(Register16::BC, 0xDEAD));
    test_success!(ld_de_imm, [0x11, 0xAD, 0xDE] => Instruction::LoadImmediate16(Register16::DE, 0xDEAD));
    test_success!(ld_hl_imm, [0x21, 0xAD, 0xDE] => Instruction::LoadImmediate16(Register16::HL, 0xDEAD));
    test_success!(ld_sp_imm, [0x31, 0xAD, 0xDE] => Instruction::LoadImmediate16(Register16::SP, 0xDEAD));
    // Add HL
    test_success!(add_hl_bc, [0x09] => Instruction::AddHLRegister(Register16::BC));
    test_success!(add_hl_de, [0x19] => Instruction::AddHLRegister(Register16::DE));
    test_success!(add_hl_hl, [0x29] => Instruction::AddHLRegister(Register16::HL));
    test_success!(add_hl_sp, [0x39] => Instruction::AddHLRegister(Register16::SP));
    // Load indirect A
    test_success!(load_indirect_a_bc, [0x02] => Instruction::LoadIndirectA(Register16Indirect::BC));
    test_success!(load_indirect_a_de, [0x12] => Instruction::LoadIndirectA(Register16Indirect::DE));
    test_success!(load_indirect_a_hli, [0x22] => Instruction::LoadIndirectA(Register16Indirect::HLI));
    test_success!(load_indirect_a_hld, [0x32] => Instruction::LoadIndirectA(Register16Indirect::HLD));
    // Load A indirect
    test_success!(load_a_bc_indirect, [0x0A] => Instruction::LoadAIndirect(Register16Indirect::BC));
    test_success!(load_a_de_indirect, [0x1A] => Instruction::LoadAIndirect(Register16Indirect::DE));
    test_success!(load_a_hli_indirect, [0x2A] => Instruction::LoadAIndirect(Register16Indirect::HLI));
    test_success!(load_a_hld_indirect, [0x3A] => Instruction::LoadAIndirect(Register16Indirect::HLD));
    // Increment 16
    test_success!(increment_bc, [0x03] => Instruction::Increment16(Register16::BC));
    test_success!(increment_de, [0x13] => Instruction::Increment16(Register16::DE));
    test_success!(increment_hl, [0x23] => Instruction::Increment16(Register16::HL));
    test_success!(increment_sp, [0x33] => Instruction::Increment16(Register16::SP));
    // Decrement 16
    test_success!(decrement_bc, [0x0B] => Instruction::Decrement16(Register16::BC));
    test_success!(decrement_de, [0x1B] => Instruction::Decrement16(Register16::DE));
    test_success!(decrement_hl, [0x2B] => Instruction::Decrement16(Register16::HL));
    test_success!(decrement_sp, [0x3B] => Instruction::Decrement16(Register16::SP));
    // Increment 8
    test_success!(increment_b, [0x04] => Instruction::Increment(Register8::B));
    test_success!(increment_d, [0x14] => Instruction::Increment(Register8::D));
    test_success!(increment_h, [0x24] => Instruction::Increment(Register8::H));
    test_success!(increment_indirect, [0x34] => Instruction::Increment(Register8::IndirectHL));
    test_success!(increment_c, [0x0C] => Instruction::Increment(Register8::C));
    test_success!(increment_e, [0x1C] => Instruction::Increment(Register8::E));
    test_success!(increment_l, [0x2C] => Instruction::Increment(Register8::L));
    test_success!(increment_a, [0x3C] => Instruction::Increment(Register8::A));
    // Decrement 8
    test_success!(decrement_b, [0x05] => Instruction::Decrement(Register8::B));
    test_success!(decrement_d, [0x15] => Instruction::Decrement(Register8::D));
    test_success!(decrement_h, [0x25] => Instruction::Decrement(Register8::H));
    test_success!(decrement_indirect, [0x35] => Instruction::Decrement(Register8::IndirectHL));
    test_success!(decrement_c, [0x0D] => Instruction::Decrement(Register8::C));
    test_success!(decrement_e, [0x1D] => Instruction::Decrement(Register8::E));
    test_success!(decrement_l, [0x2D] => Instruction::Decrement(Register8::L));
    test_success!(decrement_a, [0x3D] => Instruction::Decrement(Register8::A));
    // Load Immediate 8
    test_success!(load_b_imm, [0x06, 0x69] => Instruction::LoadImmediate(Register8::B, 0x69));
    test_success!(load_d_imm, [0x16, 0x69] => Instruction::LoadImmediate(Register8::D, 0x69));
    test_success!(load_h_imm, [0x26, 0x69] => Instruction::LoadImmediate(Register8::H, 0x69));
    test_success!(load_indirect_imm, [0x36, 0x69] => Instruction::LoadImmediate(Register8::IndirectHL, 0x69));

    test_success!(load_c_imm, [0x0E, 0x69] => Instruction::LoadImmediate(Register8::C, 0x69));
    test_success!(load_e_imm, [0x1E, 0x69] => Instruction::LoadImmediate(Register8::E, 0x69));
    test_success!(load_l_imm, [0x2E, 0x69] => Instruction::LoadImmediate(Register8::L, 0x69));
    test_success!(load_a_imm, [0x3E, 0x69] => Instruction::LoadImmediate(Register8::A, 0x69));
}
