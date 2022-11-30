use std::fs;

use bog::{Bus, Cpu, Pins};

const NES_RAM_SIZE: usize = 0x0800;

struct NromCartridge {
    prg_ram: Vec<u8>,
    prg_rom: Vec<u8>,
}

impl NromCartridge {
    fn new(rom: &[u8]) -> NromCartridge {
        let (header, data) = rom.split_at(16);

        let mapper_number = header[7] & 0xf0 | header[6] >> 4;
        if mapper_number != 0 {
            panic!("only NROM is supported")
        }

        let num_prg_rom_banks = header[4];
        let num_prg_ram_banks =
            if header[8] == 0 { 1 } else { header[8] } as usize;

        const PRG_ROM_BANK_SIZE: usize = 0x4000;
        const PRG_RAM_BANK_SIZE: usize = 0x2000;
        let prg_ram_size = num_prg_ram_banks as usize * PRG_RAM_BANK_SIZE;
        let prg_rom_size = num_prg_rom_banks as usize * PRG_ROM_BANK_SIZE;

        NromCartridge {
            prg_ram: vec![0; prg_ram_size],
            prg_rom: data[..prg_rom_size].to_vec(),
        }
    }

    fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0x6000..=0x7fff => {
                self.prg_ram[(address - 0x6000) as usize % self.prg_ram.len()]
            }
            0x8000..=0xffff => {
                self.prg_rom[(address - 0x8000) as usize % self.prg_rom.len()]
            }
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x6000..=0x7fff => {
                let address = (address - 0x6000) as usize % self.prg_ram.len();
                self.prg_ram[address] = data;
            }
            _ => unreachable!(),
        }
    }
}

struct NesBus {
    ram: [u8; NES_RAM_SIZE],
    pub cartridge: NromCartridge,
}

impl NesBus {
    fn new(cartridge: NromCartridge) -> NesBus {
        NesBus {
            ram: [0; NES_RAM_SIZE],
            cartridge,
        }
    }

    fn read_byte(&mut self, pins: &mut Pins) {
        match pins.address {
            0x0000..=0x1fff => {
                pins.data = self.ram[(pins.address & 0x07ff) as usize]
            }
            0x2000..=0x3fff => (),
            0x4000..=0x401f => (),
            0x4020..=0xffff => {
                pins.data = self.cartridge.read_byte(pins.address)
            }
        }
    }

    fn write_byte(&mut self, pins: &mut Pins) {
        match pins.address {
            0x0000..=0x1fff => {
                self.ram[(pins.address & 0x07ff) as usize] = pins.data
            }
            0x2000..=0x3fff => (),
            0x4000..=0x401f => (),
            0x4020..=0xffff => {
                self.cartridge.write_byte(pins.address, pins.data)
            }
        }
    }
}

impl Bus for NesBus {
    fn tick(&mut self, pins: &mut Pins) {
        match pins.rw {
            true => self.read_byte(pins),
            false => self.write_byte(pins),
        }
    }
}

fn run(rom_filepath: &str) {
    let rom = fs::read(rom_filepath)
        .expect(&format!("{} should exist", rom_filepath));

    let cartridge = NromCartridge::new(&rom);
    let bus = NesBus::new(cartridge);
    let mut cpu = Cpu::new(bus);

    // Run through the reset sequence.
    cpu.step();

    const STATUS_ADDRESS: u16 = 0x6000;
    const RUNNING: u8 = 0x80;
    const OUTPUT_ADDRESS: u16 = 0x6004;

    let mut status = cpu.bus.cartridge.read_byte(STATUS_ADDRESS);
    while status != RUNNING {
        cpu.step();
        status = cpu.bus.cartridge.read_byte(STATUS_ADDRESS);
    }

    while status == RUNNING {
        cpu.step();
        status = cpu.bus.cartridge.read_byte(STATUS_ADDRESS);
    }

    let mut output = Vec::new();
    let mut address = OUTPUT_ADDRESS;
    let mut byte = cpu.bus.cartridge.read_byte(address);
    while byte != b'\0' {
        output.push(byte);
        address += 1;
        byte = cpu.bus.cartridge.read_byte(address);
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
