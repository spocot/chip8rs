
pub struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    registers: [u8; 16],
    index: u16,
    pc: u16
}
