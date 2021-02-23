use emu6502::ram::MemIO;

#[derive(Default)]
pub struct PPU {
    chr_rom: Vec<u8>,
    status: Status,
}

impl PPU {
    pub fn step(&mut self) {}
    pub fn set_rom(&mut self, rom: Vec<u8>) {
        self.chr_rom = rom;
    }
}

impl MemIO for PPU {
    fn read_byte(&mut self, address: usize) -> u8 {
        match address {
            0x2002 => self.status.get_as_u8(),
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: usize, _byte: u8) {
        match address {
            _ => {}
        }
    }
}

#[derive(Default)]
struct Status {
    vblank: bool,
    sprite_zero_hit: bool,
    sprite_overflow: bool,
}

impl Status {
    pub fn get_as_u8(&self) -> u8 {
        (self.vblank as u8) << 6
            | (self.sprite_zero_hit as u8) << 5
            | (self.sprite_overflow as u8) << 4
    }
}
