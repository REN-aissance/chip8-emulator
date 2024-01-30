use crate::{HEIGHT, WIDTH};

const BUFFER_LEN: usize = (WIDTH / 8) * HEIGHT;
pub type ScreenBuffer = [u8; BUFFER_LEN];

#[repr(transparent)]
pub struct Screen(ScreenBuffer);

impl Default for Screen {
    fn default() -> Self {
        Self(Self::CLEAR)
    }
}

impl Screen {
    const CLEAR: [u8; BUFFER_LEN] = [0x00; BUFFER_LEN];

    pub fn print_sprite(&mut self, sprite: &[u8], x: u8, y: u8) -> bool {
        let (w, h) = (WIDTH as u8, HEIGHT as u8);
        let (x, y) = (x % w, y % h);
        let mut intersection = false;

        sprite.iter().enumerate().for_each(|(i, &val)| {
            let i = i as u8;
            let word_offset = (x % 8) as u32;

            let y = (y + i) % h; //Wrap screen horizontally
            if let Some(lb) = self.get_mut(x, y) {
                let val = val.checked_shr(word_offset).unwrap_or(0);
                let t = *lb | val;
                *lb ^= val;
                intersection |= *lb != t;

                #[cfg(feature = "intersection_debug")]
                if intersection {
                    println!("{:08b} {:08b}", *lb, t);
                }
            }
            //Inserts to the next word (wrapping) if sprite crosses word boundary
            let x = (x + 8) % w; //Wrap screen horizontally
            if let Some(ub) = self.get_mut(x, y) {
                let val = val.checked_shl(8 - word_offset).unwrap_or(0);
                let t = *ub | val;
                *ub ^= val;
                intersection |= *ub != t;

                #[cfg(feature = "intersection_debug")]
                if intersection {
                    println!("{:08b} {:08b}", *ub, t);
                }
            }
        });
        intersection
    }

    fn get_mut(&mut self, x: u8, y: u8) -> Option<&mut u8> {
        let i = (x as usize / 8) + (y as usize * WIDTH / 8);
        self.0.get_mut(i)
    }

    pub fn extract_buffer(&self) -> ScreenBuffer {
        self.0
    }

    pub fn clear(&mut self) {
        self.0 = Self::CLEAR;
    }

    pub fn default_buffer() -> ScreenBuffer {
        Self::CLEAR
    }
}
