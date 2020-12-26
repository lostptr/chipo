use crate::keyboard::Keyboard;
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
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Cpu {
    /// CHIP-8 has 4K memory
    pub memory: [u8; 4096],

    /// Opcodes are two bytes
    pub opcode: u16,

    /// CPU Registers; there are 16, 1 byte, registers.
    /// From V0 to VF
    pub v: [u8; 16],

    /// Index register 'I'
    pub i: u16,

    /// Program Counter (PC)
    pub pc: u16,

    /// Screen of 64x32, pixels have only one color.
    pub screen: [u32; SCREEN_WIDTH * SCREEN_HEIGHT],

    /// These two timers work the same way.
    /// Counted at 60 Hz. When set above zero, they count down to zero.
    pub delay_timer: u8,
    pub sound_timer: u8, // Makes a buzz sound when reaches zero.

    /// Call stack
    pub stack: Vec<u16>,

    /// Keyboard with 16 keys.
    ///
    /// For simplicity I'm using a bool slice but I could use
    /// a single u16 and check the key presses with bitwise operations.
    pub keys: [bool; 16],

    pub draw_flag: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Cpu {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START,

            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT],

            delay_timer: 0,
            sound_timer: 0,

            stack: vec![],

            keys: [false; 16],
            opcode: 0,

            draw_flag: false,
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

        self.draw_flag = false;

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => panic!("0x0: Unrecognized opcode {:#X}", opcode),
            },
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
            0x8000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                match opcode & 0x000F {
                    0x0000 => self.op_8xy0(x, y),
                    0x0001 => self.op_8xy1(x, y),
                    0x0002 => self.op_8xy2(x, y),
                    0x0003 => self.op_8xy3(x, y),
                    0x0004 => self.op_8xy4(x, y),
                    0x0005 => self.op_8xy5(x, y),
                    0x0006 => self.op_8xy6(x, y),
                    0x0007 => self.op_8xy7(x, y),
                    0x000E => self.op_8xye(x, y),
                    _ => panic!("0x8: Unrecognized opcode {:#X}", opcode),
                }
            }
            0x9000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                self.op_9xy0(x, y);
            }
            0xA000 => {
                let value = opcode & 0x0FFF;
                self.op_annn(value);
            }
            0xD000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                let nibble = (opcode & 0x000F) as u8;
                self.op_dxyn(x, y, nibble);
            }
            0xE000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                match opcode & 0x00FF {
                    0x009E => self.op_ex9e(x),
                    0x00A1 => self.op_exa1(x),
                    _ => panic!("0xE: Unrecognized opcode {:#X}", opcode),
                }
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
                    _ => panic!("0xF: Unrecognized opcode {:#X}", opcode),
                }
            }
            _ => panic!("Unrecognized opcode {:#X}", opcode),
        }

        println!("{:#?}", self);
    }

    /// ## 0x00E0
    /// Clears the screen.
    fn op_00e0(&mut self) {
        for pixel in self.screen.iter_mut() {
            *pixel = 0;
        }
        self.draw_flag = true;
    }

    /// ## 0x00EE
    /// Returns from subroutine.
    fn op_00ee(&mut self) {
        if let Some(value) = self.stack.pop() {
            self.pc = value;
        } else {
            panic!("Tried to pop the stack but it is empty!");
        }
    }

    /// ## 0x1NNN
    /// Jumps to address NNN (does not increment stack).
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    /// ## 0x2NNN
    /// Calls subroutine on address NNN and increments the stack.
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack.push(self.pc);
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

    /// ## 0x8XY0
    /// Sets VX to the value of VY
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
        self.inc_pc();
    }

    /// ## 0x8XY1
    /// Sets VX to (VX 'OR' VY)
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[x] | self.v[y];
        self.inc_pc();
    }

    /// ## 0x8XY2
    /// Sets VX to (VX 'AND' VY)
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[x] & self.v[y];
        self.inc_pc();
    }

    /// ## 0x8XY3
    /// Sets VX to (VX 'XOR' VY)
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[x] ^ self.v[y];
        self.inc_pc();
    }

    /// ## 0x8XY4
    /// Sets VX = VX + VY, VF = carry flag
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let sum: u16 = self.v[x] as u16 + self.v[y] as u16;

        if sum > 255 {
            self.v[0xF] = 1;
        } else {
            self.v[0xf] = 0;
        }

        self.v[x] = (sum & 0x00FF) as u8;
    }

    /// ## 0x8XY5
    /// Sets VX = VX - VY, VF = not borrow flag
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let diff: i16 = self.v[x] as i16 - self.v[y] as i16;

        if self.v[x] > self.v[y] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        // Unsure about this!
        self.v[x] = diff.abs() as u8;
    }

    /// ## 0x8XY6
    /// Set VX = VX SHIFT RIGHT 1, VF = the least significant bit.
    fn op_8xy6(&mut self, x: usize, _y: usize) {
        let least_bit = self.v[x] & 0b0000_0001;

        if least_bit == 0 {
            self.v[0xF] = 0;
        } else {
            self.v[0xF] = 1;
        }

        self.v[x] = self.v[x] >> 1;
    }

    /// ## 0x8XY7
    /// Set VX = VY - VX. VF = not borrow flag.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let diff: i16 = self.v[y] as i16 - self.v[x] as i16;

        if self.v[y] > self.v[x] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        // Unsure about this!
        self.v[x] = diff.abs() as u8;
    }

    /// ## 0x8XYE
    /// Set VX = VX SHIFT LEFT 1, VF = the most significant bit.
    fn op_8xye(&mut self, x: usize, _y: usize) {
        let most_bit = self.v[x] & 0b1000_0000;

        if most_bit == 0 {
            self.v[0xF] = 0;
        } else {
            self.v[0xF] = 1;
        }

        self.v[x] = self.v[x] << 1;
    }

    /// ## 0x9XY0
    /// Skip next instruction if VX != VY
    fn op_9xy0(&mut self, x: usize, y:usize) {
        if self.v[x] != self.v[y] {
            self.inc_pc();
        }
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
    fn op_dxyn(&mut self, x: usize, y: usize, height: u8) {
        let x_pos = self.v[x] % (SCREEN_WIDTH as u8);
        let y_pos = self.v[y] % (SCREEN_HEIGHT as u8);

        // Set pixel collision false.
        self.v[0xF] = 0;

        for row in 0..height {
            let mut pixel = self.read(self.i + (row as u16));

            // Width is 8 bytes
            for col in 0..8 {
                if self.set_screen_pixel(x_pos + col, y_pos + row, (pixel & 0b1000_0000) >> 7) {
                    self.v[0xF] = 1; // There was pixel colision.
                }
                pixel = pixel << 1;
            }
        }

        self.draw_flag = true;
        self.inc_pc();
    }

    fn set_screen_pixel(&mut self, x: u8, y: u8, value: u8) -> bool {
        let old = self.screen[x as usize + (y as usize) * SCREEN_WIDTH];

        if value > 0 {
            self.screen[x as usize + (y as usize) * SCREEN_WIDTH] ^= 0xFFFFFF;
        } else {
            self.screen[x as usize + (y as usize) * SCREEN_WIDTH] ^= 0x000000;
        }

        self.screen[x as usize + (y as usize) * SCREEN_WIDTH] != old
    }

    /// ## 0xEX9E
    /// Skips the next instruction if the key in VX is pressed.
    fn op_ex9e(&mut self, x: usize) {
        let key = self.keys[self.v[x] as usize];
        if key {
            self.inc_pc();
        }
        self.inc_pc();
    }

    /// ## 0xEXA1
    /// Skips the next instruction if the key in VX is NOT pressed.
    fn op_exa1(&mut self, x: usize) {
        let key = self.keys[self.v[x] as usize];
        if !key {
            self.inc_pc();
        }
        self.inc_pc();
    }

    /// ## 0xFX07
    /// ???
    fn op_fx07(&mut self, _x: usize) {
        todo!();
    }

    /// ## 0xFX0A
    /// ???
    fn op_fx0a(&mut self, _x: usize) {
        todo!();
    }

    /// ## 0xFX15
    /// ???
    fn op_fx15(&mut self, _x: usize) {
        todo!();
    }

    /// ## 0xFX18
    /// ???
    fn op_fx18(&mut self, _x: usize) {
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
    fn op_fx29(&mut self, _x: usize) {
        todo!();
    }

    /// ## 0xFX33
    /// ???
    fn op_fx33(&mut self, _x: usize) {
        todo!();
    }

    /// ## 0xFX55
    /// Stores the bytes from V0 to VX(inclusive) into memory starting from the address stored in I.
    fn op_fx55(&mut self, x: usize) {
        for offset in 0..x + 1 {
            self.write(self.i + offset as u16, self.v[offset]);
        }
        self.inc_pc();
    }

    /// ## 0xFX65
    /// Fills V0 to VX(inclusive) with bytes starting from the address stored in I.
    fn op_fx65(&mut self, x: usize) {
        for offset in 0..x + 1 {
            self.v[offset] = self.read(self.i + offset as u16);
        }
        self.inc_pc();
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
        write!(f, "\nStack: {:?}\n", self.stack)?;

        Ok(())
    }
}
