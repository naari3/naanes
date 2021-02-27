#[derive(Debug, Clone, Copy)]
pub enum Mapper {
    NRom(NRomMapper),
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
    fn get_nametable_mirroring_type(&self) -> Mirroring {
        self.nametable_mirroring_type
    }

    fn mapping_address(&self, address: usize) -> usize {
        let mut address = address - 0x8000;
        if self.prg_units == 1 {
            address -= 0x4000;
        }
        address
    }
}
