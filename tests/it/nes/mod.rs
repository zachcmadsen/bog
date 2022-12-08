mod instr_test_v5;
mod nestest;

use bog::{Bus, Pins};

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

    fn read_prg(&mut self, address: u16) -> u8 {
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

    fn write_prg(&mut self, address: u16, data: u8) {
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

    fn read(&mut self, pins: &mut Pins) {
        match pins.address {
            0x0000..=0x1fff => {
                pins.data = self.ram[(pins.address & 0x07ff) as usize]
            }
            0x2000..=0x3fff => (),
            0x4000..=0x401f => (),
            0x4020..=0xffff => {
                pins.data = self.cartridge.read_prg(pins.address)
            }
        }
    }

    fn write(&mut self, pins: &mut Pins) {
        match pins.address {
            0x0000..=0x1fff => {
                self.ram[(pins.address & 0x07ff) as usize] = pins.data
            }
            0x2000..=0x3fff => (),
            0x4000..=0x401f => (),
            0x4020..=0xffff => {
                self.cartridge.write_prg(pins.address, pins.data)
            }
        }
    }
}

impl Bus for NesBus {
    fn tick(&mut self, pins: &mut Pins) {
        match pins.rw {
            true => self.read(pins),
            false => self.write(pins),
        }
    }
}
