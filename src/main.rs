pub mod cpu;
pub mod instructions;
pub mod memory_bus;
pub mod ppu;
#[cfg(test)]
pub mod unit_tests;

use std::sync::{mpsc::Receiver, Arc, Mutex};

use crate::{cpu::CPU, memory_bus::MemoryBus, ppu::PPU};
use tracing::{event, trace, warn, Level};

fn main() {
    tracing_subscriber::fmt::init();

    eframe::run_native(
        "Gameboy Emulator",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(App::new(cc))),
    );
}

#[derive(Default)]
struct Cache {
    cpu_debug: String,
}

impl Cache {
    pub fn update(&mut self, cpu: &Arc<Mutex<CPU>>, _ppu: &Arc<Mutex<PPU>>) {
        if let Ok(cpu) = cpu.try_lock() {
            warn!("Cache::update got cpu guard");
            self.cpu_debug = cpu.to_string()
        }
    }
}

#[allow(dead_code)]
struct App {
    memory_bus: Arc<Mutex<MemoryBus>>,
    cpu: Arc<Mutex<CPU>>,
    ppu: Arc<Mutex<PPU>>,
    cache: Cache,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // let file = include_bytes!("../test.gb");
        let file = include_bytes!("../hello-world.gb");
        // let file = include_bytes!("../alu-test.gb");
        event!(
            Level::DEBUG,
            "Loaded hello-world.gb - {:#X} bytes",
            file.len()
        );

        let memory_bus = Arc::new(Mutex::new(MemoryBus::new(file.as_slice())));
        let cpu = Arc::new(Mutex::new(CPU::default()));
        let ppu = Arc::new(Mutex::new(PPU::default()));

        let egui_context = cc.egui_ctx.clone();
        let thread_bus = Arc::clone(&memory_bus);
        let thread_cpu = Arc::clone(&cpu);
        let thread_ppu = Arc::clone(&ppu);
        std::thread::spawn(move || {
            // Thanks to https://github.com/mvdnes/rboy/blob/c6630fa97e55a5595109a37c807038deb7a734fb/src/main.rs#L285
            let periodic = timer_periodic(16);
            let wait_ticks = 0x10000;
            let mut ticks = 0;

            loop {
                let bus = thread_bus.lock().unwrap();
                let mut cpu = thread_cpu.lock().unwrap();
                let mut ppu = thread_ppu.lock().unwrap();

                while ticks < wait_ticks {
                    trace!("Ticks: {}/{}", ticks, wait_ticks);
                    ticks += cpu.tick(&bus);
                    ppu.tick(&bus);
                }
                warn!("Hit wait ticks");

                drop(bus);
                drop(cpu);
                drop(ppu);

                ticks = 0;

                egui_context.request_repaint();

                let _ = periodic.recv();
            }
        });

        Self {
            memory_bus,
            cpu,
            ppu,
            cache: Default::default(),
        }
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.cache.update(&self.cpu, &self.ppu);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.cache.cpu_debug);
        });
    }
}
