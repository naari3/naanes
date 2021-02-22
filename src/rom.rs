#[derive(Debug)]
pub struct ROM {
    pub prg: Vec<u8>,
    pub chr: Vec<u8>,
}

pub fn parse(rom_buffer: Vec<u8>) -> ROM {
    let prg_unit_count = rom_buffer[4] as usize;
    let chr_unit_count = rom_buffer[5] as usize;

    println!("prg units: {}", prg_unit_count);
    println!("chr units: {}", chr_unit_count);

    let prg_start = 16;
    let chr_start = prg_start + 1024 * 16 * prg_unit_count;
    let chr_end = chr_start + 1024 * 8 * chr_unit_count;

    ROM {
        prg: rom_buffer[prg_start..chr_start].to_vec(),
        chr: rom_buffer[chr_start..chr_end].to_vec(),
    }
}
