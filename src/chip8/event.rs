#[derive(Clone, Debug)]
pub enum Chip8Event {
    IncrementPC,
    KBHaltOnBuffer(usize),
    RequestRedraw,
    SkipNextInstruction,
    DoNotIncrementPC,
}

unsafe impl Sync for Chip8Event {}
unsafe impl Send for Chip8Event {}
