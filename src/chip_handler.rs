mod buzzer;
mod chip8;
pub(crate) mod event;
mod keyboard;
pub(crate) mod screen;
mod stack;

use winit::event_loop::EventLoopProxy;

use self::{chip8::Chip8, event::Chip8Event, screen::ScreenBuffer};

const CPU_IPS: u32 = 15;
const FF_IPS: u32 = CPU_IPS * 32;

pub struct ChipHandler {
    ips: u32,
    cpu: Chip8,
    sys_tx: EventLoopProxy<Chip8Event>,
}

impl ChipHandler {
    pub fn new(sys_tx: EventLoopProxy<Chip8Event>) -> ChipHandler {
        ChipHandler {
            ips: CPU_IPS,
            cpu: Chip8::new().with_rom(include_bytes!("../roms/Chip-8-Other/pumpkindressup.ch8")),
            sys_tx,
        }
    }

    pub fn update(&mut self) {
        for _ in 0..CPU_IPS {
            if let Some(e) = self.cpu.update() {
                match e {
                    Chip8Event::Shutdown | Chip8Event::RequestRedraw => {
                        self.sys_tx.send_event(e).unwrap();
                        break;
                    }
                    _ => (),
                }
            }
        }
        self.cpu.decrement_timers();
    }

    pub fn update_key(&mut self, key: u8, state: bool) {
        self.cpu.set_key(key, state)
    }

    pub fn start_ff(&mut self) {
        self.ips = FF_IPS;
    }

    pub fn stop_ff(&mut self) {
        self.ips = CPU_IPS;
    }

    pub fn get_frame_buffer(&self) -> ScreenBuffer {
        self.cpu.get_display_buffer()
    }
}
