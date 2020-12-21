use std::fs::File;
use std::io;
use std::io::prelude::*;

use crate::cpu::PROGRAM_START;
use crate::cpu::Cpu;

pub struct Chip8 {
    pub cpu: Cpu,
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 { cpu: Cpu::new() }
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
    }

    fn load_rom_file(path: &str) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;
        let mut buffer = vec![];
        let bytes_read = file.read_to_end(&mut buffer)?;

        println!("Loaded '{}' ({} bytes read)", path, bytes_read);

        Ok(buffer)
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
