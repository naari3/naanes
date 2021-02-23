use emu6502::ram::{MemIO, RAM};
use emu6502::reset::Reset;

pub struct Bus {
    ram: RAM,
    rom: Vec<u8>,
}

impl Bus {
    pub fn set_rom(&mut self, rom: Vec<u8>) {
        self.rom = rom;
    }
}

impl Default for Bus {
    fn default() -> Self {
        Bus {
            ram: RAM::default(),
            rom: vec![0; 0x8000],
        }
    }
}

impl MemIO for Bus {
    fn read_byte(&mut self, address: usize) -> u8 {
        match address {
            0x0000..=0x07FF => self.ram.read_byte(address),
            0x0800..=0x1FFF => self.ram.read_byte(address & 0x07FF),
            0x8000..=0xFFFF => self.rom[address - 0x8000],
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: usize, byte: u8) {
        match address {
            0x0000..=0x07FF => self.ram.write_byte(address, byte),
            0x0800..=0x1FFF => self.ram.write_byte(address & 0x07FF, byte),
            0x8000..=0xFFFF => {}
            _ => {}
        }
    }
}

impl Reset for Bus {
    fn reset(&mut self) {
        self.ram.reset();
    }
}
