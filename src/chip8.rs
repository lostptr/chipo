use std::fs::File;
use std::io;
use std::io::prelude::*;

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
        let offset = 0x200;
        for i in 0..program_data.len() {
            self.cpu.write(offset + i, program_data[i]);
        }
        Ok(())
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
}
