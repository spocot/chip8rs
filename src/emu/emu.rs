
pub struct Chip8 {
    opcode: u16, // Current opcode
    memory: [u8; 4096],
    registers: [u8; 16],
    index: u16, // Index register
    pc: u16, // Program counter
    gfx: [u8; 2048], // Pixel values (64 x 32 screen)

    // When set > zero, these timer registers will count down to zero
    // System buzzer should sound whenever either timer reaches zero
    delay_timer: u8,
    sound_timer: u8,

    key: [u8; 16], // Current key state
}
