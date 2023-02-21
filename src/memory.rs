use std::default;

use crate::{
    cartridge::{Cartridge, NoCartridge},
    joypad::JoypadState,
    timer::Timer,
};

const RAM_SIZE: usize = 8_192;

pub struct Memory {
    ram: [u8; RAM_SIZE],
    cartridge: Box<dyn Cartridge>,
    joy: JoypadState,
    timer: Timer,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            ram: [0; RAM_SIZE],
            cartridge: Box::new(NoCartridge {}),
            joy: Default::default(),
            timer: Default::default(),
        }
    }
}

impl Memory {
    pub fn read8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cartridge.read_rom(addr),
            0xA000..=0xBFFF => self.cartridge.read_ram(addr),
            0xC000..=0xDFFF => self.ram[addr as usize & 0x0FFF],
            0xE000..=0xFDFF => self.ram[addr as usize & 0x0FFF],
            0xFF00 => self.joy.read(),
            0xFF04 => self.timer.read_div(),
            0xFF05 => self.timer.read_tima(),
            0xFF06 => self.timer.read_tma(),
            0xFF07 => self.timer.read_tac(),
            _ => 0xFF,
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.write_rom(addr, val),
            0xA000..=0xBFFF => self.cartridge.write_ram(addr, val),
            0xC000..=0xDFFF => self.ram[addr as usize & 0x0FFF] = val,
            0xE000..=0xFDFF => self.ram[addr as usize & 0x0FFF] = val,
            0xFF00 => self.joy.write(val),
            0xFF04 => self.timer.write_div(val),
            0xFF05 => self.timer.write_tima(val),
            0xFF06 => self.timer.write_tma(val),
            0xFF07 => self.timer.write_tac(val),
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
}
