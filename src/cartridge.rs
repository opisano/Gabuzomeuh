


const TITLE_START: u16 = 0x0134;
const CGB_FLAG_ADDRESS: u16 = 0x0143;
const CARTRIDGE_TYPE: u16 = 0x0147;


pub enum Mapper {
    NoMapper,
    MBC1,
    MBC2,
    MBC3,
    MBC4,
    MBC5,
    MBC6,
    MBC7,
    MMM01,
    M161,
    HuC1,
    HuC3,
    Other
}


/// Common interface for interacting with cartridges
pub trait Cartridge {

    /// Read at a specified address in ROM
    fn read_rom(&self, addr: u16) -> u8;

    /// Read at a specified address in RAM
    fn read_ram(&self, addr: u16) -> u8;

    /// Write value at specified address in ROM
    fn write_rom(&mut self, addr: u16, val: u8);

    /// Write value at specified address in RAM
    fn write_ram(&mut self, addr: u16, val: u8);


    fn title(&self) -> String {
        let title_size: u16 = 16;
        let mut result = String::with_capacity(title_size as usize);

        for i in 0..title_size {
            let c: u8 = self.read_rom((TITLE_START + i) as u16);
            match c {
                32..=0x7F => result.push(c as char),
                _ => break,

            }
        }
        result
    }

    fn mapper(&self) -> Mapper {
        let value = self.read_rom(CARTRIDGE_TYPE);

        match value {
            0|8|9       => Mapper::NoMapper,
            1..=3       => Mapper::MBC1,
            5|6         => Mapper::MBC2,
            0xb..=0xd   => Mapper::MMM01,
            0x0F..=0x13 => Mapper::MBC3,
            0x19..=0x1E => Mapper::MBC5,
            0x20        => Mapper::MBC6,
            0x22        => Mapper::MBC7,
            0xFE        => Mapper::HuC3,
            0xFF        => Mapper::HuC1,
            _           => Mapper::Other
        }
    }

}


///
/// Small games of not more than 32KBytes ROM do not require a MBC chip for ROM
///  banking. The ROM is directly mapped to memory at 0000-7FFFh. 
/// 
struct NoMapperCartridge {
    data: Vec<u8>
}

impl Cartridge for NoMapperCartridge {
    fn read_rom(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn read_ram(&self, addr: u16) -> u8 {
        0
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        ()
    }

    fn write_ram(&mut self, addr: u16, val: u8) {
        ()
    }
}