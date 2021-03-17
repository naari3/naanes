extern crate naanes;

use std::time::Instant;

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Point};

fn main() {
    let rom_buffer = include_bytes!("../../nestest.nes").to_vec();
    let rom = naanes::rom::parse(rom_buffer);
    let mut nes = naanes::nes::NES::new(rom);

    let mut display_buffer: [[[u8; 3]; 256]; 240] = [[[0; 3]; 256]; 240];

    let scale = 3.0;
    let mut buffer = image::ImageBuffer::new(256, 240);
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("naanes", 256 * scale as u32, 240 * scale as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut total_frames = 0;
    'game: loop {
        let start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'game;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::F => nes.press_button(naanes::controller::Button::A),
                    Keycode::D => nes.press_button(naanes::controller::Button::B),
                    Keycode::S => nes.press_button(naanes::controller::Button::Select),
                    Keycode::A => nes.press_button(naanes::controller::Button::Start),
                    Keycode::Up => nes.press_button(naanes::controller::Button::Up),
                    Keycode::Down => nes.press_button(naanes::controller::Button::Down),
                    Keycode::Left => nes.press_button(naanes::controller::Button::Left),
                    Keycode::Right => nes.press_button(naanes::controller::Button::Right),
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::F => nes.release_button(naanes::controller::Button::A),
                    Keycode::D => nes.release_button(naanes::controller::Button::B),
                    Keycode::S => nes.release_button(naanes::controller::Button::Select),
                    Keycode::A => nes.release_button(naanes::controller::Button::Start),
                    Keycode::Up => nes.release_button(naanes::controller::Button::Up),
                    Keycode::Down => nes.release_button(naanes::controller::Button::Down),
                    Keycode::Left => nes.release_button(naanes::controller::Button::Left),
                    Keycode::Right => nes.release_button(naanes::controller::Button::Right),
                    _ => {}
                },
                _ => {}
            }
        }

        nes.step(&mut display_buffer);

        for x in 0..256 {
            for y in 0..240 {
                let c = display_buffer[y][x];
                canvas.set_draw_color(Color::RGB(c[0], c[1], c[2]));
                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
        }
        canvas.set_scale(scale, scale).unwrap();
        canvas.present();

        let frame_duration = start.elapsed();

        total_frames += 1;
        println!(
            "{} frames, fps: {:}",
            total_frames,
            1000.0 / frame_duration.as_millis() as f32
        );
        for (x, y, pixel) in buffer.enumerate_pixels_mut() {
            let color = display_buffer[y as usize][x as usize];
            *pixel = image::Rgba([color[0], color[1], color[2], 255]);
        }

        // snapshot(display, total_cycles);
    }
}
