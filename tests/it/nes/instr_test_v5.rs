use std::fs;

use bog::Cpu;

use crate::nes::{NesBus, NromCartridge};

fn run(rom_filepath: &str) {
    let rom = fs::read(rom_filepath)
        .unwrap_or_else(|_| panic!("{} should exist", rom_filepath));

    let cartridge = NromCartridge::new(&rom);
    let bus = NesBus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    // Run through the reset sequence.
    cpu.step();

    const STATUS_ADDRESS: u16 = 0x6000;
    const RUNNING: u8 = 0x80;
    const OUTPUT_ADDRESS: u16 = 0x6004;

    let mut status = cpu.bus.cartridge.read_prg(STATUS_ADDRESS);
    while status != RUNNING {
        cpu.step();
        status = cpu.bus.cartridge.read_prg(STATUS_ADDRESS);
    }

    while status == RUNNING {
        cpu.step();
        status = cpu.bus.cartridge.read_prg(STATUS_ADDRESS);
    }

    let mut output = Vec::new();
    let mut address = OUTPUT_ADDRESS;
    let mut byte = cpu.bus.cartridge.read_prg(address);
    while byte != b'\0' {
        output.push(byte);
        address += 1;
        byte = cpu.bus.cartridge.read_prg(address);
    }

    assert!(String::from_utf8_lossy(&output).contains("Passed"));
}

#[test]
fn basics() {
    run("roms/instr_test_v5/01-basics.nes");
}

#[test]
fn implied() {
    run("roms/instr_test_v5/02-implied.nes");
}

#[test]
fn immediate() {
    run("roms/instr_test_v5/03-immediate.nes");
}

#[test]
fn zero_page() {
    run("roms/instr_test_v5/04-zero_page.nes");
}

#[test]
fn zp_xy() {
    run("roms/instr_test_v5/05-zp_xy.nes");
}

#[test]
fn absolute() {
    run("roms/instr_test_v5/06-absolute.nes");
}

#[test]
fn abs_xy() {
    run("roms/instr_test_v5/07-abs_xy.nes");
}

#[test]
fn ind_x() {
    run("roms/instr_test_v5/08-ind_x.nes");
}

#[test]
fn ind_y() {
    run("roms/instr_test_v5/09-ind_y.nes");
}

#[test]
fn branches() {
    run("roms/instr_test_v5/10-branches.nes");
}

#[test]
fn stack() {
    run("roms/instr_test_v5/11-stack.nes");
}

#[test]
fn jmp_jsr() {
    run("roms/instr_test_v5/12-jmp_jsr.nes");
}

#[test]
fn rts() {
    run("roms/instr_test_v5/13-rts.nes");
}

#[test]
fn rti() {
    run("roms/instr_test_v5/14-rti.nes");
}

#[test]
fn brk() {
    run("roms/instr_test_v5/15-brk.nes");
}

#[test]
fn special() {
    run("roms/instr_test_v5/16-special.nes");
}
