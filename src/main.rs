use emu6502;

mod bus;
mod rom;

fn main() {
    let rom_buffer = include_bytes!("../sample1.nes").to_vec();
    let rom = rom::parse(rom_buffer);
    println!("Hello, world!: {}, {}", rom.prg.len(), rom.chr.len());
    let mut cpu = emu6502::cpu::CPU::default();
    let mut ram = bus::Bus::default();
    ram.set_rom(rom.prg.clone());
    cpu.reset(&mut ram);
    loop {
        cpu.step(&mut ram);
    }
}
