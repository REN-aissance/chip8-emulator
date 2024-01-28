use std::{f32::consts::TAU, time::Duration};

use rodio::Source;

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
