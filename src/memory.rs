use crate::cartridge::Cartridge;



const RAM_SIZE: usize = 8_192;

struct Memory {
    ram: [u8; RAM_SIZE],
    cartridge: Box<dyn Cartridge>
}

impl Memory {

    fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x7FFF => self.cartridge.read_rom(addr),
            0xA000 ..= 0xBFFF => self.cartridge.read_ram(addr),
            0xC000 ..= 0xDFFF => self.ram[addr as usize & 0x0FFF],
            0xE000 ..= 0xFDFF => self.ram[addr as usize & 0x0FFF],
            _ => 0xFF
        }
    }

    fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000 ..= 0x7FFF => self.cartridge.write_rom(addr, val),
            0xA000 ..= 0xBFFF => self.cartridge.write_ram(addr, val),
            0xC000 ..= 0xDFFF => self.ram[addr as usize & 0x0FFF] = val,
            0xE000 ..= 0xFDFF => self.ram[addr as usize & 0x0FFF] = val,
            _ => ()
        }
    }

    fn read16(&self, addr: u16) -> u16 {
        let lb = self.read8(addr) as u16;
        let hb = self.read8(addr+1) as u16;
        (hb << 8) | lb
    }

    fn write16(&mut self, addr: u16, val: u16) {
        let lb = val as u8;
        let hb = (val >> 8) as u8;

        self.write8(addr, lb);
        self.write8(addr+1, hb);
    }

}

