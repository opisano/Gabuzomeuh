pub const JOYPAD_INTERRUPT: u8 = 0x10;

pub enum ButtonSelection {
    Action,
    Direction,
}

impl Default for ButtonSelection {
    fn default() -> Self {
        ButtonSelection::Action
    }
}

/*
 * These constants are the index of buttons in the joypadState struct.
 */

const A_IDX: usize = 0;
const B_IDX: usize = 1;
const SELECT_IDX: usize = 2;
const START_IDX: usize = 3;
const RIGHT_IDX: usize = 4;
const LEFT_IDX: usize = 5;
const UP_IDX: usize = 6;
const DOWN_IDX: usize = 7;

#[derive(Default)]
pub struct JoypadState {
    pub buttons: [bool; 8],
    inter: u8,
    button_selection: ButtonSelection,
}

impl JoypadState {
    pub fn write(&mut self, val: u8) {
        // Action buttons
        if (val & 0b0010_0000) == 0 {
            self.button_selection = ButtonSelection::Action;
        } else if (val & 0b0001_0000) == 0 {
            self.button_selection = ButtonSelection::Direction;
        }
    }

    pub fn read(&self) -> u8 {
        match self.button_selection {
            ButtonSelection::Action => {
                self.bit_a() | self.bit_b() | self.bit_sel() | self.bit_sta() | 0xC0
            }
            ButtonSelection::Direction => {
                self.bit_r() | self.bit_l() | self.bit_u() | self.bit_d() | 0xC0
            }
        }
    }

    pub fn interrupt(&self) -> u8 {
        self.inter
    }

    fn bit_a(&self) -> u8 {
        if self.buttons[A_IDX] {
            0
        } else {
            1
        }
    }

    fn bit_b(&self) -> u8 {
        if self.buttons[B_IDX] {
            0
        } else {
            0b10
        }
    }

    fn bit_sel(&self) -> u8 {
        if self.buttons[SELECT_IDX] {
            0
        } else {
            0b100
        }
    }

    fn bit_sta(&self) -> u8 {
        if self.buttons[START_IDX] {
            0
        } else {
            0b1000
        }
    }

    fn bit_r(&self) -> u8 {
        if self.buttons[RIGHT_IDX] {
            0
        } else {
            1
        }
    }

    fn bit_l(&self) -> u8 {
        if self.buttons[LEFT_IDX] {
            0
        } else {
            0b10
        }
    }

    fn bit_u(&self) -> u8 {
        if self.buttons[UP_IDX] {
            0
        } else {
            0b100
        }
    }

    fn bit_d(&self) -> u8 {
        if self.buttons[DOWN_IDX] {
            0
        } else {
            0b1000
        }
    }
}

#[test]
fn test_read_write() {
    // setup initial state
    let mut js: JoypadState = Default::default();
    js.buttons[B_IDX] = false;
    js.buttons[A_IDX] = true;
    js.buttons[START_IDX] = true;
    js.buttons[SELECT_IDX] = false;
    js.buttons[RIGHT_IDX] = false;
    js.buttons[LEFT_IDX] = false;
    js.buttons[UP_IDX] = true;
    js.buttons[DOWN_IDX] = false;

    js.write(0b1101_1111); // select action buttons
    let action_mask = js.read();
    assert_eq!(action_mask & 0x01, 0); // A pressed
    assert_ne!(action_mask & 0x02, 0); // B not pressed
    assert_ne!(action_mask & 0x04, 0); // select not pressed
    assert_eq!(action_mask & 0x08, 0); // start pressed

    js.write(0b1110_1111); // select direction buttons
    let dir_mask = js.read();
    assert_ne!(dir_mask & 0x01, 0); // right not pressed
    assert_ne!(dir_mask & 0x02, 0); // left not pressed
    assert_eq!(dir_mask & 0x04, 0); // up pressed
    assert_ne!(dir_mask & 0x08, 0); // down not pressed
}
