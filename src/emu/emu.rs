use std::fmt;

use rand::Rng;

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

    keys: [u8; 16], // Current key state
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c = Chip8 {
            opcode: 0,
            memory: [0; 4096],
            registers: [0; 16], // V0 - VF
            index: 0,
            pc: 0x0200, // PC starts at 0x0200
            gfx: [0; 2048],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [0; 16]
        };

        c.fontset_into_mem();
        c
    }

    pub fn key_pressed(&mut self, key_index: usize) {
        self.keys[key_index] = 1;
    }

    pub fn key_released(&mut self, key_index: usize) {
        self.keys[key_index] = 0;
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

    pub fn load_rom(&mut self, rom: &[u8;4096 - 0x200]) {
        let mut mem = self.memory[..0x200].to_vec();
        mem.extend_from_slice(rom);

        self.memory.copy_from_slice(&mem);
    }

    pub fn set_mem(&mut self, src_mem: &[u8;4096]) {
        self.memory.copy_from_slice(src_mem);
    }

    pub fn should_fill_pixel(&self, x: usize, y: usize) -> bool {
        self.gfx[x + (y * 64)] == 1
    }

    fn get_nibble(&self, i: u8) -> u8 {
        let shift = (i % 4) * 4;
        let mask = 0xF << shift;

        ((self.opcode & mask) >> shift) as u8
    }

    fn get_byte(&self, upper: bool) -> u8 {
        if upper {
            let upper_byte = (self.opcode & 0xFF00) >> 8;
            upper_byte as u8
        } else {
            (self.opcode & 0xFF) as u8
        }
    }

    fn reg_dump(&mut self, end_index: u8) {
        let mut offset = self.index;
        for i in 0..(end_index+1) {
            self.memory[offset as usize] = self.registers[i as usize];
            offset += 1;
        }
    }

    fn reg_load(&mut self, end_index: u8) {
        let mut offset = self.index;
        for i in 0..(end_index+1) {
            self.registers[i as usize] = self.memory[offset as usize];
            offset += 1;
        }
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

                // 0x00EE => Return from a subroutine
                0x000E => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];

                    self.pc += 2;

                    println!("\tReturning from subroutine.");
                },

                _ => println!("NOP"),
            },

            // 0x1NNN => jump to address NNN
            0x1000 => {
                let nnn = self.opcode & 0x0FFF;

                self.pc = nnn;
            },

            // 0x2NNN => call subroutine at NNN
            0x2000 => {
                let nnn = self.opcode & 0x0FFF;

                self.stack[self.sp as usize] = self.pc;

                self.sp += 1;

                self.pc = nnn;
            },

            // 0x3XNN => skip next instruction if register VX == NN
            0x3000 => {
                let x = self.get_nibble(2);
                let nn = self.get_byte(false);

                if self.registers[x as usize] == nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

            // 0x4XNN => skip next if VX != NN
            0x4000 => {
                let x = self.get_nibble(2);
                let nn = self.get_byte(false);

                if self.registers[x as usize] != nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

            // 0x5XY0 => skip next if VX == VY
            0x5000 => {
                let x = self.get_nibble(2);
                let y = self.get_nibble(1);

                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

            // 0x6XNN => VX = NN
            0x6000 => {
                let x = self.get_nibble(2);
                let nn = self.get_byte(false);

                self.registers[x as usize] = nn;

                self.pc += 2;
            },

            // 0x7XNN => VX += NN
            0x7000 => {
                let x = self.get_nibble(2);
                let nn = self.get_byte(false);

                self.registers[x as usize] = self.registers[x as usize].wrapping_add(nn);

                self.pc += 2;
            },

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
                0x0004 => {
                    let x = self.get_nibble(2);
                    let y = self.get_nibble(1);

                    let xval = self.registers[x as usize] as u16;
                    let yval = self.registers[y as usize] as u16;

                    let result = xval + yval;
                    self.registers[x as usize] = result as u8;

                    // Set carry flag appropriately.
                    self.registers[0xF] = if result > 0xFF { 1 } else { 0 };

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) + V{}({:#X?}) -> {:#X?}", x, x, xval, y, yval, result);
                },

                // 0x8XY5 => VX -= VY, set VF to 0 if there is a borrow, 1 if not
                0x0005 => {
                    let x = self.get_nibble(2);
                    let y = self.get_nibble(1);

                    let xval = self.registers[x as usize];
                    let yval = self.registers[y as usize];

                    let result = xval.wrapping_sub(yval);
                    self.registers[x as usize] = result;

                    // Set borrow flag appropriately.
                    self.registers[0xF] = if yval > xval { 0 } else { 1 };

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) - V{}({:#X?}) -> {:#X?}", x, x, xval, y, yval, result);
                },

                // 0x8XY6 => Store least significant bit of VX in VF, then VX >>= 1
                0x0006 => {
                    let x = self.get_nibble(2);

                    let xval = self.registers[x as usize];

                    let least_sig_bit = x & 0x1;
                    let result = xval >> 1;

                    self.registers[x as usize] = result;

                    // Store least sig in VF
                    self.registers[0xF] = least_sig_bit;

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) >> 1) -> {:#X?}", x, x, xval, result);
                },

                // 0x8XY7 => VX = VY - VX, set VF to to 0 when borrow, 1 if not
                0x0007 => {
                    let x = self.get_nibble(2);
                    let y = self.get_nibble(1);

                    let xval = self.registers[x as usize];
                    let yval = self.registers[y as usize];

                    let result = yval.wrapping_sub(xval);
                    self.registers[x as usize] = result;

                    // Set borrow flag appropriately.
                    self.registers[0xF] = if xval > yval { 0 } else { 1 };

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) - V{}({:#X?}) -> {:#X?}", x, y, yval, x, xval, result);
                },

                // 0x8XYE => VX = Store most significant bit of VX in VF, then VX <<= 1
                0x000E => {
                    let x = self.get_nibble(2);

                    let xval = self.registers[x as usize];

                    let most_sig_bit = (x & 0x80) >> 7;
                    let result = (xval & 0x7F) << 1;

                    self.registers[x as usize] = result;

                    // Store most sig in VF
                    self.registers[0xF] = most_sig_bit;

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) << 1) -> {:#X?}", x, x, xval, result);
                },

                _ => println!("NOP"),
            },

            // 0x9XY0 => skips next instruction if VX != VY
            0x9000 => {
                let x = self.get_nibble(2);
                let y = self.get_nibble(1);

                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },

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
            0xC000 => {
                let x = self.get_nibble(2);
                let nn = self.get_byte(false);

                let r: u8 = rand::thread_rng().gen();

                self.registers[x as usize] = r & nn;

                self.pc += 2;
            },

            // 0xDXYN => Draw sprite at (VX, VY) w/ width 8pixels and height N
            // See https://en.wikipedia.org/wiki/CHIP-8 for more info.
            0xD000 => {
                let x = self.get_nibble(2) as u16;
                let y = self.get_nibble(1) as u16;
                let n = self.get_nibble(0) as u16;

                // Reset VF
                self.registers[0xF] = 0;

                for dy in 0..n {
                    let pixel = self.memory[(self.index + dy) as usize];

                    for dx in 0..8 {
                        let mask = 1 << (7 - dx);

                        // If pixel bit is set in memory.
                        if pixel & mask != 0 {

                            let gfx_index = (((y + dy) * 64) + x + dx) as usize;

                            // Check if pixel is set on screen.
                            if self.gfx[gfx_index] == 1 {
                                self.registers[0xF] = 1;
                            }

                            self.gfx[gfx_index] ^= 1;
                        }
                    }
                }

                self.pc += 2;
            },

            0xE000 => match self.opcode & 0x000F {

                // 0xEX9E => Skips next instruction if the key stored in VX is pressed
                0x000E => {
                    let x = self.get_nibble(2);
                    let key = self.registers[x as usize];

                    if self.keys[key as usize] != 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                },

                // 0xEXA1 => Skips next instruction if the key stored in VX is NOT pressed
                0x0001 => {
                    let x = self.get_nibble(2);
                    let key = self.registers[x as usize];

                    if self.keys[key as usize] == 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                },

                _ => println!("NOP"),
            },

            0xF000 => match self.opcode & 0x000F {

                // 0xFX33 => Take decimal representation of VX and store:
                //           High Digit at index
                //           Middle Digit at index+1
                //           Low Digit at index+2
                0x0003 => {
                    let x = self.get_nibble(2);
                    let val = self.registers[x as usize];

                    let high: u8 = val / 100;
                    let mid: u8 = (val / 10) % 10;
                    let lower: u8 = val % 10;

                    self.memory[self.index as usize] = high;
                    self.memory[self.index as usize + 1] = mid;
                    self.memory[self.index as usize + 2] = lower;

                    self.pc += 2;
                },

                0x0005 => match self.opcode & 0x00F0 {

                    // 0xFX15 => Set delay timer to VX
                    0x0010 => {
                        let x = self.get_nibble(2);

                        self.delay_timer = x;

                        self.pc += 2;

                        println!("\tDelay Timer set to {}", x);
                    },

                    // 0xFX55 => Stores V0-VX(inclusive) in memory starting at index
                    //           Offset increases by 1 for each value stored
                    //           index remains unchanged
                    0x0050 => {
                        let x = self.get_nibble(2);
                        self.reg_dump(x);

                        self.pc += 2;

                        println!("\tStore V0-V{} in mem starting@<{:#X?}>", x, self.index);
                    },

                    // 0xFX65 => Moves values from memory into V0-VX(inclusive) starting at index
                    //           Offset increases by 1 for each value loaded
                    //           index remains unchanged
                    //
                    0x0060 => {
                        let x = self.get_nibble(2);
                        self.reg_load(x);

                        self.pc += 2;

                        println!("\tLoad from mem starting@<{:#X?}> into V0-V{}", self.index, x);
                    },

                    _ => println!("NOP"),
                },

                // 0xFX07 => Set VX to value of delay timer
                0x0007 => {
                    let x = self.get_nibble(2);

                    self.registers[x as usize] = self.delay_timer;

                    self.pc += 2;
                },

                // 0xFX18 => Set sound timer to VX
                0x0008 => {
                    let x = self.get_nibble(2);

                    self.sound_timer = x;

                    self.pc += 2;

                    println!("\tSound Timer set to {}", x);
                },

                // 0xFX29 => Sets index to the location of the sprite for the character in VX
                //           Characters 0-F are represented by a 4x5 font
                0x0009 => {
                    let x = self.get_nibble(2);

                    self.index = self.registers[x as usize] as u16 * 5;

                    self.pc += 2;
                },

                // 0xFX0A => Block execution until a key press, then store value in VX
                0x000A => {
                    let x = self.get_nibble(2);

                    let mut pressed = false;

                    for k in 0..16 {
                        if self.keys[k as usize] != 0 {
                            self.registers[x as usize] = k;
                            pressed = true;
                        }
                    }

                    // Skip cycle if we didn't get a key press
                    if !pressed {
                        return;
                    }

                    self.pc += 2;
                },

                // 0xFX1E => Adds VX to index
                0x000E => {
                    let x = self.get_nibble(2);

                    self.index += self.registers[x as usize] as u16;

                    self.pc += 2;
                },

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
