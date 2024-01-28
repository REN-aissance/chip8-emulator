use std::time::Duration;

use rand::rngs::ThreadRng;
use rand::Rng;

use crate::{HEIGHT, WIDTH};

use self::buzzer::Buzzer;
use self::screen::Screen;

mod buzzer;
pub(crate) mod screen;

pub struct Chip8 {
    pub screen: Screen,
    pub buzzer: Buzzer,
    pub pressed_keys: [bool; 16],
    awaiting_key: bool,
    rng: ThreadRng,
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
    pub fn new() -> Chip8 {
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
        // ram[0x200] = 0xF0;
        // ram[0x201] = 0x29;
        // ram[0x202] = 0xD0;
        // ram[0x203] = 0x05;

        Chip8 {
            screen: Screen::default(),
            buzzer: Buzzer::new(),
            rng: rand::thread_rng(),
            ram,
            stack: [0; 16],
            registers: [0; 16],
            dt: 0,
            st: 0,
            i: 0,
            pc: 0x200,
            sp: 0,
            pressed_keys: [false; 16],
            awaiting_key: false,
        }
    }

    pub fn execute_instruction(&mut self, i: u16) {
        match i {
            //CLS
            0x00E0 => self.screen.clear(),
            //RET
            0x00EE => {
                self.pc = self.stack[self.sp as usize];
                self.sp = self.sp.saturating_sub(1);
            }
            //JP addr
            0x1000..=0x1FFF => self.pc = i & 0x0FFF,
            //CALL addr
            0x2000..=0x2FFF => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = i & 0x0FFF;
            }
            //SE Vx, byte
            0x3000..=0x3FFF => {
                let [b1, kk] = i.to_be_bytes();
                let vx = self.registers[(b1 & 0x0F) as usize];
                if vx == kk {
                    self.pc += 2;
                }
            }
            //SNE Vx, byte
            0x4000..=0x4FFF => {
                let [b1, kk] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let vx = self.registers[x as usize];
                if vx != kk {
                    self.pc += 2;
                }
            }
            //SE
            0x5000..=0x5FF0 if i & 0x000F == 0 => {
                let [b1, b2] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let y = b2 >> 4;
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                if vx == vy {
                    self.pc += 2;
                }
            }
            //LD Vx, byte
            0x6000..=0x6FFF => {
                let [b1, kk] = i.to_be_bytes();
                let x = b1 & 0x0F;
                self.registers[x as usize] = kk;
            }
            //ADD Vx, byte
            0x7000..=0x7FFF => {
                let [b1, kk] = i.to_be_bytes();
                let x = (b1 & 0x0F) as usize;
                self.registers[x] = self.registers[x].wrapping_add(kk);
            }
            //LD Vx, Vy
            0x8000..=0x8FF0 if i & 0x000F == 0 => {
                let [b1, b2] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let y = b2 >> 4;
                self.registers[x as usize] = self.registers[y as usize];
            }
            //OR Vx, Vy
            0x8000..=0x8FF1 if i & 0x000F == 1 => {
                let [b1, b2] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let y = b2 >> 4;
                self.registers[x as usize] |= self.registers[y as usize];
            }
            //AND Vx, Vy
            0x8000..=0x8FF2 if i & 0x000F == 2 => {
                let [b1, b2] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let y = b2 >> 4;
                self.registers[x as usize] &= self.registers[y as usize];
            }
            //XOR Vx, Vy
            0x8000..=0x8FF3 if i & 0x000F == 3 => {
                let [b1, b2] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let y = b2 >> 4;
                self.registers[x as usize] ^= self.registers[y as usize];
            }
            //ADD Vx, Vy
            0x8000..=0x8FF4 if i & 0x000F == 4 => {
                let [b1, b2] = i.to_be_bytes();
                let x = (b1 & 0x0F) as usize;
                let y = (b2 >> 4) as usize;
                self.registers[x] = self.registers[x].wrapping_add(self.registers[y]);
            }
            //SUB Vx, Vy
            0x8000..=0x8FF5 if i & 0x000F == 5 => {
                let [b1, b2] = i.to_be_bytes();
                let x = (b1 & 0x0F) as usize;
                let y = (b2 >> 4) as usize;
                let vx = self.registers[x];
                let vy = self.registers[y];
                self.registers[0xF] = if vx > vy { 1 } else { 0 };
                self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
            }
            //SHR Vx {, Vy}
            0x8000..=0x8FF6 if i & 0x000F == 6 => {
                let [b1, _] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let vx = self.registers[x as usize];
                self.registers[0xF] = if vx & 0x01 == 1 { 1 } else { 0 };
                self.registers[x as usize] >>= 1;
            }
            //SUBN Vx, Vy
            0x8000..=0x8FF7 if i & 0x000F == 7 => {
                let [b1, b2] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let y = b2 >> 4;
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                self.registers[0xF] = (vy > vx) as u8;
                self.registers[x as usize] = vy.wrapping_sub(vx);
            }
            //SHL Vx {, Vy}
            0x8000..=0x8FFE if i & 0x000F == 0xE => {
                let [b1, _] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let vx = self.registers[x as usize];
                self.registers[0xF] = if vx & 0x80 == 0x80 { 1 } else { 0 };
                self.registers[x as usize] <<= 1;
            }
            //SNE Vx, Vy
            0x9000..=0x9FF0 if i & 0x000F == 0 => {
                let [b1, b2] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let y = b2 >> 4;
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                if vx != vy {
                    self.sp += 2;
                }
            }
            //LD I, addr
            0xA000..=0xAFFF => self.i = i & 0x0FFF,
            //JP V0, addr
            0xB000..=0xBFFF => self.pc = (i & 0x0FFF) + self.registers[0x0] as u16,
            //RND Vx, byte
            0xC000..=0xCFFF => {
                let [b1, kk] = i.to_be_bytes();
                let x = b1 & 0x0F;
                let r = self.rng.gen_range(0x00..=0xFF);
                self.registers[x as usize] = r & kk;
            }
            //DRW Vx, Vy, n
            0xD000..=0xDFFF => {
                let [b1, b2] = i.to_be_bytes();
                let x = (b1 & 0x0F) as usize;
                let y = (b2 >> 4) as usize;
                let vx = self.registers[x];
                let vy = self.registers[y];
                let n = (b2 & 0x0F) as usize;
                let sprite = &self.ram[(self.i as usize)..(self.i as usize + n)];
                self.screen.print_sprite(sprite, vx, vy);
            }
            //SKP Vx
            0xE09E..=0xEF9E if i & 0x00FF == 0x9E => {
                let [b1, _] = i.to_be_bytes();
                let x = b1 & 0x0F;
                if self.pressed_keys[x as usize] {
                    self.pc += 2;
                }
            }
            //SKNP Vx
            0xE0A1..=0xEFA1 if i & 0x00FF == 0xA1 => {
                let [b1, _] = i.to_be_bytes();
                let x = b1 & 0x0F;
                if !self.pressed_keys[x as usize] {
                    self.pc += 2;
                }
            }
            //LD Vx, DT
            0xF007..=0xFF07 if i & 0x00FF == 0x07 => {
                let [b1, _] = i.to_be_bytes();
                let x = b1 & 0x0F;
                self.registers[x as usize] = self.dt;
            }
            //LD Vx, K
            0xF00A..=0xFF0A if i & 0x00FF == 0x0A => self.awaiting_key = true,
            //LD DT, Vx
            0xF00A..=0xFF15 if i & 0x00FF == 0x15 => {
                self.dt = self.registers[(i.to_be_bytes()[0] & 0x0F) as usize]
            }
            //LD ST, Vx
            0xF00A..=0xFF18 if i & 0x00FF == 0x18 => {
                let x = (i.to_be_bytes()[0] & 0x0F) as usize;
                let vx = self.registers[x];
                self.buzzer.play(Duration::from_secs_f32(vx as f32 / 60.0));
                self.st = vx;
            }
            //ADD I, Vx
            0xF01E..=0xFF1E if i & 0x00FF == 0x1E => {
                let x = (i.to_be_bytes()[0] & 0x0F) as usize;
                let vx = self.registers[x];
                self.i = self.i.wrapping_add(vx as u16);
            }
            //LD F, Vx
            0xF029..=0xFF29 if i & 0x00FF == 0x29 => {
                let x = (i.to_be_bytes()[0] & 0x0F) as u16;
                self.i = x * 5;
            }
            //LD B, Vx
            0xF033..=0xFF33 if i & 0x00FF == 0x33 => {
                let x = (i.to_be_bytes()[0] & 0x0F) as usize;
                let vx = self.registers[x];
                self.ram[self.i as usize] = vx / 100;
                self.ram[(self.i + 1) as usize] = vx / 10 % 10;
                self.ram[(self.i + 2) as usize] = vx % 10;
            }
            //LD [I], Vx
            0xF055..=0xFF55 if i & 0x00FF == 0x55 => {
                let x = (i.to_be_bytes()[0] & 0x0F) as usize;
                (0..=x)
                    .map(|x| self.registers[x])
                    .enumerate()
                    .map(|(i, vx)| (self.i as usize + i, vx))
                    .for_each(|(i, vx)| {
                        self.ram[i] = vx;
                    });
            }
            //LD Vx, [I]
            0xF065..=0xFF65 if i & 0x00FF == 0x65 => {
                let x = (i.to_be_bytes()[0] & 0x0F) as usize;
                (0..=x)
                    .map(|i| self.ram[self.i as usize + i])
                    .enumerate()
                    .for_each(|(i, vx)| {
                        self.registers[i] = vx;
                    });
            }
            //SYS addr [IGNORE]
            0x0000..=0x0FFF => (),
            _ => panic!("Unknown opcode: {:04x}", i),
        }
        // println!("{:04x}", i);
    }

    pub fn get_display_buffer(&self) -> [u8; WIDTH / 8 * HEIGHT] {
        self.screen.as_bytes()
    }

    pub fn update(&mut self) {
        self.dt = self.dt.saturating_sub(1);
        self.st = self.st.saturating_sub(1);
        if !self.awaiting_key {
            let b1 = *self.ram.get(self.pc as usize).unwrap() as u16;
            let b2 = *self.ram.get((self.pc + 1) as usize).unwrap() as u16;
            self.execute_instruction(b1 << 8 | b2);
            self.pc += 2;
        }
    }

    pub fn set_key(&mut self, key: u8, state: bool) {
        match state {
            true => self.press_key(key),
            false => self.release_key(key),
        }
    }

    fn press_key(&mut self, key: u8) {
        self.pressed_keys[key as usize] = true;
        self.awaiting_key = false;
    }

    fn release_key(&mut self, key: u8) {
        self.pressed_keys[key as usize] = false;
    }

    pub fn load_rom_from_bytes(&mut self, bytes: &[u8]) {
        bytes.iter().enumerate().for_each(|(i, &b)| {
            self.ram[0x200 + i] = b;
        })
    }
}
