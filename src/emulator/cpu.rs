use bit_field::BitField;
#[allow(unused_imports)]
use tracing::{debug, error, event, info, trace};

use crate::emulator::{
    instructions::{
        AccumulatorFlagOp, AluOp, Condition, Instruction, Register16, Register16Indirect,
        Register16Stack, Register8,
    },
    memory_bus::MemoryBus,
};

use super::{
    instructions::BitwiseOp,
    memory_bus::{IE, IF},
};

pub enum Flag {
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
#[derive(Debug)]
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
    pub halted: bool,
    pub IME: bool,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            Accumulator: Default::default(),
            Flags: Default::default(),
            B: Default::default(),
            C: Default::default(),
            D: Default::default(),
            E: Default::default(),
            H: Default::default(),
            L: Default::default(),
            SP: Default::default(),
            PC: 0x100,
            stop: false,
            halted: false,
            IME: false,
        }
    }
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

    fn set_flag(&mut self, flag: Flag, value: bool) {
        match flag {
            Flag::Z => self.Flags.set_bit(7, value),
            Flag::N => self.Flags.set_bit(6, value),
            Flag::H => self.Flags.set_bit(5, value),
            Flag::C => self.Flags.set_bit(4, value),
        };
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
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

    /// Ticks in M-cycles (4 T-cycles)
    pub fn tick(&mut self, memory_bus: &MemoryBus) -> u32 {
        let old_pc = self.PC;
        let instr = self.next_instruction(memory_bus);
        let mut action_taken = false;
        debug!("Executing instruction {} at {:#X}", instr, old_pc);
        trace!(
            "Registers before: BC: {:#X} DE: {:#X} HL: {:#X} SP: {:#X}",
            self.get_bc(),
            self.get_de(),
            self.get_hl(),
            self.SP,
        );

        match self.handle_interrupt(memory_bus) {
            0 => {}
            n => return n as u32,
        };

        if self.halted {
            return 1;
        }

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
                debug!("LoadAHighPage loaded {:#X} into A", self.Accumulator);
            }
            Instruction::LoadHighPageIndirectA => {
                let offset = self.C;
                let real_address = 0xFF00 + (offset as u16);
                memory_bus.write_u8(real_address, self.Accumulator);
            }
            Instruction::LoadAHighPageIndirect => {
                let offset = self.C;
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
                if matches!(reg1, Register8::B) && matches!(reg2, Register8::B) {
                    panic!("Debug breakpoint!");
                }
                self.write_register(reg1, reg2, memory_bus);
            }
            // TODO: Handle flags
            Instruction::Increment(reg) => match reg {
                Register8::B => self.B = self.B.wrapping_add(1),
                Register8::C => self.C = self.C.wrapping_add(1),
                Register8::D => self.D = self.D.wrapping_add(1),
                Register8::E => self.E = self.E.wrapping_add(1),
                Register8::H => self.H = self.H.wrapping_add(1),
                Register8::L => self.L = self.L.wrapping_add(1),
                Register8::IndirectHL => {
                    let addr = self.get_hl();
                    memory_bus.write_u8(addr, memory_bus.get_u8(addr).wrapping_add(1));
                }
                Register8::A => self.Accumulator = self.Accumulator.wrapping_add(1),
            },
            // TODO: Handle flags
            Instruction::Decrement(reg) => match reg {
                Register8::B => self.B = self.B.wrapping_sub(1),
                Register8::C => self.C = self.C.wrapping_sub(1),
                Register8::D => self.D = self.D.wrapping_sub(1),
                Register8::E => self.E = self.E.wrapping_sub(1),
                Register8::H => self.H = self.H.wrapping_sub(1),
                Register8::L => self.L = self.L.wrapping_sub(1),
                Register8::IndirectHL => {
                    let addr = self.get_hl();
                    memory_bus.write_u8(addr, memory_bus.get_u8(addr).wrapping_sub(1));
                }
                Register8::A => self.Accumulator = self.Accumulator.wrapping_sub(1),
            },
            Instruction::Push(register) => match register {
                Register16Stack::BC => {
                    let value = self.get_bc();
                    memory_bus.write_stack_16(&mut self.SP, value);
                }
                Register16Stack::DE => {
                    let value = self.get_de();
                    memory_bus.write_stack_16(&mut self.SP, value);
                }
                Register16Stack::HL => {
                    let value = self.get_hl();
                    memory_bus.write_stack_16(&mut self.SP, value);
                }
                Register16Stack::AF => {
                    let value = self.get_af();
                    memory_bus.write_stack_16(&mut self.SP, value);
                }
            },
            Instruction::Pop(register) => match register {
                Register16Stack::BC => {
                    ALU::write_16(
                        &mut self.B,
                        &mut self.C,
                        memory_bus.get_stack_16(&mut self.SP),
                    );
                }
                Register16Stack::DE => {
                    ALU::write_16(
                        &mut self.D,
                        &mut self.E,
                        memory_bus.get_stack_16(&mut self.SP),
                    );
                }
                Register16Stack::HL => {
                    ALU::write_16(
                        &mut self.H,
                        &mut self.L,
                        memory_bus.get_stack_16(&mut self.SP),
                    );
                }
                Register16Stack::AF => {
                    ALU::write_16(
                        &mut self.Accumulator,
                        &mut self.Flags,
                        memory_bus.get_stack_16(&mut self.SP),
                    );
                }
            },
            Instruction::JumpHL => {
                self.PC = self.get_hl();
            }
            Instruction::DisableInterrupts => {
                self.IME = false;
            }
            Instruction::EnableInterrupts => {
                self.IME = true;
            }
            Instruction::RetInterrupt => {
                let addr = memory_bus.get_stack_16(&mut self.SP);
                trace!("Read {:#X} from stack @ {:#X}", addr, self.SP);
                self.PC = addr;
                self.IME = true;
            }
            Instruction::Call(imm) => {
                trace!("Writing {:#X} to stack @ {:#X}", self.PC, self.SP);
                memory_bus.write_stack_16(&mut self.SP, self.PC);
                self.PC = imm;
            }
            Instruction::Ret => {
                let addr = memory_bus.get_stack_16(&mut self.SP);
                trace!("Read {:#X} from stack @ {:#X}", addr, self.SP);
                self.PC = addr;
            }
            Instruction::RetConditional(condition) => match condition {
                Condition::NZ => {
                    if !self.get_flag(Flag::Z) {
                        self.PC = memory_bus.get_stack_16(&mut self.SP);
                        trace!("Read {:#X} from stack @ {:#X}", self.PC, self.SP);
                    }
                }
                Condition::Z => {
                    if self.get_flag(Flag::Z) {
                        self.PC = memory_bus.get_stack_16(&mut self.SP);
                        trace!("Read {:#X} from stack @ {:#X}", self.PC, self.SP);
                    }
                }
                Condition::NC => {
                    if !self.get_flag(Flag::C) {
                        self.PC = memory_bus.get_stack_16(&mut self.SP);
                        trace!("Read {:#X} from stack @ {:#X}", self.PC, self.SP);
                    }
                }
                Condition::C => {
                    if self.get_flag(Flag::C) {
                        self.PC = memory_bus.get_stack_16(&mut self.SP);
                        trace!("Read {:#X} from stack @ {:#X}", self.PC, self.SP);
                    }
                }
            },
            Instruction::JumpRelative(rel) => {
                self.PC = ALU::add_rel(self.PC, rel);
            }
            Instruction::JumpRelativeConditional(condition, rel) => match condition {
                Condition::NZ => {
                    if !self.get_flag(Flag::Z) {
                        self.PC = ALU::add_rel(self.PC, rel);
                    }
                }
                Condition::Z => {
                    if self.get_flag(Flag::Z) {
                        self.PC = ALU::add_rel(self.PC, rel);
                    }
                }
                Condition::NC => {
                    if !self.get_flag(Flag::C) {
                        self.PC = ALU::add_rel(self.PC, rel);
                    }
                }
                Condition::C => {
                    if self.get_flag(Flag::C) {
                        self.PC = ALU::add_rel(self.PC, rel);
                    }
                }
            },
            Instruction::Bit(bit, reg) => {
                self.set_flag(
                    Flag::Z,
                    match reg {
                        Register8::B => self.B.get_bit(bit as usize),
                        Register8::C => self.C.get_bit(bit as usize),
                        Register8::D => self.D.get_bit(bit as usize),
                        Register8::E => self.E.get_bit(bit as usize),
                        Register8::H => self.H.get_bit(bit as usize),
                        Register8::L => self.L.get_bit(bit as usize),
                        Register8::IndirectHL => {
                            memory_bus.get_u8(self.get_hl()).get_bit(bit as usize)
                        }
                        Register8::A => self.Accumulator.get_bit(bit as usize),
                    },
                );

                self.set_flag(Flag::N, false);
                self.set_flag(Flag::H, true);
            }
            Instruction::SetBit(bit, reg) => match reg {
                Register8::B => {
                    self.B.set_bit(bit as usize, true);
                }
                Register8::C => {
                    self.C.set_bit(bit as usize, true);
                }
                Register8::D => {
                    self.D.set_bit(bit as usize, true);
                }
                Register8::E => {
                    self.E.set_bit(bit as usize, true);
                }
                Register8::H => {
                    self.H.set_bit(bit as usize, true);
                }
                Register8::L => {
                    self.L.set_bit(bit as usize, true);
                }
                Register8::IndirectHL => {
                    memory_bus.get_u8(self.get_hl()).set_bit(bit as usize, true);
                }
                Register8::A => {
                    self.Accumulator.set_bit(bit as usize, true);
                }
            },
            Instruction::ResetBit(bit, reg) => match reg {
                Register8::B => {
                    self.B.set_bit(bit as usize, false);
                }
                Register8::C => {
                    self.C.set_bit(bit as usize, false);
                }
                Register8::D => {
                    self.D.set_bit(bit as usize, false);
                }
                Register8::E => {
                    self.E.set_bit(bit as usize, false);
                }
                Register8::H => {
                    self.H.set_bit(bit as usize, false);
                }
                Register8::L => {
                    self.L.set_bit(bit as usize, false);
                }
                Register8::IndirectHL => {
                    memory_bus
                        .get_u8(self.get_hl())
                        .set_bit(bit as usize, false);
                }
                Register8::A => {
                    self.Accumulator.set_bit(bit as usize, false);
                }
            },
            Instruction::Bitwise(op, register) => {
                ALU::handle_bitwise(self, op, register, memory_bus);
            }
            Instruction::Halt => {
                self.halted = true;
            }
            Instruction::Reset(offset) => {
                memory_bus.write_stack_16(&mut self.SP, self.PC);
                self.PC = offset as u16;
            }
            _ => {
                error!("Instruction not implemented: {}", instr);
                unimplemented!("Instruction not implemented: {}", instr)
            }
        }
        trace!(
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

    fn handle_interrupt(&mut self, memory_bus: &MemoryBus) -> u8 {
        if !self.IME && !self.halted {
            return 0;
        }

        let mut interrupts_requested = memory_bus.get_u8(IF);
        let triggered = interrupts_requested & memory_bus.get_u8(IE);
        if triggered == 0 {
            return 0;
        }

        self.halted = false;
        if !self.IME {
            return 0;
        }
        self.IME = false;

        let n = triggered.trailing_zeros();
        if n >= 5 {
            unreachable!("Invalid interrupt triggered")
        }

        interrupts_requested.set_bit(n as usize, false);
        memory_bus.write_u8(IF, interrupts_requested);

        memory_bus.write_stack_16(&mut self.SP, self.PC);
        self.PC = 0x0040 | (n as u16) << 3;

        4
    }
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

    fn handle_bitwise(cpu: &mut CPU, op: BitwiseOp, register: Register8, memory_bus: &MemoryBus) {
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
