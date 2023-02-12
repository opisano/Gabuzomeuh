use std::{default, rc::Rc};

use crate::memory::Memory;

#[derive(Default)]
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

const FLAG_ZERO: u8 = 0x80;
const FLAG_SUB: u8 = 0x40;
const FLAG_HALF: u8 = 0x20;
const FLAG_CARRY: u8 = 0x10;

impl Registers {
    /// Combine a 16 bit value write to registers A and F
    ///
    /// A := val[8..16]
    /// F := val[0..7]
    ///
    fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0xF0) as u8;
    }

    /// Combine a 16 bit value write to registers B and C
    ///
    /// B := val[8..16]
    /// C := val[0..7]
    ///
    fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = val as u8;
    }

    /// Combine a 16 bit value write to registers D and E
    ///
    /// D := val[8..16]
    /// E := val[0..7]
    ///
    fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = val as u8;
    }

    /// Combine a 16 bit value write to registers H and L
    ///
    /// H := val[8..16]
    /// L := val[0..7]
    ///
    fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = val as u8;
    }

    /// Combine a 16 bit value read from registers A and F
    ///
    /// Returns A << 8 | F
    ///
    fn get_af(&self) -> u16 {
        (self.f as u16) | ((self.a as u16) << 8)
    }

    /// Combine a 16 bit value read from registers B and C
    ///
    /// Returns B << 8 | C
    ///
    fn get_bc(&self) -> u16 {
        (self.c as u16) | ((self.b as u16) << 8)
    }

    /// Combine a 16 bit value read from registers D and E
    ///
    /// Returns D << 8 | E
    ///
    fn get_de(&self) -> u16 {
        (self.e as u16) | ((self.d as u16) << 8)
    }

    /// Combine a 16 bit value read from registers H and L
    ///
    /// Returns H << 8 | L
    ///
    fn get_hl(&self) -> u16 {
        (self.l as u16) | ((self.h as u16) << 8)
    }

    /// Toggle a flag
    fn toggle_flag(&mut self, flag: u8) {
        self.f |= flag
    }

    /// Clear a flag
    fn clear_flag(&mut self, flag: u8) {
        self.f &= !flag;
    }

    /// Toggle the zero flag
    fn toggle_zero_flag(&mut self, value: u8) {
        if value == 0 {
            self.toggle_flag(FLAG_ZERO);
        }
    }

    /// Toggle the carry flag
    fn toggle_carry_flag(&mut self, value: u32) {
        if (value & 0x100) != 0 {
            self.toggle_flag(FLAG_CARRY);
        }
    }

    /// Toggle the half flag
    fn toggle_half_flag(&mut self, value: u32) {
        if (value & 0x10) != 0 {
            self.toggle_flag(FLAG_HALF);
        }
    }
}

#[test]
fn test_get_af() {
    let regs = Registers {
        a: 0x55,
        f: 0xA0,
        ..Default::default()
    };

    assert_eq!(regs.get_af(), 0x55A0);
}

#[test]
fn test_set_af() {
    let mut regs: Registers = Default::default();
    regs.set_af(0xAA55);
    assert_eq!(regs.a, 0xAA);
    assert_eq!(regs.f, 0x50);
}

#[test]
fn test_get_bc() {
    let regs = Registers {
        b: 0x55,
        c: 0xAA,
        ..Default::default()
    };
    assert_eq!(regs.get_bc(), 0x55AA);
}

#[test]
fn test_set_bc() {
    let mut regs: Registers = Default::default();
    regs.set_bc(0xAA55);
    assert_eq!(regs.b, 0xAA);
    assert_eq!(regs.c, 0x55);
}

#[test]
fn test_get_de() {
    let regs = Registers {
        d: 0x55,
        e: 0xAA,
        ..Default::default()
    };
    assert_eq!(regs.get_de(), 0x55AA);
}

#[test]
fn test_set_de() {
    let mut regs: Registers = Default::default();
    regs.set_de(0xAA55);
    assert_eq!(regs.d, 0xAA);
    assert_eq!(regs.e, 0x55);
}

#[test]
fn test_get_hl() {
    let regs = Registers {
        h: 0x55,
        l: 0xAA,
        ..Default::default()
    };
    assert_eq!(regs.get_hl(), 0x55AA);
}

#[test]
fn test_set_hl() {
    let mut regs: Registers = Default::default();
    regs.set_hl(0xAA55);
    assert_eq!(regs.h, 0xAA);
    assert_eq!(regs.l, 0x55);
}

#[test]
fn test_toggle_zero_flag() {
    let mut regs: Registers = Default::default();
    assert_eq!(regs.f, 0);
    regs.toggle_zero_flag(42);
    assert_eq!(regs.f, 0);
    regs.toggle_zero_flag(0);
    assert_eq!(regs.f, FLAG_ZERO);
}

#[test]
fn test_clear_flag() {
    let mut regs: Registers = Default::default();
    regs.f = FLAG_CARRY | FLAG_HALF;
    regs.clear_flag(FLAG_CARRY);
    assert_eq!(regs.f & FLAG_CARRY, 0);
    assert_eq!(regs.f & FLAG_HALF, FLAG_HALF);
}

struct CPU {
    regs: Registers,
    mem: Rc<Memory>,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            regs: Default::default(),
            mem: Default::default(),
        }
    }
}

impl CPU {
    /// Add value to register A
    ///
    /// A := A + value
    ///
    /// Set Z, C, H flags
    ///
    fn add_imm(&mut self, value: u8) {
        let result = (self.regs.a as u32) + (value as u32);
        let carry = (self.regs.a as u32) ^ (value as u32) ^ result;
        self.regs.a = result as u8;
        self.regs.f = 0;
        self.regs.toggle_zero_flag(result as u8);
        self.regs.toggle_carry_flag(carry);
        self.regs.toggle_half_flag(carry);
    }

    /// Add value and Carry to register A
    ///
    /// A := A + C + value
    ///
    /// Set Z, C, H flags
    ///
    fn adc_imm(&mut self, value: u8) {
        let c = if self.regs.f & FLAG_CARRY == FLAG_CARRY {
            1u32
        } else {
            0u32
        };

        let result = (self.regs.a as u32) + (value as u32) + c;
        let carry = (self.regs.a as u32) ^ (value as u32) ^ result;
        self.regs.a = result as u8;
        self.regs.f = 0;
        self.regs.toggle_zero_flag(result as u8);
        self.regs.toggle_carry_flag(carry);
        self.regs.toggle_half_flag(carry);
    }

    /// Subtract value from register A
    ///
    /// A := A - value
    ///
    /// Set Z, C, H flags
    ///
    fn sub_imm(&mut self, value: u8) {
        let result = (self.regs.a as u32) - (value as u32);
        let carry = (self.regs.a as u32) ^ (value as u32) ^ result;
        self.regs.a = result as u8;
        self.regs.f = FLAG_SUB;
        self.regs.toggle_zero_flag(result as u8);
        self.regs.toggle_carry_flag(carry);
        self.regs.toggle_half_flag(carry);
    }

    /// Subtract value and carry from register A
    ///
    /// A := A - value
    ///
    /// Set Z, C, H flags
    ///
    fn sbc_imm(&mut self, value: u8) {
        let c = if self.regs.f & FLAG_CARRY == FLAG_CARRY {
            1u32
        } else {
            0u32
        };

        let result = (self.regs.a as u32) - (value as u32) - c;
        let carry = (self.regs.a as u32) ^ (value as u32) ^ result;
        self.regs.a = result as u8;
        self.regs.f = FLAG_SUB;
        self.regs.toggle_zero_flag(result as u8);
        self.regs.toggle_carry_flag(carry);
        self.regs.toggle_half_flag(carry);
    }
}

#[test]
fn test_add_imm() {
    let mut cpu: CPU = Default::default();
    cpu.add_imm(0);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);

    cpu.add_imm(0x80);
    assert_eq!(cpu.regs.a, 0x80);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);

    cpu.add_imm(0x80);
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);
    assert_eq!(cpu.regs.f & FLAG_CARRY, FLAG_CARRY);

    cpu.add_imm(0x08);
    assert_eq!(cpu.regs.a, 0x08);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);
    assert_eq!(cpu.regs.f & FLAG_CARRY, 0);
    assert_eq!(cpu.regs.f & FLAG_HALF, 0);

    cpu.add_imm(0x08);
    assert_eq!(cpu.regs.a, 0x10);
    assert_eq!(cpu.regs.f & FLAG_HALF, FLAG_HALF);
}

#[test]
fn test_adc_imm() {
    let mut cpu: CPU = Default::default();
    cpu.adc_imm(0);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);

    cpu.adc_imm(0x80);
    assert_eq!(cpu.regs.a, 0x80);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);

    cpu.adc_imm(0x80);
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);
    assert_eq!(cpu.regs.f & FLAG_CARRY, FLAG_CARRY);

    cpu.adc_imm(0x00);
    assert_eq!(cpu.regs.a, 0x01);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);
    assert_eq!(cpu.regs.f & FLAG_CARRY, 0);
}

#[test]
fn test_sub_imm() {
    let mut cpu: CPU = Default::default();
    cpu.add_imm(10);
    assert_eq!(cpu.regs.a, 10);
    assert_eq!(cpu.regs.f, 0);

    cpu.sub_imm(10);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);
}

#[test]
fn test_sbc_imm() {
    let mut cpu: CPU = Default::default();
    cpu.adc_imm(0x80);
    assert_eq!(cpu.regs.a, 0x80);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);

    cpu.adc_imm(0x85);
    assert_eq!(cpu.regs.a, 5);
    assert_eq!(cpu.regs.f & FLAG_CARRY, FLAG_CARRY);

    cpu.sbc_imm(2);
    assert_eq!(cpu.regs.a, 2);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
}
