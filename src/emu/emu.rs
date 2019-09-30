use std::fmt;

#[derive(Debug)]
pub struct Chip8 {
    opcode: u16, // Current opcode
    //memory: [u8; 4096],
    registers: [u8; 16],
    index: u16, // Index register
    pc: u16, // Program counter
    //gfx: [u8; 2048], // Pixel values (64 x 32 screen)

    // When set > zero, these timer registers will count down to zero
    // System buzzer should sound whenever either timer reaches zero
    delay_timer: u8,
    sound_timer: u8,

    stack: [u16; 16],
    sp: u16,

    key: [u8; 16], // Current key state
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            opcode: 0,
            registers: [0; 16],
            index: 0,
            pc: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16]
        }
    }
}

impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chip8 {{opcode: {}, index: {}, pc: {}, sp: {}}}",
            self.opcode,
            self.index,
            self.pc,
            self.sp
        )
    }
}
