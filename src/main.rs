use console::Console;

mod cartridge;
mod console;
mod cpu;
mod joypad;
mod memory;
mod ppu;
mod timer;

fn main() {
    let mut console = Console::new();
    console.cycle();
}
