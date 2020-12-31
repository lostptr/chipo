extern crate minifb;

use minifb::{Key, ScaleMode, Window, WindowOptions};
use std::fs::File;
use std::io;
use std::io::prelude::*;

use crate::cpu::{Cpu, PROGRAM_START, SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Chip8 {
    cpu: Cpu,
    window: Window,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut window = Window::new(
            "CHIPO",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            WindowOptions {
                resize: true,
                scale: minifb::Scale::X8,
                scale_mode: ScaleMode::AspectRatioStretch,
                ..WindowOptions::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Chip8 {
            window,
            cpu: Cpu::new(),
        }
    }

    pub fn load_rom(&mut self, path: &str) -> io::Result<()> {
        let program_data = Chip8::load_rom_file(path)?;
        for i in 0..program_data.len() {
            self.cpu.write(PROGRAM_START + (i as u16), program_data[i]);
        }
        Ok(())
    }

    pub fn run_cycle(&mut self) {
        self.cpu.run_instruction();

        if self.cpu.draw_flag {
            self.update_window();
        }

        self.store_key_press();
    }

    pub fn update_window(&mut self) {
        self.window
            .update_with_buffer(&self.cpu.screen, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }

    pub fn store_key_press(&mut self) {
        self.cpu.keys = [false; 16];
        if let Some(keys) = self.window.get_keys() {
            for key in keys {
                if let Some(chip8_key) = Chip8::get_chip8_key_code(key) {
                    self.cpu.keys[chip8_key as usize] = true;
                }
            }
        }
    }

    fn load_rom_file(path: &str) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;
        let mut buffer = vec![];
        let bytes_read = file.read_to_end(&mut buffer)?;

        println!("Loaded '{}' ({} bytes read)", path, bytes_read);

        Ok(buffer)
    }

    pub fn is_running(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    /// This is what maps the chip8 keys to the keyboard.
    fn get_chip8_key_code(key: minifb::Key) -> Option<u8>{
        match key {
            Key::Key1 => Some(0x1),
            Key::Key2 => Some(0x2),
            Key::Key3 => Some(0x3),
            Key::Key4 => Some(0xC),
            Key::Q => Some(0x4),
            Key::W => Some(0x5),
            Key::E => Some(0x6),
            Key::R => Some(0xD),
            Key::A => Some(0x7),
            Key::S => Some(0x8),
            Key::D => Some(0x9),
            Key::F => Some(0xE),
            Key::Z => Some(0xA),
            Key::X => Some(0x0),
            Key::C => Some(0xB),
            Key::V => Some(0xF),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn read_rom() {
        let mut chip8 = Chip8::new();
        let rom = chip8.load_rom("roms/invaders");
        match rom {
            Err(e) => panic!("Rom did not load."),
            _ => (),
        }
    }

    #[test]
    fn run_first_cycle() {
        let mut chip8 = Chip8::new();
        let rom = chip8.load_rom("roms/invaders");
        match rom {
            Err(e) => panic!("Rom did not load."),
            _ => {
                chip8.run_cycle(); // Run one cycle once.
            }
        }
    }

    #[test]
    fn opcodes(){
        
        let mut chip8 = Chip8::new();
        chip8.load_rom("roms/test_opcode.ch8").unwrap();

        while chip8.is_running() {
            chip8.run_cycle();
        }

    }
}
