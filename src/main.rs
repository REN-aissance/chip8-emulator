#![feature(let_chains)]

use chip8::{Chip8, Chip8Event};
use render::Renderer;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

mod render;
mod texture;
mod chip8;

pub const ASPECT_RATIO: f32 = 4.0 / 3.0;
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

async fn execute_event_loop(event_loop: EventLoop<Chip8Event>, window: Window) {
    let mut renderer = Renderer::new(&window).await;
    let ep = event_loop.create_proxy();
    let mut chip8 = Chip8::new(ep);
    chip8.load_rom_from_bytes(include_bytes!("../roms/test_opcode.ch8"));

    let _ = event_loop.run(|event, event_target| match event {
        Event::UserEvent(Chip8Event::RequestRedraw(buffer)) => {
            renderer.update_screen(buffer);
            window.request_redraw();
        }
        Event::AboutToWait => {
            chip8.update(Chip8Event::Update);
        }
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
                    KeyCode::Digit1 => Some(0x1),
                    KeyCode::Digit2 => Some(0x2),
                    KeyCode::Digit3 => Some(0x3),
                    KeyCode::Digit4 => Some(0xC),
                    KeyCode::KeyQ => Some(0x4),
                    KeyCode::KeyW => Some(0x5),
                    KeyCode::KeyE => Some(0x6),
                    KeyCode::KeyR => Some(0xD),
                    KeyCode::KeyA => Some(0x7),
                    KeyCode::KeyS => Some(0x8),
                    KeyCode::KeyD => Some(0x9),
                    KeyCode::KeyF => Some(0xE),
                    KeyCode::KeyZ => Some(0xA),
                    KeyCode::KeyX => Some(0x0),
                    KeyCode::KeyC => Some(0xB),
                    KeyCode::KeyV => Some(0xF),
                    _ => None,
                } {
                    chip8.set_key(key, state)
                }
            },
            _ => (),
        },
        _ => (),
    });
}

pub fn main() {
    let event_loop = EventLoopBuilder::<Chip8Event>::with_user_event().build().expect("Could not create event_loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let window = Window::new(&event_loop).expect("Could not create window");
    window.set_title("Chip-8 Emulator");
    futures::executor::block_on(execute_event_loop(event_loop, window));
}