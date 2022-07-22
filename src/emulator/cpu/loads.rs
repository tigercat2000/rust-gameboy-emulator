use bit_field::BitField;
use tracing::{debug, trace, warn};

use crate::emulator::{
    instructions::{Instruction, Register16, Register8},
    memory_bus::MemoryBus,
};

use super::{Flag, ALU, CPU};

pub fn handle_instruction(
    cpu: &mut CPU,
    instr: Instruction,
    memory_bus: &MemoryBus,
) -> Option<u32> {
    match instr {
        // 8-bit loads
        Instruction::Load(reg1, reg2) => {
            // if matches!(reg1, Register8::B) && matches!(reg2, Register8::B) {
            //     panic!("Debug breakpoint!");
            // }
            cpu.write_register(reg1, reg2, memory_bus);
        }
        Instruction::LoadImmediate(register, immediate) => match register {
            Register8::B => {
                cpu.B = immediate;
            }
            Register8::C => {
                cpu.C = immediate;
            }
            Register8::D => {
                cpu.D = immediate;
            }
            Register8::E => {
                cpu.E = immediate;
            }
            Register8::H => {
                cpu.H = immediate;
            }
            Register8::L => {
                cpu.L = immediate;
            }
            Register8::IndirectHL => {
                trace!(
                    "Writing to IndirectHL @{:#X}: {:#X}",
                    cpu.get_hl(),
                    immediate
                );
                memory_bus.write_u8(cpu.get_hl(), immediate);
            }
            Register8::A => {
                cpu.Accumulator = immediate;
            }
        },
        Instruction::LoadIndirectImmediateA(addr) => {
            trace!("Writing to Indirect @{:#X}: {:#X}", addr, cpu.Accumulator);
            memory_bus.write_u8(addr, cpu.Accumulator);
        }
        Instruction::LoadAIndirectImmediate(addr) => {
            trace!(
                "Reading Indirect @{:#X}: {:#X}",
                addr,
                memory_bus.read_u8(addr)
            );
            cpu.Accumulator = memory_bus.read_u8(addr);
        }
        Instruction::LoadHighPageAImmediate(offset) => {
            let real_address = 0xFF00 + (offset as u16);
            memory_bus.write_u8(real_address, cpu.Accumulator);
            trace!(
                "LoadHighPageA loaded A ({:#X}) into @{:#X}",
                cpu.Accumulator,
                real_address
            );
        }
        Instruction::LoadAHighPageImmediate(offset) => {
            let real_address = 0xFF00 + (offset as u16);
            cpu.Accumulator = memory_bus.read_u8(real_address);
            trace!(
                "LoadAHighPage loaded @{:#X} into A ({:#X})",
                real_address,
                cpu.Accumulator
            );
        }
        Instruction::LoadHighPageIndirectA => {
            let offset = cpu.C;
            let real_address = 0xFF00 + (offset as u16);
            memory_bus.write_u8(real_address, cpu.Accumulator);
        }
        Instruction::LoadAHighPageIndirect => {
            let offset = cpu.C;
            let real_address = 0xFF00 + (offset as u16);
            cpu.Accumulator = memory_bus.read_u8(real_address);
        }
        // 16-bit loads
        Instruction::LoadImmediate16(register, immediate) => match register {
            Register16::BC => {
                cpu.C = immediate.get_bits(0..8) as u8;
                cpu.B = immediate.get_bits(8..16) as u8;
            }
            Register16::DE => {
                cpu.E = immediate.get_bits(0..8) as u8;
                cpu.D = immediate.get_bits(8..16) as u8;
                trace!("Loaded DE with {:#X}", cpu.get_de());
            }
            Register16::HL => {
                cpu.L = immediate.get_bits(0..8) as u8;
                cpu.H = immediate.get_bits(8..16) as u8;
            }
            Register16::SP => {
                cpu.SP = immediate;
            }
        },
        Instruction::LoadAIndirect(reg_with_addr) => {
            let get_indirect_addr = cpu.get_indirect(reg_with_addr);
            trace!(
                "Loading A from {:#?} with address {:#X}",
                reg_with_addr,
                get_indirect_addr
            );
            cpu.Accumulator = memory_bus.read_u8(get_indirect_addr);
        }
        Instruction::LoadIndirectA(reg_with_addr) => {
            let addr = cpu.get_indirect(reg_with_addr);
            memory_bus.write_u8(addr, cpu.Accumulator);
        }
        // Stack pointer ops
        Instruction::LoadIndirectSP(addr) => {
            memory_bus.write_u8(addr, cpu.SP.get_bits(0..8) as u8);
            memory_bus.write_u8(addr + 1, cpu.SP.get_bits(8..16) as u8);
        }
        Instruction::LoadSPHL => {
            debug!("Put {:#X} into SP", cpu.get_hl());
            cpu.SP = cpu.get_hl();
        }
        Instruction::LoadHLSP(offset) => {
            let offset = offset as i16 as u16;
            let new_value = cpu.SP.wrapping_add(offset);
            cpu.H = new_value.get_bits(8..16) as u8;
            cpu.L = new_value.get_bits(0..8) as u8;
            cpu.set_flag(Flag::Z, false);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, ALU::test_add_carry_bit(3, cpu.SP, offset));
            cpu.set_flag(Flag::C, ALU::test_add_carry_bit(7, cpu.SP, offset));
        }
        Instruction::AddSp(offset) => {
            let offset = offset as i16 as u16;
            let new_value = cpu.SP.wrapping_add(offset);
            cpu.set_flag(Flag::Z, false);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, ALU::test_add_carry_bit(3, cpu.SP, offset));
            cpu.set_flag(Flag::C, ALU::test_add_carry_bit(7, cpu.SP, offset));
            cpu.SP = new_value;
        }
        _ => return None,
    }

    // Loads don't care about action_taken
    Some(instr.ticks(false))
}
