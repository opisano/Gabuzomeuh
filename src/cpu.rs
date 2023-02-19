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

    fn isset_flag(&self, flag: u8) -> bool {
        self.f & flag == flag
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
    assert!(!regs.isset_flag(FLAG_CARRY));
    assert!(regs.isset_flag(FLAG_HALF));
}

#[derive(Default)]
struct Cpu {
    regs: Registers,
    mem: Box<Memory>,
    interrupts: bool,
    enabled: bool,
}

impl Cpu {
    fn add8(&mut self, value: u8, use_carry: bool) {
        let carry_value = if use_carry && self.regs.isset_flag(FLAG_CARRY) {
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

    fn add_hl(&mut self, value: u16) {
        self.regs.set_hl(self.regs.get_hl().wrapping_add(value));
        self.regs.clear_flag(FLAG_SUB);
    }

    fn add16(&mut self, addr: u16, value: u16) {
        let mem_value = self.mem.read16(addr);
        let result = (mem_value as u32) + (value as u32);
        self.regs.clear_flag(FLAG_SUB);

        if (result & 0x1_0000) != 0 {
            self.regs.toggle_flag(FLAG_CARRY);
        }

        if (result & 0x1000) != 0 {
            self.regs.toggle_flag(FLAG_HALF);
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
            self.regs.toggle_flag(FLAG_CARRY);
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
            self.regs.toggle_flag(FLAG_HALF)
        }
        if (self.regs.a as u32) < (value as u32) + (carry_value as u32) {
            self.regs.toggle_flag(FLAG_CARRY);
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
    fn inc(&mut self, value: u8) -> u8 {
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
            self.regs.toggle_flag(FLAG_HALF);
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

        if (a.0 & 0x0F) > 0x09 || self.regs.isset_flag(FLAG_HALF) {
            a += Wrapping(0x06);
        }

        if (a.0 & 0xF0) > 0x90 || self.regs.isset_flag(FLAG_CARRY) {
            if (a.0 as u32) + 0x60 > 99 {
                self.regs.toggle_flag(FLAG_CARRY);
            }
            a += Wrapping(0x60);
        }

        self.regs.toggle_zero_flag(a.0);
        self.regs.clear_flag(FLAG_HALF);
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

    fn bit(&mut self, b: u8, val: u8) {
        let bit_index = b & 0b111;
        let result = val & (1 << bit_index);
        self.regs.clear_flag(FLAG_SUB);
        self.regs.toggle_flag(FLAG_HALF);
        if result == 0 {
            self.regs.toggle_flag(FLAG_ZERO);
        } else {
            self.regs.clear_flag(FLAG_ZERO);
        }
    }

    fn set(&self, b: u8, val: u8) -> u8 {
        let bit_index: u8 = b & 0b111;
        let bit_mask = 1 << bit_index;
        val | bit_mask
    }

    /// Rotate the content of A to the left
    ///
    /// The contents of bit 7 are placed in both Carry and bit 0 of A
    ///
    fn rlca(&mut self) {
        self.regs.a = self.rlc(self.regs.a);
        self.regs.clear_flag(FLAG_ZERO);
    }

    /// Rotate the content of val to the left
    ///
    /// The contents of bit 7 are placed in both Carry and bit 0
    ///
    fn rlc(&mut self, val: u8) -> u8 {
        let temp = (val as u16) << 1;
        if temp & 0x100 == 0x100 {
            self.regs.toggle_flag(FLAG_CARRY);
            self.regs.clear_flag(FLAG_ZERO);
            (temp & 0xFF) as u8 | 1
        } else {
            self.regs.clear_flag(FLAG_CARRY);
            self.regs.toggle_zero_flag((temp & 0xFF) as u8);
            (temp & 0xFF) as u8
        }
    }

    /// Rotate the content of A to the left
    ///
    /// The contents of bit 7 is placed in Carry
    /// The previous content of Carry is placed in bit 0 of A.
    ///
    fn rla(&mut self) {
        self.regs.a = self.rl(self.regs.a);
        self.regs.clear_flag(FLAG_ZERO);
    }

    /// Rotate the content of val to the left
    ///
    /// The contents of bit 7 is placed in Carry
    /// The previous content of Carry is placed in bit 0
    ///
    fn rl(&mut self, val: u8) -> u8 {
        let carry = if self.regs.isset_flag(FLAG_CARRY) {
            1u8
        } else {
            0u8
        };
        let temp = (val as u16) << 1;
        if temp & 0x100 == 0x100 {
            self.regs.toggle_flag(FLAG_CARRY);
        } else {
            self.regs.clear_flag(FLAG_CARRY);
        }
        let result = (temp & 0xFF) as u8 | carry;
        self.regs.toggle_zero_flag(result);
        result
    }

    /// Rotate the content of A to the right
    ///
    /// The contents of bit 0 are placed in both Carry and bit 7 of A
    ///
    fn rrca(&mut self) {
        self.regs.a = self.rrc(self.regs.a);
        self.regs.clear_flag(FLAG_ZERO);
    }

    /// Rotate the content of val to the right
    ///
    /// The contents of bit 0 are placed in both Carry and bit 7
    ///
    fn rrc(&mut self, val: u8) -> u8 {
        let carry = val & 1;
        if carry == 1 {
            self.regs.toggle_flag(FLAG_CARRY);
            self.regs.clear_flag(FLAG_ZERO);
            (val >> 1) | 0x80
        } else {
            self.regs.clear_flag(FLAG_CARRY);
            self.regs.toggle_zero_flag(val >> 1);
            (val >> 1)
        }
    }

    /// Rotate the content of A to the right
    ///
    /// The contents of bit 0 is placed in Carry
    /// The previous content of Carry is placed in bit 7 of A.
    ///
    fn rra(&mut self) {
        self.regs.a = self.rr(self.regs.a);
        self.regs.clear_flag(FLAG_ZERO);
    }

    /// Rotate the content of val to the right
    ///
    /// The contents of bit 0 is placed in Carry
    /// The previous content of Carry is placed in bit 7.
    ///
    fn rr(&mut self, val: u8) -> u8 {
        let carry = val & 1;
        let h = if self.regs.isset_flag(FLAG_CARRY) {
            0x80u8
        } else {
            0x00u8
        };

        if carry == 1 {
            self.regs.toggle_flag(FLAG_CARRY);
        } else {
            self.regs.clear_flag(FLAG_CARRY);
        }

        let result = (val >> 1) | h;
        self.regs.toggle_zero_flag(result);
        result
    }

    /// Shift left arithmetic
    fn sla(&mut self, val: u8) -> u8 {
        let result: u32 = (val as u32) << 1;
        let byte_result = (result & 0xFF) as u8;
        self.regs.toggle_carry_flag(result);
        self.regs.toggle_zero_flag(byte_result);
        self.regs.clear_flag(FLAG_HALF);
        self.regs.clear_flag(FLAG_SUB);
        byte_result
    }

    /// Shift right arithmetic
    fn sra(&mut self, val: u8) -> u8 {
        let carry = val & 1;
        let result = (val >> 1) | (val & 0x80);
        self.regs.clear_flag(FLAG_HALF);
        self.regs.clear_flag(FLAG_SUB);
        self.regs.toggle_zero_flag(result);
        if carry == 0 {
            self.regs.clear_flag(FLAG_CARRY);
        } else {
            self.regs.toggle_flag(FLAG_CARRY);
        }
        result
    }

    /// Shift right logical
    fn srl(&mut self, val: u8) -> u8 {
        let carry = val & 1;
        let result = (val >> 1);
        self.regs.clear_flag(FLAG_HALF);
        self.regs.clear_flag(FLAG_SUB);
        self.regs.toggle_zero_flag(result);
        if carry == 0 {
            self.regs.clear_flag(FLAG_CARRY);
        } else {
            self.regs.toggle_flag(FLAG_CARRY);
        }
        result
    }

    /// Swap the nibbles in a byte
    fn swap(&mut self, val: u8) -> u8 {
        let result = ((val >> 4) & 0x0F) | ((val << 4) & 0xF0);
        self.regs.f = 0;
        self.regs.toggle_zero_flag(result);
        result
    }

    /// Inverts the carry flag
    fn ccf(&mut self) {
        if self.regs.isset_flag(FLAG_CARRY) {
            self.regs.clear_flag(FLAG_CARRY);
        } else {
            self.regs.toggle_flag(FLAG_CARRY);
        }
    }

    fn jr(&mut self) {
        let offset = self.fetch_byte() as i8;
        self.regs.pc = ((self.regs.pc as u32 as i32) + (offset as i32)) as u16;
    }

    fn call(&mut self, val: u16) {
        self.regs.sp -= 2;
        self.mem.write16(self.regs.sp, val);
        self.regs.pc = val;
    }

    fn ret(&mut self) {
        self.regs.pc = self.mem.read16(self.regs.sp);
        self.regs.sp += 2;
    }

    fn rst(&mut self, val: u8) {
        self.push(self.regs.pc);
        self.regs.pc = val as u16;
    }

    /// Read the next byte from PC
    ///
    /// PC is incremented
    fn fetch_byte(&mut self) -> u8 {
        let b = self.mem.read8(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        b
    }

    fn fetch_word(&mut self) -> u16 {
        let w = self.mem.read16(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(2);
        w
    }

    fn execute(&mut self) -> u32 {
        let opcode = self.fetch_byte();
        match opcode {
            0x00 => 1,
            0x01 => {
                let val = self.fetch_word();
                self.regs.set_bc(val);
                3
            }
            0x02 => {
                self.mem.write8(self.regs.get_bc(), self.regs.a);
                2
            }
            0x03 => {
                self.regs.set_bc(self.inc16(self.regs.get_bc()));
                2
            }
            0x04 => {
                self.regs.b = self.inc(self.regs.b);
                1
            }
            0x05 => {
                self.regs.b = self.dec(self.regs.b);
                1
            }
            0x06 => {
                self.regs.b = self.fetch_byte();
                2
            }
            0x07 => {
                self.rlca();
                1
            }
            0x08 => {
                let addr = self.fetch_word();
                self.mem.write16(addr, self.regs.sp);
                5
            }
            0x09 => {
                self.add_hl(self.regs.get_bc());
                2
            }
            0x0A => {
                let addr = self.regs.get_bc();
                self.regs.a = self.mem.read8(addr);
                2
            }
            0x0B => {
                self.regs.set_bc(self.dec16(self.regs.get_bc()));
                2
            }
            0x0C => {
                self.regs.c = self.inc(self.regs.c);
                1
            }
            0x0D => {
                self.regs.c = self.dec(self.regs.c);
                1
            }
            0x0E => {
                self.regs.c = self.fetch_byte();
                2
            }
            0x0F => {
                self.rrca();
                1
            }
            0x10 => {
                self.enabled = false;
                1
            }
            0x11 => {
                let val = self.fetch_word();
                self.regs.set_de(val);
                3
            }
            0x12 => {
                self.mem.write8(self.regs.get_de(), self.regs.a);
                2
            }
            0x13 => {
                self.regs.set_de(self.inc16(self.regs.get_de()));
                2
            }
            0x14 => {
                self.regs.d = self.inc(self.regs.d);
                1
            }
            0x15 => {
                self.regs.d = self.dec(self.regs.d);
                1
            }
            0x16 => {
                self.regs.d = self.fetch_byte();
                2
            }
            0x17 => {
                self.rla();
                1
            }
            0x18 => {
                self.jr();
                3
            }
            0x19 => {
                self.add_hl(self.regs.get_de());
                2
            }
            0x1A => {
                let addr = self.regs.get_de();
                self.regs.a = self.mem.read8(addr);
                2
            }
            0x1B => {
                self.regs.set_bc(self.dec16(self.regs.get_bc()));
                2
            }
            0x1C => {
                self.regs.e = self.inc(self.regs.e);
                1
            }
            0x1D => {
                self.regs.e = self.dec(self.regs.e);
                1
            }
            0x1E => {
                self.regs.e = self.fetch_byte();
                2
            }
            0x1F => {
                self.rra();
                1
            }
            0x20 => {
                if !self.regs.isset_flag(FLAG_ZERO) {
                    self.jr();
                    3
                } else {
                    2
                }
            }
            0x21 => {
                let val = self.fetch_word();
                self.regs.set_hl(val);
                3
            }
            0x22 => {
                self.mem.write8(self.regs.get_hl(), self.regs.a);
                self.regs.set_hl(self.regs.get_hl().wrapping_add(1));
                2
            }
            0x23 => {
                self.regs.set_hl(self.inc16(self.regs.get_hl()));
                2
            }
            0x24 => {
                self.regs.h = self.inc(self.regs.h);
                1
            }
            0x25 => {
                self.regs.h = self.dec(self.regs.h);
                1
            }
            0x26 => {
                self.regs.h = self.fetch_byte();
                2
            }
            0x27 => {
                self.daa();
                1
            }
            0x28 => {
                if self.regs.isset_flag(FLAG_ZERO) {
                    self.jr();
                    3
                } else {
                    2
                }
            }
            0x29 => {
                self.add_hl(self.regs.get_hl());
                2
            }
            0x2A => {
                self.regs.a = self.mem.read8(self.regs.get_hl());
                self.regs.set_hl(self.regs.get_hl().wrapping_add(1));
                2
            }
            0x2B => {
                self.regs.set_hl(self.dec16(self.regs.get_hl()));
                2
            }
            0x2C => {
                self.regs.l = self.inc(self.regs.l);
                1
            }
            0x2D => {
                self.regs.l = self.dec(self.regs.l);
                1
            }
            0x2E => {
                self.regs.l = self.fetch_byte();
                2
            }
            0x2F => {
                self.cpl();
                1
            }
            0x30 => {
                if !self.regs.isset_flag(FLAG_CARRY) {
                    self.jr();
                    3
                } else {
                    2
                }
            }
            0x31 => {
                let val = self.fetch_word();
                self.regs.sp = val;
                3
            }
            0x32 => {
                self.mem.write8(self.regs.get_hl(), self.regs.a);
                self.regs.set_hl(self.regs.get_hl().wrapping_sub(1));
                2
            }
            0x33 => {
                self.regs.sp = self.inc16(self.regs.sp);
                2
            }
            0x34 => {
                let addr = self.regs.get_hl();
                let val = self.inc(self.mem.read8(addr));
                self.mem.write8(addr, val);
                3
            }
            0x35 => {
                let addr = self.regs.get_hl();
                let val = self.dec(self.mem.read8(addr));
                self.mem.write8(addr, val);
                3
            }
            0x36 => {
                let val = self.fetch_byte();
                self.mem.write8(self.regs.get_hl(), val);
                3
            }
            0x37 => {
                self.regs.toggle_flag(FLAG_HALF);
                1
            }
            0x38 => {
                if self.regs.isset_flag(FLAG_CARRY) {
                    self.jr();
                    3
                } else {
                    2
                }
            }
            0x39 => {
                self.add_hl(self.regs.sp);
                2
            }
            0x3A => {
                self.regs.a = self.mem.read8(self.regs.get_hl());
                self.regs.set_hl(self.regs.get_hl().wrapping_sub(1));
                2
            }
            0x3B => {
                self.regs.sp = self.dec16(self.regs.sp);
                2
            }
            0x3C => {
                self.regs.a = self.inc(self.regs.a);
                1
            }
            0x3D => {
                self.regs.a = self.dec(self.regs.a);
                1
            }
            0x3E => {
                self.regs.a = self.fetch_byte();
                2
            }
            0x3F => {
                self.ccf();
                1
            }
            0x40 => 1,
            0x41 => {
                self.regs.b = self.regs.c;
                1
            }
            0x42 => {
                self.regs.b = self.regs.d;
                1
            }
            0x43 => {
                self.regs.b = self.regs.e;
                1
            }
            0x44 => {
                self.regs.b = self.regs.h;
                1
            }
            0x45 => {
                self.regs.b = self.regs.l;
                1
            }
            0x46 => {
                self.regs.b = self.mem.read8(self.regs.get_hl());
                2
            }
            0x47 => {
                self.regs.b = self.regs.a;
                1
            }
            0x48 => {
                self.regs.c = self.regs.b;
                1
            }
            0x49 => 1,
            0x4A => {
                self.regs.c = self.regs.d;
                1
            }
            0x4B => {
                self.regs.c = self.regs.e;
                1
            }
            0x4C => {
                self.regs.c = self.regs.h;
                1
            }
            0x4D => {
                self.regs.c = self.regs.l;
                1
            }
            0x4E => {
                self.regs.c = self.mem.read8(self.regs.get_hl());
                2
            }
            0x4F => {
                self.regs.c = self.regs.a;
                1
            }
            0x50 => {
                self.regs.d = self.regs.b;
                1
            }
            0x51 => {
                self.regs.d = self.regs.c;
                1
            }
            0x52 => 1,
            0x53 => {
                self.regs.d = self.regs.e;
                1
            }
            0x54 => {
                self.regs.d = self.regs.h;
                1
            }
            0x55 => {
                self.regs.d = self.regs.l;
                1
            }
            0x56 => {
                self.regs.d = self.mem.read8(self.regs.get_hl());
                2
            }
            0x57 => {
                self.regs.d = self.regs.a;
                1
            }
            0x58 => {
                self.regs.e = self.regs.b;
                1
            }
            0x59 => {
                self.regs.e = self.regs.c;
                1
            }
            0x5A => {
                self.regs.e = self.regs.d;
                1
            }
            0x5B => 1,
            0x5C => {
                self.regs.e = self.regs.h;
                1
            }
            0x5D => {
                self.regs.e = self.regs.l;
                1
            }
            0x5E => {
                self.regs.e = self.mem.read8(self.regs.get_hl());
                2
            }
            0x5F => {
                self.regs.e = self.regs.a;
                1
            }
            0x60 => {
                self.regs.h = self.regs.b;
                1
            }
            0x61 => {
                self.regs.h = self.regs.c;
                1
            }
            0x62 => {
                self.regs.h = self.regs.d;
                1
            }
            0x63 => {
                self.regs.h = self.regs.e;
                1
            }
            0x64 => 1,
            0x65 => {
                self.regs.h = self.regs.l;
                1
            }
            0x66 => {
                self.regs.h = self.mem.read8(self.regs.get_hl());
                2
            }
            0x67 => {
                self.regs.h = self.regs.a;
                1
            }
            0x68 => {
                self.regs.l = self.regs.b;
                1
            }
            0x69 => {
                self.regs.l = self.regs.c;
                1
            }
            0x6A => {
                self.regs.l = self.regs.d;
                1
            }
            0x6B => {
                self.regs.l = self.regs.e;
                1
            }
            0x6C => {
                self.regs.l = self.regs.h;
                1
            }
            0x6D => 1,
            0x6E => {
                self.regs.l = self.mem.read8(self.regs.get_hl());
                2
            }
            0x6F => {
                self.regs.l = self.regs.a;
                1
            }
            0x70 => {
                self.mem.write8(self.regs.get_hl(), self.regs.b);
                2
            }
            0x71 => {
                self.mem.write8(self.regs.get_hl(), self.regs.c);
                2
            }
            0x72 => {
                self.mem.write8(self.regs.get_hl(), self.regs.d);
                2
            }
            0x73 => {
                self.mem.write8(self.regs.get_hl(), self.regs.e);
                2
            }
            0x74 => {
                self.mem.write8(self.regs.get_hl(), self.regs.h);
                2
            }
            0x75 => {
                self.mem.write8(self.regs.get_hl(), self.regs.l);
                2
            }
            0x76 => {
                self.enabled = false;
                1
            }
            0x77 => {
                self.mem.write8(self.regs.get_hl(), self.regs.a);
                2
            }
            0x78 => {
                self.regs.a = self.regs.b;
                1
            }
            0x79 => {
                self.regs.a = self.regs.c;
                1
            }
            0x7A => {
                self.regs.a = self.regs.d;
                1
            }
            0x7B => {
                self.regs.a = self.regs.e;
                1
            }
            0x7C => {
                self.regs.a = self.regs.h;
                1
            }
            0x7D => {
                self.regs.a = self.regs.l;
                1
            }
            0x7E => {
                self.regs.a = self.mem.read8(self.regs.get_hl());
                2
            }
            0x7F => 1,
            0x80 => {
                self.add_imm(self.regs.b);
                1
            }
            0x81 => {
                self.add_imm(self.regs.c);
                1
            }
            0x82 => {
                self.add_imm(self.regs.d);
                1
            }
            0x83 => {
                self.add_imm(self.regs.e);
                1
            }
            0x84 => {
                self.add_imm(self.regs.h);
                1
            }
            0x85 => {
                self.add_imm(self.regs.l);
                1
            }
            0x86 => {
                let val = self.mem.read8(self.regs.get_hl());
                self.add_imm(val);
                2
            }
            0x87 => {
                self.add_imm(self.regs.a);
                1
            }
            0x88 => {
                self.adc_imm(self.regs.b);
                1
            }
            0x89 => {
                self.adc_imm(self.regs.c);
                1
            }
            0x8A => {
                self.adc_imm(self.regs.d);
                1
            }
            0x8B => {
                self.adc_imm(self.regs.e);
                1
            }
            0x8C => {
                self.adc_imm(self.regs.h);
                1
            }
            0x8D => {
                self.adc_imm(self.regs.l);
                1
            }
            0x8E => {
                let val = self.mem.read8(self.regs.get_hl());
                self.adc_imm(val);
                2
            }
            0x8F => {
                self.adc_imm(self.regs.a);
                1
            }
            0x90 => {
                self.sub_imm(self.regs.b);
                1
            }
            0x91 => {
                self.sub_imm(self.regs.c);
                1
            }
            0x92 => {
                self.sub_imm(self.regs.d);
                1
            }
            0x93 => {
                self.sub_imm(self.regs.e);
                1
            }
            0x94 => {
                self.sub_imm(self.regs.h);
                1
            }
            0x95 => {
                self.sub_imm(self.regs.l);
                1
            }
            0x96 => {
                let val = self.mem.read8(self.regs.get_hl());
                self.sub_imm(val);
                2
            }
            0x97 => {
                self.sub_imm(self.regs.a);
                1
            }
            0x98 => {
                self.sbc_imm(self.regs.b);
                1
            }
            0x99 => {
                self.sbc_imm(self.regs.c);
                1
            }
            0x9A => {
                self.sbc_imm(self.regs.d);
                1
            }
            0x9B => {
                self.sbc_imm(self.regs.e);
                1
            }
            0x9C => {
                self.sbc_imm(self.regs.h);
                1
            }
            0x9D => {
                self.sbc_imm(self.regs.l);
                1
            }
            0x9E => {
                let val = self.mem.read8(self.regs.get_hl());
                self.sbc_imm(val);
                2
            }
            0x9F => {
                self.sbc_imm(self.regs.a);
                1
            }
            0xA0 => {
                self.and_imm(self.regs.b);
                1
            }
            0xA1 => {
                self.and_imm(self.regs.c);
                1
            }
            0xA2 => {
                self.and_imm(self.regs.d);
                1
            }
            0xA3 => {
                self.and_imm(self.regs.e);
                1
            }
            0xA4 => {
                self.and_imm(self.regs.h);
                1
            }
            0xA5 => {
                self.and_imm(self.regs.l);
                1
            }
            0xA6 => {
                let val = self.mem.read8(self.regs.get_hl());
                self.and_imm(val);
                2
            }
            0xA7 => {
                self.and_imm(self.regs.a);
                1
            }
            0xA8 => {
                self.xor_imm(self.regs.b);
                1
            }
            0xA9 => {
                self.xor_imm(self.regs.c);
                1
            }
            0xAA => {
                self.xor_imm(self.regs.d);
                1
            }
            0xAB => {
                self.xor_imm(self.regs.e);
                1
            }
            0xAC => {
                self.xor_imm(self.regs.h);
                1
            }
            0xAD => {
                self.xor_imm(self.regs.l);
                1
            }
            0xAE => {
                let val = self.mem.read8(self.regs.get_hl());
                self.xor_imm(val);
                2
            }
            0xAF => {
                self.xor_imm(self.regs.a);
                1
            }
            0xB0 => {
                self.or_imm(self.regs.b);
                1
            }
            0xB1 => {
                self.or_imm(self.regs.c);
                1
            }
            0xB2 => {
                self.or_imm(self.regs.d);
                1
            }
            0xB3 => {
                self.or_imm(self.regs.e);
                1
            }
            0xB4 => {
                self.or_imm(self.regs.h);
                1
            }
            0xB5 => {
                self.or_imm(self.regs.l);
                1
            }
            0xB6 => {
                let val = self.mem.read8(self.regs.get_hl());
                self.or_imm(val);
                2
            }
            0xB7 => {
                self.or_imm(self.regs.a);
                1
            }
            0xB8 => {
                self.cp_imm(self.regs.b);
                1
            }
            0xB9 => {
                self.cp_imm(self.regs.c);
                1
            }
            0xBA => {
                self.cp_imm(self.regs.d);
                1
            }
            0xBB => {
                self.cp_imm(self.regs.e);
                1
            }
            0xBC => {
                self.cp_imm(self.regs.h);
                1
            }
            0xBD => {
                self.cp_imm(self.regs.l);
                1
            }
            0xBE => {
                let val = self.mem.read8(self.regs.get_hl());
                self.cp_imm(val);
                2
            }
            0xBF => {
                self.cp_imm(self.regs.a);
                1
            }
            0xC0 => {
                if !self.regs.isset_flag(FLAG_ZERO) {
                    self.ret();
                    5
                } else {
                    2
                }
            }
            0xC1 => {
                let val = self.pop();
                self.regs.set_bc(val);
                3
            }
            0xC2 => {
                let addr = self.fetch_word();
                if !self.regs.isset_flag(FLAG_ZERO) {
                    self.regs.pc = addr;
                    4
                } else {
                    3
                }
            }
            0xC3 => {
                let addr = self.fetch_word();
                self.regs.pc = addr;
                4
            }
            0xC4 => {
                let addr = self.fetch_word();
                if !self.regs.isset_flag(FLAG_ZERO) {
                    self.call(addr);
                    6
                } else {
                    3
                }
            }
            0xC5 => {
                self.push(self.regs.get_bc());
                4
            }
            0xC6 => {
                let val = self.fetch_byte();
                self.add_imm(val);
                2
            }
            0xC7 => {
                self.rst(0x00);
                2
            }
            0xC8 => {
                if self.regs.isset_flag(FLAG_ZERO) {
                    self.ret();
                    5
                } else {
                    2
                }
            }
            0xC9 => {
                self.ret();
                2
            }
            0xCA => {
                let addr = self.fetch_word();
                if self.regs.isset_flag(FLAG_ZERO) {
                    self.regs.pc = addr;
                    4
                } else {
                    3
                }
            }
            0xCB => self.cb_execute(),
            0xCC => {
                let addr = self.fetch_word();
                if self.regs.isset_flag(FLAG_ZERO) {
                    self.call(addr);
                    6
                } else {
                    3
                }
            }
            0xCD => {
                let value = self.fetch_word();
                self.call(value);
                6
            }
            0xCE => {
                let value = self.fetch_byte();
                self.adc_imm(value);
                2
            }
            0xCF => {
                self.rst(0x08);
                2
            }
            0xD0 => {
                if !self.regs.isset_flag(FLAG_CARRY) {
                    self.ret();
                    5
                } else {
                    2
                }
            }
            0xD1 => {
                let val = self.pop();
                self.regs.set_de(val);
                3
            }
            0xD2 => {
                let addr = self.fetch_word();
                if !self.regs.isset_flag(FLAG_CARRY) {
                    self.regs.pc = addr;
                    4
                } else {
                    3
                }
            }
            0xD4 => {
                let addr = self.fetch_word();
                if !self.regs.isset_flag(FLAG_CARRY) {
                    self.call(addr);
                    6
                } else {
                    3
                }
            }
            0xD5 => {
                self.push(self.regs.get_de());
                4
            }
            0xD6 => {
                let val = self.fetch_byte();
                self.sub_imm(val);
                2
            }
            0xD7 => {
                self.rst(0x10);
                2
            }
            0xD8 => {
                if self.regs.isset_flag(FLAG_CARRY) {
                    self.ret();
                    5
                } else {
                    2
                }
            }
            0xD9 => {
                self.ret();
                self.interrupts = true;
                4
            }
            0xDA => {
                let addr = self.fetch_word();
                if self.regs.isset_flag(FLAG_CARRY) {
                    self.regs.pc = addr;
                    4
                } else {
                    3
                }
            }
            0xDC => {
                let addr = self.fetch_word();
                if self.regs.isset_flag(FLAG_CARRY) {
                    self.call(addr);
                    6
                } else {
                    3
                }
            }
            0xDE => {
                let value = self.fetch_byte();
                self.sbc_imm(value);
                2
            }
            0xDF => {
                self.rst(0x18);
                2
            }
            0xE0 => {
                let addr = 0xFF00 | self.fetch_byte() as u16;
                self.mem.write8(addr, self.regs.a);
                3
            }
            0xE1 => {
                let val = self.pop();
                self.regs.set_hl(val);
                3
            }
            0xE2 => {
                self.mem.write8(self.regs.c as u16, self.regs.a);
                2
            }
            0xE5 => {
                self.push(self.regs.get_hl());
                4
            }
            0xE6 => {
                let val = self.fetch_byte();
                self.and_imm(val);
                2
            }
            0xE7 => {
                self.rst(0x20);
                2
            }
            0xE8 => {
                let val = self.fetch_byte();
                self.regs.sp = self.regs.sp.wrapping_add(val as u16);
                2
            }
            0xE9 => {
                self.regs.pc = self.regs.get_hl();
                1
            }
            0xEA => {
                let addr = self.fetch_word();
                self.mem.write8(addr, self.regs.a);
                4
            }
            0xEE => {
                let value = self.fetch_byte();
                self.xor_imm(value);
                2
            }
            0xEF => {
                self.rst(0x28);
                2
            }
            0xF0 => {
                let addr = self.fetch_byte() as u16 | 0xFF00;
                self.regs.a = self.mem.read8(addr);
                3
            }
            0xF1 => {
                let val = self.pop();
                self.regs.set_af(val);
                3
            }
            0xF2 => {
                self.regs.a = self.mem.read8(self.regs.c as u16);
                2
            }
            0xF3 => {
                self.interrupts = false;
                1
            }
            0xF5 => {
                self.push(self.regs.get_af());
                4
            }
            0xF6 => {
                let val = self.fetch_byte();
                self.or_imm(val);
                2
            }
            0xF7 => {
                self.rst(0x30);
                2
            }
            0xF8 => {
                let val = self.fetch_byte() as i8 as i16;
                let result = val.wrapping_add(self.regs.sp as i16) as u16;
                self.regs.set_hl(result);
                3
            }
            0xF9 => {
                self.load_sp_hl();
                2
            }
            0xFA => {
                let addr = self.fetch_word();
                self.regs.a = self.mem.read8(addr);
                4
            }
            0xFB => {
                self.interrupts = true;
                1
            }
            0xFE => {
                let val = self.fetch_byte();
                self.cp_imm(val);
                2
            }
            0xFF => {
                self.rst(0x38);
                2
            }
            x => {
                panic!("Instruction {:2X} is not implemented", x)
            }
        }
    }

    fn cb_execute(&mut self) -> u32 {
        0
    }
}

#[test]
fn test_add_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.add_imm(0);
    assert_eq!(cpu.regs.a, 0);
    assert!(cpu.regs.isset_flag(FLAG_ZERO));

    cpu.add_imm(0x80);
    assert_eq!(cpu.regs.a, 0x80);
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));

    cpu.add_imm(0x80);
    assert_eq!(cpu.regs.a, 0x00);
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
    assert!(cpu.regs.isset_flag(FLAG_CARRY));

    cpu.add_imm(0x08);
    assert_eq!(cpu.regs.a, 0x08);
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));
    assert!(!cpu.regs.isset_flag(FLAG_HALF));

    cpu.add_imm(0x08);
    assert_eq!(cpu.regs.a, 0x10);
    assert!(cpu.regs.isset_flag(FLAG_HALF));
}

#[test]
fn test_adc_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.adc_imm(0);
    assert_eq!(cpu.regs.a, 0);
    assert!(cpu.regs.isset_flag(FLAG_ZERO));

    cpu.adc_imm(0x80);
    assert_eq!(cpu.regs.a, 0x80);
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));

    cpu.adc_imm(cpu.regs.a);
    assert_eq!(cpu.regs.a, 0x00);
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
    assert!(cpu.regs.isset_flag(FLAG_CARRY));

    cpu.adc_imm(0x00);
    assert_eq!(cpu.regs.a, 0x01);
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_sub_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.add_imm(10);
    assert_eq!(cpu.regs.a, 10);
    assert_eq!(cpu.regs.f, 0);

    cpu.sub_imm(10);
    assert_eq!(cpu.regs.a, 0);
    assert!(cpu.regs.isset_flag(FLAG_SUB));
    assert!(cpu.regs.isset_flag(FLAG_ZERO));

    cpu.sub_imm(1);
    assert_eq!(cpu.regs.a, 0xFF);
    assert!(cpu.regs.isset_flag(FLAG_SUB));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
    assert!(cpu.regs.isset_flag(FLAG_HALF));
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_sbc_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.adc_imm(0x80);
    assert_eq!(cpu.regs.a, 0x80);
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));

    cpu.adc_imm(0x85);
    assert_eq!(cpu.regs.a, 5);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));

    cpu.sbc_imm(2);
    assert_eq!(cpu.regs.a, 2);
    assert!(cpu.regs.isset_flag(FLAG_SUB));

    cpu.sbc_imm(3);
    assert_eq!(cpu.regs.a, 0xFF);
    assert!(cpu.regs.isset_flag(FLAG_SUB));
    assert!(cpu.regs.isset_flag(FLAG_HALF));
    assert!(cpu.regs.isset_flag(FLAG_SUB));
}

#[test]
fn test_and_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x05;

    cpu.and_imm(1);
    assert_eq!(cpu.regs.a, 1);
    assert!(cpu.regs.isset_flag(FLAG_HALF));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));

    cpu.and_imm(0);
    assert_eq!(cpu.regs.a, 0);
    assert!(cpu.regs.isset_flag(FLAG_HALF));
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
}

#[test]
fn test_xor_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x05;

    cpu.xor_imm(4);
    assert_eq!(cpu.regs.a, 1);
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));

    cpu.xor_imm(1);
    assert_eq!(cpu.regs.a, 0);
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
}

#[test]
fn test_or_imm() {
    let mut cpu: Cpu = Default::default();

    cpu.or_imm(0);
    assert_eq!(cpu.regs.a, 0);
    assert!(cpu.regs.isset_flag(FLAG_ZERO));

    cpu.or_imm(0x0A);
    assert_eq!(cpu.regs.a, 0x0A);
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
}

#[test]
fn test_cp_imm() {
    let mut cpu: Cpu = Default::default();
    cpu.add_imm(10);
    cpu.cp_imm(10);
    assert_eq!(cpu.regs.a, 10);
    assert!(cpu.regs.isset_flag(FLAG_SUB));
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
}

#[test]
fn test_inc() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0xF;
    cpu.regs.a = cpu.inc(cpu.regs.a);
    assert_eq!(cpu.regs.a, 0x10);
    assert!(!cpu.regs.isset_flag(FLAG_SUB));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
    assert!(cpu.regs.isset_flag(FLAG_HALF));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));

    cpu.regs.a = 0xFF;
    cpu.regs.a = cpu.inc(cpu.regs.a);
    assert_eq!(cpu.regs.a, 0x00);
    assert!(!cpu.regs.isset_flag(FLAG_SUB));
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
    assert!(!cpu.regs.isset_flag(FLAG_HALF));
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_dec() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x11;
    cpu.regs.a = cpu.dec(cpu.regs.a);
    assert_eq!(cpu.regs.a, 0x10);
    assert!(cpu.regs.isset_flag(FLAG_SUB));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));

    cpu.regs.a = 0x00;
    cpu.regs.a = cpu.dec(cpu.regs.a);
    assert_eq!(cpu.regs.a, 0xFF);
    assert!(cpu.regs.isset_flag(FLAG_SUB));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
    assert!(cpu.regs.isset_flag(FLAG_HALF));
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_daa() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x55;
    cpu.add_imm(0x11);
    cpu.daa();
    assert_eq!(cpu.regs.a, 0x66);
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));

    cpu.regs.a = 0x59;
    cpu.add_imm(0x12);
    cpu.daa();
    assert_eq!(cpu.regs.a, 0x71);
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));

    cpu.regs.a = 0x90;
    cpu.add_imm(0x10);
    cpu.daa();
    assert_eq!(cpu.regs.a, 0x00);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));

    cpu.regs.a = 0x99;
    cpu.add_imm(0x01);
    cpu.daa();
    assert_eq!(cpu.regs.a, 0x00);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_cpl() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x55;
    cpu.cpl();

    assert_eq!(cpu.regs.a, 0xAA);
    assert!(cpu.regs.isset_flag(FLAG_SUB));
    assert!(cpu.regs.isset_flag(FLAG_HALF));
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
    assert!(cpu.regs.isset_flag(FLAG_HALF));

    // check carry flag
    cpu.regs.sp = 0xFFFF;
    cpu.regs.f = 0;
    cpu.add_sp(1);
    assert_eq!(cpu.regs.sp, 0x00);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));

    cpu.regs.sp = 1;
    cpu.regs.f = 0;
    cpu.add_sp(0xFE);
    assert_eq!(cpu.regs.sp, 0xFFFF);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
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

#[test]
fn test_bit() {
    let mut cpu: Cpu = Default::default();
    let val = 0b1000_0000;

    cpu.bit(0, val);
    assert_eq!(cpu.regs.f, FLAG_HALF | FLAG_ZERO);
    cpu.bit(1, val);
    assert_eq!(cpu.regs.f, FLAG_HALF | FLAG_ZERO);
    cpu.bit(2, val);
    assert_eq!(cpu.regs.f, FLAG_HALF | FLAG_ZERO);
    cpu.bit(3, val);
    assert_eq!(cpu.regs.f, FLAG_HALF | FLAG_ZERO);
    cpu.bit(4, val);
    assert_eq!(cpu.regs.f, FLAG_HALF | FLAG_ZERO);
    cpu.bit(5, val);
    assert_eq!(cpu.regs.f, FLAG_HALF | FLAG_ZERO);
    cpu.bit(6, val);
    assert_eq!(cpu.regs.f, FLAG_HALF | FLAG_ZERO);
    cpu.bit(7, val);
    assert_eq!(cpu.regs.f, FLAG_HALF);
}

#[test]
fn test_set() {
    let cpu: Cpu = Default::default();

    for i in 0..8 {
        let result = cpu.set(i, 0);
        assert_eq!(result, (1 << i));
    }
}

#[test]
fn test_rlca() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x85;
    cpu.rlca();
    assert_eq!(cpu.regs.a, 0x0B);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_rlc() {
    let mut cpu: Cpu = Default::default();
    let value = cpu.rlc(0x85);
    assert_eq!(value, 0x0B);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_rla() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x95;
    cpu.regs.toggle_flag(FLAG_CARRY);
    cpu.rla();
    assert_eq!(cpu.regs.a, 0x2B);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_rl() {
    let mut cpu: Cpu = Default::default();
    let value = cpu.rl(0x80);
    assert_eq!(value, 0);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
    assert!(cpu.regs.isset_flag(FLAG_ZERO))
}

#[test]
fn test_rrca() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x3B;
    cpu.rrca();
    assert_eq!(cpu.regs.a, 0x9D);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_rrc() {
    let mut cpu: Cpu = Default::default();
    let value = cpu.rrc(1);
    assert_eq!(value, 0x80);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO))
}

#[test]
fn test_rra() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.a = 0x81;
    cpu.rra();
    assert_eq!(cpu.regs.a, 0x40);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
}

#[test]
fn test_rr() {
    let mut cpu: Cpu = Default::default();
    let value = cpu.rr(1);
    assert_eq!(value, 0);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
}

#[test]
fn test_sla() {
    let mut cpu: Cpu = Default::default();
    let value = cpu.sla(0x80);
    assert_eq!(value, 0);
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_sra() {
    let mut cpu: Cpu = Default::default();
    let value = cpu.sra(0x8A);
    assert_eq!(value, 0xC5);
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
}

#[test]
fn test_srl() {
    let mut cpu: Cpu = Default::default();
    let value = cpu.srl(0x01);
    assert_eq!(value, 0x00);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
}

#[test]
fn test_swap() {
    let mut cpu: Cpu = Default::default();
    let value = cpu.swap(0xA5);
    assert_eq!(value, 0x5A);
    assert!(!cpu.regs.isset_flag(FLAG_ZERO));
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));
    assert!(!cpu.regs.isset_flag(FLAG_SUB));
    assert!(!cpu.regs.isset_flag(FLAG_HALF));

    let value2 = cpu.swap(0x00);
    assert_eq!(value2, 0x00);
    assert!(cpu.regs.isset_flag(FLAG_ZERO));
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));
    assert!(!cpu.regs.isset_flag(FLAG_SUB));
    assert!(!cpu.regs.isset_flag(FLAG_HALF));
}

#[test]
fn test_ccf() {
    let mut cpu: Cpu = Default::default();
    cpu.regs.toggle_flag(FLAG_CARRY);
    assert!(cpu.regs.isset_flag(FLAG_CARRY));

    cpu.ccf();
    assert!(!cpu.regs.isset_flag(FLAG_CARRY));

    cpu.ccf();
    assert!(cpu.regs.isset_flag(FLAG_CARRY));
}

#[test]
fn test_fetch_byte() {
    let mut cpu: Cpu = Default::default();
    cpu.mem.write8(0xC000, 0x42);
    cpu.regs.pc = 0xC000;

    let value = cpu.fetch_byte();
    assert_eq!(cpu.regs.pc, 0xC001);
    assert_eq!(value, 0x42);
}
