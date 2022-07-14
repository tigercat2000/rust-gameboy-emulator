use crate::{
    instructions::{Instruction, Register8},
    memory_bus::MemoryBus,
};

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct CPU<'a> {
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

    memory_bus: &'a MemoryBus,
}

impl<'a> CPU<'a> {
    pub fn new(bus: &'a MemoryBus) -> Self {
        CPU {
            Accumulator: 0,
            Flags: 0,
            B: 0,
            C: 0,
            D: 0,
            E: 0,
            H: 0,
            L: 0,
            SP: 0,
            PC: 0,
            stop: false,
            memory_bus: bus,
        }
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.memory_bus.get_u8(self.PC);
        self.PC = self.PC.wrapping_add(1);
        byte
    }

    pub fn next_instruction(&mut self) -> Instruction {
        let instr = self.memory_bus.get_instr(self.PC);
        let (_, actual_instr) =
            Instruction::parse(&instr).expect("Instruction parsing should never fail");
        self.PC = self.PC.wrapping_add(actual_instr.byte_len());
        actual_instr
    }

    pub fn tick(&mut self) {
        let old_pc = self.PC;
        let instr = self.next_instruction();
        // println!("Executing instruction {:#X?} at {:#X}", instr, old_pc);
        match instr {
            Instruction::Nop => {}
            Instruction::Jump(target) => {
                self.PC = target;
            }
            Instruction::Stop => {
                self.stop = true;
            }
            Instruction::LoadImmediate(register, immediate) => match register {
                Register8::B => todo!(),
                Register8::C => todo!(),
                Register8::D => todo!(),
                Register8::E => todo!(),
                Register8::H => todo!(),
                Register8::L => todo!(),
                Register8::IndirectHL => todo!(),
                Register8::A => {
                    self.Accumulator = immediate;
                }
            },
            Instruction::LoadIndirectImmediateA(addr) => {
                self.memory_bus.write_u8(addr, self.Accumulator);
            }
            Instruction::LoadAIndirectImmediate(addr) => {
                self.Accumulator = self.memory_bus.get_u8(addr);
            }
            Instruction::LoadHighPageA(offset) => {
                let real_address = 0xFF00 + (offset as u16);
                self.memory_bus.write_u8(real_address, self.Accumulator);
            }
            _ => unimplemented!(),
        }
    }
}
