use bytemuck::{Pod, Zeroable};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub type ScreenBuffer = [[Tile; WIDTH]; HEIGHT];

#[derive(Debug, Clone)]
pub struct Screen {
    buffer: ScreenBuffer,
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            buffer: [[Tile::default(); WIDTH]; HEIGHT],
        }
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, p: Tile) {
        self.buffer[y as usize][x as usize] = p;
    }
    pub fn read_pixel(&self, x: u8, y: u8) -> Tile {
        self.buffer[y as usize][x as usize]
    }
    pub fn extract(&self) -> ScreenBuffer {
        self.buffer
    }
}

#[repr(align(4))]
#[repr(u32)]
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Tile {
    #[default]
    Off = 0,
    On,
    Text(u8),
}
unsafe impl Zeroable for Tile {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl Pod for Tile {}
