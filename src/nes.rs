use crate::{bus::Bus, ppu::PPU, rom::ROM};
use emu6502::cpu::CPU;
use image::{save_buffer, ColorType};

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

    pub fn run(&mut self, display: &mut [[[u8; 3]; 256]; 240]) {
        self.ppu.set_rom(self.rom.chr.clone());
        let mut loop_count = 0;
        loop {
            {
                let mut bus = Bus::new(&mut self.ppu, self.rom.prg.clone());
                self.cpu.step(&mut bus);
            }
            self.ppu.step(display);
            self.ppu.step(display);
            self.ppu.step(display);
            loop_count += 1;
            if loop_count % 10000 == 0 {
                let mut buf = vec![];
                for x in 0..256 {
                    for y in 0..240 {
                        for i in 0..3 {
                            buf.push(display[y][x][i]);
                        }
                    }
                }
                println!("buf length: {}", buf.len());
                image::save_buffer("a.png", &buf, 256, 240, image::ColorType::Rgb8).unwrap()
            }
        }
    }
}
