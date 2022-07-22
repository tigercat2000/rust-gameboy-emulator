use tracing::trace;

use crate::emulator::{instructions::Instruction, memory_bus::MemoryBus};

use super::{ALU, CPU};

pub fn handle_instruction(
    cpu: &mut CPU,
    instr: Instruction,
    memory_bus: &MemoryBus,
) -> Option<u32> {
    let mut action_taken = false;

    match instr {
        // Jumps
        Instruction::Jump(target) => {
            cpu.PC = target;
        }
        Instruction::JumpConditional(condition, addr) => {
            let jump = cpu.check_condition(condition);
            if jump {
                action_taken = true;
                cpu.PC = addr;
            }
        }
        Instruction::JumpRelative(rel) => {
            cpu.PC = ALU::add_rel(cpu.PC, rel);
        }
        Instruction::JumpRelativeConditional(condition, rel) => {
            let jump = cpu.check_condition(condition);
            if jump {
                action_taken = true;
                cpu.PC = ALU::add_rel(cpu.PC, rel);
            }
        }
        Instruction::JumpHL => {
            cpu.PC = cpu.get_hl();
        }
        // Calls/Rets
        Instruction::Call(imm) => {
            trace!("Writing {:#X} to stack @ {:#X}", cpu.PC, cpu.SP);
            memory_bus.write_stack_16(&mut cpu.SP, cpu.PC);
            cpu.PC = imm;
        }
        Instruction::CallConditional(condition, addr) => {
            let jump = cpu.check_condition(condition);
            if jump {
                action_taken = true;
                memory_bus.write_stack_16(&mut cpu.SP, cpu.PC);
                cpu.PC = addr;
            }
        }
        Instruction::Ret => {
            let addr = memory_bus.get_stack_16(&mut cpu.SP);
            trace!("Read {:#X} from stack @ {:#X}", addr, cpu.SP);
            cpu.PC = addr;
        }
        Instruction::RetConditional(condition) => {
            let condition = cpu.check_condition(condition);

            if condition {
                cpu.PC = memory_bus.get_stack_16(&mut cpu.SP);
                trace!("Read {:#X} from stack @ {:#X}", cpu.PC, cpu.SP);
            }
        }
        Instruction::RetInterrupt => {
            let addr = memory_bus.get_stack_16(&mut cpu.SP);
            trace!("Read {:#X} from stack @ {:#X}", addr, cpu.SP);
            cpu.PC = addr;
            cpu.IME = true;
        }
        // Reset Vectors
        Instruction::Reset(offset) => {
            memory_bus.write_stack_16(&mut cpu.SP, cpu.PC);
            cpu.PC = offset as u16;
        }
        _ => return None,
    }

    Some(instr.ticks(action_taken))
}
