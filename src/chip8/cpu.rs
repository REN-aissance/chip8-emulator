use crate::chip8::event::Chip8Event;

use super::buzzer::Buzzer;
use super::keyboard::Keyboard;
use super::screen::{Screen, ScreenBuffer};
use super::stack::Stack;
use anyhow::Error;
use rand::Rng;
use std::fmt;
use std::time::Duration;

#[cfg(debug_assertions)]
use std::fmt::Write;

#[derive(Debug, Copy, Clone)]
pub struct CPUError;
impl fmt::Display for CPUError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chip8Error")
    }
}
impl std::error::Error for CPUError {}

const ENTRY_POINT: u16 = 0x200;
pub const CLK_SPEED_HZ: f32 = 500.0;

pub struct Cpu {
    screen: Screen,
    buzzer: Buzzer,
    kb: Keyboard,
    stack: Stack,
    kb_halt_reg: Option<usize>,
    ram: [u8; 4096],
    reg: [u8; 16],
    dt: u8,
    st: u8,
    i: u16,
    pc: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut ram = [0x01_u8; 0x1000];
        TEXT_SPRITES
            .iter()
            .flatten()
            .enumerate()
            .for_each(|(i, &b)| {
                ram[i] = b;
            });

        /*
        //Debug time
        //Write s1 to V0
        ram[0x200] = 0x60;
        ram[0x201] = 0xFF;
        //Write s2 to V1
        ram[0x202] = 0x61;
        ram[0x203] = 0xFF;
        //Write s3 to V2
        ram[0x204] = 0x62;
        ram[0x205] = 0x18;
        //Write s4 to V3
        ram[0x206] = 0x63;
        ram[0x207] = 0x18;
        //Set I to something
        ram[0x208] = 0xA5;
        ram[0x209] = 0x00;
        //Write Vx to [I]
        ram[0x20A] = 0xF4;
        ram[0x20B] = 0x55;
        //Write posx1 to V0;
        ram[0x20C] = 0x60;
        ram[0x20D] = 0x04;
        //Write posy1 to V1;
        ram[0x20E] = 0x61;
        ram[0x20F] = 0x00;
        //Write posx2 to V2;
        ram[0x210] = 0x62;
        ram[0x211] = 0x08;
        //Write posy2 to V3;
        ram[0x212] = 0x63;
        ram[0x213] = 0x01;
        //Reset I for sprite 1
        ram[0x214] = 0xA5;
        ram[0x215] = 0x00;
        //Draw sprite at I
        ram[0x216] = 0xD0;
        ram[0x217] = 0x12;
        //Reset I for sprite 2
        ram[0x218] = 0xA5;
        ram[0x219] = 0x02;
        //Draw sprite at I
        ram[0x21A] = 0xD2;
        ram[0x21B] = 0x32;
        //Loop
        ram[0x21C] = 0x12;
        ram[0x21D] = 0x1C;
        */

        Cpu {
            screen: Screen::default(),
            buzzer: Buzzer::new(),
            stack: Stack::default(),
            kb: Keyboard::default(),
            ram,
            reg: [0; 16],
            kb_halt_reg: None,
            dt: 0,
            st: 0,
            i: 0,
            pc: ENTRY_POINT,
        }
    }

    pub fn execute_instruction(&mut self, i: u16) -> Result<Chip8Event, Error> {
        #[cfg(feature = "trace")]
        println!("{:?}", self);
        let [b, kk] = i.to_be_bytes();
        let x = (b & 0x0F) as usize;
        let y = (kk >> 4) as usize;
        let n = (kk & 0x0F) as usize;
        let vx = *self.reg.get(x).unwrap();
        let vy = *self.reg.get(y).unwrap();
        match i {
            //CLS
            0x00E0 => self.screen.clear(),
            //RET
            0x00EE => self.pc = self.stack.pop(),
            //JP addr
            0x1000..=0x1FFF => {
                self.pc = i & 0x0FFF;
                return Ok(Chip8Event::DoNotIncrementPC);
            }
            //CALL addr
            0x2000..=0x2FFF => {
                self.stack.push(self.pc);
                self.pc = i & 0x0FFF;
                return Ok(Chip8Event::DoNotIncrementPC);
            }
            //SE Vx, byte
            0x3000..=0x3FFF => {
                if vx == kk {
                    return Ok(Chip8Event::SkipNextInstruction);
                }
            }
            //SNE Vx, byte
            0x4000..=0x4FFF => {
                if vx != kk {
                    return Ok(Chip8Event::SkipNextInstruction);
                }
            }
            //SE
            0x5000..=0x5FF0 if i & 0x000F == 0 => {
                if vx == vy {
                    return Ok(Chip8Event::SkipNextInstruction);
                }
            }
            //LD Vx, byte
            0x6000..=0x6FFF => self.reg[x] = kk,
            //ADD Vx, byte
            0x7000..=0x7FFF => self.reg[x] = vx.wrapping_add(kk),
            //LD Vx, Vy
            0x8000..=0x8FF0 if i & 0x000F == 0 => self.reg[x] = vy,
            //OR Vx, Vy
            0x8000..=0x8FF1 if i & 0x000F == 1 => self.reg[x] |= vy,
            //AND Vx, Vy
            0x8000..=0x8FF2 if i & 0x000F == 2 => self.reg[x] &= vy,
            //XOR Vx, Vy
            0x8000..=0x8FF3 if i & 0x000F == 3 => self.reg[x] ^= vy,
            //ADD Vx, Vy
            0x8000..=0x8FF4 if i & 0x000F == 4 => {
                let (vx, carry) = vx.overflowing_add(vy);
                self.reg[0xF] = !carry as u8;
                self.reg[x] = vx;
            }
            //SUB Vx, Vy
            0x8000..=0x8FF5 if i & 0x000F == 5 => {
                let (vx, borrow) = vy.overflowing_sub(vx);
                self.reg[0xF] = !borrow as u8;
                self.reg[x] = vx;
            }
            //SHR Vx {, Vy}
            0x8000..=0x8FF6 if i & 0x000F == 6 => {
                self.reg[0xF] = (vx & 0x01 == 1) as u8;
                self.reg[x] >>= 1;
            }
            //SUBN Vx, Vy
            0x8000..=0x8FF7 if i & 0x000F == 7 => {
                let (vx, borrow) = vx.overflowing_sub(vy);
                self.reg[0xF] = !borrow as u8;
                self.reg[x] = vx;
            }
            //SHL Vx {, Vy}
            0x8000..=0x8FFE if i & 0x000F == 0xE => {
                self.reg[0xF] = (vx & 0x80 == 0x80) as u8;
                self.reg[x] <<= 1;
            }
            //SNE Vx, Vy
            0x9000..=0x9FF0 if i & 0x000F == 0 => {
                if vx != vy {
                    return Ok(Chip8Event::SkipNextInstruction);
                }
            }
            //LD I, addr
            0xA000..=0xAFFF => self.i = i & 0x0FFF,
            //JP V0, addr
            0xB000..=0xBFFF => {
                self.pc = (i & 0x0FFF) + self.reg[0x0] as u16;
                return Ok(Chip8Event::DoNotIncrementPC);
            }
            //RND Vx, byte
            0xC000..=0xCFFF => {
                let r = rand::thread_rng().gen_range(0x00..=0xFF);
                self.reg[x] = r & kk;
            }
            //DRW Vx, Vy, n
            0xD000..=0xDFFF => {
                let i = self.i as usize;
                let sprite = &self.ram[i..(i + n)];
                self.reg[0xF] = self.screen.print_sprite(sprite, vx, vy) as u8;
                return Ok(Chip8Event::RequestRedraw(self.get_display_buffer().into()));
            }
            //SKP Vx
            0xE09E..=0xEF9E if i & 0x00FF == 0x9E => {
                #[cfg(feature = "kb_debug")]
                println!("Checking for key press {:X}", x);
                if self.kb.is_pressed(x) {
                    return Ok(Chip8Event::SkipNextInstruction);
                };
            }
            //SKNP Vx
            0xE0A1..=0xEFA1 if i & 0x00FF == 0xA1 => {
                #[cfg(feature = "kb_debug")]
                println!("Checking key not pressed {:X}", x);
                if !self.kb.is_pressed(x) {
                    return Ok(Chip8Event::SkipNextInstruction);
                };
            }
            //LD Vx, DT
            0xF007..=0xFF07 if i & 0x00FF == 0x07 => self.reg[x] = self.dt,
            //LD Vx, K
            0xF00A..=0xFF0A if i & 0x00FF == 0x0A => {
                #[cfg(feature = "kb_debug")]
                println!("Halting for key press {:X}", x);
                return Ok(Chip8Event::KBHaltOnBuffer(x));
            }
            //LD DT, Vx
            0xF00A..=0xFF15 if i & 0x00FF == 0x15 => self.dt = vx,
            //LD ST, Vx
            0xF00A..=0xFF18 if i & 0x00FF == 0x18 => {
                #[cfg(feature = "sound_debug")]
                println!("Received opcode to play sound");
                self.buzzer.play(Duration::from_secs_f32(vx as f32 / 60.0));
                self.st = vx;
            }
            //ADD I, Vx
            0xF01E..=0xFF1E if i & 0x00FF == 0x1E => self.i = self.i.wrapping_add(vx as u16),
            //LD F, Vx
            0xF029..=0xFF29 if i & 0x00FF == 0x29 => self.i = x as u16 * 5,
            //LD B, Vx
            0xF033..=0xFF33 if i & 0x00FF == 0x33 => {
                self.ram[self.i as usize] = vx / 100;
                self.ram[(self.i + 1) as usize] = vx / 10 % 10;
                self.ram[(self.i + 2) as usize] = vx % 10;
            }
            //LD [I], Vx
            0xF055..=0xFF55 if i & 0x00FF == 0x55 => {
                (0..=x)
                    .map(|x| self.reg[x])
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
                        self.reg[i] = vx;
                    });
                self.i += (x + 1) as u16;
            }
            //SYS addr [IGNORE]
            0x0000..=0x0FFF => (),
            _ => {
                eprintln!(
                    "ERROR: Unknown opcode 0x{:04X} at 0x{:04X}\n
                    perhaps program counter ran into working memory?",
                    i, self.pc
                );
                return Err(CPUError.into());
            }
        }
        Ok(Chip8Event::None)
    }

    pub fn get_display_buffer(&self) -> ScreenBuffer {
        self.screen.extract_buffer()
    }

    pub fn increment_pc(&mut self) {
        self.pc += 2;
    }

    pub fn update(&mut self) -> Option<Chip8Event> {
        if let Some(b1) = self.ram.get(self.pc as usize).map(|&e| e as u16)
            && let Some(b2) = self.ram.get((self.pc + 1) as usize).map(|&e| e as u16)
        {
            if self.kb_halt_reg.is_some() {
                return None;
            }
            let i = b1 << 8 | b2;
            match self.execute_instruction(i) {
                Ok(e) => match e {
                    Chip8Event::SkipNextInstruction => {
                        self.increment_pc();
                        self.increment_pc();
                    }
                    Chip8Event::DoNotIncrementPC => (),
                    Chip8Event::None => self.increment_pc(),
                    Chip8Event::RequestRedraw(_) => {
                        self.increment_pc();
                        return Some(e);
                    }
                    Chip8Event::KBHaltOnBuffer(x) => {
                        self.kb_halt_reg = Some(x);
                        self.increment_pc()
                    }
                    Chip8Event::Shutdown => return Some(e),
                },
                Err(_) => return Some(Chip8Event::Shutdown),
            }
        } else {
            eprintln!("ERROR: END OF MEMORY");
            return Some(Chip8Event::Shutdown);
        }
        None
    }

    pub fn update_timers(&mut self) {
        self.dt = self.dt.saturating_sub(1);
        self.st = self.st.saturating_sub(1);
    }

    pub fn set_key(&mut self, key: u8, state: bool) {
        match state {
            true => self.press_key(key),
            false => self.release_key(key),
        }
    }

    fn press_key(&mut self, key: u8) {
        self.kb.press_key(key as usize);
        #[cfg(feature = "kb_debug")]
        if self.kb_halt_reg.is_some() {
            println!("Unhalted");
        }

        if let Some(x) = self.kb_halt_reg {
            self.reg[x] = self.kb.last_pressed();
            self.kb_halt_reg = None;
        }
    }

    fn release_key(&mut self, key: u8) {
        self.kb.release_key(key as usize)
    }

    pub fn with_rom(mut self, bytes: &[u8]) -> Self {
        bytes.iter().enumerate().for_each(|(i, &b)| {
            self.ram[ENTRY_POINT as usize + i] = b;
        });
        self
    }
}

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

#[cfg(debug_assertions)]
impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PC:{:04X} STACK:{:?} REG:{} DT:{} OP:{:02X}{:02X}",
            self.pc,
            self.stack,
            self.reg
                .iter()
                .fold(String::new(), |mut acc, &e| {
                    write!(acc, "{:02X}|", e).unwrap();
                    acc
                })
                .trim_end_matches('|'),
            self.dt,
            self.ram[self.pc as usize],
            self.ram[(self.pc + 1) as usize]
        )
    }
}
