use std::io::BufReader;

use bog::{Bus, Cpu, Pins, Status};
use serde::Deserialize;

const FUNCTIONAL_TEST_ROM: &[u8] =
    include_bytes!("../roms/6502_functional_test.bin");
const INTERRUPT_TEST_ROM: &[u8] =
    include_bytes!("../roms/6502_interrupt_test.bin");

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

#[derive(Deserialize)]
struct State {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<[u16; 2]>,
}

#[derive(Deserialize)]
struct Test {
    // name: String,
    initial: State,
    r#final: State,
    // cycles: Vec<Cycle>,
}

// #[derive(Deserialize)]
// struct Cycle {
//     address: u16,
//     data: u8,
//     kind: String,
// }

struct ProcessorTestBus {
    memory: [u8; 0x10000],
    // cycle_count: usize,
    // cycles: Vec<Cycle>,
}

impl ProcessorTestBus {
    fn new() -> ProcessorTestBus {
        ProcessorTestBus {
            memory: [0; 0x10000],
        }
    }
}

impl Bus for ProcessorTestBus {
    fn tick(&mut self, pins: &mut Pins) {
        // let cycle = &self.cycles[self.cycle_count];

        match pins.rw {
            true => {
                // assert_eq!(cycle.kind, "read");
                pins.data = self.memory[pins.address as usize]
            }
            false => {
                // assert_eq!(cycle.kind, "write");
                self.memory[pins.address as usize] = pins.data
            }
        }

        // assert_eq!(cycle.address, pins.address, "address");
        // assert_eq!(cycle.data, pins.data, "data");

        // self.cycle_count += 1;
    }
}

#[test]
fn functional_test() {
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
fn interrupt_test() {
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

#[test]
fn processor_tests() {
    (0x00..=0xffu8).for_each(|opcode| {
        // These opcodes aren't implemented yet.
        if matches!(
            opcode,
            0x02 | 0x0b
                | 0x12
                | 0x22
                | 0x2b
                | 0x32
                | 0x42
                | 0x4b
                | 0x52
                | 0x62
                | 0x6b
                | 0x72
                | 0x8b
                | 0x92
                | 0x93
                | 0x9b
                | 0x9c
                | 0x9e
                | 0x9f
                | 0xab
                | 0xb2
                | 0xbb
                | 0xcb
                | 0xd2
                | 0xf2
        ) {
            return;
        }

        let filename =
            format!("../ProcessorTests/nes6502/v1/{:02x}.json", opcode);
        let file = std::fs::File::open(filename).unwrap();
        let buf_reader = BufReader::new(file);
        let tests: Vec<Test> = serde_json::from_reader(buf_reader).unwrap();

        for test in tests {
            let mut cpu = Cpu::new(ProcessorTestBus::new());

            let initial = test.initial;
            cpu.pc = initial.pc;
            cpu.s = initial.s;
            cpu.a = initial.a;
            cpu.x = initial.x;
            cpu.y = initial.y;
            cpu.p = Status::from_bits(initial.p).unwrap();
            for [address, data] in initial.ram {
                cpu.bus.memory[address as usize] = data as u8;
            }

            cpu.step();

            let r#final = test.r#final;
            assert_eq!(cpu.pc, r#final.pc);
            assert_eq!(cpu.s, r#final.s);
            assert_eq!(cpu.a, r#final.a);
            assert_eq!(cpu.x, r#final.x);
            assert_eq!(cpu.y, r#final.y);
            assert_eq!(cpu.p.bits(), r#final.p);
            for [address, data] in r#final.ram {
                assert_eq!(cpu.bus.memory[address as usize], data as u8);
            }
        }
    });
}
