use super::screen::ScreenBuffer;

#[derive(Clone, Debug)]
pub enum Chip8Event {
    None,
    RequestRedraw(Box<ScreenBuffer>),
    SkipNextInstruction,
    DoNotIncrementPC,
    Shutdown,
}

unsafe impl Sync for Chip8Event {}
unsafe impl Send for Chip8Event {}
