use bit_field::BitField;
use tracing::trace;

use crate::emulator::{
    instructions::{AccumulatorFlagOp, AluOp, BitwiseOp, Instruction, Register16, Register8},
    memory_bus::MemoryBus,
};

use super::{Flag, CPU};

pub fn handle_instruction(
    cpu: &mut CPU,
    instr: Instruction,
    memory_bus: &MemoryBus,
) -> Option<u32> {
    match instr {
        // 8 bit ALU operations
        Instruction::AccumulatorFlag(af_op) => match af_op {
            AccumulatorFlagOp::RotateLeftCarryA => {
                cpu.Accumulator = ALU::rotate_left_carry(cpu, cpu.Accumulator);
            }
            AccumulatorFlagOp::RotateRightCarryA => {
                cpu.Accumulator = ALU::rotate_right_carry(cpu, cpu.Accumulator);
            }
            AccumulatorFlagOp::RotateLeftA => {
                cpu.Accumulator = ALU::rotate_left(cpu, cpu.Accumulator);
            }
            AccumulatorFlagOp::RotateRightA => {
                cpu.Accumulator = ALU::rotate_right(cpu, cpu.Accumulator);
            }
            AccumulatorFlagOp::DecimalAdjustAfterAddition => {
                cpu.Accumulator = ALU::decimal_adjust_after_addition(cpu, cpu.Accumulator);
            }
            AccumulatorFlagOp::ComplementAccumulator => {
                cpu.Accumulator = !cpu.Accumulator;
                cpu.set_flag(Flag::N, true);
                cpu.set_flag(Flag::H, true);
            }
            AccumulatorFlagOp::SetCarryFlag => {
                cpu.set_flag(Flag::N, false);
                cpu.set_flag(Flag::H, false);
                cpu.set_flag(Flag::C, true);
            }
            AccumulatorFlagOp::ComplementCarryFlag => {
                cpu.set_flag(Flag::N, false);
                cpu.set_flag(Flag::H, false);
                cpu.set_flag(Flag::C, !cpu.get_flag(Flag::C));
            }
        },
        Instruction::Alu(alu_op, register) => {
            ALU::handle_op(cpu, alu_op, cpu.read_register(register, memory_bus))
        }
        Instruction::AluImmediate(alu_op, immediate) => ALU::handle_op(cpu, alu_op, immediate),
        Instruction::Bitwise(op, register) => {
            ALU::handle_bitwise(cpu, op, register, memory_bus);
        }
        // 8-bit INC/DEC
        Instruction::Increment(reg) => match reg {
            Register8::B => cpu.B = ALU::increment(cpu, cpu.B),
            Register8::C => cpu.C = ALU::increment(cpu, cpu.C),
            Register8::D => cpu.D = ALU::increment(cpu, cpu.D),
            Register8::E => cpu.E = ALU::increment(cpu, cpu.E),
            Register8::H => cpu.H = ALU::increment(cpu, cpu.H),
            Register8::L => cpu.L = ALU::increment(cpu, cpu.L),
            Register8::IndirectHL => {
                let addr = cpu.get_hl();
                memory_bus.write_u8(addr, ALU::increment(cpu, memory_bus.read_u8(addr)));
            }
            Register8::A => cpu.Accumulator = ALU::increment(cpu, cpu.Accumulator),
        },
        Instruction::Decrement(reg) => match reg {
            Register8::B => cpu.B = ALU::decrement(cpu, cpu.B),
            Register8::C => cpu.C = ALU::decrement(cpu, cpu.C),
            Register8::D => cpu.D = ALU::decrement(cpu, cpu.D),
            Register8::E => cpu.E = ALU::decrement(cpu, cpu.E),
            Register8::H => cpu.H = ALU::decrement(cpu, cpu.H),
            Register8::L => cpu.L = ALU::decrement(cpu, cpu.L),
            Register8::IndirectHL => {
                let addr = cpu.get_hl();
                memory_bus.write_u8(addr, ALU::decrement(cpu, memory_bus.read_u8(addr)));
            }
            Register8::A => cpu.Accumulator = ALU::decrement(cpu, cpu.Accumulator),
        },
        // 16 bit ALU operations
        Instruction::AddHLRegister(reg) => {
            let val = ALU::add_16(cpu, cpu.read_16(reg));
            cpu.H = val.get_bits(8..16) as u8;
            cpu.L = val.get_bits(0..8) as u8;
        }
        // 16-bit INC/DEC
        Instruction::Increment16(register) => {
            if matches!(register, Register16::SP) {
                cpu.SP = cpu.SP.wrapping_add(1);
            } else {
                let (upper, lower) = match register {
                    Register16::BC => (&mut cpu.B, &mut cpu.C),
                    Register16::DE => (&mut cpu.D, &mut cpu.E),
                    Register16::HL => (&mut cpu.H, &mut cpu.L),
                    _ => unreachable!(),
                };

                trace!(
                    "Before Increment {:#?}: {:#X}",
                    register,
                    (*upper as u16) << 8 | *lower as u16
                );
                ALU::increment_16(upper, lower);
                trace!(
                    "After Increment {:#?}: {:#X}",
                    register,
                    (*upper as u16) << 8 | *lower as u16
                );
            }
        }
        Instruction::Decrement16(register) => {
            if matches!(register, Register16::SP) {
                cpu.SP = cpu.SP.wrapping_sub(1);
            } else {
                let (upper, lower) = match register {
                    Register16::BC => (&mut cpu.B, &mut cpu.C),
                    Register16::DE => (&mut cpu.D, &mut cpu.E),
                    Register16::HL => (&mut cpu.H, &mut cpu.L),
                    _ => unreachable!(),
                };

                trace!(
                    "Before Decrement {:#?}: {:#X}",
                    register,
                    (*upper as u16) << 8 | *lower as u16
                );
                ALU::decrement_16(upper, lower);
                trace!(
                    "After Decrement {:#?}: {:#X}",
                    register,
                    (*upper as u16) << 8 | *lower as u16
                );
            }
        }
        // Bitwise Ops
        Instruction::Bit(bit, reg) => {
            cpu.set_flag(
                Flag::Z,
                match reg {
                    Register8::B => cpu.B.get_bit(bit as usize),
                    Register8::C => cpu.C.get_bit(bit as usize),
                    Register8::D => cpu.D.get_bit(bit as usize),
                    Register8::E => cpu.E.get_bit(bit as usize),
                    Register8::H => cpu.H.get_bit(bit as usize),
                    Register8::L => cpu.L.get_bit(bit as usize),
                    Register8::IndirectHL => memory_bus.read_u8(cpu.get_hl()).get_bit(bit as usize),
                    Register8::A => cpu.Accumulator.get_bit(bit as usize),
                },
            );

            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, true);
        }
        Instruction::SetBit(bit, reg) => match reg {
            Register8::B => {
                cpu.B.set_bit(bit as usize, true);
            }
            Register8::C => {
                cpu.C.set_bit(bit as usize, true);
            }
            Register8::D => {
                cpu.D.set_bit(bit as usize, true);
            }
            Register8::E => {
                cpu.E.set_bit(bit as usize, true);
            }
            Register8::H => {
                cpu.H.set_bit(bit as usize, true);
            }
            Register8::L => {
                cpu.L.set_bit(bit as usize, true);
            }
            Register8::IndirectHL => {
                let mut number = memory_bus.read_u8(cpu.get_hl());
                number.set_bit(bit as usize, true);
                memory_bus.write_u8(cpu.get_hl(), number);
            }
            Register8::A => {
                cpu.Accumulator.set_bit(bit as usize, true);
            }
        },
        Instruction::ResetBit(bit, reg) => match reg {
            Register8::B => {
                cpu.B.set_bit(bit as usize, false);
            }
            Register8::C => {
                cpu.C.set_bit(bit as usize, false);
            }
            Register8::D => {
                cpu.D.set_bit(bit as usize, false);
            }
            Register8::E => {
                cpu.E.set_bit(bit as usize, false);
            }
            Register8::H => {
                cpu.H.set_bit(bit as usize, false);
            }
            Register8::L => {
                cpu.L.set_bit(bit as usize, false);
            }
            Register8::IndirectHL => {
                let mut number = memory_bus.read_u8(cpu.get_hl());
                number.set_bit(bit as usize, false);
                memory_bus.write_u8(cpu.get_hl(), number);
            }
            Register8::A => {
                cpu.Accumulator.set_bit(bit as usize, false);
            }
        },
        _ => return None,
    }

    // ALU Ops don't care about action_taken
    Some(instr.ticks(false))
}

#[allow(clippy::upper_case_acronyms)]
pub struct ALU;

impl ALU {
    pub fn handle_op(cpu: &mut CPU, op: AluOp, value: u8) {
        match op {
            AluOp::Add => {
                cpu.Accumulator = ALU::add(cpu, value);
            }
            AluOp::AddWithCarry => {
                cpu.Accumulator = ALU::adc(cpu, value);
            }
            AluOp::Subtract => {
                cpu.Accumulator = ALU::sub(cpu, value);
            }
            AluOp::SubtractWithCarry => {
                cpu.Accumulator = ALU::sbc(cpu, value);
            }
            AluOp::And => {
                cpu.Accumulator = ALU::and(cpu, value);
            }
            AluOp::Xor => {
                cpu.Accumulator = ALU::xor(cpu, value);
            }
            AluOp::Or => {
                cpu.Accumulator = ALU::or(cpu, value);
            }
            AluOp::Compare => {
                ALU::sub(cpu, value);
            }
        }
    }

    pub fn handle_bitwise(
        cpu: &mut CPU,
        op: BitwiseOp,
        register: Register8,
        memory_bus: &MemoryBus,
    ) {
        let value = cpu.read_register(register, memory_bus);
        let result = match op {
            BitwiseOp::RotateLeftCarry => ALU::rotate_left_carry(cpu, value),
            BitwiseOp::RotateRightCarry => ALU::rotate_right_carry(cpu, value),
            BitwiseOp::RotateLeft => ALU::rotate_left(cpu, value),
            BitwiseOp::RotateRight => ALU::rotate_right(cpu, value),
            BitwiseOp::ShiftLeftArithmetic => ALU::shift_left_arithmetic(cpu, value),
            BitwiseOp::ShiftRightArithmetic => ALU::shift_right_arithmetic(cpu, value),
            BitwiseOp::Swap => ALU::swap_nibble(cpu, value),
            BitwiseOp::ShiftRightLogical => ALU::shift_right_logical(cpu, value),
        };
        cpu.write_register_immediate(register, result, memory_bus);
    }

    pub fn add(cpu: &mut CPU, value: u8) -> u8 {
        let (new_value, did_overflow) = cpu.Accumulator.overflowing_add(value);
        cpu.set_flag(Flag::Z, new_value == 0);
        cpu.set_flag(Flag::N, false);
        cpu.set_flag(Flag::C, did_overflow);
        // Half-carry is set if the lower 4 bits added together overflow
        cpu.set_flag(Flag::H, (cpu.Accumulator & 0xF) + (value & 0xF) > 0xF);
        new_value
    }

    pub fn adc(cpu: &mut CPU, value: u8) -> u8 {
        let carry = cpu.get_flag(Flag::C) as u8;
        let result = cpu.Accumulator.wrapping_add(value).wrapping_add(carry);
        cpu.set_flag(Flag::Z, result == 0);
        cpu.set_flag(Flag::N, false);
        cpu.set_flag(
            Flag::H,
            (cpu.Accumulator as u16 & 0xF) + (value as u16 & 0xF) + carry as u16 > 0xF,
        );
        cpu.set_flag(
            Flag::C,
            cpu.Accumulator as u16 + value as u16 + carry as u16 > 0xFF,
        );
        result
    }

    pub fn add_16(cpu: &mut CPU, value: u16) -> u16 {
        let hl = cpu.get_hl();
        let (new_value, did_overflow) = hl.overflowing_add(value);
        cpu.set_flag(Flag::C, did_overflow);
        cpu.set_flag(Flag::N, false);
        cpu.set_flag(Flag::H, (hl & 0x07FF) + (value & 0x07FF) > 0x07FF);
        new_value
    }

    pub fn sub(cpu: &mut CPU, value: u8) -> u8 {
        let (new_value, did_overflow) = cpu.Accumulator.overflowing_sub(value);
        cpu.set_flag(Flag::Z, new_value == 0);
        cpu.set_flag(Flag::N, true);
        // Half-carry is set if the lower 4 bits subtracted overflow
        cpu.set_flag(Flag::H, (cpu.Accumulator & 0xF) < (value & 0xF));
        cpu.set_flag(Flag::C, did_overflow);
        new_value
    }

    pub fn sbc(cpu: &mut CPU, value: u8) -> u8 {
        let carry = cpu.get_flag(Flag::C) as u8;
        let result = cpu.Accumulator.wrapping_sub(value).wrapping_sub(carry);
        cpu.set_flag(Flag::Z, result == 0);
        cpu.set_flag(Flag::N, true);
        cpu.set_flag(
            Flag::H,
            (cpu.Accumulator & 0xF)
                .wrapping_sub(value & 0xF)
                .wrapping_sub(carry)
                & (0xF + 1)
                != 0,
        );
        cpu.set_flag(
            Flag::C,
            (cpu.Accumulator as u16) < (value as u16 + carry as u16),
        );
        result
    }

    pub fn and(cpu: &mut CPU, value: u8) -> u8 {
        let new_value = cpu.Accumulator & value;
        cpu.set_flag(Flag::Z, new_value == 0);
        cpu.set_flag(Flag::N, false);
        cpu.set_flag(Flag::H, true);
        cpu.set_flag(Flag::C, false);
        new_value
    }

    pub fn or(cpu: &mut CPU, value: u8) -> u8 {
        let new_value = cpu.Accumulator | value;
        cpu.set_flag(Flag::Z, new_value == 0);
        cpu.set_flag(Flag::N, false);
        cpu.set_flag(Flag::H, false);
        cpu.set_flag(Flag::C, false);
        new_value
    }

    pub fn xor(cpu: &mut CPU, value: u8) -> u8 {
        let new_value = cpu.Accumulator ^ value;
        cpu.set_flag(Flag::Z, new_value == 0);
        cpu.set_flag(Flag::N, false);
        cpu.set_flag(Flag::H, false);
        cpu.set_flag(Flag::C, false);
        new_value
    }

    fn sr_flag_update(cpu: &mut CPU, carry: bool, new_value: u8) {
        cpu.set_flag(Flag::H, false);
        cpu.set_flag(Flag::N, false);
        cpu.set_flag(Flag::Z, new_value == 0);
        cpu.set_flag(Flag::C, carry);
    }

    pub fn rotate_right(cpu: &mut CPU, value: u8) -> u8 {
        let carry = value.get_bit(0);
        let mut new_value = value >> 1;
        if cpu.get_flag(Flag::C) {
            new_value.set_bit(7, true);
        }
        Self::sr_flag_update(cpu, carry, new_value);
        new_value
    }

    pub fn rotate_right_carry(cpu: &mut CPU, value: u8) -> u8 {
        let carry = value.get_bit(0);
        let mut new_value = value >> 1;
        if carry {
            new_value.set_bit(7, true);
        }
        Self::sr_flag_update(cpu, carry, new_value);
        new_value
    }

    pub fn rotate_left(cpu: &mut CPU, value: u8) -> u8 {
        let carry = value.get_bit(7);
        let mut new_value = value << 1;
        if cpu.get_flag(Flag::C) {
            new_value.set_bit(0, true);
        }
        Self::sr_flag_update(cpu, carry, new_value);
        new_value
    }

    pub fn rotate_left_carry(cpu: &mut CPU, value: u8) -> u8 {
        let carry = value.get_bit(7);
        let mut new_value = value << 1;
        if carry {
            new_value.set_bit(0, true);
        }
        Self::sr_flag_update(cpu, carry, new_value);
        new_value
    }

    pub fn shift_left_arithmetic(cpu: &mut CPU, value: u8) -> u8 {
        let carry = value.get_bit(7);
        let new_value = value << 1;
        Self::sr_flag_update(cpu, carry, new_value);
        new_value
    }

    pub fn shift_right_arithmetic(cpu: &mut CPU, value: u8) -> u8 {
        let carry = value.get_bit(0);
        let new_value = (value as i8 >> 1) as u8;
        Self::sr_flag_update(cpu, carry, new_value);
        new_value
    }

    pub fn swap_nibble(cpu: &mut CPU, value: u8) -> u8 {
        let mut new_number = 0u8;
        new_number.set_bits(0..4, value.get_bits(4..8));
        new_number.set_bits(4..8, value.get_bits(0..4));
        Self::sr_flag_update(cpu, false, new_number);
        new_number
    }

    pub fn shift_right_logical(cpu: &mut CPU, value: u8) -> u8 {
        let carry = value.get_bit(0);
        let new_value = value >> 1;
        Self::sr_flag_update(cpu, carry, new_value);
        new_value
    }

    pub fn decimal_adjust_after_addition(cpu: &mut CPU, value: u8) -> u8 {
        let mut new_value = value;
        let mut adjust = if cpu.get_flag(Flag::C) { 0x60 } else { 0x00 };

        if cpu.get_flag(Flag::H) {
            adjust |= 0x06;
        }

        if !cpu.get_flag(Flag::N) {
            if new_value & 0x0F > 0x09 {
                adjust |= 0x06;
            }
            if new_value > 0x99 {
                adjust |= 0x60;
            }
            new_value = new_value.wrapping_add(adjust);
        } else {
            new_value = new_value.wrapping_sub(adjust);
        }

        cpu.set_flag(Flag::C, adjust >= 0x60);
        cpu.set_flag(Flag::H, false);
        cpu.set_flag(Flag::Z, new_value == 0);
        new_value
    }

    pub fn increment(cpu: &mut CPU, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        cpu.set_flag(Flag::Z, new_value == 0);
        cpu.set_flag(Flag::N, false);
        // Half-carry is set if the lower 4 bits added together overflow
        cpu.set_flag(Flag::H, (cpu.Accumulator & 0xF) + (value & 0xF) > 0xF);
        new_value
    }

    pub fn decrement(cpu: &mut CPU, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        cpu.set_flag(Flag::Z, new_value == 0);
        cpu.set_flag(Flag::N, false);
        // Half-carry is set if the lower 4 bits added together overflow
        cpu.set_flag(Flag::H, (cpu.Accumulator & 0xF) + (value & 0xF) > 0xF);
        new_value
    }

    pub fn increment_16(upper: &mut u8, lower: &mut u8) {
        let mut addr = 0u16;
        addr.set_bits(0..8, *lower as u16);
        addr.set_bits(8..16, *upper as u16);
        addr = addr.wrapping_add(1);
        *lower = addr.get_bits(0..8) as u8;
        *upper = addr.get_bits(8..16) as u8;
    }

    pub fn decrement_16(upper: &mut u8, lower: &mut u8) {
        let mut addr = 0u16;
        addr.set_bits(0..8, *lower as u16);
        addr.set_bits(8..16, *upper as u16);
        addr = addr.wrapping_sub(1);
        *lower = addr.get_bits(0..8) as u8;
        *upper = addr.get_bits(8..16) as u8;
    }

    pub fn write_16(upper: &mut u8, lower: &mut u8, value: u16) {
        *upper = value.get_bits(8..16) as u8;
        *lower = value.get_bits(0..8) as u8;
    }

    pub fn add_rel(addr: u16, rel: i8) -> u16 {
        if rel.is_negative() {
            addr.wrapping_sub(rel.wrapping_abs() as u16)
        } else {
            addr.wrapping_add(rel as u16)
        }
    }
}
