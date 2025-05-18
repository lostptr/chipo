use rand::{prelude::ThreadRng, thread_rng, Rng};
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
const FONTSET_START_ADDRESS: u16 = 0x0;

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
    pub screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],

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

    rng: ThreadRng,

    // Used to get the correct bahaviour for FX0A.
    pressed_key_index: Option<usize>,
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
            rng: thread_rng(),
            pressed_key_index: None,
        };

        // Place the font sprites int the interpreter area of the ram
        let start_of_fontset = FONTSET_START_ADDRESS as usize;
        let end_of_fontset = start_of_fontset + BUILT_IN_FONTSET.len();
        for i in start_of_fontset..end_of_fontset {
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

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn get_screen_index(x: u8, y: u8) -> usize {
        ((usize::from(y) % SCREEN_HEIGHT) * SCREEN_WIDTH) + (usize::from(x) % SCREEN_WIDTH)
    }

    /// Draws on screen memory address.
    /// Returns `true` if there's pixel collision.
    fn set_screen_pixel(&mut self, x: u8, y: u8, value: u8) -> bool {
        let old = self.screen[Cpu::get_screen_index(x, y)];

        if value > 0 {
            self.screen[Cpu::get_screen_index(x, y)] ^= 0xFF;
        } else {
            self.screen[Cpu::get_screen_index(x, y)] ^= 0x0000;
        }

        self.screen[Cpu::get_screen_index(x, y)] != old
    }

    /// Increments PC by 2
    fn inc_pc(&mut self) {
        self.pc += 2;
    }

    pub fn run_instruction(&mut self) {
        // opcodes are 16-bit (must read and combine two bytes)
        let low = self.read(self.pc) as u16;
        let high = self.read(self.pc + 1) as u16;
        let opcode = (low << 8) | high; // Big-Endian

        if high == 0 && low == 0 {
            panic!("opcode is zero");
        }

        self.draw_flag = false;
        self.opcode = opcode;

        // println!("opcode {:#x}", opcode);

        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => println!("0x0: Ignoring unrecognized opcode {:#X}", opcode),
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
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x00FF) as u8;
                self.op_4xnn(x, value);
            }
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                self.op_5xy0(x, y);
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
            0xB000 => {
                let value = opcode & 0x0FFF;
                self.op_bnnn(value);
            }
            0xC000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let value = (opcode & 0x00FF) as u8;
                self.op_cxnn(x, value);
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
    }

    /// ## 0x00E0
    /// Clears the screen.
    fn op_00e0(&mut self) {
        for pixel in self.screen.iter_mut() {
            *pixel = 0;
        }
        self.draw_flag = true;

        self.inc_pc();
    }

    /// ## 0x00EE
    /// Returns from subroutine.
    fn op_00ee(&mut self) {
        if let Some(value) = self.stack.pop() {
            self.pc = value;
            self.inc_pc();
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

    /// ## 0x4XNN
    /// Skips next instruction if VX not equals NN.
    fn op_4xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.inc_pc();
        }
        self.inc_pc();
    }

    /// ## 0x5XY0
    /// Skips next instruction if VX equals VY.
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
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
        self.v[0xF] = 0; // original chip8 quirk: reset flag register to zero.
        self.v[x] = self.v[x] | self.v[y];
        self.inc_pc();
    }

    /// ## 0x8XY2
    /// Sets VX to (VX 'AND' VY)
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[0xF] = 0; // original chip8 quirk: reset flag register to zero.
        self.v[x] = self.v[x] & self.v[y];
        self.inc_pc();
    }

    /// ## 0x8XY3
    /// Sets VX to (VX 'XOR' VY)
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[0xF] = 0; // original chip8 quirk: reset flag register to zero.
        self.v[x] = self.v[x] ^ self.v[y];
        self.inc_pc();
    }

    /// ## 0x8XY4
    /// Sets VX = VX + VY, VF = carry flag
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let sum: u16 = self.v[x] as u16 + self.v[y] as u16;

        let carry_flag = if sum > 255 { 1 } else { 0 };

        self.v[x] = (sum & 0x00FF) as u8;
        self.v[0xF] = carry_flag;
        self.inc_pc();
    }

    /// ## 0x8XY5
    /// Sets VX = VX - VY, VF = not borrow flag
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let (diff, overflow) = self.v[x].overflowing_sub(self.v[y]);

        self.v[x] = diff;
        self.v[0xF] = if overflow { 0 } else { 1 };
        self.inc_pc();
    }

    /// ## 0x8XY6
    /// Set VX = VX SHIFT RIGHT 1, VF = the least significant bit.
    fn op_8xy6(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y]; // original chip8 quirk: set VX to VY
        let least_bit = self.v[x] & 0b0000_0001;

        let carry_flag = if least_bit == 0 { 0 } else { 1 };

        self.v[x] = self.v[x] >> 1;
        self.v[0xF] = carry_flag;
        self.inc_pc();
    }

    /// ## 0x8XY7
    /// Set VX = VY - VX. VF = not borrow flag.
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let (diff, overflow) = self.v[y].overflowing_sub(self.v[x]);

        self.v[x] = diff;
        self.v[0xF] = if overflow { 0 } else { 1 };
        self.inc_pc();
    }

    /// ## 0x8XYE
    /// Set VX = VX SHIFT LEFT 1, VF = the most significant bit.
    fn op_8xye(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y]; // original chip8 quirk: set VX to VY
        let most_bit = self.v[x] & 0b1000_0000;

        let carry_flag = if most_bit == 0 { 0 } else { 1 };

        self.v[x] = self.v[x] << 1;
        self.v[0xF] = carry_flag;
        self.inc_pc();
    }

    /// ## 0x9XY0
    /// Skip next instruction if VX != VY
    fn op_9xy0(&mut self, x: usize, y: usize) {
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

    /// ## 0xBNNN
    /// Jumps to address NNN + V0
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + (self.v[0] as u16);
    }

    /// ## 0xCXNN
    /// Sets VX to a random number[0-255] bitwise `AND` NN.
    fn op_cxnn(&mut self, x: usize, nn: u8) {
        let random_num: u8 = self.rng.gen();
        self.v[x] = random_num & nn;
        self.inc_pc();
    }

    /// ## 0xDXYN
    /// Draws to the screen and checks when there's pixel collision.
    fn op_dxyn(&mut self, x: usize, y: usize, height: u8) {
        let x_pos = self.v[x] % (SCREEN_WIDTH as u8);
        let y_pos = self.v[y] % (SCREEN_HEIGHT as u8);

        // println!("drawing at ({}, {}) sprite {}x8", x_pos, y_pos, height);

        // Set pixel collision false.
        self.v[0xF] = 0;

        for row in 0..height {
            // Clip sprite if it goes past the bottom of the screen.
            if (y_pos + row) >= (SCREEN_HEIGHT as u8) {
                // println!("skipping drawing at row {}", row);
                break;
            }
            let mut pixel = self.read(self.i + (row as u16));

            // Width is 8 bytes
            for col in 0..8 {
                // Clip sprite if it goes past the left side of the screen.
                if (x_pos + col) >= (SCREEN_WIDTH as u8) {
                    // println!("skipping drawing at col {}, row {}", col, row);
                    break;
                }

                if self.set_screen_pixel(x_pos + col, y_pos + row, (pixel & 0b1000_0000) >> 7) {
                    self.v[0xF] = 1; // There was pixel colision.
                }

                pixel = pixel << 1;
            }
        }

        self.draw_flag = true;
        self.inc_pc();
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
    /// Sets VX to the value in the delay timer.
    fn op_fx07(&mut self, x: usize) {
        self.v[x] = self.delay_timer;
        self.inc_pc();
    }

    /// ## 0xFX0A
    /// Waits for a key press and then stores that key in VX.
    /// We only resume once the key is released.
    fn op_fx0a(&mut self, x: usize) {
        if let Some(key_index) = self.pressed_key_index {
            if !self.keys[key_index] {
                self.pressed_key_index = None;
                self.v[x] = key_index as u8;
                self.inc_pc();
                return;
            }
        } else {
            for i in 0..16 {
                if self.keys[i] {
                    self.pressed_key_index = Some(i);
                }
            }
        }
    }

    /// ## 0xFX15
    /// Sets delay timer to VX.
    fn op_fx15(&mut self, x: usize) {
        self.delay_timer = self.v[x];
        self.inc_pc();
    }

    /// ## 0xFX18
    /// Sets sound timer to VX.
    fn op_fx18(&mut self, x: usize) {
        self.sound_timer = self.v[x];
        self.inc_pc();
    }

    /// ## 0xFX1E
    /// Adds VX to I, does not affect VF(carry flag).
    fn op_fx1e(&mut self, x: usize) {
        self.i += self.v[x] as u16;
        self.inc_pc();
    }

    /// ## 0xFX29
    /// Sets I to the address of the sprite for digit in VX.
    fn op_fx29(&mut self, x: usize) {
        self.i = FONTSET_START_ADDRESS + (self.v[x] as u16 * 5);
        self.inc_pc();
    }

    /// ## 0xFX33
    /// Takes the decimal value of VX and store the digits in I, I+1 and I+2.
    /// ### Example:
    /// Let VX = 0xFE => 254 in decimal.
    /// Then... I = 2, I+1 = 5, I+2 = 4
    fn op_fx33(&mut self, x: usize) {
        let mut value = self.v[x];
        self.write(self.i + 2, value % 10);
        value /= 10;

        self.write(self.i + 1, value % 10);
        value /= 10;

        self.write(self.i, value % 10);
        self.inc_pc();
    }

    /// ## 0xFX55
    /// Stores the bytes from V0 to VX(inclusive) into memory starting from the address stored in I.
    fn op_fx55(&mut self, x: usize) {
        for offset in 0..x + 1 {
            self.write(self.i + offset as u16, self.v[offset]);
        }
        self.i += 1; // original chip8 quirk: I is incremented after save.
        self.inc_pc();
    }

    /// ## 0xFX65
    /// Fills V0 to VX(inclusive) with bytes starting from the address stored in I.
    fn op_fx65(&mut self, x: usize) {
        for offset in 0..x + 1 {
            self.v[offset] = self.read(self.i + offset as u16);
        }
        self.i += 1; // original chip8 quirk: I is incremented after load.
        self.inc_pc();
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rng() {
        let mut cpu = Cpu::new();

        for _ in 0..10 {
            let n: u8 = cpu.rng.gen();

            if n < 0 || n > 255 {
                panic!("Random number generator out of bounds [0-255]");
            } else {
                println!("Number generated: {}", n);
            }
        }
    }
}
