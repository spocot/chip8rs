use std::fmt;

const FONTSET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

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

    stack: [u16; 16],
    sp: u16,

    key: [u8; 16], // Current key state
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c = Chip8 {
            opcode: 0x200, // pc starts here
            memory: [0; 4096],
            registers: [0; 16],
            index: 0,
            pc: 0,
            gfx: [0; 2048],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16]
        };

        c.fontset_into_mem();
        c
    }

    fn fontset_into_mem(&mut self) {
        // Load fontset into memory.
        for i in 0..80 {
            self.memory[i] = FONTSET[i];
        }
    }

    pub fn init(&mut self) {
        // Chip8 program counter starts at 0x200
        self.pc = 0x200;

        // Reset opcode, index, and stack pointer.
        self.opcode = 0;
        self.index = 0;
        self.sp = 0;

        // Clear display, stack, registers, and memory.
        self.gfx.iter_mut().for_each(|x| *x = 0);
        self.stack.iter_mut().for_each(|x| *x = 0);
        self.registers.iter_mut().for_each(|x| *x = 0);
        self.memory.iter_mut().for_each(|x| *x = 0);

        // Reset timers
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.fontset_into_mem();
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
