#![feature(let_chains)]
#![feature(get_many_mut)]

use chip_handler::{event::Chip8Event, ChipHandler};
use render::Renderer;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window},
};

mod render;
mod texture;
mod chip_handler;

pub const ASPECT_RATIO: f32 = 4.0 / 3.0;
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

async fn execute_event_loop(event_loop: EventLoop<Chip8Event>, window: Window) {
    let mut renderer = Renderer::new(&window).await;
    let mut chip8 = ChipHandler::new(event_loop.create_proxy());

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
                if state == ElementState::Pressed {
                    match (keycode, window.fullscreen()) {
                        (KeyCode::Escape, _,) => {
                            event_target.exit();
                        },
                        (KeyCode::Enter, Some(_)) => window.set_fullscreen(None),
                        (KeyCode::Enter, None) => window.set_fullscreen(Some(Fullscreen::Borderless(None))),
                        _ => (),
                    }
                }
            },
            _ => (),
        },
        _ => (),
    }).unwrap();
}

//FIXME This is disgusting
fn handle_chip8_input(state: ElementState, keycode: KeyCode, chip8: &mut ChipHandler) {
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
                true => chip8.start_ff(),
                false => chip8.stop_ff(),
            };
            None
        }
        _ => None,
    } {
        chip8.update_key(key, state);
    }
}

pub fn main() {
    let event_loop = EventLoopBuilder::<Chip8Event>::with_user_event().build().expect("Could not create event_loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let window = Window::new(&event_loop).expect("Could not create window");
    window.set_title("Chip-8 Emulator");
    futures::executor::block_on(execute_event_loop(event_loop, window));
}