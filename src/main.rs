#![allow(warnings)]

extern crate minifb;

use std::io;

use chipo::chip8::Chip8;
//use minifb::{Key, ScaleMode, Window, WindowOptions};

fn main() -> io::Result<()> {
    // let opcode: u16 = 0x6A55;
    // let x: usize = ((opcode & 0x0F00) >> 8) as usize;
    // let nn: u8 = (opcode & 0x00FF) as u8;
    // println!("{:#X} --> x:{} | nn: {:#X}", opcode, x, nn);

    // let mut chip8 = Chip8::new();
    // chip8.load_rom("roms/invaders")?;

    // while chip8.is_running() {
    //     chip8.run_cycle();
    // }

    let a: u8 = 0b0111_0000;    

    let bit = a & 0b1000_0000;

    if bit == 0 {
        println!("bit is zero");
    } else {
        println!("bit is one");
    }
    

    println!("a: {:#010b}\nnew a: {:#010b}\n???: {:#010b}", a, a >> 1, a / 2);

    // let mut buffer: Vec<u32> = vec![0; 32 * 64];
    // let mut window = Window::new(
    //     "CHIPO",
    //     64,
    //     32,
    //     WindowOptions {
    //         resize: true,
    //         scale: minifb::Scale::X8,
    //         scale_mode: ScaleMode::AspectRatioStretch,
    //         ..WindowOptions::default()
    //     },
    // )
    // .unwrap_or_else(|e| {
    //     panic!("{}", e);
    // });

    // // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // let mut chip8_keys: [bool; 16];

    // while window.is_open() && !window.is_key_down(Key::Escape) {
    //     chip8_keys = [false; 16];
    //     if let Some(keys) = window.get_keys() {
    //         for key in keys {
    //             if let Some(chip8_key) = get_chip8_key_code(key) {
    //                 chip8_keys[chip8_key as usize] = true;
    //             }
    //         }
    //     }

    //     println!("keys: {:?}", chip8_keys);
    //     window.update_with_buffer(&buffer, 32, 32).unwrap();
    // }

    Ok(())
}
