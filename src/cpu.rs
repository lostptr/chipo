struct Cpu {
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
    pub gfx: [u8; 64 * 32],

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