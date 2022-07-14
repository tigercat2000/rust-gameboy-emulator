pub mod cpu;
pub mod instructions;
pub mod memory_bus;
#[cfg(test)]
pub mod unit_tests;

use crate::{cpu::CPU, memory_bus::MemoryBus};
use tracing::{event, Level};

fn main() {
    tracing_subscriber::fmt::init();
    // let file = include_bytes!("../test.gb");
    let file = include_bytes!("../hello-world.gb");
    // let file = include_bytes!("../alu-test.gb");
    event!(
        Level::DEBUG,
        "Loaded hello-world.gb - {:#X} bytes",
        file.len()
    );
    let memory_bus = MemoryBus::new(file.as_slice());
    let mut cpu = CPU::new(&memory_bus);

    loop {
        cpu.tick();
        if cpu.stop {
            break;
        }
    }

    event!(Level::WARN, "CPU at exit:\n{}", cpu);
}
