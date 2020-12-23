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
    }

    pub fn update_window(&mut self){
        self.window
            .update_with_buffer(&self.cpu.screen, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
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
            },
        }
    }
}
