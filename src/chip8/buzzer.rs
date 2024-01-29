use mpsc::Receiver;
use rodio::{OutputStream, Source};
use std::{
    sync::mpsc::{self, Sender},
    thread::{self},
    time::Duration,
};

use crate::chip8::square_wave::SquareWave;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum AudioEvent {
    Play(Duration),
    Terminate,
}

pub struct Buzzer {
    tx: Sender<AudioEvent>,
}

impl Buzzer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || Self::event_handler(rx));
        Self { tx }
    }

    pub fn play(&self, d: Duration) {
        self.tx.send(AudioEvent::Play(d)).unwrap();
    }

    fn event_handler(rx: Receiver<AudioEvent>) {
        loop {
            match rx.recv() {
                Ok(AudioEvent::Play(d)) => {
                    let sound = SquareWave::new(261.60).amplify(0.1);
                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                    let sound = sound.clone().take_duration(d);
                    let _ = stream_handle.play_raw(sound);
                    thread::sleep(d);
                }
                Ok(AudioEvent::Terminate) => break,
                Err(e) => panic!("{}", e),
            }
        }
    }

    pub fn exit(&self) {
        self.tx.send(AudioEvent::Terminate).unwrap();
    }
}

impl Drop for Buzzer {
    fn drop(&mut self) {
        self.exit();
    }
}
