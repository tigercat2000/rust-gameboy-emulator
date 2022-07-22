use renderer::Renderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

pub mod emulator;
pub mod renderer;

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(true)
        .with_transparent(false)
        .with_title("Gameboy Emulator")
        .build(&event_loop)
        .expect("Failed to create window with winit");

    let buffer = emulator::run();
    let mut renderer = Renderer::new(&window, buffer);

    event_loop.run(move |event, _, control_flow| {
        if renderer.handle_event(&window, &event, control_flow) {
            return;
        }
        match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == window.id() && matches!(event, WindowEvent::CloseRequested) => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    })
}
