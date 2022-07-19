pub mod cpu;
pub mod instructions;
pub mod memory_bus;
pub mod ppu;
#[cfg(test)]
pub mod unit_tests;

use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::Receiver,
        Arc, Mutex,
    },
};

const GAMEBOY_WIDTH: usize = 160;
const GAMEBOY_HEIGHT: usize = 144;

use crate::{cpu::CPU, memory_bus::MemoryBus, ppu::PPU};
use egui::{color, Color32, ColorImage, PaintCallback, Pos2, Rect, Rounding};
use tracing::{debug, event, trace, warn, Level};

struct DoubleBuffer {
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
    fn get_current(&self) -> &Mutex<ppu::FrameBuffer> {
        &self.buffers[self.curr_buffer.load(Ordering::Relaxed)]
    }

    fn get_off(&self) -> &Mutex<ppu::FrameBuffer> {
        &self.buffers[(self.curr_buffer.load(Ordering::Relaxed) + 1) % 2]
    }

    fn swap(&self) {
        let mut num = self.curr_buffer.load(Ordering::Acquire);
        num = (num + 1) % 2;
        self.curr_buffer.store(num, Ordering::Release);
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
            let buffer = Arc::new(DoubleBuffer::default());

            // Spawn emulator thread
            let emuthread_buffer = Arc::clone(&buffer);
            let emuthread_egui_ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let egui_ctx = emuthread_egui_ctx;
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

                    let mut lock = emuthread_buffer.get_off().lock().unwrap();
                    ppu.render(&memory_bus, &mut lock);

                    // Reduce contention by dropping this lock before swap
                    // Contention can still happen if the render thread is rendering when we swap
                    drop(lock);
                    emuthread_buffer.swap();

                    egui_ctx.request_repaint();
                    debug!("Asked for repaint");

                    periodic.recv().unwrap();
                }
            });

            Box::new(App::new(cc, buffer))
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
    buffer: Arc<DoubleBuffer>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, buffer: Arc<DoubleBuffer>) -> Self {
        Self { buffer }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let fb = self.buffer.get_current().lock().unwrap();

            let mut color_vec = vec![];
            for y in 0..GAMEBOY_HEIGHT {
                for x in 0..GAMEBOY_WIDTH {
                    let color = fb[x + (y * GAMEBOY_WIDTH)];
                    color_vec.push(Color32::from_gray(color))
                }
            }

            drop(fb);

            let image = ColorImage {
                size: [GAMEBOY_WIDTH, GAMEBOY_HEIGHT],
                pixels: color_vec,
            };

            let texture = ui.ctx().load_texture("auto", image);

            ui.image(&texture, ui.max_rect().size());

            // let lock = self.buffer.get_current().try_lock().unwrap();
            // for x in 0..160 {
            //     for y in 0..140 {
            //         painter.rect_filled(
            //             Rect::from_two_pos(
            //                 Pos2::new(x as f32 * SCALE, y as f32 * SCALE),
            //                 Pos2::new(x as f32 * SCALE + SCALE, y as f32 * SCALE + SCALE),
            //             ),
            //             Rounding::none(),
            //             Color32::from_gray(lock[x + (y * GAMEBOY_WIDTH)]),
            //         )
            //     }
            // }
            // drop(lock);
        });
    }
}
