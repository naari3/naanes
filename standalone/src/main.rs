extern crate naanes;

use std::time::Instant;

use piston_window::{
    clear, image as im_pis, G2dTexture, PistonWindow, Texture, TextureContext, TextureSettings,
    Transformed, WindowSettings,
};

fn main() {
    let rom_buffer = include_bytes!("../../nestest.nes").to_vec();
    let rom = naanes::rom::parse(rom_buffer);
    let mut nes = naanes::nes::NES::new(rom);

    let mut display_buffer: [[[u8; 3]; 256]; 240] = [[[0; 3]; 256]; 240];

    let scale = 3.0;
    let mut buffer = image::ImageBuffer::new(256, 240);
    let mut window: PistonWindow =
        WindowSettings::new("naanes", [256 as f64 * scale, 240 as f64 * scale])
            .exit_on_esc(true)
            .samples(0)
            .build()
            .unwrap();
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let mut texture: G2dTexture =
        Texture::from_image(&mut texture_context, &buffer, &TextureSettings::new()).unwrap();

    let mut total_frames = 0;
    loop {
        if let Some(event) = window.next() {
            let start = Instant::now();

            nes.step(&mut display_buffer);

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

            texture.update(&mut texture_context, &buffer).unwrap();
            window.draw_2d(&event, |context, graphics, device| {
                texture_context.encoder.flush(device);
                clear([1.0; 4], graphics);
                im_pis(&texture, context.transform.scale(scale, scale), graphics);
            });
        }
    }
}
