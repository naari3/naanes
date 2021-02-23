mod bus;
mod nes;
mod ppu;
mod rom;

fn main() {
    let rom_buffer = include_bytes!("../sample1.nes").to_vec();
    let rom = rom::parse(rom_buffer);
    println!("Hello, world!: {}, {}", rom.prg.len(), rom.chr.len());
    let mut nes = nes::NES::new(rom);
    nes.run()
}
