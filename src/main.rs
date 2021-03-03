// use std::{fs::File, io::Write};

mod bus;
mod color;
mod mapper;
mod nes;
mod ppu;
mod rom;

fn main() {
    let rom_buffer = include_bytes!("../nestest.nes").to_vec();
    let rom = rom::parse(rom_buffer);
    // let mut file = File::create("sample1.prg").unwrap();
    // file.write_all(&rom.prg).unwrap();
    println!("Hello, world!: {}, {}", rom.prg.len(), rom.chr.len());
    let mut nes = nes::NES::new(rom);

    let mut display_buffer: [[[u8; 3]; 256]; 240] = [[[0; 3]; 256]; 240];
    nes.run(&mut display_buffer)
}
