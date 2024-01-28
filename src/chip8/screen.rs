use crate::{HEIGHT, WIDTH};
use std::{error::Error, fmt::Display};

#[derive(Debug, Copy, Clone)]
pub enum ScreenError {
    OutOfBounds(u8, u8),
    InvalidSprite(u8),
}

impl Display for ScreenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScreenError::OutOfBounds(x, y) => write!(f, "Out of bounds: x:{},y:{}", x, y)?,
            ScreenError::InvalidSprite(i) => write!(f, "Invalid sprite index: {:x}", i)?,
        }
        Ok(())
    }
}
impl Error for ScreenError {}

type ScreenResult = Result<(), ScreenError>;

#[repr(transparent)]
pub struct Screen([u8; (WIDTH / 8) * HEIGHT]);

impl Default for Screen {
    fn default() -> Self {
        Self([0x00_u8; (WIDTH / 8) * HEIGHT])
    }
}

impl Screen {
    pub const CLEAR: [u8; (WIDTH / 8) * HEIGHT] = [0x00; (WIDTH / 8) * HEIGHT];

    pub fn print_sprite(&mut self, sprite: &[u8], x: u8, y: u8) -> bool {
        let (x,y) = (x % WIDTH as u8, y % HEIGHT as u8);
        let mut intersection = false;

        sprite.iter().enumerate().for_each(|(i, &val)| {
            let word_offset = (x % 8) as u32;
            if let Some(lb) = self.get(x, y + i as u8) {
                let val = val.checked_shr(word_offset).unwrap_or(0);
                *lb ^= val;
                intersection |= lb.checked_shr(word_offset).unwrap_or(0) != val
            }
            //Inserts to the next word if sprite crosses word boundary
            if let Some(ub) = self.get(x + 8, y + i as u8) {
                let val = val.checked_shl(8 - word_offset).unwrap_or(0);
                *ub ^= val;
                intersection |= ub.checked_shr(8 - word_offset).unwrap_or(0) != val
            }
        });

        intersection
    }

    fn get(&mut self, x: u8, y: u8) -> Option<&mut u8> {
        if x as usize > WIDTH {
            return None;
        }
        self.0.get_mut((x as usize / 8) + (y as usize * WIDTH / 8))
    }

    pub fn as_bytes(&self) -> [u8; HEIGHT / 8 * WIDTH] {
        self.0
    }

    pub fn clear(&mut self) {
        self.0 = Self::CLEAR;
    }
}
