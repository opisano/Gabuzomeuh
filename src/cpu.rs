use std::num::Wrapping;

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

#[derive(Default)]
struct Cpu {
    regs: Registers,
    mem: Box<Memory>,
}

impl Cpu {
    fn add8(&mut self, value: u8, use_carry: bool) {
        let carry_value = if use_carry && (self.regs.f & FLAG_CARRY) == FLAG_CARRY {
            1u32
        } else {
            0u32
        };
        let result = carry_value + self.regs.a as u32 + value as u32;
        self.regs.a = result as u8;
        self.regs.f = 0;
        self.regs.toggle_zero_flag(result as u8);
        self.regs.toggle_carry_flag(result);
        self.regs.toggle_half_flag(result);
    }

    fn add16(&mut self, addr: u16, value: u16) {
        let mem_value = self.mem.read16(addr);
        let result = (mem_value as u32) + (value as u32);
        self.regs.f &= !FLAG_SUB;

        if (result & 0x1_0000) != 0 {
            self.regs.f |= FLAG_CARRY;
        }

        if (result & 0x1000) != 0 {
            self.regs.f |= FLAG_HALF;
        }
        self.mem.write16(addr, result as u16);
    }

    fn add_sp(&mut self, value: u8) {
        let signed_value = (value as i8) as i16;
        let signed_sp = self.regs.sp as i16;
        let new_signed_sp_value = Wrapping(signed_sp) + Wrapping(signed_value);
        let new_sp_value = new_signed_sp_value.0 as u16;

        self.regs.toggle_half_flag(new_sp_value as u32);

        // overflow did occur ?
        if (new_sp_value < self.regs.sp && value < 128)
            || (new_sp_value > self.regs.sp && value >= 128)
        {
            self.regs.f |= FLAG_CARRY;
        }
        self.regs.sp = new_sp_value;
    }

    fn sub8(&mut self, value: u8, use_carry: bool) {
        let carry_value = if use_carry && (self.regs.f & FLAG_CARRY) == FLAG_CARRY {
            1
        } else {
            0
        };

        let result = Wrapping(self.regs.a) - Wrapping(value) - Wrapping(carry_value);
        self.regs.f = FLAG_SUB;
        self.regs.toggle_zero_flag(result.0);
        if (self.regs.a & 0x0F) < (value & 0x0F) + carry_value {
            self.regs.f |= FLAG_HALF;
        }
        if (self.regs.a as u32) < (value as u32) + (carry_value as u32) {
            self.regs.f |= FLAG_CARRY;
        }
        self.regs.a = result.0;
    }

    /// Add value to register A
    ///
    /// A := A + value
    ///
    /// Set Z, C, H flags
    ///
    fn add_imm(&mut self, value: u8) {
        self.add8(value, false);
    }

    /// Add value and Carry to register A
    ///
    /// A := A + C + value
    ///
    /// Set Z, C, H flags
    ///
    fn adc_imm(&mut self, value: u8) {
        self.add8(value, true);
    }

    /// Subtract value from register A
    ///
    /// A := A - value
    ///
    /// Set Z, C, H flags
    ///
    fn sub_imm(&mut self, value: u8) {
        self.sub8(value, false);
    }

    /// Subtract value and carry from register A
    ///
    /// A := A - value
    ///
    /// Set Z, C, H flags
    ///
    fn sbc_imm(&mut self, value: u8) {
        self.sub8(value, true);
    }

    /// Perform binary AND operation
    ///
    /// A := A & value
    /// F |= FLAG_HALF
    ///
    /// Set Z flag
    fn and_imm(&mut self, value: u8) {
        self.regs.a &= value;
        self.regs.f = FLAG_HALF;
        self.regs.toggle_zero_flag(self.regs.a);
    }

    /// Perform binary XOR operation
    ///
    /// A := A & value
    ///
    /// Set Z flag
    fn xor_imm(&mut self, value: u8) {
        self.regs.a ^= value;
        self.regs.f = 0;
        self.regs.toggle_zero_flag(self.regs.a);
    }

    /// Perform binary OR operation
    ///
    /// A := A & value
    ///
    /// Set Z flag
    fn or_imm(&mut self, value: u8) {
        self.regs.a |= value;
        self.regs.f = 0;
        self.regs.toggle_zero_flag(self.regs.a);
    }

    /// Compare A to value
    ///
    /// Similar to a SUB, but leaves A untouched.
    ///
    fn cp_imm(&mut self, value: u8) {
        let save = self.regs.a;
        self.sub8(value, false);
        self.regs.a = save;
    }

    /// Increment a value by 1
    ///
    /// Set H and Z flags
    ///
    /// Return new value
    fn inc8(&mut self, value: u8) -> u8 {
        let result = ((value as u32) + 1) as u8;
        self.regs.f = 0;
        self.regs.toggle_half_flag(result as u32);
        self.regs.toggle_zero_flag(result);
        result
    }

    fn inc16(&self, value: u16) -> u16 {
        let result = ((value as u32) + 1) as u16;
        result
    }

    /// Decrement a value by 1
    ///
    /// Set H and Z flags
    ///
    /// Return new value
    fn dec(&mut self, value: u8) -> u8 {
        let result = Wrapping(value) - Wrapping(1u8);
        self.regs.f = FLAG_SUB;
        if (value & 0x0F) == 0 {
            self.regs.f |= FLAG_HALF;
        }
        self.regs.toggle_zero_flag(result.0);
        result.0
    }

    fn dec16(&self, value: u16) -> u16 {
        let result = Wrapping(value) - Wrapping(1u16);
        result.0
    }

    fn daa(&mut self) {
        let mut a = Wrapping(self.regs.a);

        if (a.0 & 0x0F) > 0x09 || (self.regs.f & FLAG_HALF) == FLAG_HALF {
            a += Wrapping(0x06);
        }

        if (a.0 & 0xF0) > 0x90 || (self.regs.f & FLAG_CARRY) == FLAG_CARRY {
            if (a.0 as u32) + 0x60 > 99 {
                self.regs.f |= FLAG_CARRY;
            }
            a += Wrapping(0x60);
        }

        self.regs.toggle_zero_flag(a.0);
        self.regs.f &= !FLAG_HALF;
        self.regs.a = a.0;
    }

    fn cpl(&mut self) {
        self.regs.a = !self.regs.a;
        self.regs.f = FLAG_SUB | FLAG_HALF;
    }

    /// Write the value of Stack pointer at specified address
    ///
    /// (addr) := SP
    ///
    fn store_sp(&mut self, addr: u16) {
        self.mem.write16(addr, self.regs.sp);
    }

    /// Copy HL into SP
    ///
    /// SP := HL
    ///
    fn load_sp_hl(&mut self) {
        self.regs.sp = self.regs.get_hl();
    }

    fn push(&mut self, val: u16) {
        self.regs.sp -= 2;
        self.mem.write16(self.regs.sp, val);
    }

    fn pop(&mut self) -> u16 {
        let temp = self.mem.read16(self.regs.sp);
        self.regs.sp += 2;
        temp
    }
}

#[test]
fn test_add_imm() {
    let mut cpu: Cpu = Default::default();
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
    let mut cpu: Cpu = Default::default();
    cpu.adc_imm(0);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);

    cpu.adc_imm(0x80);
    assert_eq!(cpu.regs.a, 0x80);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);

    cpu.adc_imm(cpu.regs.a);
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
    let mut cpu: Cpu = Default::default();
    cpu.add_imm(10);
    assert_eq!(cpu.regs.a, 10);
    assert_eq!(cpu.regs.f, 0);

    cpu.sub_imm(10);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);

    cpu.sub_imm(1);
    assert_eq!(cpu.regs.a, 0xFF);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);
    assert_eq!(cpu.regs.f & FLAG_HALF, FLAG_HALF);
    assert_eq!(cpu.regs.f & FLAG_CARRY, FLAG_CARRY);
}

#[test]
fn test_sbc_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.adc_imm(0x80);
    assert_eq!(cpu.regs.a, 0x80);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);

    cpu.adc_imm(0x85);
    assert_eq!(cpu.regs.a, 5);
    assert_eq!(cpu.regs.f & FLAG_CARRY, FLAG_CARRY);

    cpu.sbc_imm(2);
    assert_eq!(cpu.regs.a, 2);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);

    cpu.sbc_imm(3);
    assert_eq!(cpu.regs.a, 0xFF);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
    assert_eq!(cpu.regs.f & FLAG_HALF, FLAG_HALF);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
}

#[test]
fn test_and_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x05;

    cpu.and_imm(1);
    assert_eq!(cpu.regs.a, 1);
    assert_eq!(cpu.regs.f & FLAG_HALF, FLAG_HALF);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);

    cpu.and_imm(0);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f & FLAG_HALF, FLAG_HALF);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);
}

#[test]
fn test_xor_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x05;

    cpu.xor_imm(4);
    assert_eq!(cpu.regs.a, 1);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);

    cpu.xor_imm(1);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);
}

#[test]
fn test_or_imm() {
    let mut cpu: Cpu = Default::default();

    cpu.or_imm(0);
    assert_eq!(cpu.regs.a, 0);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);

    cpu.or_imm(0x0A);
    assert_eq!(cpu.regs.a, 0x0A);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);
}

#[test]
fn test_cp_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.add_imm(10);
    cpu.cp_imm(10);
    assert_eq!(cpu.regs.a, 10);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);
}

#[test]
fn test_inc() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0xF;
    cpu.regs.a = cpu.inc8(cpu.regs.a);
    assert_eq!(cpu.regs.a, 0x10);
    assert_eq!(cpu.regs.f & FLAG_SUB, 0);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);
    assert_eq!(cpu.regs.f & FLAG_HALF, FLAG_HALF);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);

    cpu.regs.a = 0xFF;
    cpu.regs.a = cpu.inc8(cpu.regs.a);
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f & FLAG_SUB, 0);
    assert_eq!(cpu.regs.f & FLAG_ZERO, FLAG_ZERO);
    assert_eq!(cpu.regs.f & FLAG_HALF, 0);
    assert_eq!(cpu.regs.f & FLAG_CARRY, 0)
}

#[test]
fn test_dec() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x11;
    cpu.regs.a = cpu.dec(cpu.regs.a);
    assert_eq!(cpu.regs.a, 0x10);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);

    cpu.regs.a = 0x00;
    cpu.regs.a = cpu.dec(cpu.regs.a);
    assert_eq!(cpu.regs.a, 0xFF);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
    assert_eq!(cpu.regs.f & FLAG_ZERO, 0);
    assert_eq!(cpu.regs.f & FLAG_HALF, FLAG_HALF);
    assert_eq!(cpu.regs.f & FLAG_CARRY, 0)
}

#[test]
fn test_daa() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x55;
    cpu.add_imm(0x11);
    cpu.daa();
    assert_eq!(cpu.regs.a, 0x66);
    assert_eq!(cpu.regs.f & FLAG_CARRY, 0);

    cpu.regs.a = 0x59;
    cpu.add_imm(0x12);
    cpu.daa();
    assert_eq!(cpu.regs.a, 0x71);
    assert_eq!(cpu.regs.f & FLAG_CARRY, 0);

    cpu.regs.a = 0x90;
    cpu.add_imm(0x10);
    cpu.daa();
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f & FLAG_CARRY, FLAG_CARRY);

    cpu.regs.a = 0x99;
    cpu.add_imm(0x01);
    cpu.daa();
    assert_eq!(cpu.regs.a, 0x00);
    assert_eq!(cpu.regs.f & FLAG_CARRY, FLAG_CARRY);
}

#[test]
fn test_cpl() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x55;
    cpu.cpl();

    assert_eq!(cpu.regs.a, 0xAA);
    assert_eq!(cpu.regs.f & FLAG_SUB, FLAG_SUB);
    assert_eq!(cpu.regs.f & FLAG_HALF, FLAG_HALF);
}

#[test]
fn test_add16() {
    let mut cpu: Cpu = Default::default();
    cpu.add16(0xC000, 0xBABD);
    cpu.add16(0xC000, 1);

    assert_eq!(cpu.mem.read8(0xC000), 0xBE);
    assert_eq!(cpu.mem.read8(0xC001), 0xBA);
}

#[test]
fn test_inc16() {
    let cpu: Cpu = Default::default();
    let value = cpu.inc16(0x4241);
    assert_eq!(value, 0x4242);

    let value2 = cpu.inc16(0xFFFF);
    assert_eq!(value2, 0);
}

#[test]
fn test_dec16() {
    let cpu: Cpu = Default::default();
    let value = cpu.dec16(0x4243);
    assert_eq!(value, 0x4242);

    let value2 = cpu.dec16(0);
    assert_eq!(value2, 0xFFFF);
}

#[test]
fn test_add_sp() {
    let mut cpu: Cpu = Default::default();

    // add positive value
    cpu.regs.sp = 0x32;
    cpu.add_sp(0x11);
    assert_eq!(cpu.regs.sp, 0x43);

    // add negative value
    cpu.add_sp(0xFF);
    assert_eq!(cpu.regs.sp, 0x42);

    // check half flag
    cpu.add_sp(0x0E);
    assert_eq!(cpu.regs.sp, 0x50);
    assert_eq!(cpu.regs.f & FLAG_HALF, FLAG_HALF);

    // check carry flag
    cpu.regs.sp = 0xFFFF;
    cpu.regs.f = 0;
    cpu.add_sp(1);
    assert_eq!(cpu.regs.sp, 0x00);
    assert_eq!(cpu.regs.f & FLAG_CARRY, FLAG_CARRY);

    cpu.regs.sp = 1;
    cpu.regs.f = 0;
    cpu.add_sp(0xFE);
    assert_eq!(cpu.regs.sp, 0xFFFF);
    assert_eq!(cpu.regs.f & FLAG_CARRY, FLAG_CARRY);
}

#[test]
fn test_store_sp() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.sp = 0x0042;

    assert_eq!(cpu.mem.read16(0xC000), 0);
    cpu.store_sp(0xC000);
    assert_eq!(cpu.mem.read16(0xC000), 0x0042);
}

#[test]
fn test_load_sp_hl() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.set_hl(0x1234);

    assert_eq!(cpu.regs.sp, 0);
    cpu.load_sp_hl();
    assert_eq!(cpu.regs.sp, 0x1234);
}

#[test]
fn test_push() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.sp = 0xC002;
    cpu.push(0x1234);

    assert_eq!(cpu.regs.sp, 0xC000);
    assert_eq!(cpu.mem.read16(cpu.regs.sp), 0x1234);
}

#[test]
fn test_pop() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.sp = 0xC000;
    cpu.mem.write16(cpu.regs.sp, 0x1234);
    let value = cpu.pop();

    assert_eq!(cpu.regs.sp, 0xC002);
    assert_eq!(value, 0x1234);
}
