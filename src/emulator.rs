use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::Receiver,
    Arc, Mutex,
};

pub mod cpu;
use cpu::CPU;
pub mod instructions;
pub mod memory_bus;
use memory_bus::MemoryBus;
pub mod ppu;
use ppu::PPU;
#[cfg(test)]
pub mod unit_tests;

const GAMEBOY_WIDTH: usize = 160;
const GAMEBOY_HEIGHT: usize = 144;

pub struct DoubleBuffer {
    buffers: [Mutex<ppu::FrameBuffer>; 2],
    curr_buffer: AtomicUsize,
}

impl Default for DoubleBuffer {
    fn default() -> Self {
        Self {
            buffers: [
                Mutex::new([0; GAMEBOY_HEIGHT * GAMEBOY_WIDTH]),
                Mutex::new([0; GAMEBOY_HEIGHT * GAMEBOY_WIDTH]),
            ],
            curr_buffer: AtomicUsize::new(0),
        }
    }
}

impl DoubleBuffer {
    pub fn get_current(&self) -> &Mutex<ppu::FrameBuffer> {
        &self.buffers[self.curr_buffer.load(Ordering::Relaxed)]
    }

    pub fn get_off(&self) -> &Mutex<ppu::FrameBuffer> {
        &self.buffers[(self.curr_buffer.load(Ordering::Relaxed) + 1) % 2]
    }

    pub fn swap(&self) {
        let mut num = self.curr_buffer.load(Ordering::Acquire);
        num = (num + 1) % 2;
        self.curr_buffer.store(num, Ordering::Release);
    }
}

/// From https://github.com/mvdnes/rboy/blob/c6630fa97e55a5595109a37c807038deb7a734fb/src/main.rs#L323
fn timer_periodic(ms: u64) -> Receiver<()> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(ms));
        if tx.send(()).is_err() {
            break;
        }
    });
    rx
}

pub fn run() -> Arc<DoubleBuffer> {
    let buffer = Arc::new(DoubleBuffer::default());

    let emu_buffer = Arc::clone(&buffer);
    std::thread::spawn(move || {
        let buffer = emu_buffer;
        // let file = include_bytes!("../roms/test.gb");
        // let file = include_bytes!("../roms/hello-world.gb");
        // let file = include_bytes!("../roms/tetris.gb");
        // let file = include_bytes!("../roms/alu-test.gb");
        // let file = include_bytes!("../roms/dmg-acid2.gb");
        // let file = include_bytes!("../roms/cpu_instrs.gb");
        // let file = include_bytes!("../roms/01-special.gb");
        // let file = include_bytes!("../roms/04-op r,imm.gb");
        let file = include_bytes!("../roms/03-op sp,hl.gb");

        let memory_bus = MemoryBus::new(file.as_slice());
        let mut cpu = CPU::default();
        let mut ppu = PPU::default();

        // Thanks to https://github.com/mvdnes/rboy/blob/c6630fa97e55a5595109a37c807038deb7a734fb/src/main.rs#L285
        // 16ms period = 60fps
        let periodic = timer_periodic(16);

        loop {
            let mut lock = buffer.get_off().lock().unwrap();
            while !ppu.updated {
                let ticks = cpu.tick(&memory_bus);
                ppu.tick(&memory_bus, &mut *lock, ticks * 4);
            }
            ppu.updated = false;

            // Reduce contention by dropping this lock before swap
            // Contention can still happen if the render thread is rendering when we swap
            drop(lock);
            buffer.swap();

            periodic.recv().unwrap();
        }
    });

    buffer
}
