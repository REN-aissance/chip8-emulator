#[cfg(feature = "kb_trace")]
use std::fmt::Write;

#[derive(Clone, Debug, Default)]
pub struct Keyboard {
    pressed_keys: [bool; 16],
    last_pressed: u8,
}

impl Keyboard {
    pub fn is_pressed(&self, key: usize) -> bool {
        #[cfg(feature = "kb_trace")]
        println!(
            "{}",
            self.pressed_keys.iter().fold(String::new(), |mut acc, &x| {
                write!(acc, "{}", x as u8).unwrap();
                acc
            })
        );
        self.pressed_keys
            .get(key)
            .copied()
            .expect("Attempted to access invalid key")
    }

    pub fn press_key(&mut self, key: usize) {
        if let Some(state) = self.pressed_keys.get_mut(key) {
            *state = true;
            self.last_pressed = key as u8;
        } else {
            panic!("Attempted to access invalid key");
        }
    }

    pub fn release_key(&mut self, key: usize) {
        if let Some(state) = self.pressed_keys.get_mut(key) {
            *state = false;
            self.last_pressed = key as u8;
        } else {
            panic!("Attempted to access invalid key");
        }
    }

    pub fn last_pressed(&self) -> u8 {
        self.last_pressed
    }
}
