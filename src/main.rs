extern crate piston_window;
extern crate image as im;
extern crate fps_counter;

use piston_window::*;

mod emu;
use emu::Chip8;

const SCALE: u32 = 2;
const SCALING_FACTOR: u32 = SCALE * 4;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

const SCREEN_WIDTH: u32 = WIDTH * SCALING_FACTOR;
const SCREEN_HEIGHT: u32 = HEIGHT * SCALING_FACTOR;

fn main() {

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

    // Initialize the chip8 emulator
    let mut c8 = Chip8::new();
    //draw_buf.put_pixel(0,0,im::Rgba([0,0,0,255]));

    while let Some(event) = window.next() {
        if let Some(_) = event.render_args() {
            texture.update(&mut texture_context, &draw_buf).unwrap();
            window.draw_2d(&event, |context, graphics, device| {
                texture_context.encoder.flush(device);
                clear([1.0, 0.0, 1.0, 1.0], graphics);

                image(&texture, context.transform, graphics);
            });

            // Very gross looking way to draw pixels from c8 gfx
            // TODO: find more efficient way to do this?
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let dx = x * SCALING_FACTOR;
                    let dy = y * SCALING_FACTOR;

                    let should_fill = c8.should_fill_pixel(x as usize, y as usize);

                    for ry in dy..(dy + SCALING_FACTOR) {
                        for rx in dx..(dx + SCALING_FACTOR) {
                            draw_buf.put_pixel(rx, ry,
                                if should_fill {
                                    im::Rgba([0,0,0,255])
                                } else {
                                    im::Rgba([1,1,1,255])
                                }
                            );
                        }
                    }
                }
            }

            let fps = fps_cnt.tick();
            let title = format!("Chip8-rs {}FPS", fps);
            window.set_title(title);
        } // end renger_args

        if let Some(_) = event.update_args() {
            println!("Update tick");
            c8.cycle();
        } // end update_args
    }

    println!("Exited...");
}
