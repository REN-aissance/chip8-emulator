use mpsc::Receiver;
use rodio::{OutputStream, Sink, Source};
use std::{
    sync::mpsc::{self, Sender},
    thread::{self},
    time::Duration,
};

use std::f32::consts::TAU;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum AudioEvent {
    Play,
    Pause,
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

    pub fn play(&self) {
        self.tx.send(AudioEvent::Play).unwrap();
    }

    pub fn pause(&self) {
        self.tx.send(AudioEvent::Pause).unwrap();
    }

    fn event_handler(rx: Receiver<AudioEvent>) {
        let sound = SquareWave::new(261.60);
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(0.2);
        sink.pause();
        sink.append(sound);
        loop {
            match rx.recv() {
                Ok(AudioEvent::Play) => {
                    #[cfg(feature = "sound_debug")]
                    println!("Sound start");
                    sink.play();
                }
                Ok(AudioEvent::Pause) => {
                    #[cfg(feature = "sound_debug")]
                    println!("Sound stop");
                    sink.pause();
                }
                Ok(AudioEvent::Terminate) => break,
                Err(e) => panic!("{}", e),
            }
            //Prevent this loop from running at full speed wastng cycles
            thread::sleep(Duration::from_secs_f64(1.0 / 60.0))
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

#[derive(Debug, Clone)]
pub struct SquareWave {
    freq: f32,
    n_samples: usize,
    sample_rate: u32,
}

impl SquareWave {
    pub fn new(freq: f32) -> SquareWave {
        SquareWave {
            freq,
            ..Default::default()
        }
    }
}

impl Default for SquareWave {
    fn default() -> Self {
        Self {
            freq: 440.0,
            n_samples: 0,
            sample_rate: 44100,
        }
    }
}

impl Iterator for SquareWave {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.n_samples = self.n_samples.wrapping_add(1);
        let value = TAU * self.freq * (self.n_samples as f32 / self.sample_rate as f32);
        Some(value.sin().signum())
    }
}

impl Source for SquareWave {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
