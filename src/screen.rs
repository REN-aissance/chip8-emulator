use crate::{HEIGHT, WIDTH};
use std::{error::Error, fmt::Display};

#[derive(Debug, Copy, Clone)]
pub struct ScreenError(usize, usize);
impl Display for ScreenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pixel does not exist in screen")?;
        Ok(())
    }
}
impl Error for ScreenError {}

#[repr(transparent)]
pub struct Screen([u8; (WIDTH / 8) * HEIGHT]);

impl Default for Screen {
    fn default() -> Self {
        Self([0x00_u8; (WIDTH / 8) * HEIGHT])
    }
}

impl Screen {
    //Bitwise not due to pbm P4 file format specification
    pub const CLEAR: [u8; (WIDTH / 8) * HEIGHT] = [!0x00; (WIDTH / 8) * HEIGHT];

    pub fn put_pixel(&mut self, x: usize, y: usize) -> Result<(), Box<dyn Error>> {
        let index = Self::get_index(x, y);
        *self.0.get_mut(index).ok_or(ScreenError(x, y))? |= 0x01_u8.rotate_left(7 - (x as u32 % 8));
        Ok(())
    }
    pub fn clear_pixel(&mut self, x: usize, y: usize) -> Result<(), Box<dyn Error>> {
        let index = Self::get_index(x, y);
        *self.0.get_mut(index).ok_or(ScreenError(x, y))? &= 0x01_u8.rotate_left(7 - (x as u32 % 8));
        Ok(())
    }

    fn get_index(x: usize, y: usize) -> usize {
        (x / 8) + (y * WIDTH / 8)
    }

    pub fn as_bytes(&self) -> [u8; HEIGHT / 8 * WIDTH] {
        self.0.map(|n| !n)
    }
}
