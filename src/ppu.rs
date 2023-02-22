const OAM_SEARCH_CYCLES: i32 = 20;
const PIXEL_CYCLES: i32 = 42;
const HBLANK_CYCLES: i32 = 51;

const COLS: i32 = 160;
const ROWS: i32 = 144;
const VBLANK_ROWS: i32 = 10;

const VRAM_SIZE: usize = 8_192;
const OAM_SIZE: usize = 160;

#[derive(Copy, Clone)]
#[repr(u8)]
enum Mode {
    HBlank,
    VBlank,
    SearchingOam,
    Transfering,
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum Color {
    White,
    LGray,
    DGray,
    Black,
}

impl Color {
    fn from_int(value: u8) -> Color {
        match value & 0x03 {
            0 => Color::White,
            1 => Color::LGray,
            2 => Color::DGray,
            3 => Color::Black,
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_from_int() {
    // Ensures 6 higher bits are not taken into account
    assert!(matches!(Color::from_int(0b1111_1111), Color::Black));
    assert!(matches!(Color::from_int(0b1010_1010), Color::DGray));
    assert!(matches!(Color::from_int(0b0101_0101), Color::LGray));
    assert!(matches!(Color::from_int(0b1001_1000), Color::White));
}

#[derive(Copy, Clone)]
struct SpriteInfo {
    y: u8,
    x: u8,
    index: u8,
}

pub struct Ppu {
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    bg_palette: [Color; 4],
    sprite_palette1: [Color; 3],
    sprite_palette2: [Color; 3],
    win_tile_map_addr: u16,
    bg_tile_data_addr: u16,
    bg_tile_map_addr: u16,
    mode: Mode,
    sprite_height: u8,
    ly: u8,
    lyc: u8,
    scy: u8,
    scx: u8,
    winx: u8,
    winy: u8,
    lcd_display_enabled: bool,
    win_enable: bool,
    sprite_enabled: bool,
    bg_window_enable: bool,
    hblank_interrupt_enabled: bool,
    vblank_interrupt_enabled: bool,
    oam_interrupt_enabled: bool,
    lyc_interrupt_enabled: bool,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            bg_palette: [Color::White, Color::LGray, Color::DGray, Color::Black],
            sprite_palette1: [Color::LGray, Color::DGray, Color::Black],
            sprite_palette2: [Color::LGray, Color::DGray, Color::Black],
            win_tile_map_addr: Default::default(),
            bg_tile_data_addr: Default::default(),
            bg_tile_map_addr: Default::default(),
            mode: Mode::SearchingOam,
            sprite_height: Default::default(),
            ly: Default::default(),
            lyc: Default::default(),
            scy: Default::default(),
            scx: Default::default(),
            winx: Default::default(),
            winy: Default::default(),
            lcd_display_enabled: Default::default(),
            win_enable: Default::default(),
            sprite_enabled: Default::default(),
            bg_window_enable: Default::default(),
            hblank_interrupt_enabled: Default::default(),
            vblank_interrupt_enabled: Default::default(),
            oam_interrupt_enabled: Default::default(),
            lyc_interrupt_enabled: Default::default(),
        }
    }
}

impl Ppu {
    pub fn read_control(&self) -> u8 {
        let bit7 = if self.lcd_display_enabled { 0x80 } else { 0x00 };
        let bit6 = if self.win_tile_map_addr == 0x9C00 {
            0x40
        } else {
            0x00
        };
        let bit5 = if self.win_enable { 0x20 } else { 0x00 };
        let bit4 = if self.bg_tile_data_addr == 0x8000 {
            0x10
        } else {
            0x00
        };
        let bit3 = if self.bg_tile_map_addr == 0x9C00 {
            0x08
        } else {
            0x00
        };
        let bit2 = if self.sprite_height == 16 { 0x04 } else { 0x00 };
        let bit1 = if self.sprite_enabled { 0x02 } else { 0x00 };
        let bit0 = if self.bg_window_enable { 0x01 } else { 0x00 };
        bit7 | bit6 | bit5 | bit4 | bit3 | bit2 | bit1 | bit0
    }

    pub fn write_control(&mut self, value: u8) {
        self.lcd_display_enabled = (value & 0x80) == 0x80;
        self.win_tile_map_addr = if (value & 0x40) == 0x40 {
            0x9C00
        } else {
            0x9800
        };
        self.win_enable = (value & 0x20) == 0x20;
        self.bg_tile_data_addr = if (value & 0x10) == 0x10 {
            0x8000
        } else {
            0x8800
        };
        self.bg_tile_map_addr = if (value & 0x08) == 0x08 { 0x9C00 } else { 9800 };
        self.sprite_height = if (value & 0x04) == 0x04 { 16 } else { 8 };
        self.sprite_enabled = (value & 0x02) == 0x02;
        self.bg_window_enable = (value & 0x01) == 0x01;
    }

    pub fn read_status(&self) -> u8 {
        let bits01 = self.mode as u8;
        let bit2 = if self.ly == self.lyc { 0x04 } else { 0x00 };
        let bit3 = if self.hblank_interrupt_enabled {
            0x08
        } else {
            0x00
        };
        let bit4 = if self.vblank_interrupt_enabled {
            0x10
        } else {
            0x00
        };
        let bit5 = if self.oam_interrupt_enabled {
            0x20
        } else {
            0x00
        };
        let bit6 = if self.lyc_interrupt_enabled {
            0x40
        } else {
            0x00
        };
        0x80 | bit6 | bit5 | bit4 | bit3 | bit2 | bits01
    }

    pub fn write_status(&mut self, value: u8) {
        self.hblank_interrupt_enabled = (value & 0x08) == 0x08;
        self.vblank_interrupt_enabled = (value & 0x10) == 0x10;
        self.oam_interrupt_enabled = (value & 0x20) == 0x20;
        self.lyc_interrupt_enabled = (value & 0x40) == 0x40;
    }

    pub fn read_scy(&self) -> u8 {
        self.scy
    }

    pub fn write_scy(&mut self, value: u8) {
        self.scy = value;
    }

    pub fn read_scx(&self) -> u8 {
        self.scx
    }

    pub fn write_scx(&mut self, value: u8) {
        self.scx = value;
    }

    pub fn read_ly(&self) -> u8 {
        self.ly
    }

    pub fn read_lyc(&self) -> u8 {
        self.lyc
    }

    pub fn write_lyc(&mut self, value: u8) {
        self.lyc = value;
    }

    pub fn read_bgp(&self) -> u8 {
        let bits01 = self.bg_palette[0] as u8;
        let bits23 = (self.bg_palette[1] as u8) << 2;
        let bits45 = (self.bg_palette[2] as u8) << 4;
        let bits67 = (self.bg_palette[3] as u8) << 6;
        bits67 | bits45 | bits23 | bits01
    }

    pub fn write_bgp(&mut self, value: u8) {
        self.bg_palette[0] = Color::from_int(value);
        self.bg_palette[1] = Color::from_int(value >> 2);
        self.bg_palette[2] = Color::from_int(value >> 4);
        self.bg_palette[3] = Color::from_int(value >> 6);
    }

    pub fn read_obp0(&self) -> u8 {
        let bits23 = (self.sprite_palette1[0] as u8) << 2;
        let bits45 = (self.sprite_palette1[1] as u8) << 4;
        let bits67 = (self.sprite_palette1[2] as u8) << 6;
        bits67 | bits45 | bits23
    }

    pub fn write_obp0(&mut self, value: u8) {
        self.sprite_palette1[0] = Color::White;
        self.sprite_palette1[1] = Color::from_int(value >> 2);
        self.sprite_palette1[2] = Color::from_int(value >> 4);
        self.sprite_palette1[3] = Color::from_int(value >> 6);
    }

    pub fn read_obp1(&self) -> u8 {
        let bits23 = (self.sprite_palette2[0] as u8) << 2;
        let bits45 = (self.sprite_palette2[1] as u8) << 4;
        let bits67 = (self.sprite_palette2[2] as u8) << 6;
        bits67 | bits45 | bits23
    }

    pub fn write_obp1(&mut self, value: u8) {
        self.sprite_palette2[0] = Color::White;
        self.sprite_palette2[1] = Color::from_int(value >> 2);
        self.sprite_palette2[2] = Color::from_int(value >> 4);
        self.sprite_palette2[3] = Color::from_int(value >> 6);
    }

    pub fn read_wy(&self) -> u8 {
        self.winy
    }

    pub fn write_wy(&mut self, value: u8) {
        self.winy = value;
    }

    pub fn read_wx(&self) -> u8 {
        self.winx
    }

    pub fn write_wx(&mut self, value: u8) {
        self.winx = value;
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        let local_index = (addr & 0x1FFF) as usize;
        self.vram[local_index]
    }

    pub fn write_vram(&mut self, addr: u16, value: u8) {
        let local_index = (addr & 0x1FFF) as usize;
        self.vram[local_index] = value;
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        let local_index = addr as usize - 0xFE00;
        self.oam[local_index]
    }

    pub fn write_oam(&mut self, addr: u16, value: u8) {
        let local_index = addr as usize - 0xFE00;
        self.oam[local_index] = value;
    }

    /// Search for the up to 10 first sprites to draw for current line
    fn oam_search(&self, values: &mut [SpriteInfo; 10]) {
        let entries = self
            .oam
            .chunks_exact(4)
            .enumerate()
            .filter(|(i, e)| self.ly >= e[0] && self.ly < e[0] + self.sprite_height)
            .take(10);

        let mut arr_idx = 0;
        for (i, entry) in entries {
            values[arr_idx] = SpriteInfo {
                y: entry[0],
                x: entry[1],
                index: (i * 4) as u8,
            };
            arr_idx += 1;
        }
    }
}
