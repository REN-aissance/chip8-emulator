use super::screen::ScreenBuffer;

#[derive(Clone, Debug)]
pub enum Chip8Event {
    None,
    RequestRedraw(Box<ScreenBuffer>),
    KBHaltOnBuffer(usize),
    SkipNextInstruction,
    DoNotIncrementPC,
    Shutdown,
}

unsafe impl Sync for Chip8Event {}
unsafe impl Send for Chip8Event {}
