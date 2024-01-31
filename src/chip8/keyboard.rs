#[derive(Clone, Debug, Default)]
pub struct Keyboard {
    pressed_keys: [bool; 16],
    last_pressed: u8,
}

impl Keyboard {
    pub fn is_pressed(&self, key: usize) -> bool {
        self.pressed_keys
            .get(key)
            .copied()
            .expect("Attempted to access invalid key")
    }

    pub fn press_key(&mut self, key: usize) {
        #[cfg(feature = "kb_debug")]
        eprintln!("Key pressed! {:0X}", key);
        *self
            .pressed_keys
            .get_mut(key)
            .expect("Attempted to access invalid key") = true;
        self.last_pressed = key as u8;
    }

    pub fn release_key(&mut self, key: usize) {
        #[cfg(feature = "kb_debug")]
        eprintln!("Key released! {:0X}", key);
        self.pressed_keys
            .get(key)
            .expect("Attempted to access invalid key");
    }

    pub fn last_pressed(&self) -> u8 {
        self.last_pressed
    }
}
