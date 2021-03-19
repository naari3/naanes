use std::time::Instant;

use crate::{
    bus::Bus,
    controller::{Button, ControllerInput},
    ppu::{OAMDMAStatus, PPU},
    rom::ROM,
};
use emu6502::{
    cpu::{Interrupt, CPU},
    ram::{MemIO, RAM},
};

pub struct NES {
    cpu: CPU,
    ppu: PPU,
    wram: RAM,
    rom: ROM,
    nmi: bool,
    controller: ControllerInput,
}

impl NES {
    pub fn new(rom: ROM) -> NES {
        let ppu = PPU::new(rom.chr.clone(), rom.mapper);
        let mut prg = rom.prg.clone();
        let mut nes = NES {
            cpu: CPU::default(),
            ppu,
            wram: RAM::default(),
            rom,
            nmi: false,
            controller: ControllerInput::new(0),
        };
        nes.cpu.reset(&mut Bus::new(
            &mut nes.wram,
            &mut nes.ppu,
            &mut prg,
            nes.rom.mapper,
            &mut nes.controller,
        ));
        nes
    }

    // TODO: more consider interrupt timing
    pub fn step(&mut self, display: &mut [[[u8; 3]; 256]; 240]) {
        let mut cycles = 0;
        while cycles < (341 / 3) * (262 + 1) {
            if let OAMDMAStatus::NotRunning = self.ppu.oam_dma_status() {
                let mut bus = Bus::new(
                    &mut self.wram,
                    &mut self.ppu,
                    &mut self.rom.prg,
                    self.rom.mapper,
                    &mut self.controller,
                );
                self.cpu.step(&mut bus);
            }
            if let OAMDMAStatus::Running(address) = self.ppu.oam_dma_status() {
                let byte = {
                    let mut bus = Bus::new(
                        &mut self.wram,
                        &mut self.ppu,
                        &mut self.rom.prg,
                        self.rom.mapper,
                        &mut self.controller,
                    );
                    bus.read_byte(address)
                };
                self.ppu.oam_dma_write(byte)
            }

            for _ in 0..3 {
                self.ppu.step(display, &mut self.nmi);
                if self.nmi {
                    let mut bus = Bus::new(
                        &mut self.wram,
                        &mut self.ppu,
                        &mut self.rom.prg,
                        self.rom.mapper,
                        &mut self.controller,
                    );
                    self.cpu.interrupt(&mut bus, Interrupt::NMI);
                    self.cpu.remain_cycles = 0;
                    self.nmi = false;
                }
            }

            if let OAMDMAStatus::Waiting = self.ppu.oam_dma_status() {
                // TODO: Check it's correctly
                if cycles % 2 == 0 {
                    self.ppu.start_oam_dma();
                    self.cpu.remain_cycles = 512;
                }
            }
            cycles += 1;
        }
    }

    pub fn run(&mut self, display: &mut [[[u8; 3]; 256]; 240]) {
        self.ppu.set_rom(self.rom.chr.clone(), self.rom.mapper);
        let mut total_frames = 0;

        loop {
            let start = Instant::now();
            self.step(display);
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

    #[allow(dead_code)]
    pub fn update_input(&mut self, input: u8) {
        self.controller.update_input(input);
    }

    #[allow(dead_code)]
    pub fn press_button(&mut self, button: Button) {
        self.controller.press_button(button);
    }

    #[allow(dead_code)]
    pub fn release_button(&mut self, button: Button) {
        self.controller.release_button(button);
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
