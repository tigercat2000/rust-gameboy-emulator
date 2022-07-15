pub mod cpu;
pub mod instructions;
pub mod memory_bus;
pub mod ppu;
#[cfg(test)]
pub mod unit_tests;

use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
        Arc, Mutex,
    },
};

const GAMEBOY_WIDTH: usize = 160;
const GAMEBOY_HEIGHT: usize = 144;

use crate::{cpu::CPU, memory_bus::MemoryBus, ppu::PPU};
use egui::{Color32, Pos2, Rect, Rounding};
use tracing::{debug, event, trace, warn, Level};

struct TransitBuffer {
    is_ready: AtomicBool,
    rcell: Mutex<RefCell<ppu::FrameBuffer>>,
}

impl Default for TransitBuffer {
    fn default() -> Self {
        Self {
            is_ready: AtomicBool::new(false),
            rcell: Mutex::new(RefCell::new([0; GAMEBOY_WIDTH * GAMEBOY_HEIGHT])),
        }
    }
}

const SCALE: f32 = 3.0;

fn main() {
    // let format = tracing_subscriber::fmt::format()
    //     .with_level(false)
    //     .with_ansi(false)
    //     .compact();
    // tracing_subscriber::fmt()
    //     .event_format(format)
    //     .with_max_level(Level::TRACE)
    //     .init();

    tracing_subscriber::fmt::init();

    // Spawn renderer thread
    eframe::run_native(
        "Gameboy Emulator",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            let cpu_buffer = RefCell::new([0; GAMEBOY_WIDTH * GAMEBOY_HEIGHT]);
            let transit_buffer = Arc::new(TransitBuffer::default());
            let render_buffer = RefCell::new([0; GAMEBOY_WIDTH * GAMEBOY_HEIGHT]);

            // Spawn emulator thread
            let emuthread_buffer_ref = Arc::clone(&transit_buffer);
            let emuthread_egui_ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let egui_ctx = emuthread_egui_ctx;
                let transit_buffer = emuthread_buffer_ref;
                // let file = include_bytes!("../test.gb");
                let file = include_bytes!("../hello-world.gb");
                // let file = include_bytes!("../alu-test.gb");
                event!(
                    Level::DEBUG,
                    "Loaded hello-world.gb - {:#X} bytes",
                    file.len()
                );

                let memory_bus = MemoryBus::new(file.as_slice());
                let mut cpu = CPU::default();
                let mut ppu = PPU;

                // Thanks to https://github.com/mvdnes/rboy/blob/c6630fa97e55a5595109a37c807038deb7a734fb/src/main.rs#L285
                let periodic = timer_periodic(16);
                let wait_ticks = 0x10000;
                let mut ticks = 0;

                loop {
                    while ticks < wait_ticks {
                        trace!("\n----------------\nTicks: {}/{}", ticks, wait_ticks);
                        ticks += cpu.tick(&memory_bus);
                        ppu.tick(&memory_bus);
                    }
                    ticks = 0;

                    ppu.render(&memory_bus, &mut cpu_buffer.borrow_mut());

                    if transit_buffer
                        .is_ready
                        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                    {
                        transit_buffer.rcell.lock().unwrap().swap(&cpu_buffer);
                    }

                    egui_ctx.request_repaint();
                    debug!("Asked for repaint");

                    periodic.recv().unwrap();
                }
            });

            Box::new(App::new(cc, transit_buffer, render_buffer))
        }),
    );
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

#[allow(dead_code)]
struct App {
    transit_buffer: Arc<TransitBuffer>,
    render_buffer: RefCell<ppu::FrameBuffer>,
}

impl App {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        transit_buffer: Arc<TransitBuffer>,
        render_buffer: RefCell<ppu::FrameBuffer>,
    ) -> Self {
        Self {
            transit_buffer,
            render_buffer,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self
            .transit_buffer
            .is_ready
            .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            self.transit_buffer
                .rcell
                .lock()
                .unwrap()
                .swap(&self.render_buffer);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            for x in 0..160 {
                for y in 0..140 {
                    painter.rect_filled(
                        Rect::from_two_pos(
                            Pos2::new(x as f32 * SCALE, y as f32 * SCALE),
                            Pos2::new(x as f32 * SCALE + SCALE, y as f32 * SCALE + SCALE),
                        ),
                        Rounding::none(),
                        Color32::from_gray(self.render_buffer.borrow()[x + (y * GAMEBOY_WIDTH)]),
                    )
                }
            }
        });
    }
}
