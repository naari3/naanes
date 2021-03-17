use std::time::Instant;

use crate::{bus::Bus, ppu::PPU, rom::ROM};
use emu6502::{
    cpu::{Interrupt, CPU},
    ram::RAM,
};

pub struct NES {
    cpu: CPU,
    ppu: PPU,
    wram: RAM,
    rom: ROM,
    nmi: bool,
}

impl NES {
    pub fn new(rom: ROM) -> NES {
        let ppu = PPU::new(rom.chr.clone(), rom.mapper);
        let prg = rom.prg.clone();
        let mut nes = NES {
            cpu: CPU::default(),
            ppu,
            wram: RAM::default(),
            rom,
            nmi: false,
        };
        nes.cpu.reset(&mut Bus::new(
            &mut nes.wram,
            &mut nes.ppu,
            prg,
            nes.rom.mapper,
        ));
        nes
    }

    pub fn step(&mut self, display: &mut [[[u8; 3]; 256]; 240]) {
        let mut cycles = 0;
        while cycles < (341 / 3) * (262 + 1) {
            {
                let mut bus = Bus::new(
                    &mut self.wram,
                    &mut self.ppu,
                    self.rom.prg.clone(),
                    self.rom.mapper,
                );
                if self.nmi {
                    self.cpu.interrupt(&mut bus, Interrupt::NMI);
                    self.cpu.remain_cycles = 0;
                    self.nmi = false;
                }
                self.cpu.step(&mut bus);
            }
            for _ in 0..3 {
                self.ppu.step(display, &mut self.nmi);
            }
            cycles += 1;
        }
    }

    pub fn run(&mut self, display: &mut [[[u8; 3]; 256]; 240]) {
        let guard = pprof::ProfilerGuard::new(100).unwrap();

        self.ppu.set_rom(self.rom.chr.clone(), self.rom.mapper);
        let mut total_frames = 0;

        loop {
            let start = Instant::now();
            let mut cycles = 0;
            while cycles < (341 / 3) * (262 + 1) {
                {
                    let mut bus = Bus::new(
                        &mut self.wram,
                        &mut self.ppu,
                        self.rom.prg.clone(),
                        self.rom.mapper,
                    );
                    if self.nmi {
                        self.cpu.interrupt(&mut bus, Interrupt::NMI);
                        self.cpu.remain_cycles = 0;
                        self.nmi = false;
                    }
                    self.cpu.step(&mut bus);
                }
                for _ in 0..3 {
                    self.ppu.step(display, &mut self.nmi);
                }
                cycles += 1;
            }
            let frame_duration = start.elapsed();
            total_frames += 1;
            println!(
                "{} frames, fps: {:}",
                total_frames,
                1000.0 / frame_duration.as_millis() as f32
            );

            snapshot(display, total_frames);
        }
    }
}

fn snapshot(&mut display: &mut [[[u8; 3]; 256]; 240], frame_count: usize) {
    let scale = 4;
    let mut imgbuf = image::ImageBuffer::new(256 * scale, 240 * scale);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let color = display[(y / scale) as usize][(x / scale) as usize];
        *pixel = image::Rgb([color[0], color[1], color[2]]);
    }
    imgbuf
        .save(format!("./tmp/a_{:0>10}.png", frame_count))
        .unwrap();
}
