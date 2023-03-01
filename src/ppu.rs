use std::{f32::consts::PI, thread::current};

const OAM_SEARCH_CYCLES: u32 = 80;
const PIXEL_CYCLES: u32 = 172;
const HBLANK_CYCLES: u32 = 204;
const TILE_SIZE: u8 = 8;

const COLS: usize = 160;
const ROWS: usize = 144;
const VBLANK_ROWS: usize = 10;

const VRAM_SIZE: usize = 8_192;
const OAM_SIZE: usize = 160;

pub const PPU_VBLANK_INTERRUPT: u8 = 0x01;
pub const PPU_STAT_INTERRUPT: u8 = 0x02;

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
enum Mode {
    HBlank,
    VBlank,
    SearchOam,
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
    data: [u8; COLS * ROWS],
    sprites: [SpriteInfo; 10],
    ticks: u32,
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
    inter: u8,
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
            data: [0; COLS * ROWS],
            sprites: [SpriteInfo {
                y: 0,
                x: 0,
                index: 0,
            }; 10],
            ticks: 0u32,
            bg_palette: [Color::White, Color::LGray, Color::DGray, Color::Black],
            sprite_palette1: [Color::LGray, Color::DGray, Color::Black],
            sprite_palette2: [Color::LGray, Color::DGray, Color::Black],
            win_tile_map_addr: Default::default(),
            bg_tile_data_addr: Default::default(),
            bg_tile_map_addr: Default::default(),
            mode: Mode::SearchOam,
            sprite_height: Default::default(),
            ly: Default::default(),
            lyc: Default::default(),
            scy: Default::default(),
            scx: Default::default(),
            winx: Default::default(),
            winy: Default::default(),
            inter: Default::default(),
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

    pub fn interrupt(&self) -> u8 {
        self.inter
    }

    pub fn cycle(&mut self, ticks: u32) {
        if !self.lcd_display_enabled {
            return;
        }

        let mut ticks_left = ticks;
        while ticks_left > 0 {
            let current_ticks = if ticks_left >= 80 { 80 } else { ticks_left };
            self.ticks += current_ticks;
            ticks_left -= current_ticks;

            if self.ticks >= 456 {
                self.ticks -= 456;
                self.ly = (self.ly + 1) % (ROWS + VBLANK_ROWS) as u8;
                self.check_interrupt_lyc();

                if self.ly >= 144 && self.mode != Mode::VBlank {
                    self.switch_to_vblank_mode();
                }
            }

            if self.ly < ROWS as u8 {
                if self.ticks <= OAM_SEARCH_CYCLES {
                    if self.mode != Mode::SearchOam {
                        self.switch_to_search_oam_mode();
                        self.oam_search();
                    }
                } else if self.ticks <= OAM_SEARCH_CYCLES + PIXEL_CYCLES {
                    if self.mode != Mode::Transfering {
                        self.switch_to_transfering_mode();
                        self.draw();
                    }
                } else {
                    if self.mode != Mode::HBlank {
                        self.switch_to_hblank_mode();
                    }
                }
            }
        }
    }

    fn draw(&mut self) {
        self.draw_bg();
        self.draw_sprites();
    }

    fn draw_bg(&mut self) {
        // if drawing background/window is disabled, we have nothing to do
        if !self.bg_window_enable {
            return;
        }

        let bg_y = self.ly.wrapping_add(self.scy) as u16;

        for x in 0..COLS {
            let bg_x = self.scx.wrapping_add(x as u8) as u16;
            let tile_y = bg_y / TILE_SIZE as u16;
            let tile_x = bg_x / TILE_SIZE as u16;
            let tile_idx = self.read_vram(self.bg_tile_map_addr + tile_y * 32 + tile_x);

            let tile_addr = if self.bg_tile_data_addr == 0x8000 {
                self.bg_tile_data_addr + tile_idx as u16 * 16
            } else {
                self.bg_tile_data_addr + (tile_idx as i8 as i16 + 128) as u16 * 16
            };

            let pixel_y = bg_y & 0x07;
            let byte1 = self.read_vram(tile_addr + (pixel_y * 2));
            let byte2 = self.read_vram(tile_addr + (pixel_y * 2) + 1);

            let pixel_x = bg_x & 0x07;
            let color = if byte1 & (1 << pixel_x) != 0 {
                0b01u8
            } else {
                0
            } | if byte2 & (1 << pixel_x) != 0 {
                0b10u8
            } else {
                0
            };
            self.data[self.ly as usize * COLS + x] = color;
        }

        // TODO draw window
    }

    fn draw_sprites(&mut self) {}

    /// Search for the up to 10 first sprites to draw for current line
    fn oam_search(&mut self) {
        let entries = self
            .oam
            .chunks_exact(4)
            .enumerate()
            .filter(|(i, e)| self.ly >= e[0] && self.ly < e[0] + self.sprite_height)
            .take(10);

        let mut arr_idx = 0;
        for (i, entry) in entries {
            self.sprites[arr_idx] = SpriteInfo {
                y: entry[0],
                x: entry[1],
                index: (i * 4) as u8,
            };
            arr_idx += 1;
        }
    }

    fn check_interrupt_lyc(&mut self) {
        if self.lyc_interrupt_enabled && self.ly == self.lyc {
            self.inter |= PPU_STAT_INTERRUPT;
        }
    }

    fn switch_to_vblank_mode(&mut self) {
        self.mode = Mode::VBlank;
        self.inter |= PPU_VBLANK_INTERRUPT;
        if self.vblank_interrupt_enabled {
            self.inter |= PPU_STAT_INTERRUPT;
        }
    }

    fn switch_to_search_oam_mode(&mut self) {
        self.mode = Mode::SearchOam;
        if self.oam_interrupt_enabled {
            self.inter |= PPU_STAT_INTERRUPT;
        }
    }

    fn switch_to_transfering_mode(&mut self) {
        self.mode = Mode::Transfering;
    }

    fn switch_to_hblank_mode(&mut self) {
        self.mode = Mode::HBlank;
        if self.hblank_interrupt_enabled {
            self.inter |= PPU_STAT_INTERRUPT;
        }
    }
}
