use crate::{
    cartridge::{Cartridge, NoCartridge},
    joypad::JoypadState,
    ppu::Ppu,
    timer::Timer,
};

const RAM_SIZE: usize = 8_192;
const HRAM_SIZE: usize = 127;

pub struct Memory {
    ram: [u8; RAM_SIZE],
    hram: [u8; HRAM_SIZE],
    cartridge: Box<dyn Cartridge>,
    joy: JoypadState,
    timer: Timer,
    ppu: Ppu,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            ram: [0; RAM_SIZE],
            hram: [0; HRAM_SIZE],
            cartridge: Box::new(NoCartridge {}),
            joy: Default::default(),
            timer: Default::default(),
            ppu: Default::default(),
        }
    }
}

impl Memory {
    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cartridge.read_rom(addr),
            0x8000..=0x9FFF => self.ppu.read_vram(addr),
            0xA000..=0xBFFF => self.cartridge.read_ram(addr),
            0xC000..=0xDFFF => self.ram[addr as usize & 0x0FFF],
            0xE000..=0xFDFF => self.ram[addr as usize & 0x0FFF],
            0xFE00..=0xFE9F => self.ppu.read_oam(addr),
            0xFF00 => self.joy.read(),
            0xFF04 => self.timer.read_div(),
            0xFF05 => self.timer.read_tima(),
            0xFF06 => self.timer.read_tma(),
            0xFF07 => self.timer.read_tac(),
            0xFF40 => self.ppu.read_control(),
            0xFF41 => self.ppu.read_status(),
            0xFF42 => self.ppu.read_scy(),
            0xFF43 => self.ppu.read_scx(),
            0xFF44 => self.ppu.read_ly(),
            0xFF45 => self.ppu.read_lyc(),
            0xFF46 => 0, // OAM DMA register is write-only
            0xFF47 => self.ppu.read_bgp(),
            0xFF48 => self.ppu.read_obp0(),
            0xFF49 => self.ppu.read_obp1(),
            0xFF4A => self.ppu.read_wy(),
            0xFF4B => self.ppu.read_wx(),
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
            _ => 0xFF,
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.write_rom(addr, val),
            0x8000..=0x9FFF => self.ppu.write_vram(addr, val),
            0xA000..=0xBFFF => self.cartridge.write_ram(addr, val),
            0xC000..=0xDFFF => self.ram[addr as usize & 0x0FFF] = val,
            0xE000..=0xFDFF => self.ram[addr as usize & 0x0FFF] = val,
            0xFE00..=0xFE9F => self.ppu.write_oam(addr, val),
            0xFF00 => self.joy.write(val),
            0xFF04 => self.timer.write_div(val),
            0xFF05 => self.timer.write_tima(val),
            0xFF06 => self.timer.write_tma(val),
            0xFF07 => self.timer.write_tac(val),
            0xFF40 => self.ppu.write_control(val),
            0xFF41 => self.ppu.write_status(val),
            0xFF42 => self.ppu.write_scy(val),
            0xFF43 => self.ppu.write_scx(val),
            0xFF45 => self.ppu.write_lyc(val),
            0xFF46 => self.oam_dma(val),
            0xFF47 => self.ppu.write_bgp(val),
            0xFF48 => self.ppu.write_obp0(val),
            0xFF49 => self.ppu.write_obp1(val),
            0xFF4A => self.ppu.write_wy(val),
            0xFF4B => self.ppu.write_wx(val),
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80] = val,
            _ => (),
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let lb = self.read8(addr) as u16;
        let hb = self.read8(addr + 1) as u16;
        (hb << 8) | lb
    }

    pub fn write16(&mut self, addr: u16, val: u16) {
        let lb = val as u8;
        let hb = (val >> 8) as u8;

        self.write8(addr, lb);
        self.write8(addr + 1, hb);
    }

    fn oam_dma(&mut self, value: u8) {
        let source_address = (value as u16) << 8;
        for offset in 0..0xA0 {
            let byte = self.read8(source_address + offset);
            self.write8(0xFE00 + offset, byte);
        }
    }
}
