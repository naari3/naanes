use emu6502::ram::{MemIO, RAM};
use emu6502::reset::Reset;

use crate::{mapper::Mapper, ppu::PPU};

pub struct Bus<'a> {
    wram: &'a mut RAM,
    prg_rom: Vec<u8>,
    ppu: &'a mut PPU,
    mapper: Mapper,
    input: u8,
}

impl<'a> Bus<'a> {
    pub fn new(
        wram: &'a mut RAM,
        ppu: &'a mut PPU,
        prg_rom: Vec<u8>,
        mapper: Mapper,
        input: u8,
    ) -> Bus<'a> {
        Bus {
            wram,
            prg_rom,
            ppu,
            mapper,
            input,
        }
    }
}

impl<'a> MemIO for Bus<'a> {
    fn read_byte(&mut self, address: usize) -> u8 {
        match address {
            0x0000..=0x07FF => self.wram.read_byte(address),
            0x0800..=0x1FFF => self.wram.read_byte(address & 0x07FF),
            0x2000..=0x2007 => self.ppu.read_byte(address),
            0x8000..=0xFFFF => {
                let a = self.mapper.mapping_address(address);
                self.prg_rom[a]
            }
            _ => 0,
        }
    }

    fn read_byte_without_effect(&mut self, address: usize) -> u8 {
        match address {
            0x0000..=0x07FF => self.wram.read_byte_without_effect(address),
            0x0800..=0x1FFF => self.wram.read_byte_without_effect(address & 0x07FF),
            0x2000..=0x2007 => self.ppu.read_byte_without_effect(address),
            0x8000..=0xFFFF => {
                let a = self.mapper.mapping_address(address);
                self.prg_rom[a]
            }
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: usize, byte: u8) {
        match address {
            0x0000..=0x07FF => self.wram.write_byte(address, byte),
            0x0800..=0x1FFF => self.wram.write_byte(address & 0x07FF, byte),
            0x2000..=0x2007 => self.ppu.write_byte(address, byte),
            0x8000..=0xFFFF => {}
            _ => {}
        }
    }
}

impl<'a> Reset for Bus<'a> {
    fn reset(&mut self) {
        self.wram.reset();
    }
}
