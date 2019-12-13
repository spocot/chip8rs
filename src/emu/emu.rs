use std::fmt;
use std::collections::VecDeque;

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
    pub gfx: [[u8; 64]; 32], // Pixel values (64 x 32 screen)

    // When set > zero, these timer registers will count down to zero
    // System buzzer should sound whenever either timer reaches zero
    delay_timer: u8,
    sound_timer: u8,

    stack: [u16; 16],
    sp: u16,

    keys: [u8; 16], // Current key state

    pub redraw: bool, // Should gfx be completely redrawn?
    pub draw_queue: VecDeque<(u16, u16, u8)>,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c = Chip8 {
            opcode: 0,
            memory: [0; 4096],
            registers: [0; 16], // V0 - VF
            index: 0,
            pc: 0x0200, // PC starts at 0x0200
            gfx: [[0; 64]; 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [0; 16],
            redraw: true,
            draw_queue: VecDeque::new(),
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
        self.gfx = [[0; 64]; 32];
        self.redraw = true;
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

        self.redraw = true;
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
        self.gfx[y][x] == 1
    }

    fn get_nibble(&self, i: u8) -> u8 {
        let shift = (i % 4) * 4;
        let mask = 0xF << shift;

        ((self.opcode & mask) >> shift) as u8
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

        // Store values that some opcodes need to use.
        let x: u8 = self.get_nibble(2);
        let y: u8 = self.get_nibble(1);
        let n: u8 = (self.opcode & 0xF) as u8;
        let nn: u8 = (self.opcode & 0xFF) as u8;
        let nnn: u16 = self.opcode & 0xFFF;


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

                    println!("\tReturning from subroutine, setting sp={} pc={}+2", self.sp, self.pc - 2);
                },

                _ => println!("NOP"),
            },

            // 0x1NNN => jump to address NNN
            0x1000 => {
                self.pc = nnn;

                println!("\tJumping to address {}", nnn);
            },

            // 0x2NNN => call subroutine at NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;

                self.sp += 1;

                self.pc = nnn;

                println!("\tCalling subroutine at {}", nnn);
            },

            // 0x3XNN => skip next instruction if register VX == NN
            0x3000 => {
                let val = self.registers[x as usize];
                if val == nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                println!("\tSkip next if V{}({})=={}", x, val, nn);
            },

            // 0x4XNN => skip next if VX != NN
            0x4000 => {
                let val = self.registers[x as usize];
                if val != nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                println!("\tSkip next if V{}({})!={}", x, val, nn);
            },

            // 0x5XY0 => skip next if VX == VY
            0x5000 => {
                let valx = self.registers[x as usize];
                let valy = self.registers[y as usize];

                if valx == valy {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                println!("\tSkip next if V{}({})==V{}({})", x, valx, y, valy);
            },

            // 0x6XNN => VX = NN
            0x6000 => {
                self.registers[x as usize] = nn;

                self.pc += 2;

                println!("\tSet V{}={}", x, nn);
            },

            // 0x7XNN => VX += NN
            0x7000 => {

                let val = &mut self.registers[x as usize];
                let prev_val = *val;

                *val = (*val).wrapping_add(nn);

                self.pc += 2;

                println!("\tV{}={} wrapadd {} = {}", x, prev_val, nn, *val);
            },

            0x8000 => match self.opcode & 0x000F {

                // 0x8XY0 => VX = VY
                0x0000 => {
                    let val = self.registers[y as usize];

                    self.registers[x as usize] = val;

                    self.pc += 2;

                    println!("\tV{}=V{} ({:#X?})", x, y, val);
                },

                // 0x8XY1 => VX = VX | VY
                0x0001 => {
                    let xval = self.registers[x as usize];
                    let yval = self.registers[y as usize];

                    let result = xval | yval;
                    self.registers[x as usize] = result;

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) | V{}({:#X?}) -> {:#X?}", x, x, xval, y, yval, result);
                },

                // 0x8XY2 => VX = VX & VY
                0x0002 => {
                    let xval = self.registers[x as usize];
                    let yval = self.registers[y as usize];

                    let result = xval & yval;
                    self.registers[x as usize] = result;

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) & V{}({:#X?}) -> {:#X?}", x, x, xval, y, yval, result);
                },

                // 0x8XY3 => VX = VX ^(bitwise xor) VY
                0x0003 => {
                    let xval = self.registers[x as usize];
                    let yval = self.registers[y as usize];

                    let result = xval ^ yval;
                    self.registers[x as usize] = result;

                    self.pc += 2;

                    println!("\tV{}=V{}({:#X?}) ^ V{}({:#X?}) -> {:#X?}", x, x, xval, y, yval, result);
                },

                // 0x8XY4 => VX += VY, set VF to 1 if there is a carry, 0 if not
                0x0004 => {
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
                let xval = self.registers[x as usize];
                let yval = self.registers[y as usize];

                if xval != yval {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }

                println!("\tSkip next if V{}({})!=V{}({})", x, xval, y, yval);
            },

            // 0xANNN => set index to NNN
            0xA000 => {
                self.index = nnn;

                self.pc += 2;

                println!("\tSetting I(index) to {}.", nnn);
            },

            // 0xBNNN => set PC to V0 + NNN
            0xB000 => {
                self.pc = self.registers[0] as u16 + nnn;

                println!("\tSetting PC to V0 ({:#?}) + {:X?} = ({:#?})", self.registers[0], nnn, self.pc);
            },

            // 0xCXNN => set VX to some random number (0-255), R & NN
            0xC000 => {
                let r: u8 = rand::thread_rng().gen();

                let result = r & nn;

                self.registers[x as usize] = result;

                self.pc += 2;

                println!("\tSet V{} to random# {}", x, result);
            },

            // 0xDXYN => Draw sprite at (VX, VY) w/ width 8pixels and height N
            // See https://en.wikipedia.org/wiki/CHIP-8 for more info.
            0xD000 => {

                // Reset VF
                self.registers[0xF] = 0;

                for dy in 0..n {
                    let pixel = self.memory[(self.index + dy as u16) as usize];

                    for dx in 0..8 {
                        let mask = 0b10000000 >> dx;

                        // If pixel bit is set in memory.
                        if pixel & mask != 0 {

                            let gfx_index = ((x + dx) as usize, (y + dy) as usize);

                            let data = &mut (self.gfx[gfx_index.1][gfx_index.0]);

                            // Check if pixel is set on screen.
                            if *data == 1 {
                                self.registers[0xF] = 1;
                            }

                            *data = *data ^ 1;

                            let locx = (x + dx) as u16;
                            let locy = (y + dy) as u16;
                            self.draw_queue.push_back((locx, locy, *data));
                        }
                    }
                }

                self.pc += 2;

                println!("\tDraw sprite at ({},{}) with height {}", x, y, n);
            },

            0xE000 => match self.opcode & 0x000F {

                // 0xEX9E => Skips next instruction if the key stored in VX is pressed
                0x000E => {
                    let key = self.registers[x as usize];

                    if self.keys[key as usize] != 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                },

                // 0xEXA1 => Skips next instruction if the key stored in VX is NOT pressed
                0x0001 => {
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
                        self.delay_timer = x;

                        self.pc += 2;

                        println!("\tDelay Timer set to {}", x);
                    },

                    // 0xFX55 => Stores V0-VX(inclusive) in memory starting at index
                    //           Offset increases by 1 for each value stored
                    //           index remains unchanged
                    0x0050 => {
                        self.reg_dump(x);

                        self.pc += 2;

                        println!("\tStore V0-V{} in mem starting@<{:#X?}>", x, self.index);
                    },

                    // 0xFX65 => Moves values from memory into V0-VX(inclusive) starting at index
                    //           Offset increases by 1 for each value loaded
                    //           index remains unchanged
                    //
                    0x0060 => {
                        self.reg_load(x);

                        self.pc += 2;

                        println!("\tLoad from mem starting@<{:#X?}> into V0-V{}", self.index, x);
                    },

                    _ => println!("NOP"),
                },

                // 0xFX07 => Set VX to value of delay timer
                0x0007 => {
                    self.registers[x as usize] = self.delay_timer;

                    self.pc += 2;

                    println!("\tSet V{}={} (delay timer)", x, self.delay_timer);
                },

                // 0xFX18 => Set sound timer to VX
                0x0008 => {
                    let xval = self.registers[x as usize];

                    self.sound_timer = xval;

                    self.pc += 2;

                    println!("\tSound Timer set to {}", xval);
                },

                // 0xFX29 => Sets index to the location of the sprite for the character in VX
                //           Characters 0-F are represented by a 4x5 font
                0x0009 => {
                    self.index = self.registers[x as usize] as u16 * 5;

                    self.pc += 2;

                    println!("\tSet index to loc of sprite for character in V{} = {}", x, self.index);
                },

                // 0xFX0A => Block execution until a key press, then store value in VX
                0x000A => {
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
                    let xval = self.registers[x as usize] as u16;
                    self.index += xval;

                    self.pc += 2;

                    println!("\tAdd V{}({}) to index = {}", x, xval, self.index);
                },

                _ => println!("NOP"),
            },

            _ => println!("NOP"),
        } // End of Opcode matching
    }

    pub fn cycle(&mut self) {

        // Decode and perform the current opcode.
        self.perform_opcode();

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
