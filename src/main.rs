use std::io;

use chipo::chip8::Chip8;

fn main() -> io::Result<()> {
        
    // let opcode: u16 = 0x6A55;
    // let x: usize = ((opcode & 0x0F00) >> 8) as usize;
    // let nn: u8 = (opcode & 0x00FF) as u8;
    // println!("{:#X} --> x:{} | nn: {:#X}", opcode, x, nn);
    
    
    let mut chip8 = Chip8::new();
    chip8.load_rom("roms/invaders")?;

    loop {
        chip8.run_cycle();
    }
}
