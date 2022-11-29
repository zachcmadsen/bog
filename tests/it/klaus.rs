use std::fs;

use bog::{Bus, Cpu, Pins};

const ZERO_PAGE_START: usize = 0xa;
const CODE_SEGMENT_START: u16 = 0x400;
const INTERRUPT_FEEDBACK_REGISTER: u16 = 0xbffc;
const IRQ_MASK: u8 = 0x1;
const NMI_MASK: u8 = 0x2;
const FUNCTIONAL_TEST_SUCCESS: u16 = 0x336d;
const INTERRUPT_TEST_SUCCESS: u16 = 0x6f5;

struct KlausTestBus {
    memory: [u8; 0x10000],
}

impl KlausTestBus {
    fn new(rom: &[u8]) -> KlausTestBus {
        let mut memory = [0; 0x10000];
        memory[ZERO_PAGE_START..].copy_from_slice(&rom);

        KlausTestBus { memory }
    }
}

impl Bus for KlausTestBus {
    fn tick(&mut self, pins: &mut Pins) {
        match pins.rw {
            true => pins.data = self.memory[pins.address as usize],
            false => {
                if pins.address == INTERRUPT_FEEDBACK_REGISTER {
                    let old_data = self.memory[pins.address as usize];
                    let prev_nmi = old_data & NMI_MASK != 0;
                    let new_nmi = pins.data & NMI_MASK != 0;

                    pins.irq = pins.data & IRQ_MASK != 0;
                    pins.nmi = !prev_nmi && new_nmi;
                }

                self.memory[pins.address as usize] = pins.data;
            }
        }
    }
}

#[test]
fn functional() {
    let rom = fs::read("roms/klaus/6502_functional_test.bin")
        .expect("roms/klaus/6502_functional_test.bin should exist");
    let mut cpu = Cpu::new(KlausTestBus::new(&rom));

    cpu.pc = CODE_SEGMENT_START;
    let mut prev_pc = cpu.pc;

    loop {
        cpu.step();

        if prev_pc == cpu.pc {
            if cpu.pc == FUNCTIONAL_TEST_SUCCESS {
                break;
            }

            panic!("trapped at 0x{:04X}", cpu.pc);
        }

        prev_pc = cpu.pc;
    }
}

#[test]
fn interrupt() {
    let rom = fs::read("roms/klaus/6502_interrupt_test.bin")
        .expect("roms/klaus/6502_interrupt_test.bin should exist");
    let mut cpu = Cpu::new(KlausTestBus::new(&rom));

    cpu.pc = CODE_SEGMENT_START;
    let mut prev_pc = cpu.pc;

    loop {
        cpu.step();

        if prev_pc == cpu.pc {
            if cpu.pc == INTERRUPT_TEST_SUCCESS {
                break;
            }

            panic!("trapped at 0x{:04X}", cpu.pc);
        }

        prev_pc = cpu.pc;
    }
}
