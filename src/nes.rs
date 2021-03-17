use std::fs::File;

use piston_window::{
    clear, image as im_pis, G2dTexture, PistonWindow, Texture, TextureContext, TextureSettings,
    Transformed, WindowSettings,
};

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
        let ppu = PPU::new(rom.mapper);
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

    pub fn run(&mut self, display: &mut [[[u8; 3]; 256]; 240]) {
        let guard = pprof::ProfilerGuard::new(100).unwrap();

        self.ppu.set_rom(self.rom.chr.clone(), self.rom.mapper);
        let mut total_cycles = 0;
        let mut total_frames = 0;

        let scale = 3.0;
        let mut buffer = image::ImageBuffer::new(256, 240);
        let mut window: PistonWindow =
            WindowSettings::new("naanes", [256 as f64 * scale, 240 as f64 * scale])
                .exit_on_esc(true)
                .samples(0)
                .build()
                .unwrap();
        let mut texture_context = TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into(),
        };
        let mut texture: G2dTexture =
            Texture::from_image(&mut texture_context, &buffer, &TextureSettings::new()).unwrap();

        loop {
            if let Some(event) = window.next() {
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
                    total_cycles += 1;
                }
                total_frames += 1;
                println!("{} frames", total_frames);
                for (x, y, pixel) in buffer.enumerate_pixels_mut() {
                    let color = display[y as usize][x as usize];
                    *pixel = image::Rgba([color[0], color[1], color[2], 255]);
                }

                // snapshot(display, total_cycles);

                texture.update(&mut texture_context, &buffer).unwrap();
                window.draw_2d(&event, |context, graphics, device| {
                    texture_context.encoder.flush(device);
                    clear([1.0; 4], graphics);
                    im_pis(&texture, context.transform.scale(scale, scale), graphics);
                });

                if total_frames > 20 {
                    if let Ok(report) = guard.report().build() {
                        let file = File::create("flamegraph.svg").unwrap();
                        let mut options = pprof::flamegraph::Options::default();
                        options.image_width = Some(2500);
                        report.flamegraph_with_options(file, &mut options).unwrap();
                    };
                    return;
                }
            }
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
    imgbuf.save(format!("a_{:0>10}.png", frame_count)).unwrap();
}
