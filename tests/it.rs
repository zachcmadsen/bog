use bog::{Bus, Cpu, Pins};

const FUNCTIONAL_TEST_ROM: &[u8] =
    include_bytes!("../roms/6502_functional_test.bin");

// These values are based on 6502_functional_test.lst.
const FUNCTIONAL_TEST_ZERO_PAGE_START: usize = 0xa;
const FUNCTIONAL_TEST_CODE_SEGMENT_START: u16 = 0x0400;
const FUNCTIONAL_TEST_SUCCESS: u16 = 0x336d;

struct TestBus {
    memory: [u8; 0x10000],
}

impl Bus for TestBus {
    fn tick(&mut self, pins: &mut Pins) {
        match pins.rw {
            true => pins.data = self.memory[pins.address as usize],
            false => self.memory[pins.address as usize] = pins.data,
        }
    }
}

#[test]
fn functional_test() {
    let mut memory = [0; 0x10000];
    memory[FUNCTIONAL_TEST_ZERO_PAGE_START..]
        .copy_from_slice(FUNCTIONAL_TEST_ROM);
    let mut cpu = Cpu::new(TestBus { memory });

    cpu.pc = FUNCTIONAL_TEST_CODE_SEGMENT_START;
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
