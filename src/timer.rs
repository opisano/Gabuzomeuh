const TICKS: u32 = 16;
const DIVIDER_PERIOD: u32 = 256;

pub struct Timer {
    div: u8,
    tima: u8,
    tma: u8,
    enabled: bool,
    step: u32,
    internal_div: u32,
    internal_count: u32,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            div: 0x18,
            tima: 0x00,
            tma: 0x00,
            enabled: false,
            step: 256,
            internal_div: 0,
            internal_count: 0,
        }
    }
}

impl Timer {
    pub fn read_div(&self) -> u8 {
        self.div
    }

    pub fn write_div(&mut self, _value: u8) {
        self.div = 0;
    }

    pub fn read_tima(&self) -> u8 {
        self.tima
    }

    pub fn write_tima(&mut self, value: u8) {
        self.tima = value;
    }

    pub fn read_tma(&self) -> u8 {
        self.tma
    }

    pub fn read_tac(&self) -> u8 {
        let clk = match self.step {
            1024 => 0b00,
            16 => 0b01,
            64 => 0b10,
            256 => 0b11,
            _ => panic!("Inconsistent step value"),
        };

        let en = if self.enabled { 0b100 } else { 0 };
        en | clk | 0xF8
    }

    pub fn write_tac(&mut self, value: u8) {
        self.step = match value & 0b11 {
            0b00 => 1024,
            0b01 => 16,
            0b10 => 64,
            0b11 => 256,
            _ => panic!("Inconsistent step value"),
        };

        self.enabled = if value & 0b100 == 0 { false } else { true };
    }

    pub fn write_tma(&mut self, value: u8) {
        self.tma = value
    }

    /// For performance reasons, this method shall only be called every 16 cycles
    /// since this is the highest frequency this timer supports
    pub fn cycle(&mut self) {
        self.internal_div += TICKS;

        if self.internal_div >= DIVIDER_PERIOD {
            self.div = self.div.wrapping_add(1u8);
            self.internal_div -= DIVIDER_PERIOD;
        }

        if self.enabled {
            self.internal_count += TICKS;
            if self.internal_count >= self.step {
                self.tima = if self.tima == 0xFF {
                    self.tma
                } else {
                    self.tima + 1
                };
                // TODO interrupt
                self.internal_count -= self.step;
            }
        }
    }
}

#[test]
fn test_divider() {
    let mut t: Timer = Default::default();
    t.write_div(0x42);
    assert_eq!(t.div, 0); // writing any value to div sets it to 0

    t.write_tima(0);
    t.write_tma(0x42);
    t.write_tac(0b001); // timer disabled

    assert_eq!(t.enabled, false);
    assert_eq!(t.step, 16);

    for _ in 0..10 {
        t.cycle();
    }
    assert_eq!(t.div, 0);
    assert_eq!(t.tima, 0);

    for _ in 0..10 {
        t.cycle();
    }

    assert_eq!(t.div, 1);
    assert_eq!(t.tima, 0);
}

#[test]
fn test_timer() {
    let mut t: Timer = Default::default();
    t.write_div(0x42);
    assert_eq!(t.div, 0); // writing any value to div sets it to 0

    t.write_tima(0);
    t.write_tma(0x42);
    t.write_tac(0b101); // timer enabled at max frequency

    assert_eq!(t.enabled, true);
    assert_eq!(t.step, 16);

    for _ in 0..10 {
        t.cycle();
    }
    assert_eq!(t.div, 0);
    assert_eq!(t.tima, 10);

    for _ in 0..246 {
        t.cycle();
    }
    // check tima is set to tma when it overflows
    assert_eq!(t.tima, 0x42);
    assert_eq!(t.div, 16);
}
