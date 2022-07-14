pub mod cpu;
pub mod instructions;
pub mod memory_bus;
#[cfg(test)]
pub mod unit_tests;

use crate::{cpu::CPU, memory_bus::MemoryBus};

fn main() {
    let file = include_bytes!("../test.gb");
    let memory_bus = MemoryBus::new(file.as_slice());
    let mut cpu = CPU::new(&memory_bus);

    loop {
        cpu.tick();
        if cpu.stop {
            break;
        }
    }

    println!("Exit");
}
