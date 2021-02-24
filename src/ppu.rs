use std::usize;

use emu6502::ram::{MemIO, RAM};

#[derive(Debug)]
pub struct PPU {
    vram: RAM,
    palette_ram: RAM,
    chr_rom: Vec<u8>,
    control: Control, // $2000
    mask: Mask,       // $2001
    status: Status,   // $2002
    // $2003
    // $2004
    scroll: Scroll,   // $2005
    address: Address, // $2006
    data: Data,       // $2007
}

impl Default for PPU {
    fn default() -> Self {
        PPU {
            vram: RAM::new(vec![0; 0x4000]),
            palette_ram: RAM::new(vec![0; 0x20]),
            chr_rom: vec![],
            control: Control::default(),
            mask: Mask::default(),
            status: Status::default(),
            scroll: Scroll::default(),
            address: Address::default(),
            data: Data::default(),
        }
    }
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
            0x0000..=0x1FFF => self.chr_rom[address],
            0x2002 => self.status.get_as_u8(),
            0x2007 => {
                let mut addr = self.address.get() as usize & 0x3FFF;
                println!("addr: 0x{:x}", addr);
                if addr >= 0x3000 && addr <= 0x3EFF {
                    addr -= 0x1000;
                }
                let byte = match address {
                    // https://wiki.nesdev.com/w/index.php/PPU_memory_map
                    0x0000..=0x0FFF => self.chr_rom[address],
                    0x1000..=0x1FFF => self.chr_rom[address],
                    0x2000..=0x23FF => self.vram.read_byte(addr),
                    0x2400..=0x27FF => self.vram.read_byte(addr),
                    0x2800..=0x2BFF => self.vram.read_byte(addr - 0x400),
                    0x2C00..=0x2FFF => self.vram.read_byte(addr - 0x400),
                    0x3F00..=0x3F1F => self.palette_ram.read_byte(addr - 0x3F00),
                    0x3F20..=0x3FFF => self.palette_ram.read_byte((addr - 0x3F20) & 0x1F),
                    _ => 0,
                };
                self.address
                    .increment_address(self.control.increment_address);
                byte
            }
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: usize, byte: u8) {
        match address {
            0x2000 => self.control.set_as_u8(byte),
            0x2001 => self.mask.set_as_u8(byte),
            0x2005 => self.scroll.set_as_u8(byte),
            0x2006 => self.address.set_as_u8(byte),
            0x2007 => {
                let mut addr = self.address.get() as usize & 0x3FFF;
                println!("addr: 0x{:x}", addr);
                if addr >= 0x3000 && addr <= 0x3EFF {
                    addr -= 0x1000;
                }
                match addr {
                    // https://wiki.nesdev.com/w/index.php/PPU_memory_map
                    0x0000..=0x0FFF => {}
                    0x1000..=0x1FFF => {}
                    0x2000..=0x23FF => {
                        self.vram.write_byte(addr, byte);
                    }
                    0x2400..=0x27FF => {
                        self.vram.write_byte(addr, byte);
                    }
                    0x2800..=0x2BFF => {
                        self.vram.write_byte(addr - 0x400, byte);
                    }
                    0x2C00..=0x2FFF => {
                        self.vram.write_byte(addr - 0x400, byte);
                    }
                    0x3F00..=0x3F1F => {
                        self.palette_ram.write_byte(addr - 0x3F00, byte);
                    }
                    0x3F20..=0x3FFF => {
                        self.palette_ram.write_byte((addr - 0x3F20) & 0x1F, byte);
                    }
                    _ => {}
                };
                self.address
                    .increment_address(self.control.increment_address);
            }
            _ => {}
        }
    }
}

// $2000
#[derive(Default, Debug)]
struct Control {
    name_table: u8,                 // 0-1
    increment_address: bool,        // 2
    sprites_pattern_table: bool,    // 3
    background_pattern_table: bool, // 4
    sprites_size: bool,             // 5
    master_slave: bool,             // 6
    nmi_vblank: bool,               // 7
}

impl Control {
    pub fn set_as_u8(&mut self, byte: u8) {
        self.name_table = byte & 0b00000011;
        self.increment_address = byte & 0b00000100 > 0;
        self.sprites_pattern_table = byte & 0b00001000 > 0;
        self.background_pattern_table = byte & 0b00010000 > 0;
        self.sprites_size = byte & 0b00100000 > 0;
        self.master_slave = byte & 0b01000000 > 0;
        self.nmi_vblank = byte & 0b10000000 > 0;
    }
}

// $2001
#[derive(Default, Debug)]
struct Mask {
    greyscale: bool,              // 0
    background_left_column: bool, // 1
    sprite_left_column: bool,     // 2
    background: bool,             // 3
    sprite: bool,                 // 4
    emphasis_red: bool,           // 5
    emphasis_green: bool,         // 6
    emphasis_blue: bool,          // 7
}

impl Mask {
    pub fn set_as_u8(&mut self, byte: u8) {
        self.greyscale = byte & 0b00000001 > 0;
        self.background_left_column = byte & 0b00000010 > 0;
        self.sprite_left_column = byte & 0b00000100 > 0;
        self.background = byte & 0b00001000 > 0;
        self.sprite = byte & 0b00010000 > 0;
        self.emphasis_red = byte & 0b00100000 > 0;
        self.emphasis_green = byte & 0b01000000 > 0;
        self.emphasis_blue = byte & 0b10000000 > 0;
    }
}

// $2002
#[derive(Default, Debug)]
struct Status {
    sprite_overflow: bool, // 5
    sprite_zero_hit: bool, // 6
    vblank: bool,          // 7
}

impl Status {
    pub fn get_as_u8(&self) -> u8 {
        (self.vblank as u8) << 6
            | (self.sprite_zero_hit as u8) << 5
            | (self.sprite_overflow as u8) << 4
    }
}

// $2005
#[derive(Default, Debug)]
struct Scroll {
    x: u8,
    y: u8,
    is_stored_first: bool,
}

impl Scroll {
    pub fn set_as_u8(&mut self, byte: u8) {
        if self.is_stored_first {
            self.x = byte;
        } else {
            self.y = byte;
        }
        self.is_stored_first = !self.is_stored_first;
    }
}

// $2006
#[derive(Default, Debug)]
struct Address {
    addr: u16,
    is_stored_first: bool,
}

impl Address {
    pub fn set_as_u8(&mut self, byte: u8) {
        if !self.is_stored_first {
            self.addr = byte as u16;
        } else {
            self.addr += (byte as u16) << 8;
        }
        self.is_stored_first = !self.is_stored_first;
    }

    pub fn increment_address(&mut self, large_increment: bool) {
        self.addr += if large_increment { 32 } else { 1 }
    }

    pub fn get(&self) -> u16 {
        self.addr
    }
}

// $2007
#[derive(Default, Debug)]
struct Data {
    addr: u16,
}
