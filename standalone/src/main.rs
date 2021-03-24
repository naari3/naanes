extern crate naanes;

use std::time::Instant;

use piston_window::{
    clear, image as im_pis, Button, CloseEvent, EventLoop, G2dTexture, Key, PistonWindow,
    PressEvent, ReleaseEvent, RenderEvent, Texture, TextureContext, TextureSettings, Transformed,
    WindowSettings,
};

fn main() {
    let rom_buffer = include_bytes!("../../roms/nestest.nes").to_vec();
    let rom = naanes::rom::parse(rom_buffer);
    let mut nes = naanes::nes::NES::new(rom);

    let mut display_buffer: [[[u8; 3]; 256]; 240] = [[[0; 3]; 256]; 240];

    let scale = 2.0;
    let mut buffer = image::ImageBuffer::new(256, 240);

    let mut window: PistonWindow =
        WindowSettings::new("naanes", [256 as f64 * scale, 240 as f64 * scale])
            .build()
            .unwrap();
    let mut max_fps_mode = false;
    window.set_max_fps(68);
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let mut texture: G2dTexture =
        Texture::from_image(&mut texture_context, &buffer, &TextureSettings::new()).unwrap();

    let mut total_frames = 0;
    let mut fps = fps_counter::FPSCounter::default();

    loop {
        if let Some(event) = window.next() {
            if let Some(_) = event.render_args() {
                let start = Instant::now();
                nes.step(&mut display_buffer);

                for (x, y, pixel) in buffer.enumerate_pixels_mut() {
                    let color = display_buffer[y as usize][x as usize];
                    *pixel = image::Rgba([color[0], color[1], color[2], 255]);
                }

                // snapshot(display, total_cycles);

                texture.update(&mut texture_context, &buffer).unwrap();
                window.draw_2d(&event, |c, g, d| {
                    texture_context.encoder.flush(d);
                    clear([0.0; 4], g);
                    im_pis(&texture, c.transform.scale(scale, scale), g);
                });
                total_frames += 1;

                let duration = start.elapsed();
                println!(
                    "{} frames, fps: {:}, took: {:?} ms",
                    total_frames,
                    fps.tick(),
                    duration
                );
            }

            if let Some(Button::Keyboard(key)) = event.press_args() {
                match key {
                    Key::F => nes.press_button(naanes::controller::Button::A),
                    Key::D => nes.press_button(naanes::controller::Button::B),
                    Key::S => nes.press_button(naanes::controller::Button::Select),
                    Key::A => nes.press_button(naanes::controller::Button::Start),
                    Key::Up => nes.press_button(naanes::controller::Button::Up),
                    Key::Down => nes.press_button(naanes::controller::Button::Down),
                    Key::Left => nes.press_button(naanes::controller::Button::Left),
                    Key::Right => nes.press_button(naanes::controller::Button::Right),
                    Key::Tab => {
                        max_fps_mode = !max_fps_mode;
                        let fps = if max_fps_mode { 10000 } else { 68 };
                        window.set_max_fps(fps);
                    }
                    _ => {}
                }
            }

            if let Some(Button::Keyboard(key)) = event.release_args() {
                match key {
                    Key::F => nes.release_button(naanes::controller::Button::A),
                    Key::D => nes.release_button(naanes::controller::Button::B),
                    Key::S => nes.release_button(naanes::controller::Button::Select),
                    Key::A => nes.release_button(naanes::controller::Button::Start),
                    Key::Up => nes.release_button(naanes::controller::Button::Up),
                    Key::Down => nes.release_button(naanes::controller::Button::Down),
                    Key::Left => nes.release_button(naanes::controller::Button::Left),
                    Key::Right => nes.release_button(naanes::controller::Button::Right),
                    Key::Escape => break,
                    _ => {}
                }
            }

            if let Some(_) = event.close_args() {
                break;
            }
        }
    }
}
