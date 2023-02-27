use crate::cpu::Cpu;

pub struct Console {
    cpu: Cpu,
}

impl Console {
    pub fn new() -> Self {
        let console = Console {
            cpu: Default::default(),
        };

        console
    }

    pub fn cycle(&mut self) {
        self.cpu.cycle();
    }
}
