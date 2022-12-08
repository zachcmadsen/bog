use std::{
    fs::{self, File},
    io::BufReader,
};

use bincode::Decode;
use bog::{Cpu, Status};

use crate::nes::{NesBus, NromCartridge};

const NESTEST_START_ADDRESS: u16 = 0xc000;

#[derive(Decode)]
struct State {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    s: u8,
    cycles: u64,
}

#[test]
fn nestest() {
    let rom = fs::read("roms/nestest/nestest.nes")
        .expect("roms/nestest/nestest.nes should exist");

    let cartridge = NromCartridge::new(&rom);
    let bus = NesBus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    // Run through the reset sequence.
    cpu.step();
    cpu.pc = NESTEST_START_ADDRESS;
    // The nestest log has a different initial values for the status register
    // and stack pointer.
    cpu.p = Status::from_bits(0x24).unwrap();
    cpu.s = 0xfd;

    let log = File::open("roms/nestest/nestest_log.bincode")
        .expect("roms/nestest/nestest_log.bincode should exist");
    let mut buf_reader = BufReader::new(log);
    let expected_states: Vec<State> = bincode::decode_from_std_read(
        &mut buf_reader,
        bincode::config::standard(),
    )
    .unwrap();

    for state in expected_states {
        assert_eq!(cpu.pc, state.pc);
        assert_eq!(cpu.a, state.a);
        assert_eq!(cpu.x, state.x);
        assert_eq!(cpu.y, state.y);
        assert_eq!(cpu.p.bits(), state.p);
        assert_eq!(cpu.s, state.s);
        assert_eq!(cpu.cycles, state.cycles);

        cpu.step();
    }
}
