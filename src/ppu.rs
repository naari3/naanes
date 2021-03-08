use emu6502::ram::{MemIO, RAM};

use crate::color::Color;
use crate::mapper::{Mapper, Mirroring};

#[derive(Debug)]
pub struct PPU {
    mapper: Mapper,

    vram: RAM,
    palette_ram: RAM,
    chr_rom: Vec<u8>,
    control: Control,        // $2000
    mask: Mask,              // $2001
    status: Status,          // $2002
    oam_address: OAMAddress, // $2003
    oam_data: OAMData,       // $2004
    scroll: Scroll,          // $2005
    address: Address,        // $2006
    data: Data,              // $2007

    oam_dma: OAMDMA, // $4014

    cycles: usize,
    scan_line: usize,
}

impl PPU {
    pub fn new(mapper: Mapper) -> Self {
        PPU {
            mapper,
            vram: RAM::new(vec![0; 0x4000]),
            palette_ram: RAM::new(vec![0; 0x20]),
            chr_rom: vec![],
            control: Control::default(),
            mask: Mask::default(),
            status: Status::default(),
            oam_address: OAMAddress::default(),
            oam_data: OAMData::default(),
            scroll: Scroll::default(),
            address: Address::default(),
            data: Data::default(),
            oam_dma: OAMDMA::default(),
            cycles: 0,
            scan_line: 0,
        }
    }

    pub fn step(&mut self, display: &mut [[[u8; 3]; 256]; 240], nmi: &mut bool) {
        self.render_pixel(display);
        self.update_status(display, nmi);
        self.tick();
    }

    fn tick(&mut self) {
        self.cycles += 1;
        if self.cycles > 340 {
            self.cycles = 0;
            self.scan_line += 1;

            // maybe set frame increments near by scan_line == 240

            if self.scan_line > 261 {
                self.scan_line = 0;
            }
        }
    }

    fn update_status(&mut self, _display: &mut [[[u8; 3]; 256]; 240], nmi: &mut bool) {
        // at leach new scan line...
        if self.cycles == 1 {
            if self.scan_line == 241 {
                self.status.set_vblank();
                if self.control.nmi_vblank {
                    *nmi = true;
                }
            } else if self.scan_line == 261 {
                self.status.clear_vblank();
                self.status.clear_zero_hit();
            }
        }
    }

    fn render_pixel(&mut self, display: &mut [[[u8; 3]; 256]; 240]) {
        if self.cycles >= 257 || self.scan_line >= 240 || self.cycles == 0 {
            return;
        }

        let x = self.cycles - 1;
        let y = self.scan_line;

        let c = self.get_background_pixel(x, y);
        let c = Color::from(c);
        display[y][x][0] = c.0;
        display[y][x][1] = c.1;
        display[y][x][2] = c.2;
    }

    fn get_background_pixel(&mut self, x: usize, y: usize) -> u8 {
        // it can be cached
        let tile_number = self.vram[0x2000 + x / 8 + (y / 8) * 0x20];
        let tile = self.get_tile(tile_number);
        let pal = self.get_palette_number((x / 8 + (y / 8) * 0x20) as u8);
        let c = tile[y % 8][7 - (x % 8)];

        if c == 0 {
            self.palette_ram.read_byte(0)
        } else {
            self.palette_ram.read_byte((pal * 3 + c) as usize)
        }
    }

    fn get_tile(&mut self, tile_number: u8) -> Vec<Vec<u8>> {
        let start_addr = tile_number as usize * 0x10;
        let bytes = [start_addr]
            .iter()
            .cycle()
            .take(0x10)
            .enumerate()
            .map(|(i, addr)| self.read_byte(*addr as usize + i))
            .collect::<Vec<_>>();
        let to_bits = |uint| {
            [()].iter()
                .cycle()
                .take(8)
                .enumerate()
                .map(|(j, _)| uint >> j & 1 == 1)
                .collect::<Vec<_>>()
        };
        let mut tile = Vec::new();
        for i in 0..8 {
            let one_bar = to_bits(bytes[i])
                .iter()
                .zip(to_bits(bytes[i + 8]))
                .map(|(b1, b2)| *b1 as u8 + ((b2 as u8) << 1))
                .collect::<Vec<_>>();
            tile.push(one_bar);
        }
        tile
    }

    fn get_palette_number(&mut self, nametable_number: u8) -> u8 {
        let attr_addr_lower = nametable_number / 2;
        let attr_addr_higher = (nametable_number / 0x10) / 2;
        let attr_addr = attr_addr_lower + attr_addr_higher * 8;
        if attr_addr != 0 && attr_addr != 0x01 {
            println!(
                "attr_addr: 0x{:02X}, nametable_number: 0x{:02X}",
                attr_addr, nametable_number
            );
        }
        let attr_byte = self.palette_ram.read_byte(attr_addr as usize);
        let low_addr = nametable_number % 2;
        let high_addr = (nametable_number / 0x10) % 2;
        let specific_bits = low_addr + high_addr * 2;

        match specific_bits {
            0 => attr_byte & 0b00000011,
            1 => (attr_byte & 0b00001100) >> 2,
            2 => (attr_byte & 0b00110000) >> 4,
            3 => (attr_byte & 0b11000000) >> 6,
            _ => panic!("wrong calc!"),
        }
    }

    pub fn set_rom(&mut self, rom: Vec<u8>, mapper: Mapper) {
        self.chr_rom = rom;
        self.mapper = mapper;
    }

    fn write_byte_to_nametable(&mut self, address: usize, byte: u8) {
        let addr = self.get_mirrored_name_space_address(address);
        self.vram.write_byte(addr, byte);
    }

    fn read_byte_from_nametable(&mut self, address: usize) -> u8 {
        let addr = self.get_mirrored_name_space_address(address);
        self.vram.read_byte(addr)
    }

    fn get_mirrored_name_space_address(&mut self, address: usize) -> usize {
        // TODO: support to another mirroring by mapper
        // see also https://wiki.nesdev.com/w/index.php/Mirroring
        match self.mapper.get_nametable_mirroring_type() {
            Mirroring::Horizontal => match address {
                0x2000..=0x23FF => address,
                0x2400..=0x27FF => address - 0x400, // mirror
                0x2800..=0x2BFF => address,
                0x2C00..=0x2FFF => address - 0x400, // mirror
                _ => {
                    panic!("out of index: {:x}", address)
                }
            },
            Mirroring::Vertical => match address {
                0x2000..=0x23FF => address,
                0x2400..=0x27FF => address,
                0x2800..=0x2BFF => address - 0x400, // mirror
                0x2C00..=0x2FFF => address - 0x400, // mirror
                _ => {
                    panic!("out of index: {:x}", address)
                }
            },
        }
    }
}

impl MemIO for PPU {
    fn read_byte(&mut self, address: usize) -> u8 {
        match address {
            0x0000..=0x1FFF => self.chr_rom[address],
            0x2002 => {
                let byte = self.status.get_as_u8();
                self.status.clear_vblank();
                byte
            }
            0x2007 => {
                let mut addr = self.address.get() as usize & 0x3FFF;
                if addr >= 0x3000 && addr <= 0x3EFF {
                    addr -= 0x1000;
                }
                let byte = match address {
                    // https://wiki.nesdev.com/w/index.php/PPU_memory_map
                    0x0000..=0x0FFF => self.chr_rom[address],
                    0x1000..=0x1FFF => self.chr_rom[address],
                    0x2000..=0x2FFF => self.read_byte_from_nametable(address),
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

    fn read_byte_without_effect(&mut self, address: usize) -> u8 {
        match address {
            0x0000..=0x1FFF => self.chr_rom[address],
            0x2002 => self.status.get_as_u8(),
            0x2007 => {
                let mut addr = self.address.get() as usize & 0x3FFF;
                if addr >= 0x3000 && addr <= 0x3EFF {
                    addr -= 0x1000;
                }
                let byte = match address {
                    // https://wiki.nesdev.com/w/index.php/PPU_memory_map
                    0x0000..=0x0FFF => self.chr_rom[address],
                    0x1000..=0x1FFF => self.chr_rom[address],
                    0x2000..=0x2FFF => self.read_byte_from_nametable(address),
                    0x3F00..=0x3F1F => self.palette_ram.read_byte(addr - 0x3F00),
                    0x3F20..=0x3FFF => self.palette_ram.read_byte((addr - 0x3F20) & 0x1F),
                    _ => 0,
                };
                byte
            }
            _ => 0,
        }
    }

    fn write_byte(&mut self, address: usize, byte: u8) {
        match address {
            0x2000 => self.control.set_as_u8(byte),
            0x2001 => self.mask.set_as_u8(byte),
            0x2003 => self.oam_address.write_byte(byte),
            0x2004 => self.oam_data.write_byte(byte),
            0x2005 => self.scroll.set_as_u8(byte),
            0x2006 => self.address.set_as_u8(byte),
            0x2007 => {
                let mut addr = self.address.get() as usize & 0x3FFF;
                if addr >= 0x3000 && addr <= 0x3EFF {
                    addr -= 0x1000;
                }
                match addr {
                    // https://wiki.nesdev.com/w/index.php/PPU_memory_map
                    0x0000..=0x0FFF => {}
                    0x1000..=0x1FFF => {}
                    0x2000..=0x2FFF => {
                        self.write_byte_to_nametable(addr, byte);
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
            0x4014 => {
                self.oam_dma.write_byte(byte);
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
        (self.vblank as u8) << 7
            | (self.sprite_zero_hit as u8) << 6
            | (self.sprite_overflow as u8) << 5
    }

    fn set_vblank(&mut self) {
        self.vblank = true;
    }

    fn clear_vblank(&mut self) {
        self.vblank = false;
    }

    fn set_zero_hit(&mut self) {
        self.sprite_zero_hit = true;
    }

    fn clear_zero_hit(&mut self) {
        self.sprite_zero_hit = false;
    }
}

// $2003
#[derive(Default, Debug)]
struct OAMAddress {
    addr: u8,
}

impl OAMAddress {
    pub fn write_byte(&mut self, byte: u8) {
        println!("OAMAddress byte: 0x{:02X}", byte);
        self.addr = byte;
    }
}

// $2004
#[derive(Default, Debug)]
struct OAMData {}

impl OAMData {
    pub fn write_byte(&mut self, byte: u8) {
        println!("OAMData byte: 0x{:02X}", byte);
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
            self.addr = (byte as u16) << 8;
        } else {
            self.addr += byte as u16;
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

// $4014
#[derive(Default, Debug)]
struct OAMDMA {}

impl OAMDMA {
    pub fn write_byte(&mut self, byte: u8) {
        println!("OAMDMA byte: 0x{:02X}", byte);
    }
}
