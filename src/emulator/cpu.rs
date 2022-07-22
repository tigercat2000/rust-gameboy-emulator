use bit_field::BitField;
#[allow(unused_imports)]
use tracing::{debug, error, event, info, trace};

use crate::emulator::{
    instructions::{Instruction, Register16Indirect, Register16Stack, Register8},
    memory_bus::MemoryBus,
};

use super::memory_bus::{IE, IF};

pub mod alu;
pub use alu::ALU;
pub mod control_flow;
pub mod helpers;
pub mod loads;

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
            SP: 0xFFFE,
            PC: 0x100,
            stop: false,
            halted: false,
            IME: false,
        }
    }
}

impl CPU {
    /// Ticks in M-cycles (4 T-cycles)
    pub fn tick(&mut self, memory_bus: &MemoryBus) -> u32 {
        let old_pc = self.PC;
        let instr = self.next_instruction(memory_bus);
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

        if let Some(num) = alu::handle_instruction(self, instr, memory_bus) {
            return num;
        }

        if let Some(num) = control_flow::handle_instruction(self, instr, memory_bus) {
            return num;
        }

        if let Some(num) = loads::handle_instruction(self, instr, memory_bus) {
            return num;
        }

        match instr {
            Instruction::Nop => {}
            Instruction::Stop => {
                self.stop = true;
            }
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
                        memory_bus.read_stack_16(&mut self.SP),
                    );
                }
                Register16Stack::DE => {
                    ALU::write_16(
                        &mut self.D,
                        &mut self.E,
                        memory_bus.read_stack_16(&mut self.SP),
                    );
                }
                Register16Stack::HL => {
                    ALU::write_16(
                        &mut self.H,
                        &mut self.L,
                        memory_bus.read_stack_16(&mut self.SP),
                    );
                }
                Register16Stack::AF => {
                    ALU::write_16(
                        &mut self.Accumulator,
                        &mut self.Flags,
                        memory_bus.read_stack_16(&mut self.SP),
                    );
                    self.Flags &= 0xF0;
                }
            },
            Instruction::DisableInterrupts => {
                self.IME = false;
            }
            Instruction::EnableInterrupts => {
                self.IME = true;
            }
            Instruction::Halt => {
                self.halted = true;
            }
            _ => {
                error!("Instruction not implemented: {}", instr);

                // error!("High Ram Dump");
                // memory_bus.hram_dump();

                unimplemented!("Instruction not implemented: {}", instr)
            }
        }

        trace!(
            "Registers after: BC: {:#X} DE: {:#X} HL: {:#X}",
            self.get_bc(),
            self.get_de(),
            self.get_hl()
        );

        // Actions never taken in this code
        instr.ticks(false)
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
            Register8::IndirectHL => memory_bus.read_u8(self.get_hl()),
            Register8::A => self.Accumulator,
        }
    }

    fn handle_interrupt(&mut self, memory_bus: &MemoryBus) -> u8 {
        if !self.IME && !self.halted {
            return 0;
        }

        let mut interrupts_requested = memory_bus.read_u8(IF);
        let triggered = interrupts_requested & memory_bus.read_u8(IE);
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
