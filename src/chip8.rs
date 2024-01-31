mod buzzer;
mod cpu;
pub(crate) mod event;
mod keyboard;
pub(crate) mod screen;
mod stack;

use self::{
    cpu::{Cpu, CLK_SPEED_HZ},
    event::Chip8Event,
};
use crate::SystemEvent;
use std::{
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use winit::event_loop::EventLoopProxy;

const FF_SPEED_HZ: f32 = CLK_SPEED_HZ * 32.0;

pub struct Chip8 {
    _thread_handle: JoinHandle<()>,
    tx: Sender<SystemEvent>,
}

impl Chip8 {
    pub fn new(sys_tx: EventLoopProxy<Chip8Event>) -> Chip8 {
        let mut cpu = Cpu::new().with_rom(include_bytes!("../roms/TEST_KEYPAD_OK"));
        let mut clock_speed = CLK_SPEED_HZ;
        let (tx, rx) = mpsc::channel();
        let thread_handle = thread::spawn(move || loop {
            let dt = Instant::now();
            //This does not seem to drop any events. Causes input lag at clock speed < 60HZ
            match rx.try_recv() {
                Ok(SystemEvent::CloseRequested) => break,
                Ok(SystemEvent::KeyEvent(key, state)) => cpu.set_key(key, state),
                Ok(SystemEvent::StartFastForward) => clock_speed = FF_SPEED_HZ,
                Ok(SystemEvent::StopFastForward) => clock_speed = CLK_SPEED_HZ,
                Ok(SystemEvent::UpdateTimer) => {
                    cpu.update_timers();
                }
                _ => (),
            }
            if let Some(e) = cpu.update() {
                match e {
                    Chip8Event::RequestRedraw(_) => sys_tx.send_event(e).unwrap(),
                    Chip8Event::Shutdown => {
                        sys_tx.send_event(e).unwrap();
                        break;
                    }
                    _ => (),
                }
            }
            thread::sleep(Duration::from_secs_f32(1.0 / clock_speed).saturating_sub(dt.elapsed()));
        });
        Chip8 {
            _thread_handle: thread_handle,
            tx,
        }
    }

    pub fn send_event(&self, e: SystemEvent) {
        let _ = self.tx.send(e);
    }
}
