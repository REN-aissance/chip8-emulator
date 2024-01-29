use crate::{HEIGHT, WIDTH};

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
        let (w, h) = (WIDTH as u8, HEIGHT as u8);
        let (x, y) = (x % w, y % h);
        let mut intersection = false;

        sprite.iter().enumerate().for_each(|(i, &val)| {
            let i = i as u8;
            let word_offset = (x % 8) as u32;

            let y = (y + i) % h; //Wrap screen horizontally
            if let Some(lb) = self.get(x, y) {
                let val = val.checked_shr(word_offset).unwrap_or(0);
                *lb ^= val;
                intersection |= lb.checked_shr(word_offset).unwrap_or(0) != val
            }
            //Inserts to the next word (wrapping) if sprite crosses word boundary
            let x = (x + 8) % w; //Wrap screen horizontally
            if let Some(ub) = self.get(x, y) {
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
