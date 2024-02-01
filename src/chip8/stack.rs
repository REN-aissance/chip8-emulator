#[derive(Clone, Default)]
pub struct Stack {
    contents: [u16; 16],
    sp: usize,
}

impl Stack {
    pub fn push(&mut self, data: u16) {
        if self.sp == 16 {
            panic!("Chip-8 internal stack overflow");
        }
        *self.contents.get_mut(self.sp).unwrap() = data;
        self.sp += 1;
    }

    pub fn pop(&mut self) -> u16 {
        self.sp = self.sp.saturating_sub(1);
        self.contents[self.sp]
    }
}

#[cfg(debug_assertions)]
use std::fmt::Debug;
#[cfg(debug_assertions)]
impl Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:0X}|{:04X}",
            self.sp,
            self.contents[self.sp.saturating_sub(1)]
        )
    }
}
