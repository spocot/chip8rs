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
    pc: usize, // Program counter
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

    pub fn cycle(&mut self) {

        // Get next opcode.
        self.opcode = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;

        // Decode opcode.
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x000F {
                // Clear Screen
                0x0000 => {},

                // Return from a subroutine
                0x000E => {},

                _ => println!("NOP"),
            },

            // 0x1NNN => jump to address NNN
            0x1000 => {},

            // 0x2NNN => call subroutine at NNN
            0x2000 => {},

            // 0x3XNN => skip next instruction if register X == NN
            0x3000 => {},

            // 0x4XNN => skip next if reg X != NN
            0x4000 => {},

            // 0x5XY0 => skip next if reg X == reg Y
            0x5000 => {},

            // 0x6XNN => X = NN
            0x6000 => {},

            // 0x7XNN => X += NN
            0x7000 => {},

            0x8000 => match self.opcode & 0x000F {

                // 0x8XY0 => X = Y
                0x0000 => {},

                // 0x8XY1 => X = X | Y
                0x0001 => {},

                // 0x8XY2 => X = X & Y
                0x0002 => {},

                // 0x8XY3 => X = X ^(bitwise xor) Y
                0x0003 => {},

                // 0x8XY4 => X += Y, set F to 1 if there is a carry, 0 if not
                0x0004 => {},

                // 0x8XY5 => X -= Y, set F to 0 if there is a borrow, 1 if not
                0x0005 => {},

                // 0x8XY6 => Store least significant bit of X in F, then X >>= 1
                0x0006 => {},

                // 0x8XY7 => X = X | Y
                0x0007 => {},

                // 0x8XYE => X = X & Y
                0x000E => {},

                _ => println!("NOP"),
            },

            _ => println!("NOP"),
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
