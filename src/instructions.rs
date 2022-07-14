use bit_field::BitField;
use nom::{
    combinator::map,
    error::VerboseError,
    number::complete::{i8, le_u16, u8},
    IResult,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Condition {
    NZ,
    Z,
    NC,
    C,
}

// Must only pass 2 bits to this
impl From<u8> for Condition {
    fn from(condition: u8) -> Self {
        match condition {
            0 => Self::NZ,
            1 => Self::Z,
            2 => Self::NC,
            3 => Self::C,
            _ => unreachable!(), // Only 2 bits are allowed to be passed here
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Register16Stack {
    BC,
    DE,
    HL,
    AF,
}

/// Must only pass 2 bits to this
impl From<u8> for Register16Stack {
    fn from(register: u8) -> Self {
        match register {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            3 => Self::AF,
            _ => unreachable!(), // Only 2 bits are allowed to be passed here
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Accumulator / Flags operations
pub enum AccumulatorFlagOp {
    /// RLCA
    RotateLeftCarryA,
    /// RRCA
    RotateRightCarryA,
    /// RLA
    RotateLeftA,
    /// RRA
    RotateRightA,
    /// DAA
    DecimalAdjustAfterAddition,
    /// CPL
    ComplementAccumulator,
    /// SCF
    SetCarryFlag,
    /// CCF
    ComplementCarryFlag,
}

/// Must only pass 3 bits to this
impl From<u8> for AccumulatorFlagOp {
    fn from(register: u8) -> Self {
        match register {
            0 => Self::RotateLeftCarryA,
            1 => Self::RotateRightCarryA,
            2 => Self::RotateLeftA,
            3 => Self::RotateRightA,
            4 => Self::DecimalAdjustAfterAddition,
            5 => Self::ComplementAccumulator,
            6 => Self::SetCarryFlag,
            7 => Self::ComplementCarryFlag,
            _ => unreachable!(), // Only 3 bits are allowed to be passed here
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AluOp {
    Add,
    /// ADC
    AddWithCarry,
    Subtract,
    /// SBC
    SubtractWithCarry,
    And,
    Xor,
    Or,
    /// CP
    Compare,
}

/// Must only pass 3 bits to this
impl From<u8> for AluOp {
    fn from(register: u8) -> Self {
        match register {
            0 => Self::Add,
            1 => Self::AddWithCarry,
            2 => Self::Subtract,
            3 => Self::SubtractWithCarry,
            4 => Self::And,
            5 => Self::Xor,
            6 => Self::Or,
            7 => Self::Compare,
            _ => unreachable!(), // Only 3 bits are allowed to be passed here
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BitwiseOp {
    /// RLC
    RotateLeftCarry,
    /// RRC
    RotateRightCarry,
    /// RL
    RotateLeft,
    /// RR
    RotateRight,
    /// SLA
    ShiftLeftArithmetic,
    /// SRA
    ShiftRightArithmetic,
    /// SWAP
    Swap,
    /// SRL
    ShiftRightLogical,
}

/// Must only pass 3 bits to this
impl From<u8> for BitwiseOp {
    fn from(register: u8) -> Self {
        match register {
            0 => Self::RotateLeftCarry,
            1 => Self::RotateRightCarry,
            2 => Self::RotateLeft,
            3 => Self::RotateRight,
            4 => Self::ShiftLeftArithmetic,
            5 => Self::ShiftRightArithmetic,
            6 => Self::Swap,
            7 => Self::ShiftRightLogical,
            _ => unreachable!(), // Only 3 bits are allowed to be passed here
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    /// NOP
    Nop,
    /// LD SP, u16
    LoadSP(u16),
    /// STOP
    Stop,
    /// JR i8
    JumpRelative(i8),
    /// JR Z, i8
    JumpRelativeConditional(Condition, i8),
    /// LD r16, u16
    LoadImmediate16(Register16, u16),
    /// ADD HL, r16
    AddHLRegister(Register16),
    /// LD (r16), A
    LoadIndirectA(Register16Indirect),
    /// LD A, (r16)
    LoadAIndirect(Register16Indirect),
    /// INC r16
    Increment16(Register16),
    /// DEC r16
    Decrement16(Register16),
    /// INC r8
    Increment(Register8),
    /// DEC r8
    Decrement(Register8),
    /// LD r8, u8
    LoadImmediate(Register8, u8),
    /// See [`AccumulatorFlagOp`]
    AccumulatorFlag(AccumulatorFlagOp),
    /// HALT
    Halt,
    /// LD r8, r8
    Load(Register8, Register8),
    /// See [`AluOp`]
    Alu(AluOp, Register8),
    /// RET Z
    RetConditional(Condition),
    /// LDH (n),A / LD (0xFF00 + u8) (n),A
    LoadHighPageA(u8),
    /// ADD SP, i8
    AddSp(i8),
    /// LDH A,(n) / LD (0xFF00 + u8) A,(n)
    LoadAHighPage(u8),
    /// LD HL, SP+u8
    LoadHLSP(i8),
    /// POP r16
    Pop(Register16Stack),
    /// RET
    Ret,
    /// RETI
    RetInterrupt,
    /// JP HL
    JumpHL,
    /// LD SP, HL
    LoadSPHL,
    /// JP Z, u16
    JumpConditional(Condition, u16),
    /// LD (C), A
    LoadHighPageIndirectA,
    /// LD A, (C)
    LoadAHighPageIndirect,
    /// LD (u16), A
    LoadIndirectImmediateA(u16),
    /// LD A, (u16)
    LoadAIndirectImmediate(u16),
    /// JP u16
    Jump(u16),
    /// DI
    DisableInterrupts,
    /// EI
    EnableInterrupts,
    /// CALL Z, u16
    CallConditional(Condition, u16),
    /// CALL u16
    Call(u16),
    /// PUSH r16
    Push(Register16Stack),
    /// See [`AluOp`]
    AluImmediate(AluOp, u8),
    /// RST 00/08/...
    /// Call to 00EXP000
    Reset(u8),
    /// See [`BitwiseOp`]
    Bitwise(BitwiseOp, Register8),
    /// BIT u8, r8
    Bit(u8, Register8),
    /// RES u8, r8
    ResetBit(u8, Register8),
    /// SET u8, r8
    SetBit(u8, Register8),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "Instruction::")?;
        match self {
            Instruction::Nop => write!(f, "Nop"),
            Instruction::LoadSP(immediate) => write!(f, "LoadSP({:#X})", immediate),
            Instruction::Stop => write!(f, "Stop"),
            Instruction::JumpRelative(offset) => write!(f, "JumpRelative({:#X})", offset),
            Instruction::JumpRelativeConditional(condition, offset) => {
                write!(
                    f,
                    "JumpRelativeConditional({:#X?}, {:#X})",
                    condition, offset
                )
            }
            Instruction::LoadImmediate16(register, immediate) => {
                write!(f, "LoadImmediate16({:#X?}, {:#X})", register, immediate)
            }
            Instruction::AddHLRegister(register) => write!(f, "AddHLRegister({:#X?})", register),
            Instruction::LoadIndirectA(register) => write!(f, "LoadIndirectA({:#X?})", register),
            Instruction::LoadAIndirect(register) => write!(f, "LoadAIndirect({:#X?})", register),
            Instruction::Increment16(register) => write!(f, "Increment16({:#X?})", register),
            Instruction::Decrement16(register) => write!(f, "Decrement16({:#X?})", register),
            Instruction::Increment(register) => write!(f, "Increment({:#X?})", register),
            Instruction::Decrement(register) => write!(f, "Decrement({:#X?})", register),
            Instruction::LoadImmediate(register, immediate) => {
                write!(f, "LoadImmediate({:#X?}, {:#X})", register, immediate)
            }
            Instruction::AccumulatorFlag(flag_op) => write!(f, "AccumulatorFlag({:#X?})", flag_op),
            Instruction::Halt => write!(f, "Halt"),
            Instruction::Load(register1, register2) => {
                write!(f, "Load({:#X?}, {:#X?})", register1, register2)
            }
            Instruction::Alu(alu_op, register) => {
                write!(f, "Alu({:#X?}, {:#X?})", alu_op, register)
            }
            Instruction::RetConditional(condition) => {
                write!(f, "RetConditional({:#X?})", condition)
            }
            Instruction::LoadHighPageA(offset) => write!(f, "LoadHighPageA({:#X})", offset),
            Instruction::AddSp(offset) => write!(f, "AddSp({:#X})", offset),
            Instruction::LoadAHighPage(offset) => write!(f, "LoadAHighPage({:#X})", offset),
            Instruction::LoadHLSP(offset) => write!(f, "LoadHLSP({:#X})", offset),
            Instruction::Pop(register) => write!(f, "Pop({:#X?})", register),
            Instruction::Ret => write!(f, "Ret"),
            Instruction::RetInterrupt => write!(f, "RetInterrupt"),
            Instruction::JumpHL => write!(f, "JumpHL"),
            Instruction::LoadSPHL => write!(f, "LoadSPHL"),
            Instruction::JumpConditional(condition, register) => {
                write!(f, "JumpConditional({:#X?}, {:#X?})", condition, register)
            }
            Instruction::LoadHighPageIndirectA => write!(f, "LoadHighPageIndirectA"),
            Instruction::LoadAHighPageIndirect => write!(f, "LoadAHighPageIndirect"),
            Instruction::LoadIndirectImmediateA(immediate) => {
                write!(f, "LoadIndirectImmediateA({:#X})", immediate)
            }
            Instruction::LoadAIndirectImmediate(immediate) => {
                write!(f, "LoadAIndirectImmediate({:#X})", immediate)
            }
            Instruction::Jump(addr) => write!(f, "Jump({:#X})", addr),
            Instruction::DisableInterrupts => write!(f, "DisableInterrupts"),
            Instruction::EnableInterrupts => write!(f, "EnableInterrupts"),
            Instruction::CallConditional(condition, addr) => {
                write!(f, "CallConditional({:#X?}, {:#X})", condition, addr)
            }
            Instruction::Call(addr) => write!(f, "Call({:#X})", addr),
            Instruction::Push(register) => write!(f, "Push({:#X?})", register),
            Instruction::AluImmediate(alu_op, immediate) => {
                write!(f, "AluImmediate({:#X?}, {:#X})", alu_op, immediate)
            }
            Instruction::Reset(reset_vector) => write!(f, "Reset({:#X})", reset_vector),
            Instruction::Bitwise(bit_op, register) => {
                write!(f, "Bitwise({:#X?}, {:#X?})", bit_op, register)
            }
            Instruction::Bit(bit, register) => write!(f, "Bit({:#X}, {:#X?})", bit, register),
            Instruction::ResetBit(bit, register) => {
                write!(f, "ResetBit({:#X}, {:#X?})", bit, register)
            }
            Instruction::SetBit(bit, register) => write!(f, "SetBit({:#X}, {:#X?})", bit, register),
        }
    }
}

impl Instruction {
    pub fn byte_len(&self) -> u16 {
        match *self {
            Instruction::Nop => 1,
            Instruction::LoadSP(_) => 3,
            Instruction::Stop => 2,
            Instruction::JumpRelative(_) => 2,
            Instruction::JumpRelativeConditional(_, _) => 2,
            Instruction::LoadImmediate16(_, _) => 3,
            Instruction::AddHLRegister(_) => 1,
            Instruction::LoadIndirectA(_) => 1,
            Instruction::LoadAIndirect(_) => 1,
            Instruction::Increment16(_) => 1,
            Instruction::Decrement16(_) => 1,
            Instruction::Increment(_) => 1,
            Instruction::Decrement(_) => 1,
            Instruction::LoadImmediate(_, _) => 2,
            Instruction::AccumulatorFlag(_) => 1,
            Instruction::Halt => 1,
            Instruction::Load(_, _) => 1,
            Instruction::Alu(_, _) => 1,
            Instruction::RetConditional(_) => 1,
            Instruction::LoadHighPageA(_) => 2,
            Instruction::AddSp(_) => 2,
            Instruction::LoadAHighPage(_) => 2,
            Instruction::LoadHLSP(_) => 2,
            Instruction::Pop(_) => 1,
            Instruction::Ret => 1,
            Instruction::RetInterrupt => 1,
            Instruction::JumpHL => 1,
            Instruction::LoadSPHL => 1,
            Instruction::JumpConditional(_, _) => 3,
            Instruction::LoadHighPageIndirectA => 1,
            Instruction::LoadAHighPageIndirect => 1,
            Instruction::LoadIndirectImmediateA(_) => 3,
            Instruction::LoadAIndirectImmediate(_) => 3,
            Instruction::Jump(_) => 3,
            Instruction::DisableInterrupts => 1,
            Instruction::EnableInterrupts => 1,
            Instruction::CallConditional(_, _) => 3,
            Instruction::Call(_) => 3,
            Instruction::Push(_) => 1,
            Instruction::AluImmediate(_, _) => 2,
            Instruction::Reset(_) => 1,
            Instruction::Bitwise(_, _) => 2,
            Instruction::Bit(_, _) => 2,
            Instruction::ResetBit(_, _) => 2,
            Instruction::SetBit(_, _) => 2,
        }
    }
}

impl Instruction {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self, VerboseError<&[u8]>> {
        let (rest, opcode) = u8(input)?;

        let (p1, p2, p3) = (
            opcode.get_bits(0..3), // 3 bits
            opcode.get_bits(3..6), // 3 bits
            opcode.get_bits(6..8), // 2 bits
        );

        // We're checking bits and we should be explicit about 0 (false) or 1 (true).
        #[allow(clippy::bool_comparison)]
        match (p3, p2, p1) {
            // Group one
            (0b00, 0b000, 0b000) => Ok((rest, Instruction::Nop)),
            (0b00, 0b001, 0b000) => map(le_u16, Instruction::LoadSP)(rest),
            (0b00, 0b010, 0b000) => Ok((rest, Instruction::Stop)),
            (0b00, 0b011, 0b000) => map(i8, Instruction::JumpRelative)(rest),
            (0b00, condition, 0b000) if condition.get_bit(2) == true => {
                let (rest, offset) = i8(rest)?;
                Ok((
                    rest,
                    Instruction::JumpRelativeConditional(condition.get_bits(0..2).into(), offset),
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
            (0b00, acc_flag_op, 0b111) => {
                Ok((rest, Instruction::AccumulatorFlag(acc_flag_op.into())))
            }
            // Group 2
            (0b01, 0b110, 0b110) => Ok((rest, Instruction::Halt)),
            // Note: (HL, HL) is not a valid LD r8, r8 as it would conflict with HALT
            // This isn't a concern for us because it'll just be read as Halt, which takes precedence as an
            // earlier match expression
            (0b01, reg1, reg2) => Ok((rest, Instruction::Load(reg1.into(), reg2.into()))),
            (0b10, opcode, register) => {
                Ok((rest, Instruction::Alu(opcode.into(), register.into())))
            }
            (0b11, condition, 0b000) if condition.get_bit(2) == false => Ok((
                rest,
                Instruction::RetConditional(condition.get_bits(0..2).into()),
            )),
            (0b11, 0b100, 0b000) => {
                let (rest, offset) = u8(rest)?;
                Ok((rest, Instruction::LoadHighPageA(offset)))
            }
            (0b11, 0b101, 0b000) => {
                let (rest, offset) = i8(rest)?;
                Ok((rest, Instruction::AddSp(offset)))
            }
            (0b11, 0b110, 0b000) => {
                let (rest, offset) = u8(rest)?;
                Ok((rest, Instruction::LoadAHighPage(offset)))
            }
            (0b11, 0b111, 0b000) => {
                let (rest, offset) = i8(rest)?;
                Ok((rest, Instruction::LoadHLSP(offset)))
            }
            (0b11, register, 0b001) if register.get_bit(0) == false => {
                Ok((rest, Instruction::Pop(register.get_bits(1..=2).into())))
            }
            (0b11, opcode, 0b001) if opcode.get_bit(0) == true => {
                let instr = match opcode.get_bits(1..=2) {
                    0 => Instruction::Ret,
                    1 => Instruction::RetInterrupt,
                    2 => Instruction::JumpHL,
                    3 => Instruction::LoadSPHL,
                    _ => unreachable!(), // Only 2 bits are given
                };
                Ok((rest, instr))
            }
            (0b11, condition, 0b010) if condition.get_bit(2) == false => {
                let (rest, address) = le_u16(rest)?;
                Ok((
                    rest,
                    Instruction::JumpConditional(condition.get_bits(0..2).into(), address),
                ))
            }
            (0b11, 0b100, 0b010) => Ok((rest, Instruction::LoadHighPageIndirectA)),
            (0b11, 0b101, 0b010) => {
                let (rest, address) = le_u16(rest)?;
                Ok((rest, Instruction::LoadIndirectImmediateA(address)))
            }
            (0b11, 0b110, 0b010) => Ok((rest, Instruction::LoadAHighPageIndirect)),
            (0b11, 0b111, 0b010) => {
                let (rest, address) = le_u16(rest)?;
                Ok((rest, Instruction::LoadAIndirectImmediate(address)))
            }
            // Group 3
            (0b11, 0b000, 0b011) => {
                let (rest, address) = le_u16(rest)?;
                Ok((rest, Instruction::Jump(address)))
            }
            // CB-prefixed
            (0b11, 0b001, 0b011) => {
                let (rest, opcode) = u8(rest)?;
                Ok((rest, Self::parse_prefixed_cb(opcode)))
            }
            (0b11, 0b110, 0b011) => Ok((rest, Instruction::DisableInterrupts)),
            (0b11, 0b111, 0b011) => Ok((rest, Instruction::EnableInterrupts)),
            (0b11, condition, 0b100) if condition.get_bit(2) == false => {
                let (rest, address) = le_u16(rest)?;
                Ok((
                    rest,
                    Instruction::CallConditional(condition.get_bits(0..2).into(), address),
                ))
            }
            (0b11, 0b001, 0b101) => {
                let (rest, address) = le_u16(rest)?;
                Ok((rest, Instruction::Call(address)))
            }
            (0b11, register, 0b101) if register.get_bit(0) == false => {
                Ok((rest, Instruction::Push(register.get_bits(1..=2).into())))
            }
            (0b11, opcode, 0b110) => {
                let (rest, immediate) = u8(rest)?;
                Ok((rest, Instruction::AluImmediate(opcode.into(), immediate)))
            }
            (0b11, exp, 0b111) => Ok((rest, Instruction::Reset(exp))),
            (a, b, c) => {
                eprintln!(
                    "Illegal instruction {:#X?} ({:#04b}, {:#05b}, {:#05b})",
                    opcode, a, b, c
                );
                unimplemented!()
            }
        }
    }

    fn parse_prefixed_cb(opcode: u8) -> Instruction {
        let (p1, p2, p3) = (
            opcode.get_bits(0..3), // 3 bits
            opcode.get_bits(3..6), // 3 bits
            opcode.get_bits(6..8), // 2 bits
        );
        match (p3, p2, p1) {
            (0b00, opcode, register) => Instruction::Bitwise(opcode.into(), register.into()),
            (0b01, bit, register) => Instruction::Bit(bit, register.into()),
            (0b10, bit, register) => Instruction::ResetBit(bit, register.into()),
            (0b11, bit, register) => Instruction::SetBit(bit, register.into()),
            _ => unimplemented!(),
        }
    }
}
