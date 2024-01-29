use std::error::Error;
use std::time::{Duration, Instant};

use rand::Rng;
use winit::event_loop::EventLoopProxy;

use self::buzzer::Buzzer;
use self::screen::Screen;

mod buzzer;
pub(crate) mod screen;
mod square_wave;

#[derive(Debug, Copy, Clone)]
pub struct Chip8Error;
impl std::fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chip8Error")
    }
}
impl Error for Chip8Error {}

#[derive(Clone, Debug)]
pub enum Chip8Event {
    RequestRedraw(Vec<u8>),
    KeyEvent(u8, bool),
    Update,
}
unsafe impl Sync for Chip8Event {}
unsafe impl Send for Chip8Event {}

const ENTRY_POINT: u16 = 0x200;

pub struct Chip8 {
    pub screen: Screen,
    pub buzzer: Buzzer,
    pub pressed_keys: [bool; 16],
    event_sender: EventLoopProxy<Chip8Event>,
    halt: bool,
    halt_register: usize,
    ram: [u8; 4096],
    stack: [u16; 16],
    registers: [u8; 16],
    dt: u8,
    st: u8,
    i: u16,
    pc: u16,
    sp: u8,
}

impl Chip8 {
    pub fn new(ep: EventLoopProxy<Chip8Event>) -> Chip8 {
        const TEXT_SPRITES: [[u8; 5]; 16] = [
            [0xF0, 0x90, 0x90, 0x90, 0xF0], //0
            [0x20, 0x60, 0x20, 0x20, 0x70], //1
            [0xF0, 0x10, 0xF0, 0x80, 0xF0], //2
            [0xF0, 0x10, 0xF0, 0x10, 0xF0], //3
            [0x90, 0x90, 0xF0, 0x10, 0x10], //4
            [0xF0, 0x80, 0xF0, 0x10, 0xF0], //5
            [0xF0, 0x80, 0xF0, 0x90, 0xF0], //6
            [0xF0, 0x10, 0x20, 0x40, 0x40], //7
            [0xF0, 0x90, 0xF0, 0x90, 0xF0], //8
            [0xF0, 0x90, 0xF0, 0x10, 0xF0], //9
            [0xF0, 0x90, 0xF0, 0x90, 0x90], //A
            [0xE0, 0x90, 0xE0, 0x90, 0xE0], //B
            [0xF0, 0x80, 0x80, 0x80, 0xF0], //C
            [0xE0, 0x90, 0x90, 0x90, 0xE0], //D
            [0xF0, 0x80, 0xF0, 0x80, 0xF0], //E
            [0xF0, 0x80, 0xF0, 0x80, 0x80], //F
        ];
        let mut ram = [0_u8; 4096];
        TEXT_SPRITES
            .iter()
            .flatten()
            .enumerate()
            .for_each(|(i, &b)| {
                ram[i] = b;
            });
        // ram[0x200] = 0x60;
        // ram[0x201] = 0xC0;

        // ram[0x202] = 0x61;
        // ram[0x203] = 0xC0;

        // // ram[0x204] = 0x62;
        // // ram[0x205] = 0x03;

        // // ram[0x206] = 0x63;
        // // ram[0x207] = 0x03;

        // // ram[0x208] = 0x64;
        // // ram[0x209] = 0x03;

        // // ram[0x20A] = 0x65;
        // // ram[0x20B] = 0x03;

        // // ram[0x20C] = 0xA9;
        // // ram[0x20D] = 0x00;

        // //Copy to i
        // ram[0x20E] = 0xF6;
        // ram[0x20F] = 0x55;

        // //Reset x pos
        // ram[0x210] = 0x60;
        // ram[0x211] = 0x3F;

        // //Reset y pos
        // ram[0x212] = 0x61;
        // ram[0x213] = 0x1F;

        // //print
        // ram[0x214] = 0xD0;
        // ram[0x215] = 0x12;

        // //loop
        // ram[0x216] = 0x12;
        // ram[0x217] = 0x14;

        Chip8 {
            screen: Screen::default(),
            buzzer: Buzzer::new(),
            event_sender: ep,
            ram,
            stack: [0; 16],
            registers: [0; 16],
            halt: false,
            halt_register: 0,
            dt: 0,
            st: 0,
            i: 0,
            pc: ENTRY_POINT,
            sp: 0,
            pressed_keys: [false; 16],
        }
    }

    pub fn execute_instruction(&mut self, i: u16) -> Result<(), Box<dyn Error>> {
        let [b, kk] = i.to_be_bytes();
        let x = (b & 0x0F) as usize;
        let y = (kk >> 4) as usize;
        let n = (kk & 0x0F) as usize;
        let vx = self.registers.get(x).ok_or(Chip8Error).copied();
        let vy = self.registers.get(y).ok_or(Chip8Error).copied();
        match i {
            //CLS
            0x00E0 => self.screen.clear(),
            //RET
            0x00EE => {
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            //JP addr
            0x1000..=0x1FFF => self.pc = (i & 0x0FFF) - 2,
            //CALL addr
            0x2000..=0x2FFF => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = i & 0x0FFF;
            }
            //SE Vx, byte
            0x3000..=0x3FFF => self.pc += if vx? == kk { 2 } else { 0 },
            //SNE Vx, byte
            0x4000..=0x4FFF => self.pc += if vx? != kk { 2 } else { 0 },
            //SE
            0x5000..=0x5FF0 if i & 0x000F == 0 => self.pc += if vx? == vy? { 2 } else { 0 },
            //LD Vx, byte
            0x6000..=0x6FFF => self.registers[x] = kk,
            //ADD Vx, byte
            0x7000..=0x7FFF => self.registers[x] = vx?.wrapping_add(kk),
            //LD Vx, Vy
            0x8000..=0x8FF0 if i & 0x000F == 0 => self.registers[x] = vy?,
            //OR Vx, Vy
            0x8000..=0x8FF1 if i & 0x000F == 1 => self.registers[x] |= vy?,
            //AND Vx, Vy
            0x8000..=0x8FF2 if i & 0x000F == 2 => self.registers[x] &= vy?,
            //XOR Vx, Vy
            0x8000..=0x8FF3 if i & 0x000F == 3 => self.registers[x] ^= vy?,
            //ADD Vx, Vy
            0x8000..=0x8FF4 if i & 0x000F == 4 => {
                let (vx, carry) = vx?.overflowing_add(vy?);
                self.registers[x] = vx;
                self.registers[0xF] = carry as u8;
            }
            //SUB Vx, Vy
            0x8000..=0x8FF5 if i & 0x000F == 5 => {
                self.registers[0xF] = (vx? > vy?) as u8;
                self.registers[x] = vx?.wrapping_sub(vy?);
            }
            //SHR Vx {, Vy}
            0x8000..=0x8FF6 if i & 0x000F == 6 => {
                self.registers[0xF] = (vx? & 0x01 == 1) as u8;
                self.registers[x] >>= 1;
            }
            //SUBN Vx, Vy
            0x8000..=0x8FF7 if i & 0x000F == 7 => {
                self.registers[0xF] = (vy? > vx?) as u8;
                self.registers[x] = vy?.wrapping_sub(vx?);
            }
            //SHL Vx {, Vy}
            0x8000..=0x8FFE if i & 0x000F == 0xE => {
                self.registers[0xF] = (vx? & 0x80 == 0x80) as u8;
                self.registers[x] <<= 1;
            }
            //SNE Vx, Vy
            0x9000..=0x9FF0 if i & 0x000F == 0 => self.sp += if vx? != vy? { 2 } else { 0 },
            //LD I, addr
            0xA000..=0xAFFF => self.i = i & 0x0FFF,
            //JP V0, addr
            0xB000..=0xBFFF => self.pc = (i & 0x0FFF) + self.registers[0x0] as u16,
            //RND Vx, byte
            0xC000..=0xCFFF => {
                let r = rand::thread_rng().gen_range(0x00..=0xFF);
                self.registers[x] = r & kk;
            }
            //DRW Vx, Vy, n
            0xD000..=0xDFFF => {
                let i = self.i as usize;
                let sprite = &self.ram[i..(i + n)];
                self.screen.print_sprite(sprite, vx?, vy?);
                self.screen_update();
            }
            //SKP Vx
            0xE09E..=0xEF9E if i & 0x00FF == 0x9E => {
                // println!("Checking for key press");
                self.pc += if self.pressed_keys[x] { 2 } else { 0 };
            }
            //SKNP Vx
            0xE0A1..=0xEFA1 if i & 0x00FF == 0xA1 => {
                // println!("Checking key not pressed");
                self.pc += if !self.pressed_keys[x] { 2 } else { 0 };
            }
            //LD Vx, DT
            0xF007..=0xFF07 if i & 0x00FF == 0x07 => self.registers[x] = self.dt,
            //LD Vx, K
            0xF00A..=0xFF0A if i & 0x00FF == 0x0A => {
                #[cfg(debug_assertions)]
                println!("Halting for key press");
                self.halt_register = x;
                self.halt = true;
            }
            //LD DT, Vx
            0xF00A..=0xFF15 if i & 0x00FF == 0x15 => self.dt = vx?,
            //LD ST, Vx
            0xF00A..=0xFF18 if i & 0x00FF == 0x18 => {
                println!("TRYING TO PLAY SOUND");
                self.buzzer.play(Duration::from_secs_f32(vx? as f32 / 60.0));
                self.st = vx?;
            }
            //ADD I, Vx
            0xF01E..=0xFF1E if i & 0x00FF == 0x1E => self.i = self.i.wrapping_add(vx? as u16),
            //LD F, Vx
            0xF029..=0xFF29 if i & 0x00FF == 0x29 => self.i = x as u16 * 5,
            //LD B, Vx
            0xF033..=0xFF33 if i & 0x00FF == 0x33 => {
                self.ram[self.i as usize] = vx? / 100;
                self.ram[(self.i + 1) as usize] = vx? / 10 % 10;
                self.ram[(self.i + 2) as usize] = vx? % 10;
            }
            //LD [I], Vx
            0xF055..=0xFF55 if i & 0x00FF == 0x55 => {
                (0..=x)
                    .map(|x| self.registers[x])
                    .enumerate()
                    .map(|(i, vx)| (self.i as usize + i, vx))
                    .for_each(|(i, vx)| {
                        self.ram[i] = vx;
                    });
                self.i += (x + 1) as u16;
            }
            //LD Vx, [I]
            0xF065..=0xFF65 if i & 0x00FF == 0x65 => {
                (0..=x)
                    .map(|i| self.ram[self.i as usize + i])
                    .enumerate()
                    .for_each(|(i, vx)| {
                        self.registers[i] = vx;
                    });
                self.i += (x + 1) as u16;
            }
            //SYS addr [IGNORE]
            0x0000..=0x0FFF => (),
            _ => {
                eprintln!("Unknown opcode: {:04x}", i);
                return Err(Chip8Error.into());
            }
        }
        #[cfg(debug_assertions)]
        if i & 0x0FFF != 0 {
            println!("{:04X}", i);
        }
        Ok(())
    }

    pub fn get_display_buffer(&self) -> Vec<u8> {
        Vec::from(self.screen.as_bytes())
    }

    pub fn screen_update(&mut self) {
        self.dt = self.dt.saturating_sub(1);
        self.st = self.st.saturating_sub(1);
        self.event_sender
            .send_event(Chip8Event::RequestRedraw(self.get_display_buffer()))
            .unwrap();
    }

    pub fn update(&mut self, event: Chip8Event) {
        let t = Instant::now();
        while !self.halt
            && t.elapsed() < Duration::from_secs(1)
            && let Some(b1) = self.ram.get(self.pc as usize).map(|&e| e as u16)
            && let Some(b2) = self.ram.get((self.pc + 1) as usize).map(|&e| e as u16)
        {
            if let Chip8Event::KeyEvent(key, state) = event {
                self.set_key(key, state);
            }
            let i = b1 << 8 | b2;
            self.execute_instruction(i).unwrap();
            self.pc += 2;
            if i & 0xF000 == 0xD000 {
                break;
            }
        }
    }

    pub fn sound_test(&self) {
        self.buzzer.play(Duration::from_millis(200));
    }

    pub fn set_key(&mut self, key: u8, state: bool) {
        match state {
            true => self.press_key(key),
            false => self.release_key(key),
        }
    }

    fn press_key(&mut self, key: u8) {
        self.pressed_keys[key as usize] = true;
        self.registers[self.halt_register] = key;
        self.halt = false;
    }

    fn release_key(&mut self, key: u8) {
        self.pressed_keys[key as usize] = false;
    }

    pub fn load_rom_from_bytes(&mut self, bytes: &[u8]) {
        bytes.iter().enumerate().for_each(|(i, &b)| {
            self.ram[ENTRY_POINT as usize + i] = b;
        })
    }
}
