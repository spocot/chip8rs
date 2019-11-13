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
    registers: [u8; 16], // V0 - VF
    index: u16, // Index register
    pc: u16, // Program counter
    pub gfx: [u8; 2048], // Pixel values (64 x 32 screen)

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
            registers: [0; 16], // V0 - VF
            index: 0,
            pc: 0,
            gfx: [1; 2048],
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

    fn clear_screen(&mut self) {
        self.gfx.iter_mut().for_each(|x| *x = 0);
    }

    pub fn init(&mut self) {
        // Chip8 program counter starts at 0x200
        self.pc = 0x200;

        // Reset opcode, index, and stack pointer.
        self.opcode = 0;
        self.index = 0;
        self.sp = 0;

        // Clear display, stack, registers, and memory.
        self.clear_screen();
        self.stack.iter_mut().for_each(|x| *x = 0);
        self.registers.iter_mut().for_each(|x| *x = 0);
        self.memory.iter_mut().for_each(|x| *x = 0);

        // Reset timers
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.fontset_into_mem();
    }

    pub fn load_mem(&mut self, src_mem: &[u8;4096]) {
        self.memory.clone_from_slice(src_mem);
    }

    pub fn should_fill_pixel(&self, x: usize, y: usize) -> bool {
        self.gfx[x + (y * 32)] == 1
    }

    fn get_nibble(&self, i: u8) -> u8 {
        let shift = (i % 4) * 4;
        let mask = 0xF << shift;

        return ((self.opcode & mask) >> shift) as u8;
    }

    fn perform_opcode(&mut self) {

        // Get next opcode.
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | self.memory[self.pc as usize + 1] as u16;

        println!("PC: {}, opcode: <{:#X?}>", self.pc, self.opcode);

        // Decode opcode.
        match self.opcode & 0xF000 {
            0x0000 => match self.opcode & 0x000F {
                // Clear Screen
                0x0000 => {
                    self.clear_screen();
                    self.pc += 2;

                    println!("\tClearing screen.");
                },

                // Return from a subroutine
                0x000E => {
                    println!("\tReturning from subroutine.");
                },

                _ => println!("NOP"),
            },

            // 0x1NNN => jump to address NNN
            0x1000 => {},

            // 0x2NNN => call subroutine at NNN
            0x2000 => {},

            // 0x3XNN => skip next instruction if register VX == NN
            0x3000 => {},

            // 0x4XNN => skip next if VX != NN
            0x4000 => {},

            // 0x5XY0 => skip next if VX == reg Y
            0x5000 => {},

            // 0x6XNN => VX = NN
            0x6000 => {},

            // 0x7XNN => VX += NN
            0x7000 => {},

            0x8000 => match self.opcode & 0x000F {

                // 0x8XY0 => VX = VY
                0x0000 => {
                    let x = self.get_nibble(2);
                    let y = self.get_nibble(1);

                    let val = self.registers[y as usize];

                    self.registers[x as usize] = val;

                    self.pc += 2;

                    println!("\tV{}=V{} ({:#X?})", x, y, val);
                },

                // 0x8XY1 => VX = VX | VY
                0x0001 => {
                    let x = self.get_nibble(2);
                    let y = self.get_nibble(1);

                    let xval = self.registers[x as usize];
                    let yval = self.registers[y as usize];

                    let result = xval | yval;
                    self.registers[x as usize] = result;

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) | V{}({:#X?}) -> {:#X?}", x, x, xval, y, yval, result);
                },

                // 0x8XY2 => VX = VX & VY
                0x0002 => {
                    let x = self.get_nibble(2);
                    let y = self.get_nibble(1);

                    let xval = self.registers[x as usize];
                    let yval = self.registers[y as usize];

                    let result = xval & yval;
                    self.registers[x as usize] = result;

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) & V{}({:#X?}) -> {:#X?}", x, x, xval, y, yval, result);
                },

                // 0x8XY3 => VX = VX ^(bitwise xor) VY
                0x0003 => {
                    let x = self.get_nibble(2);
                    let y = self.get_nibble(1);

                    let xval = self.registers[x as usize];
                    let yval = self.registers[y as usize];

                    let result = xval ^ yval;
                    self.registers[x as usize] = result;

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) ^ V{}({:#X?}) -> {:#X?}", x, x, xval, y, yval, result);
                },

                // 0x8XY4 => VX += VY, set VF to 1 if there is a carry, 0 if not
                0x0004 => {},

                // 0x8XY5 => VX -= VY, set VF to 0 if there is a borrow, 1 if not
                0x0005 => {},

                // 0x8XY6 => Store least significant bit of VX in VF, then VX >>= 1
                0x0006 => {},

                // 0x8XY7 => VX = VY - VX, set VF to to 0 when borrow, 1 if not
                0x0007 => {},

                // 0x8XYE => VX = Store most significant bit of VX in VF, then VX <<= 1
                0x000E => {},

                _ => println!("NOP"),
            },

            // 0x9XY0 => skips next instruction if VX != VY
            0x9000 => {},

            // 0xANNN => set index to NNN
            0xA000 => {
                let i = self.opcode & 0x0FFF;
                self.index = i;
                self.pc += 2;

                println!("\tSetting I(index) to {}.", i);
            },

            // 0xBNNN => set PC to V0 + NNN
            0xB000 => {
                let nnn = self.opcode & 0x0FFF;
                self.pc = self.registers[0] as u16 + nnn;

                println!("\tSetting PC to V0 ({:#?}) + {:X?} = ({:#?})", self.registers[0], nnn, self.pc);
            },

            // 0xCXNN => set VX to some random number (0-255), R & NN
            0xC000 => {},

            // 0xDXYN => Draw sprite at (VX, VY) w/ width 8pixels and height N
            // See https://en.wikipedia.org/wiki/CHIP-8 for more info.
            0xD000 => {},

            0xE000 => match self.opcode & 0x000F {

                // 0xEX9E => Skips next instruction if the key stored in VX is pressed
                0x000E => {},

                // 0xEXA1 => Skips next instruction if the key stored in VX is NOT pressed
                0x0001 => {},

                _ => println!("NOP"),
            },

            0xF000 => match self.opcode & 0x000F {

                // 0xFX33 => Take decimal representation of VX and store:
                //           High Digit at index
                //           Middle Digit at index+1
                //           Low Digit at index+2
                0x0003 => {},

                0x0005 => match self.opcode & 0x00F0 {

                    // 0xFX15 => Set delay timer to VX
                    0x0010 => {},

                    // 0xFX55 => Stores V0-VX(inclusive) in memory starting at index
                    //           Offset increases by 1 for each value stored
                    //           index remains unchanged
                    0x0050 => {},

                    // 0xFX65 => Moves values from memory into V0-VX(inclusive) starting at index
                    //           Offset increases by 1 for each value loaded
                    //           index remains unchanged
                    //
                    0x0060 => {},

                    _ => println!("NOP"),
                },

                // 0xFX07 => Set VX to value of display timer
                0x0007 => {},

                // 0xFX18 => Set sound timer to VX
                0x0008 => {},

                // 0xFX29 => Sets index to the location of the sprite for the character in VX
                //           Characters 0-F are represented by a 4x5 font
                0x0009 => {},

                // 0xFX0A => Block execution until a key press, then store value in VX
                0x000A => {},

                // 0xFX1E => Adds VX to index
                0x000E => {},

                _ => println!("NOP"),
            },

            _ => println!("NOP"),
        } // End of Opcode matching
    }

    pub fn cycle(&mut self) {

        // Decode and perform the current opcode.
        self.perform_opcode();

        let x = self.registers[0];
        let y = self.registers[1];

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Make a beep
                println!("BEEP");
            }
            self.sound_timer -= 1;
        }

    } // End of fn cycle()
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
