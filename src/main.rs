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
    chip8.load_rom_from_bytes(include_bytes!("../roms/GUESS"));

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
                    KeyCode::Space => {
                        chip8.sound_test();
                        None
                    }
                    _ => None,
                } {
                    chip8.update(Chip8Event::KeyEvent(key, state))
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