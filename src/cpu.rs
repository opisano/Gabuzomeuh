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
const FLAG_NONE: u8 = 0x00;

impl Default for Registers {
    fn default() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }
}

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
}

#[test]
fn test_get_af() {
    let mut regs: Registers = Default::default();
    regs.a = 0x55;
    regs.f = 0xA0;
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
    let mut regs: Registers = Default::default();
    regs.b = 0x55;
    regs.c = 0xAA;
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
    let mut regs: Registers = Default::default();
    regs.d = 0x55;
    regs.e = 0xAA;
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
    let mut regs: Registers = Default::default();
    regs.h = 0x55;
    regs.l = 0xAA;
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
