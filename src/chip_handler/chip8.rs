use crate::chip_handler::event::Chip8Event;

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
pub enum CPUError {
    UnknownOpcode(u16, u16),
    RamOutOfBounds,
}

impl fmt::Display for CPUError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CPUError::UnknownOpcode(i, pc) => write!(f,
                "ERROR: Unknown opcode 0x{:04X} at 0x{:04X}\nPerhaps program counter ran into working memory?",
                i, pc
            ),
            CPUError::RamOutOfBounds => write!(f, "ERROR: Ran out of RAM"),
        }
    }
}
impl std::error::Error for CPUError {}

const ENTRY_POINT: u16 = 0x200;

pub struct Chip8 {
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

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut ram = [0x00_u8; 0x1000];
        TEXT_SPRITES
            .iter()
            .flatten()
            .enumerate()
            .for_each(|(i, &b)| {
                ram[i] = b;
            });

        Chip8 {
            screen: Screen::default(),
            buzzer: Buzzer::new(),
            stack: Stack::default(),
            kb: Keyboard::default(),
            ram,
            reg: [0x00; 16],
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
        let [ub, lb] = i.to_be_bytes();
        let x = (ub & 0x0F) as usize;
        let y = (lb >> 4) as usize;
        let n = lb & 0x0F;
        let vx = *self.reg.get(x).unwrap();
        let vy = *self.reg.get(y).unwrap();
        match i {
            //00E0 CLS
            0x00E0 => self.screen.clear(),
            //00EE RET
            0x00EE => self.pc = self.stack.pop(),
            //1nnn JP addr
            0x1000..=0x1FFF => {
                self.pc = i & 0x0FFF;
                return Ok(Chip8Event::DoNotIncrementPC);
            }
            //2nnn CALL addr
            0x2000..=0x2FFF => {
                self.stack.push(self.pc);
                self.pc = i & 0x0FFF;
                return Ok(Chip8Event::DoNotIncrementPC);
            }
            //3nnn SE Vx, byte
            0x3000..=0x3FFF => {
                if vx == lb {
                    return Ok(Chip8Event::SkipNextInstruction);
                }
            }
            //4nnn SNE Vx, byte
            0x4000..=0x4FFF => {
                if vx != lb {
                    return Ok(Chip8Event::SkipNextInstruction);
                }
            }
            //5xy0 SE
            0x5000..=0x5FF0 if i & 0x000F == 0 => {
                if vx == vy {
                    return Ok(Chip8Event::SkipNextInstruction);
                }
            }
            //6nnn LD Vx, byte
            0x6000..=0x6FFF => self.reg[x] = lb,
            //7nnn ADD Vx, byte
            0x7000..=0x7FFF => self.reg[x] = vx.wrapping_add(lb),
            //8
            0x8000..=0x8FFF => match n {
                //8xy0 LD Vx, Vy
                0x00 => self.reg[x] = vy,
                //8xy1 OR Vx, Vy
                0x01 => {
                    self.reg[x] |= vy;
                    self.reg[0xF] = 0;
                }
                //8xy2 AND Vx, Vy
                0x02 => {
                    self.reg[x] &= vy;
                    self.reg[0xF] = 0;
                }
                //8xy3 XOR Vx, Vy
                0x03 => {
                    self.reg[x] ^= vy;
                    self.reg[0xF] = 0;
                }
                //8xy4 ADD Vx, Vy
                0x04 => {
                    let (vx, carry) = vx.overflowing_add(vy);
                    self.reg[x] = vx;
                    self.reg[0xF] = carry as u8;
                }
                //8xy5 SUB Vx, Vy
                0x05 => {
                    let (vx, borrow) = vx.overflowing_sub(vy);
                    self.reg[x] = vx;
                    self.reg[0xF] = !borrow as u8;
                }
                //8xy6 SHR Vx {, Vy}
                0x06 => {
                    let out_bit = (vy & 0x01 == 1) as u8;
                    self.reg[x] = vy >> 1;
                    self.reg[0xF] = out_bit;
                }
                //8xy7 SUBN Vx, Vy
                0x07 => {
                    let (vx, borrow) = vy.overflowing_sub(vx);
                    self.reg[x] = vx;
                    self.reg[0xF] = !borrow as u8;
                }
                //8xyE SHL Vx {, Vy}
                0x0E => {
                    let out_bit = (vy & 0x80 == 0x80) as u8;
                    self.reg[x] = vy << 1;
                    self.reg[0xF] = out_bit;
                }
                _ => return Err(CPUError::UnknownOpcode(i, self.pc).into()),
            },
            //9xy0 SNE Vx, Vy
            0x9000..=0x9FF0 if i & 0xF == 0 => {
                if vx != vy {
                    return Ok(Chip8Event::SkipNextInstruction);
                }
            }
            //Annn LD I, addr
            0xA000..=0xAFFF => self.i = i & 0x0FFF,
            //Bnnn JP V0, addr
            0xB000..=0xBFFF => {
                self.pc = (i & 0x0FFF) + self.reg[0x0] as u16;
                return Ok(Chip8Event::DoNotIncrementPC);
            }
            //Cnnn RND Vx, byte
            0xC000..=0xCFFF => {
                let r = rand::thread_rng().gen_range(0x00..=0xFF);
                self.reg[x] = r & lb;
            }
            //Dxyn DRW Vx, Vy, n
            0xD000..=0xDFFF => {
                let i = self.i as usize;
                let sprite = &self.ram[i..(i + n as usize)];
                self.reg[0xF] = self.screen.print_sprite(sprite, vx, vy) as u8;
                return Ok(Chip8Event::RequestRedraw);
            }
            //E
            0xE000..=0xEFFF => match lb {
                //Ex9E SKP Vx
                0x9E => {
                    // #[cfg(feature = "kb_debug")]
                    // println!("Checking for key press {:X}", x);
                    if self.kb.is_pressed(vx as usize) {
                        return Ok(Chip8Event::SkipNextInstruction);
                    };
                }
                //ExA1 SKNP Vx
                0xA1 => {
                    // #[cfg(feature = "kb_debug")]
                    // println!("Checking key not pressed {:X}", x);
                    if !self.kb.is_pressed(vx as usize) {
                        return Ok(Chip8Event::SkipNextInstruction);
                    };
                }
                _ => return Err(CPUError::RamOutOfBounds.into()),
            },
            //F
            0xF000..=0xFFFF => match lb {
                //Fx07 LD Vx, DT
                0x07 => self.reg[x] = self.dt,
                //Fx0A LD Vx, K
                0x0A => {
                    #[cfg(feature = "kb_debug")]
                    println!("Halting for key press {:X}", x);
                    return Ok(Chip8Event::KBHaltOnBuffer(x));
                }
                //Fx15 LD DT, Vx
                0x15 => self.dt = vx,
                //Fx18 LD ST, Vx
                0x18 => {
                    #[cfg(feature = "sound_debug")]
                    println!("Received opcode to play sound");
                    self.buzzer.play(Duration::from_secs_f32(vx as f32 / 60.0));
                    self.st = vx;
                }
                //Fx1E ADD I, Vx
                0x1E => self.i = self.i.wrapping_add(vx as u16),
                //Fx29 LD F, Vx
                0x29 => self.i = x as u16 * 5,
                //Fx33 LD B, Vx
                0x33 => {
                    let i = self.i as usize;
                    if let Ok([b0, b1, b2]) = self.ram.get_many_mut([i, i + 1, i + 2]) {
                        *b0 = vx / 100;
                        *b1 = vx / 10 % 10;
                        *b2 = vx % 10;
                    } else {
                        return Err(CPUError::RamOutOfBounds.into());
                    }
                }
                //Fx55 LD [I], Vx
                0x55 => {
                    (0..=x)
                        .flat_map(|x| self.reg.get(x))
                        .copied()
                        .enumerate()
                        .for_each(|(i, vx)| {
                            self.ram[self.i as usize + i] = vx;
                        });
                    self.i += (x + 1) as u16;
                }
                //Fx65 LD Vx, [I]
                0x65 => {
                    (0..=x)
                        .filter_map(|i| self.ram.get(self.i as usize + i))
                        .copied()
                        .enumerate()
                        .for_each(|(i, vx)| {
                            self.reg[i] = vx;
                        });
                    self.i += (x + 1) as u16;
                }
                _ => return Err(CPUError::UnknownOpcode(i, self.pc).into()),
            },
            //Treat blank memory as NOP
            0x0000 => return Ok(Chip8Event::DoNotIncrementPC),
            _ => {
                return Err(CPUError::UnknownOpcode(i, self.pc).into());
            }
        }
        Ok(Chip8Event::IncrementPC)
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
                    Chip8Event::IncrementPC => self.increment_pc(),
                    Chip8Event::KBHaltOnBuffer(x) => {
                        self.kb_halt_reg = Some(x);
                        self.increment_pc()
                    }
                    Chip8Event::Shutdown => return Some(e),
                    Chip8Event::RequestRedraw => {
                        self.increment_pc();
                        return Some(e);
                    }
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                    return Some(Chip8Event::Shutdown);
                }
            }
        } else {
            eprintln!("{}", CPUError::RamOutOfBounds);
            return Some(Chip8Event::Shutdown);
        }
        None
    }

    pub fn decrement_timers(&mut self) {
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

        if self.kb_halt_reg.is_some() {
            self.buzzer.play(Duration::from_millis(200));
        }
    }

    fn release_key(&mut self, key: u8) {
        self.kb.release_key(key as usize);
        #[cfg(feature = "kb_debug")]
        if self.kb_halt_reg.is_some() {
            println!("Unhalted");
        }

        if let Some(x) = self.kb_halt_reg {
            self.reg[x] = self.kb.last_pressed();
            self.kb_halt_reg = None;
        }
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
impl fmt::Debug for Chip8 {
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
