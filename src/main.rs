#![feature(let_chains)]

use chip8::{event::Chip8Event, screen::Screen, Chip8};
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

pub const ASPECT_RATIO: f32 = 16.0 / 9.0;
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

#[derive(Debug)]
enum SystemEvent {
    CloseRequested,
    KeyEvent(u8, bool),
    StartFastForward,
    StopFastForward,
}
unsafe impl Send for SystemEvent {}
unsafe impl Sync for SystemEvent {}

async fn execute_event_loop(chip8: Chip8, event_loop: EventLoop<Chip8Event>, window: Window) {
    let mut renderer = Renderer::new(&window).await;
    let mut screen_buffer = Box::new(Screen::default_buffer());

    event_loop.run(|event, event_target| match event {
        Event::UserEvent(Chip8Event::RequestRedraw(buffer)) => {
            screen_buffer = buffer;
        }
        Event::AboutToWait => {
            window.request_redraw()
        },
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                chip8.send_event(SystemEvent::CloseRequested);
                event_target.exit()
            },
            WindowEvent::Resized(new_size) => renderer.resize(new_size),
            WindowEvent::RedrawRequested => {
                renderer.update_screen(*screen_buffer);
                renderer.render()
            },
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
                    KeyCode::Space => {
                        match state  {
                            true => chip8.send_event(SystemEvent::StartFastForward),
                            false => chip8.send_event(SystemEvent::StopFastForward),
                        };
                        None
                    }
                    _ => None,
                } {
                    chip8.send_event(SystemEvent::KeyEvent(key, state));
                }
            },
            _ => (),
        },
        _ => (),
    }).unwrap();
}

pub fn main() {
    let event_loop = EventLoopBuilder::<Chip8Event>::with_user_event().build().expect("Could not create event_loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let window = Window::new(&event_loop).expect("Could not create window");
    window.set_title("Chip-8 Emulator");

    let ep = event_loop.create_proxy();
    let chip8 = Chip8::new(ep);

    futures::executor::block_on(execute_event_loop(chip8, event_loop, window));
}