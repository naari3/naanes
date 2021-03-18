pub mod bus;
pub mod color;
pub mod controller;
pub mod mapper;
pub mod nes;
pub mod ppu;
pub mod rom;

#[cfg(test)]
mod tests {
    use image::GenericImageView;

    use super::*;

    #[test]
    fn test_nestest() {
        let rom_buffer = include_bytes!("../roms/nestest.nes").to_vec();
        let rom = rom::parse(rom_buffer);
        let mut nes = nes::NES::new(rom);

        let mut display_buffer: [[[u8; 3]; 256]; 240] = [[[0; 3]; 256]; 240];
        // wait menu displayed
        for _ in 0..4 {
            nes.step(&mut display_buffer);
        }

        // start valid tests
        nes.press_button(controller::Button::Start);
        for _ in 0..17 {
            nes.step(&mut display_buffer);
        }
        nes.release_button(controller::Button::Start);
        validate_snapshot(
            &mut display_buffer,
            "screenshots/nestest_valid_ops.png".to_string(),
        );

        // swap to invalids
        nes.press_button(controller::Button::Select);
        nes.step(&mut display_buffer);
        nes.release_button(controller::Button::Select);
        nes.step(&mut display_buffer);

        // start invalid tests
        nes.press_button(controller::Button::Start);
        for _ in 0..20 {
            nes.step(&mut display_buffer);
        }
        validate_snapshot(
            &mut display_buffer,
            "screenshots/nestest_invalid_ops.png".to_string(),
        );
    }

    fn validate_snapshot(&mut display: &mut [[[u8; 3]; 256]; 240], screenshot_path: String) {
        let valid = image::open(screenshot_path).unwrap();
        for x in 0..256 {
            for y in 0..240 {
                let v = valid.get_pixel(x, y);
                assert_eq!(v.0[0], display[y as usize][x as usize][0]);
                assert_eq!(v.0[1], display[y as usize][x as usize][1]);
                assert_eq!(v.0[2], display[y as usize][x as usize][2]);
            }
        }
    }
}
