#![feature(test)]

mod bench {
    use naanes::*;

    extern crate test;
    use test::Bencher;

    /// Hello, World表示にかかる時間計測
    #[bench]
    fn bench_hello(b: &mut Bencher) {
        let rom_buffer = include_bytes!("../nestest.nes").to_vec();
        let rom = rom::parse(rom_buffer);
        let mut nes = nes::NES::new(rom);

        let mut display_buffer: [[[u8; 3]; 256]; 240] = [[[0; 3]; 256]; 240];
        b.iter(|| {
            // wait menu displayed
            for _ in 0..4 {
                nes.step(&mut display_buffer);
            }
        });
    }
}
