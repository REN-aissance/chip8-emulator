use chip8::Chip8;
use render::Renderer;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

mod render;
mod square_wave;
mod texture;
mod chip8;

pub const ASPECT_RATIO: f32 = 4.0 / 3.0;
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

async fn execute_event_loop(event_loop: EventLoop<()>, window: Window) {
    let mut renderer = Renderer::new(&window).await;
    let mut chip8 = Chip8::new();
    chip8.load_rom_from_bytes(include_bytes!("../roms/MISSILE"));

    let _ = event_loop.run(|event, event_target| match event {
        Event::AboutToWait => {
            chip8.update();
            renderer.update_screen(chip8.get_display_buffer());
            window.request_redraw();
        },
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => event_target.exit(),
            WindowEvent::Resized(new_size) => renderer.resize(new_size),
            WindowEvent::RedrawRequested => renderer.render(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(keycode),
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                let state = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                if let Some(key) = match keycode {
                    KeyCode::Digit1 => Some(0),
                    KeyCode::Digit2 => Some(1),
                    KeyCode::Digit3 => Some(2),
                    KeyCode::Digit4 => Some(3),
                    KeyCode::KeyQ => Some(4),
                    KeyCode::KeyW => Some(5),
                    KeyCode::KeyE => Some(6),
                    KeyCode::KeyR => Some(7),
                    KeyCode::KeyA => Some(8),
                    KeyCode::KeyS => Some(9),
                    KeyCode::KeyD => Some(10),
                    KeyCode::KeyF => Some(11),
                    KeyCode::KeyZ => Some(12),
                    KeyCode::KeyX => Some(13),
                    KeyCode::KeyC => Some(14),
                    KeyCode::KeyV => Some(15),
                    _ => None,
                } {
                    chip8.set_key(key, state);
                }
            },
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