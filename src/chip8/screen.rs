use std::{cell::RefCell, rc::Rc};

use crate::{HEIGHT, WIDTH};

const BUFFER_LEN: usize = (WIDTH / 8) * HEIGHT;
pub(crate) type ScreenBuffer = Rc<RefCell<[u8]>>;

#[repr(transparent)]
pub struct Screen(ScreenBuffer);

impl Default for Screen {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(Self::CLEAR)))
    }
}

impl Screen {
    const CLEAR: [u8; BUFFER_LEN] = [0x00_u8; BUFFER_LEN];

    pub fn print_sprite(&mut self, sprite: &[u8], x: u8, y: u8) -> bool {
        let (w, h) = (WIDTH as u8, HEIGHT as u8);
        let (x, y) = (x % w, y % h);
        let mut intersection = false;

        sprite.iter().enumerate().for_each(|(i, &val)| {
            let i = i as u8;
            let word_offset = (x % 8) as u32;

            let y = y.wrapping_add(i); //Wrapping due to cpu wrapping sub
                                       // let y = y % h; //Wrap screen horizontally
            if let Some(i) = Self::fix_index(x, y)
                && let Some(lb) = self.0.borrow_mut().get_mut(i)
            {
                let val = val.checked_shr(word_offset).unwrap_or(0);
                let t = *lb | val;
                *lb ^= val;
                intersection |= *lb != t;

                #[cfg(feature = "intersection_debug")]
                print!("{:08b} {:08b} ", *lb, t);
            }
            //Inserts to the next word (wrapping) if sprite crosses word boundary
            let x = x.wrapping_add(8); //Wrapping due to cpu wrapping sub
                                       // let x = x % w; //Wrap screen horizontally
            if x < 64
                && let Some(i) = Self::fix_index(x, y)
                && let Some(ub) = self.0.borrow_mut().get_mut(i)
            {
                let val = val.checked_shl(8 - word_offset).unwrap_or(0);
                let t = *ub | val;
                *ub ^= val;
                intersection |= *ub != t;

                #[cfg(feature = "intersection_debug")]
                println!("{:08b} {:08b}", *ub, t);
            }
        });
        #[cfg(feature = "intersection_debug")]
        dbg!(intersection);
        intersection
    }

    fn fix_index(x: u8, y: u8) -> Option<usize> {
        if x >= WIDTH as u8 {
            None
        } else {
            Some((x as usize / 8) + (y as usize * WIDTH / 8))
        }
    }

    pub fn extract_buffer(&self) -> ScreenBuffer {
        self.0.clone()
    }

    pub fn clear(&mut self) {
        self.0 = Rc::new(RefCell::new(Self::CLEAR));
    }

    pub fn default_buffer() -> ScreenBuffer {
        Rc::new(RefCell::new(Self::CLEAR))
    }
}
