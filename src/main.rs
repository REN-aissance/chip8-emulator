#![feature(let_chains)]
#![feature(get_many_mut)]

use chip8::event::Chip8Event;
use chip8handler::Chip8Handler;
use render::Renderer;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window},
};

mod render;
mod texture;
mod chip8handler;
mod chip8;

pub const ASPECT_RATIO: f32 = 4.0 / 3.0;
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

async fn execute_event_loop(event_loop: EventLoop<Chip8Event>, window: Window) {
    let mut renderer = Renderer::new(&window).await;
    let mut chip8 = Chip8Handler::new(event_loop.create_proxy());

    event_loop.run(|event, event_target| match event {
        Event::UserEvent(Chip8Event::Shutdown) => {
            event_target.exit();
        }
        Event::UserEvent(Chip8Event::RequestRedraw) => {
            window.request_redraw();
        }
        Event::AboutToWait => {
            window.request_redraw()
        },
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                event_target.exit()
            },
            WindowEvent::Resized(new_size) => renderer.resize(new_size),
            WindowEvent::RedrawRequested => {
                chip8.update();
                renderer.update_screen(&chip8.get_frame_buffer().borrow());
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
                handle_chip8_input(state, keycode, &mut chip8);
                if state == ElementState::Pressed && keycode == KeyCode::Enter {
                    match window.fullscreen() {
                        Some(_) => window.set_fullscreen(None),
                        None => window.set_fullscreen(Some(Fullscreen::Borderless(None))),
                    }
                }
                if keycode == KeyCode::Escape {
                    event_target.exit();
                }
            },
            _ => (),
        },
        _ => (),
    }).unwrap();
}

fn handle_chip8_input(state: ElementState, keycode: KeyCode, chip8: &mut Chip8Handler) {
    let state = match state {
        ElementState::Pressed => true,
        ElementState::Released => false,
    };
    match keycode {
        KeyCode::Space => {
            match state {
                true => chip8.start_ff(),
                false => chip8.stop_ff(),
            }
        },
        KeyCode::Backslash => chip8.reset(),
        KeyCode::Digit1 => chip8.update_key(0x1, state),
        KeyCode::Digit2 => chip8.update_key(0x2, state),
        KeyCode::Digit3 => chip8.update_key(0x3, state),
        KeyCode::Digit4 => chip8.update_key(0xC, state),
        KeyCode::KeyQ => chip8.update_key(0x4, state),
        KeyCode::KeyW => chip8.update_key(0x5, state),
        KeyCode::KeyE => chip8.update_key(0x6, state),
        KeyCode::KeyR => chip8.update_key(0xD, state),
        KeyCode::KeyA => chip8.update_key(0x7, state),
        KeyCode::KeyS => chip8.update_key(0x8, state),
        KeyCode::KeyD => chip8.update_key(0x9, state),
        KeyCode::KeyF => chip8.update_key(0xE, state),
        KeyCode::KeyZ => chip8.update_key(0xA, state),
        KeyCode::KeyX => chip8.update_key(0x0, state),
        KeyCode::KeyC => chip8.update_key(0xB, state),
        KeyCode::KeyV => chip8.update_key(0xF, state),
        _ => (),
    }
}

pub fn main() {
    let event_loop = EventLoopBuilder::<Chip8Event>::with_user_event().build().expect("Could not create event_loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let window = Window::new(&event_loop).expect("Could not create window");
    window.set_title("Chip-8 Emulator");
    futures::executor::block_on(execute_event_loop(event_loop, window));
}