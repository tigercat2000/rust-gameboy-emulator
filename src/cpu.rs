use bit_field::BitField;
#[allow(unused_imports)]
use tracing::{debug, error, event, info, trace};

use crate::{
    instructions::{
        AccumulatorFlagOp, AluOp, Condition, Instruction, Register16, Register16Indirect, Register8,
    },
    memory_bus::MemoryBus,
};

enum Flag {
    /// Zero flag
    Z,
    /// Subtraction flag (BCD)
    N,
    /// Half Carry flag (BCD)
    H,
    /// Carry flag
    C,
}

#[allow(non_snake_case)]
#[derive(Debug, Default)]
pub struct CPU {
    pub Accumulator: u8,
    pub Flags: u8,
    pub B: u8,
    pub C: u8,
    pub D: u8,
    pub E: u8,
    pub H: u8,
    pub L: u8,

    pub SP: u16,
    pub PC: u16,
    pub stop: bool,
}

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
    fn get_bc(&self) -> u16 {
        ((self.B as u16) << 8) | (self.C as u16)
    }

    fn get_de(&self) -> u16 {
        ((self.D as u16) << 8) | (self.E as u16)
    }

    fn get_hl(&self) -> u16 {
        ((self.H as u16) << 8) | (self.L as u16)
    }

    fn set_flag(&mut self, flag: Flag, value: bool) {
        match flag {
            Flag::Z => self.Flags.set_bit(7, value),
            Flag::N => self.Flags.set_bit(6, value),
            Flag::H => self.Flags.set_bit(5, value),
            Flag::C => self.Flags.set_bit(4, value),
        };
    }

    fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Z => self.Flags.get_bit(7),
            Flag::N => self.Flags.get_bit(6),
            Flag::H => self.Flags.get_bit(5),
            Flag::C => self.Flags.get_bit(4),
        }
    }

    // fn next_byte(&mut self) -> u8 {
    //     let byte = self.memory_bus.get_u8(self.PC);
    //     self.PC = self.PC.wrapping_add(1);
    //     byte
    // }

    pub fn next_instruction(&mut self, memory_bus: &MemoryBus) -> Instruction {
        let instr = memory_bus.get_instr(self.PC);
        let (_, actual_instr) =
            Instruction::parse(&instr).expect("Instruction parsing should never fail");
        self.PC = self.PC.wrapping_add(actual_instr.byte_len());
        actual_instr
    }

    pub fn tick(&mut self, memory_bus: &MemoryBus) -> u32 {
        let old_pc = self.PC;
        let instr = self.next_instruction(memory_bus);
        let mut action_taken = false;
        debug!("Executing instruction {} at {:#X}", instr, old_pc);
        debug!(
            "Registers before: BC: {:#X} DE: {:#X} HL: {:#X}",
            self.get_bc(),
            self.get_de(),
            self.get_hl()
        );
        match instr {
            Instruction::Nop => {}
            Instruction::Jump(target) => {
                self.PC = target;
            }
            Instruction::Stop => {
                self.stop = true;
            }
            Instruction::LoadImmediate(register, immediate) => match register {
                Register8::B => {
                    self.B = immediate;
                }
                Register8::C => {
                    self.C = immediate;
                }
                Register8::D => {
                    self.D = immediate;
                }
                Register8::E => {
                    self.E = immediate;
                }
                Register8::H => {
                    self.H = immediate;
                }
                Register8::L => {
                    self.L = immediate;
                }
                Register8::IndirectHL => {
                    let addr = self.get_hl();
                    memory_bus.write_u8(addr, immediate);
                }
                Register8::A => {
                    self.Accumulator = immediate;
                }
            },
            Instruction::LoadIndirectImmediateA(addr) => {
                memory_bus.write_u8(addr, self.Accumulator);
            }
            Instruction::LoadAIndirectImmediate(addr) => {
                self.Accumulator = memory_bus.get_u8(addr);
            }
            Instruction::LoadHighPageA(offset) => {
                let real_address = 0xFF00 + (offset as u16);
                memory_bus.write_u8(real_address, self.Accumulator);
            }
            Instruction::LoadAHighPage(offset) => {
                let real_address = 0xFF00 + (offset as u16);
                self.Accumulator = memory_bus.get_u8(real_address);
            }
            Instruction::AccumulatorFlag(af_op) => match af_op {
                AccumulatorFlagOp::RotateLeftCarryA => {
                    self.Accumulator = ALU::rotate_left_carry(self, self.Accumulator);
                }
                AccumulatorFlagOp::RotateRightCarryA => {
                    self.Accumulator = ALU::rotate_right_carry(self, self.Accumulator);
                }
                AccumulatorFlagOp::RotateLeftA => {
                    self.Accumulator = ALU::rotate_left(self, self.Accumulator);
                }
                AccumulatorFlagOp::RotateRightA => {
                    self.Accumulator = ALU::rotate_right(self, self.Accumulator);
                }
                AccumulatorFlagOp::DecimalAdjustAfterAddition => todo!(),
                AccumulatorFlagOp::ComplementAccumulator => {
                    self.Accumulator = !self.Accumulator;
                    self.set_flag(Flag::N, true);
                    self.set_flag(Flag::H, true);
                }
                AccumulatorFlagOp::SetCarryFlag => {
                    self.set_flag(Flag::N, false);
                    self.set_flag(Flag::H, false);
                    self.set_flag(Flag::C, true);
                }
                AccumulatorFlagOp::ComplementCarryFlag => {
                    self.set_flag(Flag::N, false);
                    self.set_flag(Flag::H, false);
                    self.set_flag(Flag::C, !self.get_flag(Flag::C));
                }
            },
            Instruction::Alu(alu_op, register) => {
                ALU::handle_op(self, alu_op, self.read_register(register, memory_bus))
            }
            Instruction::AluImmediate(alu_op, immediate) => ALU::handle_op(self, alu_op, immediate),
            Instruction::JumpConditional(condition, addr) => {
                let jump = match condition {
                    Condition::NZ => !self.get_flag(Flag::Z),
                    Condition::Z => self.get_flag(Flag::Z),
                    Condition::NC => !self.get_flag(Flag::C),
                    Condition::C => self.get_flag(Flag::C),
                };
                if jump {
                    action_taken = true;
                    self.PC = addr;
                }
            }
            Instruction::LoadImmediate16(register, immediate) => match register {
                Register16::BC => {
                    self.C = immediate.get_bits(0..8) as u8;
                    self.B = immediate.get_bits(8..16) as u8;
                }
                Register16::DE => {
                    self.E = immediate.get_bits(0..8) as u8;
                    self.D = immediate.get_bits(8..16) as u8;
                    trace!("Loaded DE with {:#X}", self.get_de());
                }
                Register16::HL => {
                    self.L = immediate.get_bits(0..8) as u8;
                    self.H = immediate.get_bits(8..16) as u8;
                }
                Register16::SP => {
                    self.SP = immediate;
                }
            },
            Instruction::LoadAIndirect(reg_with_addr) => {
                let get_indirect_addr = self.get_indirect(reg_with_addr);
                trace!(
                    "Loading A from {:#?} with address {:#X}",
                    reg_with_addr,
                    get_indirect_addr
                );
                self.Accumulator = memory_bus.get_u8(get_indirect_addr);
            }
            Instruction::LoadIndirectA(reg_with_addr) => {
                let addr = self.get_indirect(reg_with_addr);
                memory_bus.write_u8(addr, self.Accumulator);
            }
            Instruction::Increment16(register) => match register {
                Register16::BC => {
                    trace!(
                        "Increment BC: {:#X} (B: {:#X}, C: {:#X})",
                        self.get_bc(),
                        self.B,
                        self.C
                    );
                    ALU::increment_16(&mut self.B, &mut self.C);
                    trace!(
                        "After Increment BC: {:#X} (B: {:#X}, C: {:#X})",
                        self.get_bc(),
                        self.B,
                        self.C
                    );
                }
                Register16::DE => {
                    trace!(
                        "Increment DE: {:#X} (D: {:#X}, E: {:#X})",
                        self.get_de(),
                        self.D,
                        self.E
                    );
                    ALU::increment_16(&mut self.D, &mut self.E);
                    trace!(
                        "After Increment DE: {:#X} (D: {:#X}, E: {:#X})",
                        self.get_de(),
                        self.D,
                        self.E
                    );
                }
                Register16::HL => {
                    trace!(
                        "Increment HL: {:#X} (H: {:#X}, L: {:#X})",
                        self.get_hl(),
                        self.H,
                        self.L
                    );
                    ALU::increment_16(&mut self.H, &mut self.L);
                    trace!(
                        "After Increment HL: {:#X} (H: {:#X}, L: {:#X})",
                        self.get_hl(),
                        self.H,
                        self.L
                    );
                }
                Register16::SP => {
                    trace!("Increment SP: {:#X}", self.SP);
                    self.SP += 1;
                    trace!("After Increment SP: {:#X}", self.SP);
                }
            },
            Instruction::Decrement16(register) => match register {
                Register16::BC => {
                    trace!(
                        "Decrement BC: {:#X} (B: {:#X}, C: {:#X})",
                        self.get_bc(),
                        self.B,
                        self.C
                    );
                    ALU::decrement_16(&mut self.B, &mut self.C);
                    trace!(
                        "After Decrement BC: {:#X} (B: {:#X}, C: {:#X})",
                        self.get_bc(),
                        self.B,
                        self.C
                    );
                }
                Register16::DE => {
                    trace!(
                        "Decrement DE: {:#X} (D: {:#X}, E: {:#X})",
                        self.get_de(),
                        self.D,
                        self.E
                    );
                    ALU::decrement_16(&mut self.D, &mut self.E);
                    trace!(
                        "After Decrement DE: {:#X} (D: {:#X}, E: {:#X})",
                        self.get_de(),
                        self.D,
                        self.E
                    );
                }
                Register16::HL => {
                    trace!(
                        "Decrement HL: {:#X} (H: {:#X}, L: {:#X})",
                        self.get_hl(),
                        self.H,
                        self.L
                    );
                    ALU::decrement_16(&mut self.H, &mut self.L);
                    trace!(
                        "After Decrement HL: {:#X} (H: {:#X}, L: {:#X})",
                        self.get_hl(),
                        self.H,
                        self.L
                    );
                }
                Register16::SP => {
                    trace!("Decrement SP: {:#X}", self.SP);
                    self.SP += 1;
                    trace!("After Decrement SP: {:#X}", self.SP);
                }
            },
            Instruction::Load(reg1, reg2) => {
                self.write_register(reg1, reg2, memory_bus);
            }
            _ => {
                error!("Instruction not implemented: {}", instr);
                unimplemented!("Instruction not implemented: {}", instr)
            }
        }
        debug!(
            "Registers after: BC: {:#X} DE: {:#X} HL: {:#X}",
            self.get_bc(),
            self.get_de(),
            self.get_hl()
        );

        instr.ticks(action_taken)
    }

    fn get_indirect(&mut self, register: Register16Indirect) -> u16 {
        match register {
            Register16Indirect::BC => self.get_bc(),
            Register16Indirect::DE => self.get_de(),
            Register16Indirect::HLI => {
                let addr = self.get_hl();
                trace!(
                    "Reading indirect address HL: {:#X} (H: {:#X}, L: {:#X})",
                    addr,
                    self.H,
                    self.L
                );
                let addr1 = addr.wrapping_add(1);
                trace!("Added 1 to HL: {:#X}", addr1);
                self.H = addr1.get_bits(8..16) as u8;
                self.L = addr1.get_bits(0..8) as u8;
                trace!(
                    "Wroteback address to HL: {:#X} (H: {:#X}, L: {:#X})",
                    addr1,
                    self.H,
                    self.L
                );
                trace!("Returning addr {:#X}", addr);
                addr
            }
            Register16Indirect::HLD => {
                let addr = self.get_hl();
                trace!(
                    "Reading indirect address HL: {:#X} (H: {:#X}, L: {:#X})",
                    addr,
                    self.H,
                    self.L
                );
                let addr1 = addr.wrapping_sub(1);
                trace!("Subtracted 1 from HL: {:#X}", addr1);
                self.H = addr1.get_bits(8..16) as u8;
                self.L = addr1.get_bits(0..8) as u8;
                trace!(
                    "Wroteback address to HL: {:#X} (H: {:#X}, L: {:#X})",
                    addr1,
                    self.H,
                    self.L
                );
                addr
            }
        }
    }

    fn write_register(&mut self, target: Register8, source: Register8, memory_bus: &MemoryBus) {
        let read = self.read_register(source, memory_bus);
        trace!("Writing {:?} -> {:?}", source, target);
        self.write_register_immediate(target, read, memory_bus);
    }

    fn write_register_immediate(
        &mut self,
        target: Register8,
        immediate: u8,
        memory_bus: &MemoryBus,
    ) {
        trace!("Writing {:#X} -> {:?}", immediate, target);
        match target {
            Register8::B => self.B = immediate,
            Register8::C => self.C = immediate,
            Register8::D => self.D = immediate,
            Register8::E => self.E = immediate,
            Register8::H => self.H = immediate,
            Register8::L => self.L = immediate,
            Register8::IndirectHL => memory_bus.write_u8(self.get_hl(), immediate),
            Register8::A => self.Accumulator = immediate,
        }
    }

    fn read_register(&self, register: Register8, memory_bus: &MemoryBus) -> u8 {
        match register {
            Register8::B => self.B,
            Register8::C => self.C,
            Register8::D => self.D,
            Register8::E => self.E,
            Register8::H => self.H,
            Register8::L => self.L,
            Register8::IndirectHL => memory_bus.get_u8(self.get_hl()),
            Register8::A => self.Accumulator,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
struct ALU;

impl ALU {
    pub fn handle_op(cpu: &mut CPU, op: AluOp, value: u8) {
        match op {
            AluOp::Add => {
                cpu.Accumulator = ALU::add(cpu, value);
            }
            AluOp::AddWithCarry => {
                cpu.Accumulator = ALU::add(cpu, cpu.get_flag(Flag::C) as u8);
                cpu.Accumulator = ALU::add(cpu, value);
            }
            AluOp::Subtract => {
                cpu.Accumulator = ALU::sub(cpu, value);
            }
            AluOp::SubtractWithCarry => {
                cpu.Accumulator = ALU::sub(cpu, cpu.get_flag(Flag::C) as u8);
                cpu.Accumulator = ALU::sub(cpu, value);
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

    pub fn add(cpu: &mut CPU, value: u8) -> u8 {
        let (new_value, did_overflow) = cpu.Accumulator.overflowing_add(value);
        cpu.set_flag(Flag::Z, new_value == 0);
        cpu.set_flag(Flag::N, false);
        cpu.set_flag(Flag::C, did_overflow);
        // Half-carry is set if the lower 4 bits added together overflow
        cpu.set_flag(Flag::H, (cpu.Accumulator & 0xF) + (value & 0xF) > 0xF);
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
}
