#[derive(Clone, Debug, Default)]
pub struct Keyboard {
    pressed_keys: [bool; 16],
    last_pressed: u8,
}

impl Keyboard {
    pub fn is_pressed(&self, key: usize) -> bool {
        self.pressed_keys[key]
    }

    pub fn press_key(&mut self, key: usize) {
        #[cfg(feature = "kb_debug")]
        eprintln!("Key pressed! {:0X}", key);
        self.pressed_keys[key] = true;
        self.last_pressed = key as u8;
    }

    pub fn release_key(&mut self, key: usize) {
        #[cfg(feature = "kb_debug")]
        eprintln!("Key released! {:0X}", key);
        self.pressed_keys[key] = false;
    }

    pub fn last_pressed(&self) -> u8 {
        self.last_pressed
    }
}
