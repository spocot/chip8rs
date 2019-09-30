mod emu;

use emu::Chip8;

fn main() {
    let c8 = Chip8::new();
    println!("{}", c8);
}
