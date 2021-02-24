use crate::{bus::Bus, ppu::PPU, rom::ROM};
use emu6502::cpu::CPU;

pub struct NES {
    cpu: CPU,
    ppu: PPU,
    rom: ROM,
}

impl NES {
    pub fn new(rom: ROM) -> NES {
        let ppu = PPU::default();
        let mut nes = NES {
            cpu: CPU::default(),
            ppu,
            rom,
        };
        nes.cpu
            .reset(&mut Bus::new(&mut nes.ppu, nes.rom.prg.clone()));
        nes
    }

    pub fn run(&mut self) {
        self.ppu.set_rom(self.rom.chr.clone());
        loop {
            {
                let mut bus = Bus::new(&mut self.ppu, self.rom.prg.clone());
                self.cpu.step(&mut bus);
            }
            self.ppu.step();
            self.ppu.step();
            self.ppu.step();
        }
    }
}
