extern crate piston_window;
extern crate image as im;
extern crate fps_counter;

use piston_window::*;
use piston_window::keyboard::Key;

mod emu;
use emu::Chip8;

use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: u32 = 2;
const SCALING_FACTOR: u32 = SCALE * 4;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

const SCREEN_WIDTH: u32 = WIDTH * SCALING_FACTOR;
const SCREEN_HEIGHT: u32 = HEIGHT * SCALING_FACTOR;

const STEP_BY_ONE: bool = false;
const DEBUG_MSG: bool = false;

// Map keys to which key register will hold them (the array index).
const KEYS: [Key; 16] = [
    Key::D1, Key::D2, Key::D3, Key::D4,
    Key::Q, Key::W, Key::E, Key::R,
    Key::A, Key::S, Key::D, Key::F,
    Key::Z, Key::X, Key::C, Key::V
];

fn main() {

    let mut should_step = STEP_BY_ONE;
    let mut should_debug = DEBUG_MSG;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 4 {
        // Weird number of command line args.
        println!("Usage: {} <romfile> [stepbyone=1|0] [debug=1|0]", &args[0]);
        return;
    }

    if args.len() > 2 {
        if let Ok(by_one) = (&args[2]).parse::<bool>() {
            should_step = by_one;
        } else {
            // We weren't given a bool show usage and return.
            println!("Usage: {} <romfile> [stepbyone=1|0] [debug=1|0]", &args[0]);
            return;
        }
    }

    if args.len() == 3 {
        if let Ok(debug_msg) = (&args[3]).parse::<bool>() {
            should_debug = debug_msg;
        } else {
            // We weren't given a bool show usage and return.
            println!("Usage: {} <romfile> [stepbyone=1|0] [debug=1|0]", &args[0]);
            return;
        }
    }


    println!("Loading memory into emulator...");

    // Load game ROM into buffer.
    let mut rom = [0;4096 - 0x200];

    let mut rom_file = File::open(&args[1]).expect("File not found");

    if let Ok(_) = rom_file.read(&mut rom) {
        println!("ROM loaded!");
    } else {
        println!("[-] ROM couldn't be loaded.");
        return;
    }

    // Create graphics display
    let mut window: PistonWindow = WindowSettings::new(
        "Chip8",
        (SCREEN_WIDTH, SCREEN_HEIGHT)
    ).exit_on_esc(true).build().unwrap();

    // Buffer for drawing
    let mut draw_buf = im::ImageBuffer::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    let mut texture: G2dTexture = Texture::from_image(
        &mut texture_context,
        &draw_buf,
        &TextureSettings::new()
    ).unwrap();

    let mut fps_cnt = fps_counter::FPSCounter::new();

    // Create a new chip8 emulator
    let mut c8 = Chip8::new();
    c8.show_debug = should_debug;
    c8.load_rom(&rom);

    while let Some(event) = window.next() {
        if let Some(_) = event.render_args() {

            texture.update(&mut texture_context, &draw_buf).unwrap();
            window.draw_2d(&event, |context, graphics, device| {
                texture_context.encoder.flush(device);
                clear([0.0, 0.0, 0.0, 1.0], graphics);

                image(&texture, context.transform, graphics);
            });

            let fps = fps_cnt.tick();
            let title = format!("Chip8-rs {}FPS", fps);
            window.set_title(title);
        } // end renger_args

        if let Some(_) = event.update_args() {
            if !should_step {
                c8.cycle();
            }
        } // end update_args

        if let Some(button_args) = event.button_args() {

            // Check if it was a key press.
            if let Button::Keyboard(key) = button_args.button {

                // Check if it's a key we care about.
                if let Some(key_index) = KEYS.iter().position(|&x| x == key) {

                    // Set/unset keystate based on press/release.
                    if button_args.state == ButtonState::Press {
                        c8.key_pressed(key_index);
                    } else {
                        c8.key_released(key_index);
                    }
                } else if key == Key::Return && should_step {
                    c8.cycle();
                }
            }

        } // end button_args

        if c8.redraw {
            draw_buf = im::ImageBuffer::new(SCREEN_WIDTH, SCREEN_HEIGHT);
            c8.redraw = false;
        }

        // Draw pixels from queue
        while !c8.draw_queue.is_empty() {

            if let Some((x, y, to_draw)) = c8.draw_queue.pop_front() {

                let dx = x as u32 * SCALING_FACTOR;
                let dy = y as u32 * SCALING_FACTOR;

                for ry in dy..(dy + SCALING_FACTOR) {
                    for rx in dx..(dx + SCALING_FACTOR) {
                        draw_buf.put_pixel(rx, ry,
                            if to_draw == 1 {
                                im::Rgba([255,255,255,255])
                            } else {
                                im::Rgba([0,0,0,255])
                            }
                        );
                    }
                }
            }
            /*if let Some((x, y, val)) = c8.draw_queue.pop_front() {
              draw_buf.put_pixel(x as u32, y as u32,
              if val == 0 {
              im::Rgba([0,0,0,255])
              } else {
              im::Rgba([255,255,255,255])
              }
              );
              }*/
        }
    }

    println!("Exited...");
}
