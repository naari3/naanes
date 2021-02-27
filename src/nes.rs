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
                let scale = 4;
                let mut imgbuf = image::ImageBuffer::new(256 * scale, 240 * scale);
                for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
                    let color = display[(y / scale) as usize][(x / scale) as usize];
                    *pixel = image::Rgb([color[0], color[1], color[2]]);
                }
                imgbuf.save("a.png").unwrap();
            }
        }
    }
}
