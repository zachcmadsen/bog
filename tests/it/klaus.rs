use bog::{Bus, Cpu, Pins};

const FUNCTIONAL_TEST_ROM: &[u8] =
    include_bytes!("../../roms/6502_functional_test.bin");
const INTERRUPT_TEST_ROM: &[u8] =
    include_bytes!("../../roms/6502_interrupt_test.bin");

const ZERO_PAGE_START: usize = 0xa;
const CODE_SEGMENT_START: u16 = 0x400;
const INTERRUPT_FEEDBACK_REGISTER: u16 = 0xbffc;
const IRQ_MASK: u8 = 0x1;
const NMI_MASK: u8 = 0x2;
const FUNCTIONAL_TEST_SUCCESS: u16 = 0x336d;
const INTERRUPT_TEST_SUCCESS: u16 = 0x6f5;

struct FunctionalTestBus {
    memory: [u8; 0x10000],
}

impl FunctionalTestBus {
    fn new() -> FunctionalTestBus {
        let mut memory = [0; 0x10000];
        memory[ZERO_PAGE_START..].copy_from_slice(FUNCTIONAL_TEST_ROM);
        FunctionalTestBus { memory }
    }
}

impl Bus for FunctionalTestBus {
    fn tick(&mut self, pins: &mut Pins) {
        match pins.rw {
            true => pins.data = self.memory[pins.address as usize],
            false => self.memory[pins.address as usize] = pins.data,
        }
    }
}

struct InterruptTestBus {
    memory: [u8; 0x10000],
}

impl InterruptTestBus {
    fn new() -> InterruptTestBus {
        let mut memory = [0; 0x10000];
        memory[ZERO_PAGE_START..].copy_from_slice(INTERRUPT_TEST_ROM);
        InterruptTestBus { memory }
    }
}

impl Bus for InterruptTestBus {
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
    let mut cpu = Cpu::new(FunctionalTestBus::new());
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
    let mut cpu = Cpu::new(InterruptTestBus::new());
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
