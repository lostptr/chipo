// Chip-8 has 16 sprites of 5 bytes (16 * 5 = 80)
// They represent the hex digits of 0..F
static BUILT_IN_FONTSET: [u8; 80] = [
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
    pub sp: u8,

    // Keyboard
    pub key: [u8; 16],
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Cpu {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0,

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

    pub fn read(&self, address: usize) -> u8 {
        self.memory[address]
    }

    pub fn write(&mut self, address: usize, value: u8) {
        self.memory[address] = value;
    }
}
