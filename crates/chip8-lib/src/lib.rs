#![allow(non_snake_case)]

use instant::Instant;
use log::error;
#[cfg(not(target_arch = "wasm32"))]
use std::{io::Read, path::Path};

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_BYTES: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

const STACK_SIZE: usize = 16;
const MEM_SIZE: usize = 4096;
const ROM_START: usize = 0x200;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    mem: [u8; MEM_SIZE],
    display: [u8; DISPLAY_BYTES],
    pc: usize,
    stack: [usize; STACK_SIZE],
    sp: usize,

    I: u16,
    V: [u8; 16],

    sound_timer: f32,
    delay_timer: f32,
    pressed_keys: [bool; 16],
    last_pressed_key: Option<u8>,

    last_time: Instant,
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8::new()
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let mut mem = [0; 4096];
        (&mut mem[0..FONT.len()]).copy_from_slice(&FONT);

        Self {
            mem,
            display: [0; DISPLAY_BYTES],
            pc: ROM_START,
            stack: [0; STACK_SIZE],
            sp: 0,
            I: 0,
            V: [0; 16],
            sound_timer: 0.0,
            delay_timer: 0.0,
            pressed_keys: [false; 16],
            last_pressed_key: None,
            last_time: Instant::now(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_rom_from_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        self.reset();

        std::fs::File::open(path).and_then(|mut file| file.read(&mut self.mem[ROM_START..]))?;
        Ok(())
    }

    pub fn load_rom_from_slice(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        if bytes.len() > self.mem.len() - ROM_START {
            return Err("The provided slice was too large");
        }
        self.reset();
        self.mem[ROM_START..ROM_START + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.pc = ROM_START;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.display = [0; DISPLAY_BYTES];
        self.last_pressed_key = None;
        self.delay_timer = 0.0;
        self.sound_timer = 0.0;
        self.last_time = Instant::now();
        self.I = 0;
        self.V = [0; 16];
    }

    fn next_op(&mut self) -> u32 {
        let op = ((self.mem[self.pc] as u32) << 8) | self.mem[self.pc + 1] as u32;
        self.pc += 2;
        op
    }

    pub fn set_pressed_key(&mut self, key: u8, pressed: bool) {
        if key <= 0xF {
            if pressed && !self.pressed_keys[key as usize] {
                self.last_pressed_key = Some(key);
            }
            self.pressed_keys[key as usize] = pressed;
        }
    }

    pub fn tick(&mut self) {
        let op = self.next_op();

        match [
            (op >> 12) & 0xF,
            (op >> 8) & 0xF,
            (op >> 4) & 0xF,
            op & 0xF, //
        ] {
            // 00E0
            [0x0, 0x0, 0xE, 0x0] => {
                self.display = [0; DISPLAY_BYTES];
            }

            // 00EE
            [0x0, 0x0, 0xE, 0xE] => {
                self.pc = self.stack[self.sp - 1];
                self.sp -= 1;
            }

            // 1NNN
            [0x1, ..] => {
                self.pc = op as usize & 0xFFF;
            }

            // 2NNN
            [0x2, ..] => {
                self.sp += 1;
                self.stack[self.sp - 1] = self.pc;
                self.pc = op as usize & 0xFFF;
            }

            // 3XKK
            [0x3, x, ..] => {
                if self.V[x as usize] == (op & 0xFF) as u8 {
                    self.pc += 2;
                }
            }

            // 4XKK
            [0x4, x, ..] => {
                if self.V[x as usize] != (op & 0xFF) as u8 {
                    self.pc += 2;
                }
            }

            // 5XY0
            [0x5, x, y, 0x0] => {
                if self.V[x as usize] == self.V[y as usize] {
                    self.pc += 2;
                }
            }

            // 6XKK
            [0x6, x, ..] => {
                self.V[x as usize] = (op & 0xFF) as u8;
            }

            // 7XKK
            [0x7, x, ..] => {
                self.V[x as usize] = self.V[x as usize].wrapping_add((op & 0xFF) as u8);
            }

            // 8XY0
            [0x8, x, y, 0x0] => {
                self.V[x as usize] = self.V[y as usize];
            }

            // 8XY1
            [0x8, x, y, 0x1] => {
                self.V[x as usize] |= self.V[y as usize];
            }

            // 8XY2
            [0x8, x, y, 0x2] => {
                self.V[x as usize] &= self.V[y as usize];
            }

            // 8XY3
            [0x8, x, y, 0x3] => {
                self.V[x as usize] ^= self.V[y as usize];
            }

            // 8XY4
            [0x8, x, y, 0x4] => {
                let sum = self.V[x as usize].wrapping_add(self.V[y as usize]);
                self.V[0xF] = (sum < self.V[x as usize]) as u8;
                self.V[x as usize] = sum;
            }

            // 8XY5
            [0x8, x, y, 0x5] => {
                self.V[0xF] = (self.V[x as usize] > self.V[y as usize]) as u8;
                self.V[x as usize] = self.V[x as usize].wrapping_sub(self.V[y as usize]);
            }

            // 8XY6
            [0x8, x, _, 0x6] => {
                self.V[0xF] = self.V[x as usize] & 0x1;
                self.V[x as usize] >>= 0x1;
            }

            // 8XY7
            [0x8, x, y, 0x7] => {
                self.V[0xF] = (self.V[y as usize] > self.V[x as usize]) as u8;
                self.V[x as usize] = self.V[y as usize].wrapping_sub(self.V[x as usize]);
            }

            // 8XYE
            [0x8, x, _, 0xE] => {
                self.V[0xF] = self.V[x as usize] >> 0x7;
                self.V[x as usize] <<= 0x1;
            }

            // 9XY0
            [0x9, x, y, 0x0] => {
                if self.V[x as usize] != self.V[y as usize] {
                    self.pc += 2;
                }
            }

            // ANNN
            [0xA, ..] => {
                self.I = (op & 0xFFF) as u16;
            }

            // BNNN
            [0xB, ..] => {
                self.pc = (op as usize & 0xFFF) + self.V[0x0] as usize;
            }

            // CXKK
            [0xC, x, ..] => {
                self.V[x as usize] = rand::random::<u8>() & ((op & 0xFF) as u8);
            }

            // DXYN
            [0xD, x, y, n] => {
                let start_x = self.V[x as usize] as usize;
                let start_y = self.V[y as usize] as usize;

                let mem_start = self.I as usize;
                let mem_end = mem_start + n as usize;
                let sprite_mem = &self.mem[mem_start..mem_end];

                let mut any_flipped = 0;

                for y2 in 0..n as usize {
                    let sprite_row = sprite_mem[y2 as usize];

                    for x2 in 0..8 {
                        let sprite_pixel = ((sprite_row >> (7 - x2)) & 0x1) * 0xFF;

                        if sprite_pixel > 0 {
                            let frame_y = (start_y + y2) % DISPLAY_HEIGHT;
                            let frame_x = (start_x + x2) % DISPLAY_WIDTH;
                            let frame_idx = frame_y * DISPLAY_WIDTH + frame_x;

                            if self.display[frame_idx] != 0 {
                                any_flipped = 1;
                            }

                            self.display[frame_idx] ^= sprite_pixel;
                        }
                    }
                }
                self.V[0xF] = any_flipped;
            }

            // Ex9E
            [0xE, x, 0x9, 0xE] => {
                if self.pressed_keys[self.V[x as usize] as usize] {
                    self.pc += 2;
                }
            }

            // EXA1
            [0xE, x, 0xA, 0x1] => {
                if !self.pressed_keys[self.V[x as usize] as usize] {
                    self.pc += 2;
                }
            }

            // FX07
            [0xF, x, 0x0, 0x7] => {
                self.V[x as usize] = (self.delay_timer as u32 & 0xFF) as u8;
            }

            // FX0A
            [0xF, x, 0x0, 0xA] => match self.last_pressed_key {
                Some(key) => {
                    self.V[x as usize] = key;
                }
                None => {
                    self.pc -= 2;
                }
            },

            // FX15
            [0xF, x, 0x1, 0x5] => {
                self.delay_timer = self.V[x as usize] as f32;
            }

            // FX18
            [0xF, x, 0x1, 0x8] => {
                self.sound_timer = self.V[x as usize] as f32;
            }

            // FX1E
            [0xF, x, 0x1, 0xE] => {
                self.I = (self.I + self.V[x as usize] as u16) & 0xFFF;
            }

            // FX29
            [0xF, x, 0x2, 0x9] => {
                self.I = (self.V[x as usize] * 5) as u16 & 0xFFF;
            }

            // FX33
            [0xF, x, 0x3, 0x3] => {
                let vx = self.V[x as usize];
                self.mem[self.I as usize] = vx / 100;
                self.mem[self.I as usize + 1] = (vx / 10) % 10;
                self.mem[self.I as usize + 2] = vx % 10;
            }

            // FX55
            [0xF, x, 0x5, 0x5] => {
                for i in 0..=x as usize {
                    self.mem[self.I as usize + i] = self.V[i];
                }
            }

            // FX65
            [0xF, x, 0x6, 0x5] => {
                for i in 0..=x as usize {
                    self.V[i] = self.mem[self.I as usize + i];
                }
            }

            _ => {
                error!("Unknown op: {:04X}", op)
            }
        }

        let now = Instant::now();
        self.delay_timer -= f32::min(
            self.delay_timer,
            now.duration_since(self.last_time).as_secs_f32() * 60.0,
        );
        self.sound_timer -= f32::min(
            self.sound_timer,
            now.duration_since(self.last_time).as_secs_f32() * 60.0,
        );
        self.last_time = now;
        self.last_pressed_key = None;
    }

    pub fn display(&self) -> &[u8] {
        &self.display
    }

    pub fn sound_timer(&self) -> u8 {
        self.sound_timer as u8
    }
}
