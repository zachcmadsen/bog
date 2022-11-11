#![no_std]

mod cpu;

pub use cpu::Cpu;

pub struct Pins {
    pub address: u16,
    pub data: u8,
    pub rw: bool,
    pub irq: bool,
    pub nmi: bool,
}

impl Default for Pins {
    fn default() -> Self {
        Pins {
            address: 0,
            data: 0,
            rw: true,
            irq: false,
            nmi: false,
        }
    }
}

pub trait Bus {
    fn tick(&mut self, pins: &mut Pins);
}
