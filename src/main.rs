use std::time::Duration;

use render::Renderer;
use sound::Buzzer;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

mod render;
mod screen;
mod sound;

pub const ASPECT_RATIO: f32 = 4.0 / 3.0;

async fn execute_event_loop(event_loop: EventLoop<()>, window: Window) {
    let mut state = Renderer::new(&window).await;
    let bz = Buzzer::new();

    let _ = event_loop.run(|event, event_target| match event {
        Event::AboutToWait => {
            window.request_redraw();
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                event_target.exit();
            }
            WindowEvent::Resized(new_size) => {
                state.resize(new_size);
            }
            WindowEvent::RedrawRequested => {
                state.render();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Space),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => bz.play(Duration::from_millis(250)),
            _ => (),
        },
        _ => (),
    });
}

pub fn main() {
    let event_loop = EventLoop::new().expect("Could not create event_loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let window = Window::new(&event_loop).expect("Could not create window");
    window.set_title("Chip-8 Emulator");
    futures::executor::block_on(execute_event_loop(event_loop, window));
}