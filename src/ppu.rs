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

    oam: OAM,
    secondary_oam: OAM,
    sprite_temporary_buffer: [u8; 256],
    next_line_sprite_temporary_buffer: [u8; 256],
    zero_sprite_temporary_buffer: [u8; 256],
    next_line_zero_sprite_temporary_buffer: [u8; 256],

    cycles: usize,
    scan_line: usize,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mapper: Mapper) -> Self {
        PPU {
            mapper,
            vram: RAM::new(vec![0; 0x4000]),
            palette_ram: RAM::new(vec![0; 0x20]),
            chr_rom,
            control: Control::default(),
            mask: Mask::default(),
            status: Status::default(),
            oam_address: OAMAddress::default(),
            oam_data: OAMData::default(),
            scroll: Scroll::default(),
            address: Address::default(),
            data: Data::default(),
            oam_dma: OAMDMA::default(),
            oam: OAM::new(64),
            secondary_oam: OAM::new(8),
            sprite_temporary_buffer: [0; 256],
            next_line_sprite_temporary_buffer: [0; 256],
            zero_sprite_temporary_buffer: [0; 256],
            next_line_zero_sprite_temporary_buffer: [0; 256],
            cycles: 0,
            scan_line: 0,
        }
    }

    pub fn step(&mut self, display: &mut [[[u8; 3]; 256]; 240], nmi: &mut bool) {
        self.render_pixel(display);
        self.evaluate_sprites();
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

    // ref: https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
    fn evaluate_sprites(&mut self) {
        if self.scan_line >= 240 {
            return;
        }

        // Cycles 1-64: Secondary OAM (32-byte buffer for current sprites on scanline) is initialized to $FF
        if self.cycles == 0 {
            self.secondary_oam.initialize();
        } else if self.cycles == 64 {
            // Cycles 65-256: Sprite evaluation
            self.next_line_sprite_temporary_buffer = [0; 256];
            self.next_line_zero_sprite_temporary_buffer = [0; 256];
            let mut found_count = 0;
            let mut sprites = Vec::with_capacity(8);
            for (id, &s) in self.oam.iter().enumerate() {
                if s.y >= 240 {
                    continue;
                }
                if ((s.y)..(s.y + 8)).contains(&(self.scan_line as u8)) {
                    self.secondary_oam.set_sprite(found_count, s.clone());
                    sprites.push((id, s));
                    found_count += 1;
                    if found_count == 7 {
                        self.status.sprite_overflow = true;
                        break;
                    }
                }
            }
            // rendering...
            for (id, s) in sprites.iter() {
                if s.tile_number == 0xFF {
                    println!("0");
                }
                let cs = self.get_specified_in_sprite_tile(s, self.scan_line - s.y as usize);

                for (i, &c) in cs.iter().enumerate() {
                    if s.x as usize + i >= 256 {
                        continue;
                    }
                    self.next_line_sprite_temporary_buffer[s.x as usize + i] = if c == 0 {
                        self.next_line_sprite_temporary_buffer[s.x as usize + i]
                    } else {
                        self.palette_ram
                            .read_byte(((s.attribute.palette + 4) * 4 + c) as usize)
                    };
                    if *id == 0 {
                        self.next_line_zero_sprite_temporary_buffer[s.x as usize + i] = c;
                    }
                }
            }
        } else if self.cycles == 257 {
            self.sprite_temporary_buffer
                .copy_from_slice(&self.next_line_sprite_temporary_buffer);
            self.zero_sprite_temporary_buffer
                .copy_from_slice(&self.next_line_zero_sprite_temporary_buffer);
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

        let sprite_c = self.sprite_temporary_buffer[x];
        let background_c = self.get_background_pixel(x, y);

        let c_byte = if sprite_c != 0 {
            sprite_c
        } else {
            background_c
        };

        let c = Color::from(c_byte);
        display[y][x][0] = c.0;
        display[y][x][1] = c.1;
        display[y][x][2] = c.2;

        // https://wiki.nesdev.com/w/index.php/PPU_OAM#Sprite_zero_hits
        // not (sprite 0 hit has already occurred this frame)
        if !self.status.sprite_zero_hit
            // not (At x=255)
            && !(x == 255)
            // not (If background or sprite rendering is disabled in PPUMASK ($2001))
            && self.mask.background && self.mask.sprite
            // opaque pixel of sprite 0 overlaps an opaque pixel of the background
            && self.zero_sprite_temporary_buffer[x] != 0
            // not (At x=0 to x=7 if the left-side clipping window is enabled (if bit 2 or bit 1 of PPUMASK is 0))
            && !((0..=7).contains(&x) && (self.mask.sprite_left_column || self.mask.background_left_column))
            // not (At any pixel where the background or sprite pixel is transparent)
            && !(background_c == 0)
        {
            self.status.set_zero_hit();
            // println!("HIT cycles: {}, scan: {}", self.cycles, self.scan_line);
            // println!("zero: {:?}", self.oam.get(0));
            // println!("zero buffer: {:?}", self.zero_sprite_temporary_buffer);
        }
    }

    fn get_background_pixel(&mut self, x: usize, y: usize) -> u8 {
        let mut nametable_address_offset = 0;
        let mut x = x + self.scroll.x as usize;
        let mut y = y + self.scroll.y as usize;
        if x > 0xFF {
            x %= 0x100;
            nametable_address_offset += 0x400;
        }
        // maybe ?
        if y > 0xEF {
            y %= 0xF0;
            nametable_address_offset += 0x800;
        }

        let nametable_number = x / 0x08 + (y / 0x08) * 0x20;
        let tile_address = self.get_mirrored_name_space_address(
            self.control.get_nametable_base_address() + nametable_address_offset + nametable_number,
        );
        let tile_number = self.vram[tile_address];

        let c = self.get_specified_in_tile(tile_number, x % 8, y % 8);
        let pal = self.get_palette_number(nametable_number, nametable_address_offset);

        if c == 0 {
            self.palette_ram.read_byte(0)
        } else {
            self.palette_ram.read_byte((pal * 4 + c) as usize) // maybe?
        }
    }

    #[allow(dead_code)]
    fn get_tile(&mut self, tile_number: u8) -> Vec<Vec<u8>> {
        let start_addr =
            self.control.get_background_pattern_table_base_address() + tile_number as usize * 0x10;
        let bytes = [start_addr]
            .iter()
            .cycle()
            .take(0x10)
            .enumerate()
            .map(|(i, addr)| self.read_byte(*addr as usize + i))
            .collect::<Vec<_>>();

        let mut tile = Vec::with_capacity(8);
        for i in 0..8 {
            let byte1 = bytes[i];
            let byte2 = bytes[i + 8];

            let mut one_bar = Vec::with_capacity(8);

            for j in 0..8 {
                one_bar
                    .push(u8::from(byte1 & (1 << j) != 0) + u8::from(byte2 & (1 << j) != 0) << 1);
            }
            tile.push(one_bar);
        }
        tile
    }

    // x: 0-7
    // y: 0-7
    fn get_specified_in_tile(&mut self, tile_number: u8, x: usize, y: usize) -> u8 {
        let start_addr =
            self.control.get_background_pattern_table_base_address() + tile_number as usize * 0x10;

        let byte1 = self.read_byte(start_addr + y);
        let byte2 = self.read_byte(start_addr + y + 8);

        u8::from(byte1 & (1 << (7 - x)) != 0) + (u8::from(byte2 & (1 << (7 - x)) != 0) << 1)
    }

    // y: 0-7
    fn get_specified_in_sprite_tile(&mut self, s: &Sprite, y: usize) -> [u8; 8] {
        let y = if s.attribute.vflip { 7 - y } else { y };

        let start_addr =
            self.control.get_sprites_pattern_table_base_address() + s.tile_number as usize * 0x10;

        let byte1 = self.read_byte(start_addr + y);
        let byte2 = self.read_byte(start_addr + y + 8);

        let mut pixels = [0; 8];
        for i in 0..pixels.len() {
            let x = if s.attribute.hflip { i } else { 7 - i };
            pixels[i] = u8::from(byte1 & (1 << x) != 0) + (u8::from(byte2 & (1 << x) != 0) << 1)
        }
        if s.tile_number == 0xFF {
            println!("y: {} {:?}", s.y + y as u8, &pixels);
        }
        pixels
    }

    fn get_palette_number(
        &mut self,
        nametable_number: usize,
        nametable_address_offset: usize,
    ) -> u8 {
        let attr_addr_lower = (nametable_number & 0x1F) / 4;
        let attr_addr_higher = (nametable_number / 0x20) / 4;
        let attr_addr = attr_addr_lower + attr_addr_higher * 8;
        let attr_address = self.get_mirrored_name_space_address(
            (attr_addr as usize)
                + self.control.get_nametable_base_address()
                + nametable_address_offset
                + 0x3C0,
        );
        let attr_byte = self.read_byte_from_nametable(attr_address);
        let low_addr = (nametable_number % 4) / 2;
        let high_addr = ((nametable_number / 0x20) % 4) / 2;
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

    pub fn oam_dma_status(&self) -> OAMDMAStatus {
        self.oam_dma.status
    }

    pub fn start_oam_dma(&mut self) {
        self.oam_dma.start();
    }

    pub fn oam_dma_write(&mut self, byte: u8) {
        self.oam.write_byte(
            (0x100 - self.oam_dma.remains + self.oam_address.addr as usize) % 0x100,
            byte,
        );
        self.oam_dma.tick();
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
            0x2004 => self
                .oam_data
                .read_byte(self.oam_address.addr as usize, &mut self.oam),
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
            0x2004 => self
                .oam_data
                .write_byte(self.oam_address.addr as usize, byte, &mut self.oam),
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

    pub fn get_nametable_base_address(&self) -> usize {
        match self.name_table {
            0b00 => 0x2000,
            0b01 => 0x2400,
            0b10 => 0x2800,
            0b11 => 0x2C00,
            _ => 0,
        }
    }

    pub fn get_sprites_pattern_table_base_address(&self) -> usize {
        if self.sprites_pattern_table {
            0x1000
        } else {
            0x0
        }
    }

    pub fn get_background_pattern_table_base_address(&self) -> usize {
        if self.background_pattern_table {
            0x1000
        } else {
            0x0
        }
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
        self.addr = byte;
    }
}

// $2004
#[derive(Default, Debug)]
struct OAMData {}

impl OAMData {
    pub fn write_byte(&mut self, address: usize, byte: u8, oam: &mut OAM) {
        oam.write_byte(address, byte);
    }

    pub fn read_byte(&mut self, address: usize, oam: &mut OAM) -> u8 {
        oam.read_byte(address)
    }
}

#[derive(Debug)]
enum ScrollNext {
    X,
    Y,
}

impl Default for ScrollNext {
    fn default() -> Self {
        ScrollNext::X
    }
}

// $2005
#[derive(Default, Debug)]
struct Scroll {
    x: u8,
    y: u8,
    next: ScrollNext,
}

impl Scroll {
    pub fn set_as_u8(&mut self, byte: u8) {
        match self.next {
            ScrollNext::X => {
                self.x = byte;
                self.next = ScrollNext::Y;
            }
            ScrollNext::Y => {
                self.y = byte;
                self.next = ScrollNext::X;
            }
        }
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

#[derive(Debug, Clone, Copy)]
pub enum OAMDMAStatus {
    NotRunning,
    Waiting,
    Running(usize),
}

impl Default for OAMDMAStatus {
    fn default() -> Self {
        Self::NotRunning
    }
}

// $4014
#[derive(Default, Debug)]
struct OAMDMA {
    status: OAMDMAStatus,
    current_addr: usize,
    remains: usize,
}

impl OAMDMA {
    fn write_byte(&mut self, byte: u8) {
        self.current_addr = (byte as usize) << 8;
        self.status = OAMDMAStatus::Waiting;
        self.remains = 0x100;
    }

    fn start(&mut self) {
        self.status = OAMDMAStatus::Running(self.current_addr);
    }

    fn tick(&mut self) {
        self.current_addr += 1;
        self.remains -= 1;
        if self.remains == 0 {
            self.status = OAMDMAStatus::NotRunning;
        } else {
            self.status = OAMDMAStatus::Running(self.current_addr);
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct Sprite {
    y: u8,
    tile_number: u8,
    attribute: SpriteAttribute,
    x: u8,
}

impl Sprite {
    fn set_y(&mut self, y: u8) {
        self.y = y;
    }

    fn set_tile_number(&mut self, tile_number: u8) {
        self.tile_number = tile_number;
    }

    fn set_attribute(&mut self, attribute: u8) {
        self.attribute.set_as_u8(attribute);
    }

    fn set_x(&mut self, x: u8) {
        self.x = x;
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct SpriteAttribute {
    vflip: bool,
    hflip: bool,
    priority: bool,
    palette: u8,
}

impl SpriteAttribute {
    fn set_as_u8(&mut self, byte: u8) {
        self.palette = byte & 0b11;
        self.priority = byte & 0b00100000 > 0;
        self.hflip = byte & 0b01000000 > 0;
        self.vflip = byte & 0b10000000 > 0;
    }

    fn get_as_u8(&mut self) -> u8 {
        self.palette
            + (u8::from(self.priority) << 5)
            + (u8::from(self.hflip) << 6)
            + (u8::from(self.vflip) << 7)
    }
}

#[derive(Debug, Clone)]
struct OAM {
    inner: Vec<Sprite>,
}

impl OAM {
    fn new(capacity: usize) -> Self {
        Self {
            inner: vec![Sprite::default(); capacity],
        }
    }

    fn initialize(&mut self) {
        for s in self.inner.iter_mut() {
            s.y = 0xFF;
            s.tile_number = 0xFF;
            s.attribute.set_as_u8(0xFF);
            s.x = 0xFF;
        }
    }
}

impl OAM {
    fn write_byte(&mut self, address: usize, byte: u8) {
        let idx = address / 4;
        match address % 4 {
            0 => {
                self.inner[idx].set_y(byte);
            }
            1 => {
                self.inner[idx].set_tile_number(byte);
            }
            2 => {
                self.inner[idx].set_attribute(byte);
            }
            3 => {
                self.inner[idx].set_x(byte);
            }
            _ => panic!("unreechable"),
        }
    }

    fn read_byte(&mut self, address: usize) -> u8 {
        let idx = address / 4;
        match address % 4 {
            0 => self.inner[idx].y,
            1 => self.inner[idx].tile_number,
            2 => self.inner[idx].attribute.get_as_u8(),
            3 => self.inner[idx].x,
            _ => panic!("unreechable"),
        }
    }

    fn get(&self, index: usize) -> &Sprite {
        &self.inner[index]
    }

    fn set_sprite(&mut self, index: usize, sprite: Sprite) {
        self.inner[index] = sprite;
    }

    fn iter<'a>(&'a self) -> IterOAM<'a> {
        IterOAM {
            inner: self.inner.iter(),
        }
    }
}

struct IterOAM<'a> {
    inner: std::slice::Iter<'a, Sprite>,
}

impl<'a> Iterator for IterOAM<'a> {
    type Item = &'a Sprite;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
