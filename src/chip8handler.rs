use std::{
    env, fs,
    io::{BufReader, Read},
};
use winit::event_loop::EventLoopProxy;

use crate::chip8::{event::Chip8Event, screen::ScreenBuffer, Chip8, ENTRY_POINT};

const CPU_IPF: u32 = 15;
const FF_IPF: u32 = CPU_IPF * 16;
const MAX_FILESIZE: u64 = 0x1000 - ENTRY_POINT as u64;

pub struct Chip8Handler {
    ipf: u32,
    cpu: Chip8,
    sys_tx: EventLoopProxy<Chip8Event>,
    ff: bool,
}

impl Chip8Handler {
    pub fn new(sys_tx: EventLoopProxy<Chip8Event>) -> Chip8Handler {
        Chip8Handler {
            ipf: CPU_IPF,
            cpu: Chip8::new().with_rom(&Self::read_rom_from_fs()),
            sys_tx,
            ff: false,
        }
    }

    pub fn update(&mut self) {
        for _ in 0..self.ipf {
            if let Some(Chip8Event::RequestRedraw) = self.cpu.update()
                //Ignore cpu-requested redraw events while in fast-forward
                    && !self.ff
            {
                self.sys_tx.send_event(Chip8Event::RequestRedraw).unwrap();
                break;
            }
        }
        self.cpu.update_timers();
    }

    pub fn update_key(&mut self, key: u8, state: bool) {
        self.cpu.set_key(key, state)
    }

    pub fn start_ff(&mut self) {
        self.ff = true;
        self.ipf = FF_IPF;
    }

    pub fn stop_ff(&mut self) {
        self.ff = false;
        self.ipf = CPU_IPF;
    }

    pub fn get_frame_buffer(&self) -> ScreenBuffer {
        self.cpu.get_display_buffer()
    }

    pub fn reset(&mut self) {
        self.cpu = Chip8::new().with_rom(&Self::read_rom_from_fs());
        self.sys_tx.send_event(Chip8Event::RequestRedraw).unwrap();
    }

    fn read_rom_from_fs() -> Vec<u8> {
        let rom_path = env::args().nth(1).expect("Please provide a path to rom");
        let rom = fs::File::open(rom_path).expect("Cannot open rom, does it exist in the path?");
        let rom_metadata = rom.metadata().expect("Cannot access file metadata");
        if rom_metadata.len() > MAX_FILESIZE {
            panic!("File too large to be a Chip-8 rom");
        }
        let mut buffer: Vec<u8> = Vec::with_capacity(MAX_FILESIZE as usize);
        let mut reader = BufReader::new(rom);
        let read_bytes = reader
            .read_to_end(&mut buffer)
            .expect("Unsuccessfully read rom");
        if read_bytes != rom_metadata.len() as usize {
            panic!("File length does not match metadata");
        }
        buffer
    }
}
