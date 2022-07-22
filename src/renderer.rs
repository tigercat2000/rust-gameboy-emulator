use std::sync::Arc;

use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

mod wgpu_core;
use wgpu_core::WGPUCore;

mod gameboy_pass;
use gameboy_pass::GameBoyPass;

use crate::emulator;

pub struct Renderer {
    core: WGPUCore,
    gameboy_pass: GameBoyPass,
}

impl Renderer {
    pub fn new(window: &Window, buffer: Arc<emulator::DoubleBuffer>) -> Self {
        let core = WGPUCore::new(window);
        let gameboy_pass = GameBoyPass::new(&core, buffer);
        Self { core, gameboy_pass }
    }

    pub fn handle_event(
        &mut self,
        window: &Window,
        event: &Event<()>,
        control_flow: &mut ControlFlow,
    ) -> bool {
        match event {
            Event::WindowEvent { window_id, event } if *window_id == window.id() => match event {
                WindowEvent::Resized(new_inner_size) => {
                    self.resize(*new_inner_size);
                    true
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    self.resize(**new_inner_size);
                    true
                }
                _ => false,
            },
            Event::MainEventsCleared => {
                window.request_redraw();
                true
            }
            Event::RedrawRequested(window_id) if *window_id == window.id() => {
                let output = match self.core.surface.get_current_texture() {
                    Ok(texture) => texture,
                    Err(wgpu::SurfaceError::Lost) => {
                        self.resize(self.core.size);
                        return true;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = ControlFlow::Exit;
                        return true;
                    }
                    Err(wgpu::SurfaceError::Outdated) => return true, // Minimized
                    Err(e) => {
                        eprintln!("{:?}", e);
                        return true;
                    }
                };

                let output_view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // TODO: Intermediate texture
                self.gameboy_pass.render(&self.core, &output_view);
                output.present();

                true
            }
            _ => false,
        }
    }
}

impl Renderer {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.core.resize(new_size);
        }
    }
}
