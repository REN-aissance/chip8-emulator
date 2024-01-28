use std::time::Duration;
use render::Renderer;
use screen::Screen;
use sound::Buzzer;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

mod render;
mod sound;
mod square_wave;
mod texture;
mod screen;

pub const ASPECT_RATIO: f32 = 4.0 / 3.0;
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

async fn execute_event_loop(event_loop: EventLoop<()>, window: Window) {
    let mut renderer = Renderer::new(&window).await;
    let bz = Buzzer::new();
    let mut screen = Screen::default();

    let mut x = 0;
    let mut y = 0;

    let _ = event_loop.run(|event, event_target| match event {
        Event::AboutToWait => window.request_redraw(),
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => event_target.exit(),
            WindowEvent::Resized(new_size) => renderer.resize(new_size),
            WindowEvent::RedrawRequested => renderer.render(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Space),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                screen.put_pixel(x, y).unwrap();
                renderer.update(&screen.as_bytes());
                bz.play(Duration::from_millis(200));
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                match keycode {
                    KeyCode::KeyW => y -= 1,
                    KeyCode::KeyA => x -= 1,
                    KeyCode::KeyS => y += 1,
                    KeyCode::KeyD => x += 1,
                    _ => (),
                }
            }
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