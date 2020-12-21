use crate::screen::Screen;
use std::fmt;

/// Chip-8 has 16 sprites of 5 bytes (16 * 5 = 80)
///
/// They represent the hex digits of 0..F
const BUILT_IN_FONTSET: [u8; 80] = [
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

pub const PROGRAM_START: u16 = 0x200;

pub struct Cpu {
    // CHIP-8 has 4K memory
    pub memory: [u8; 4096],

    // Opcodes are two bytes
    pub opcode: u16,

    // CPU Registers; there are 16, 1 byte, registers.
    // From V0 to VF
    pub v: [u8; 16],

    // Index register 'I'
    pub i: u16,

    // Program Counter (PC)
    pub pc: u16,

    // Screen of 64x32, pixels have only one color.
    pub screen: [u8; 64 * 32],

    // These two timers work the same way.
    // Counted at 60 Hz. When set above zero, they count down to zero.
    pub delay_timer: u8,
    pub sound_timer: u8, // Makes a buzz sound when reaches zero.

    // Stack and stack pointer (sp)
    pub stack: [u16; 16],
    pub sp: usize,

    // Keyboard
    pub key: [u8; 16],
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Cpu {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START,

            screen: [0; 64 * 32],

            delay_timer: 0,
            sound_timer: 0,

            stack: [0; 16],
            sp: 0,

            key: [0; 16],
            opcode: 0,
        };

        // Place the font sprites int the interpreter area of the ram
        for i in 0..BUILT_IN_FONTSET.len() {
            cpu.memory[i] = BUILT_IN_FONTSET[i];
        }

        cpu
    }

    pub fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn run_instruction(&mut self) {
        // opcodes are 16-bit (must read and combine two bytes)
        let low = self.read(self.pc) as u16;
        let high = self.read(self.pc + 1) as u16;
        let opcode = (low << 8) | high; // Big-Endian

        //println!("instruction read: {:x?} (high: {:x?}, low: {:x?})", opcode, high, low);

        //println!("instruction {:#x?} --> {:#x?}", opcode, opcode & 0xF000);
        if high == 0 && low == 0 {
            panic!()
        }
        match opcode & 0xF000 {
            0x1000 => {
                let address = opcode & 0x0FFF;
                self.op_1nnn(address);
            }
            0x2000 => {
                let address = opcode & 0x0FFF;
                self.op_2nnn(address);
            }
            0x3000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x00FF) as u8;
                self.op_3xnn(x, value);
            }
            0x6000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x00FF) as u8;
                self.op_6xnn(x, value);
            }
            0x7000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x00FF) as u8;
                self.op_7xnn(x, value);
            }
            0xA000 => {
                let value = opcode & 0x0FFF;
                self.op_annn(value);
            }
            0xD000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                let nibble = (opcode & 0x000F) as u8;
                self.op_dxyn(x, y, nibble);
            }
            0xF000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                match opcode & 0x00FF {
                    0x0007 => self.op_fx07(x),
                    0x000A => self.op_fx0a(x),
                    0x0015 => self.op_fx15(x),
                    0x0018 => self.op_fx18(x),
                    0x001E => self.op_fx1e(x),
                    0x0029 => self.op_fx29(x),
                    0x0033 => self.op_fx33(x),
                    0x0055 => self.op_fx55(x),
                    0x0065 => self.op_fx65(x),
                    _ => panic!("Unrecognized opcode {:#X}", opcode),
                }
            }
            _ => panic!("Unrecognized opcode {:#X}", opcode),
        }

        println!("{:#?}", self);
    }

    /// ## 0x1NNN
    /// Jumps to address NNN (does not increment stack).
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    /// ## 0x2NNN
    /// Calls subroutine on address NNN and increments the stack.
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = nnn;
    }

    /// ## 0x3XNN
    /// Skips next instruction if VX equals NN.
    fn op_3xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.inc_pc();
        }
        self.inc_pc();
    }

    /// ## 0x6XNN
    /// Sets V[X] to NN
    fn op_6xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
        self.inc_pc();
    }

    /// ## 0x7XNN
    /// Adds NN to VX (Does not change carry flag)
    fn op_7xnn(&mut self, x: usize, nn: u8) {
        self.v[x] = self.v[x].wrapping_add(nn);
        self.inc_pc();
    }

    /// ## 0xANNN
    /// Sets I to NNN
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
        self.inc_pc();
    }

    /// ## 0xDXYN
    /// Draws to the screen and checks when there's pixel collision.
    fn op_dxyn(&mut self, x: u8, y: u8, height: u8) {
        println!("Drawing sprite at ({}, {})", x, y);
        for h in 0..height {
            let mut pixel = self.read(self.i + (h as u16));

            // Width is 8 bytes
            for _ in 0..8 {
                match (pixel & 0b1000_0000) >> 7 {
                    0 => print!("░"),
                    1 => print!("█"),
                    _ => unreachable!(),
                }
                pixel = pixel << 1;
            }
            print!("\n");
        }

        print!("\n");
        self.inc_pc();
    }

    /// ## 0xFX07
    /// ???
    fn op_fx07(&mut self, x: usize) {
        todo!();
    }

    /// ## 0xFX0A
    /// ???
    fn op_fx0a(&mut self, x: usize) {
        todo!();
    }

    /// ## 0xFX15
    /// ???
    fn op_fx15(&mut self, x: usize) {
        todo!();
    }

    /// ## 0xFX18
    /// ???
    fn op_fx18(&mut self, x: usize) {
        todo!();
    }

    /// ## 0xFX1E
    /// ???
    fn op_fx1e(&mut self, x: usize) {
        self.i += self.v[x] as u16;
        self.inc_pc();
    }

    /// ## 0xFX29
    /// ???
    fn op_fx29(&mut self, x: usize) {
        todo!();
    }

    /// ## 0xFX33
    /// ???
    fn op_fx33(&mut self, x: usize) {
        todo!();
    }

    /// ## 0xFX55
    /// ???
    fn op_fx55(&mut self, x: usize) {
        todo!();
    }

    /// ## 0xFX65
    /// ???
    fn op_fx65(&mut self, x: usize) {
        todo!();
    }

    /// Increments PC by 2
    fn inc_pc(&mut self) {
        self.pc += 2;
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "PC: {:#X} | I: {:#X}\n", self.pc, self.i)?;

        write!(f, "Registers: ")?;
        for vx in self.v.iter() {
            write!(f, "{:#X}|", vx)?;
        }
        write!(f, "\nStack: (sp {}) [", self.sp)?;
        for sp in 0..self.stack.len() {
            if sp == self.sp {
                write!(f, " *{:#X},", self.stack[sp])?;
            } else {
                write!(f, " {:#X},", self.stack[sp])?;
            }
        }
        write!(f, "]\n")?;

        Ok(())
    }
}
