use std::{
    sync::mpsc,
    thread::{self},
    time::Duration,
};

use mpsc::{Receiver, Sender};
use rodio::{
    source::{SineWave, Source},
    OutputStream, Sink,
};

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

    fn play_internal(d: Duration) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let sound = SineWave::new(220.0).take_duration(d).amplify(0.2);
        sink.append(sound);
        sink.sleep_until_end();
    }

    pub fn play(&self, d: Duration) {
        self.tx.send(AudioEvent::Play(d)).unwrap();
    }

    fn event_handler(rx: Receiver<AudioEvent>) {
        loop {
            match rx.recv() {
                Ok(AudioEvent::Play(d)) => Self::play_internal(d),
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
