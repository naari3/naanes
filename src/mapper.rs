#[derive(Debug, Clone, Copy)]
pub enum Mapper {
    NRom(NRomMapper),
}

impl Mapper {
    pub fn get_nametable_mirroring_type(&self) -> Mirroring {
        match self {
            Mapper::NRom(m) => m.get_nametable_mirroring_type(),
        }
    }

    pub fn mapping_address(&self, address: usize) -> usize {
        match self {
            Mapper::NRom(m) => m.mapping_address(address),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mirroring {
    Horizontal,
    Vertical,
}

pub trait Mapping {
    fn get_nametable_mirroring_type(&self) -> Mirroring;
    fn mapping_address(&self, address: usize) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct NRomMapper {
    prg_units: usize, // 1 or 2
    nametable_mirroring_type: Mirroring,
}

impl NRomMapper {
    pub fn new(prg_units: usize, nametable_mirroring_type: Mirroring) -> Self {
        Self {
            prg_units,
            nametable_mirroring_type,
        }
    }
}

impl Mapping for NRomMapper {
    // https://wiki.nesdev.com/w/index.php/NROM
    fn get_nametable_mirroring_type(&self) -> Mirroring {
        self.nametable_mirroring_type
    }

    fn mapping_address(&self, address: usize) -> usize {
        let mut address = address;
        if self.prg_units == 1 && address >= 0xC000 {
            address -= 0x4000;
        }
        address -= 0x8000;
        address
    }
}
