use std::io::BufReader;

use bincode::Decode;
use bog::{Bus, Cpu, Pins, Status};

#[derive(Decode)]
struct State {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<[u16; 2]>,
}

#[derive(Decode)]
#[allow(dead_code)]
struct Cycle {
    address: u16,
    data: u8,
    kind: String,
}

#[derive(Decode)]
#[allow(dead_code)]
struct Test {
    name: String,
    initial: State,
    r#final: State,
    cycles: Vec<Cycle>,
}

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
fn all() {
    let mut cpu = Cpu::new(ProcessorTestBus::new());

    for opcode in 0x00..=0xff {
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
            continue;
        }

        let filename = format!("roms/ProcessorTests/{:02x}.bincode", opcode);
        let file = std::fs::File::open(filename).unwrap();
        let mut buf_reader = BufReader::new(file);
        let tests: Vec<Test> = bincode::decode_from_std_read(
            &mut buf_reader,
            bincode::config::standard(),
        )
        .unwrap();

        for test in tests {
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
    }
}
