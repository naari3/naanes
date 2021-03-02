use crate::{bus::Bus, ppu::PPU, rom::ROM};
use emu6502::{cpu::CPU, ram::RAM};

pub struct NES {
    cpu: CPU,
    ppu: PPU,
    wram: RAM,
    rom: ROM,
}

impl NES {
    pub fn new(rom: ROM) -> NES {
        let ppu = PPU::new(rom.mapper);
        let prg = rom.prg.clone();
        let mut nes = NES {
            cpu: CPU::default(),
            ppu,
            wram: RAM::default(),
            rom,
        };
        nes.cpu.reset(&mut Bus::new(
            &mut nes.wram,
            &mut nes.ppu,
            prg,
            nes.rom.mapper,
        ));
        nes
    }

    pub fn run(&mut self, display: &mut [[[u8; 3]; 256]; 240]) {
        self.ppu.set_rom(self.rom.chr.clone(), self.rom.mapper);
        let mut loop_count = 0;
        self.cpu.flags.i = true;
        self.cpu.pc = 0xC000;
        self.cpu.sp = 0xFD;
        loop {
            {
                let mut bus = Bus::new(
                    &mut self.wram,
                    &mut self.ppu,
                    self.rom.prg.clone(),
                    self.rom.mapper,
                );
                self.cpu.step(&mut bus);
            }
            self.ppu.step(display);
            self.ppu.step(display);
            self.ppu.step(display);
            loop_count += 1;
            if loop_count % 10000 == 0 {
                snapshot(display);
            }
        }
    }
}

fn snapshot(&mut display: &mut [[[u8; 3]; 256]; 240]) {
    let scale = 4;
    let mut imgbuf = image::ImageBuffer::new(256 * scale, 240 * scale);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let color = display[(y / scale) as usize][(x / scale) as usize];
        *pixel = image::Rgb([color[0], color[1], color[2]]);
    }
    imgbuf.save("a.png").unwrap();
}
