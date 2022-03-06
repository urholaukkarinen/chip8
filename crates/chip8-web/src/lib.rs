#![deny(clippy::all)]
#![allow(clippy::unused_unit, clippy::new_without_default)]

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

pub const PONG: &[u8] = include_bytes!("../pong.rom");

#[wasm_bindgen]
pub struct Chip8 {
    chip8: chip8_lib::Chip8,
    key_mapping: HashMap<u32, u8>,
}

#[wasm_bindgen]
impl Chip8 {
    pub fn new() -> Self {
        let mut key_mapping: HashMap<u32, u8> = HashMap::new();
        key_mapping.insert(48, 0);
        key_mapping.insert(49, 1);
        key_mapping.insert(50, 2);
        key_mapping.insert(51, 3);
        key_mapping.insert(52, 4);
        key_mapping.insert(53, 5);
        key_mapping.insert(54, 6);
        key_mapping.insert(55, 7);
        key_mapping.insert(56, 8);
        key_mapping.insert(57, 9);
        key_mapping.insert(97, 0xA);
        key_mapping.insert(98, 0xB);
        key_mapping.insert(99, 0xC);
        key_mapping.insert(100, 0xD);
        key_mapping.insert(101, 0xE);
        key_mapping.insert(102, 0xF);

        let mut chip8 = chip8_lib::Chip8::new();
        chip8.load_rom_from_slice(PONG).unwrap();

        Self { chip8, key_mapping }
    }

    pub fn load(&mut self, rom: &[u8]) -> Result<(), JsValue> {
        self.chip8
            .load_rom_from_slice(rom)
            .map_err(JsValue::from_str)
    }

    pub fn set_pressed_key(&mut self, key: u32, pressed: bool) {
        if let Some(key) = self.key_mapping.get(&key) {
            self.chip8.set_pressed_key(*key, pressed);
        }
    }

    pub fn tick(&mut self) {
        self.chip8.tick();
    }

    pub fn display_data(&self) -> *const u8 {
        self.chip8.display().as_ptr()
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_log::init().unwrap();
    console_error_panic_hook::set_once();

    Ok(())
}
