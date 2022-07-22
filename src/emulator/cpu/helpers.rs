use bit_field::BitField;

use crate::emulator::{
    instructions::{Condition, Instruction, Register16},
    memory_bus::MemoryBus,
};

use super::{Flag, CPU};

impl std::fmt::Display for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CPU Dump")?;
        writeln!(f, "\tFlags")?;
        writeln!(f, "\t\tZero: {}", self.get_flag(Flag::Z))?;
        writeln!(f, "\t\tSubtraction: {}", self.get_flag(Flag::N))?;
        writeln!(f, "\t\tHalfCarry: {}", self.get_flag(Flag::H))?;
        writeln!(f, "\t\tCarry: {}", self.get_flag(Flag::C))?;
        writeln!(f, "\tRegisters")?;
        writeln!(f, "\t\tA: {:#X}", self.Accumulator)?;
        writeln!(f, "\t\tB: {:#X}", self.B)?;
        writeln!(f, "\t\tC: {:#X}", self.C)?;
        writeln!(f, "\t\tD: {:#X}", self.D)?;
        writeln!(f, "\t\tE: {:#X}", self.E)?;
        writeln!(f, "\t\tL: {:#X}", self.H)?;
        writeln!(f, "\t\tH: {:#X}", self.L)?;
        writeln!(f)?;
        writeln!(f, "\t\tSP: {:#X}", self.SP)?;
        writeln!(f, "\t\tPC: {:#X}", self.PC)?;
        writeln!(f, "\t\tStopped: {}", self.stop)?;

        Ok(())
    }
}

impl CPU {
    // Instruction Fetcher
    pub fn next_instruction(&mut self, memory_bus: &MemoryBus) -> Instruction {
        let instr = memory_bus.get_instr(self.PC);
        let (_, actual_instr) = Instruction::parse(&instr)
            .unwrap_or_else(|_| panic!("Instruction parsing failed at {:#X}", self.PC));
        self.PC = self.PC.wrapping_add(actual_instr.byte_len());
        actual_instr
    }

    // 16 bit helpers
    pub fn get_af(&self) -> u16 {
        ((self.Accumulator as u16) << 8) | (self.Flags as u16)
    }

    pub fn get_bc(&self) -> u16 {
        ((self.B as u16) << 8) | (self.C as u16)
    }

    pub fn get_de(&self) -> u16 {
        ((self.D as u16) << 8) | (self.E as u16)
    }

    pub fn get_hl(&self) -> u16 {
        ((self.H as u16) << 8) | (self.L as u16)
    }

    pub fn read_16(&self, register: Register16) -> u16 {
        match register {
            Register16::BC => self.get_bc(),
            Register16::DE => self.get_de(),
            Register16::HL => self.get_hl(),
            Register16::SP => self.SP,
        }
    }

    // Flags
    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Z => self.Flags.get_bit(7),
            Flag::N => self.Flags.get_bit(6),
            Flag::H => self.Flags.get_bit(5),
            Flag::C => self.Flags.get_bit(4),
        }
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        match flag {
            Flag::Z => self.Flags.set_bit(7, value),
            Flag::N => self.Flags.set_bit(6, value),
            Flag::H => self.Flags.set_bit(5, value),
            Flag::C => self.Flags.set_bit(4, value),
        };
    }

    // Conditions
    pub fn check_condition(&self, condition: Condition) -> bool {
        match condition {
            Condition::NZ => !self.get_flag(Flag::Z),
            Condition::Z => self.get_flag(Flag::Z),
            Condition::NC => !self.get_flag(Flag::C),
            Condition::C => self.get_flag(Flag::C),
        }
    }
}
