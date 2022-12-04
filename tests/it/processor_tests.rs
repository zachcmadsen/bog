use std::{fs::File, io::BufReader};

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

fn run(opcode: u8) {
    let mut cpu = Cpu::new(ProcessorTestBus::new());

    // Run through the reset sequence.
    cpu.step();

    let filename = format!("roms/ProcessorTests/{:02x}.bincode", opcode);
    let file =
        File::open(&filename).expect(&format!("{} should exist", &filename));
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

#[test]
fn opcode_00() {
    run(0x00);
}

#[test]
fn opcode_01() {
    run(0x01);
}

#[test]
fn opcode_03() {
    run(0x03);
}

#[test]
fn opcode_04() {
    run(0x04);
}

#[test]
fn opcode_05() {
    run(0x05);
}

#[test]
fn opcode_06() {
    run(0x06);
}

#[test]
fn opcode_07() {
    run(0x07);
}

#[test]
fn opcode_08() {
    run(0x08);
}

#[test]
fn opcode_09() {
    run(0x09);
}

#[test]
fn opcode_0a() {
    run(0x0a);
}

#[test]
fn opcode_0b() {
    run(0x0b);
}

#[test]
fn opcode_0c() {
    run(0x0c);
}

#[test]
fn opcode_0d() {
    run(0x0d);
}

#[test]
fn opcode_0e() {
    run(0x0e);
}

#[test]
fn opcode_0f() {
    run(0x0f);
}

#[test]
fn opcode_10() {
    run(0x10);
}

#[test]
fn opcode_11() {
    run(0x11);
}

#[test]
fn opcode_13() {
    run(0x13);
}

#[test]
fn opcode_14() {
    run(0x14);
}

#[test]
fn opcode_15() {
    run(0x15);
}

#[test]
fn opcode_16() {
    run(0x16);
}

#[test]
fn opcode_17() {
    run(0x17);
}

#[test]
fn opcode_18() {
    run(0x18);
}

#[test]
fn opcode_19() {
    run(0x19);
}

#[test]
fn opcode_1a() {
    run(0x1a);
}

#[test]
fn opcode_1b() {
    run(0x1b);
}

#[test]
fn opcode_1c() {
    run(0x1c);
}

#[test]
fn opcode_1d() {
    run(0x1d);
}

#[test]
fn opcode_1e() {
    run(0x1e);
}

#[test]
fn opcode_1f() {
    run(0x1f);
}

#[test]
fn opcode_20() {
    run(0x20);
}

#[test]
fn opcode_21() {
    run(0x21);
}

#[test]
fn opcode_23() {
    run(0x23);
}

#[test]
fn opcode_24() {
    run(0x24);
}

#[test]
fn opcode_25() {
    run(0x25);
}

#[test]
fn opcode_26() {
    run(0x26);
}

#[test]
fn opcode_27() {
    run(0x27);
}

#[test]
fn opcode_28() {
    run(0x28);
}

#[test]
fn opcode_29() {
    run(0x29);
}

#[test]
fn opcode_2a() {
    run(0x2a);
}

#[test]
fn opcode_2b() {
    run(0x2b);
}

#[test]
fn opcode_2c() {
    run(0x2c);
}

#[test]
fn opcode_2d() {
    run(0x2d);
}

#[test]
fn opcode_2e() {
    run(0x2e);
}

#[test]
fn opcode_2f() {
    run(0x2f);
}

#[test]
fn opcode_30() {
    run(0x30);
}

#[test]
fn opcode_31() {
    run(0x31);
}

#[test]
fn opcode_33() {
    run(0x33);
}

#[test]
fn opcode_34() {
    run(0x34);
}

#[test]
fn opcode_35() {
    run(0x35);
}

#[test]
fn opcode_36() {
    run(0x36);
}

#[test]
fn opcode_37() {
    run(0x37);
}

#[test]
fn opcode_38() {
    run(0x38);
}

#[test]
fn opcode_39() {
    run(0x39);
}

#[test]
fn opcode_3a() {
    run(0x3a);
}

#[test]
fn opcode_3b() {
    run(0x3b);
}

#[test]
fn opcode_3c() {
    run(0x3c);
}

#[test]
fn opcode_3d() {
    run(0x3d);
}

#[test]
fn opcode_3e() {
    run(0x3e);
}

#[test]
fn opcode_3f() {
    run(0x3f);
}

#[test]
fn opcode_40() {
    run(0x40);
}

#[test]
fn opcode_41() {
    run(0x41);
}

#[test]
fn opcode_43() {
    run(0x43);
}

#[test]
fn opcode_44() {
    run(0x44);
}

#[test]
fn opcode_45() {
    run(0x45);
}

#[test]
fn opcode_46() {
    run(0x46);
}

#[test]
fn opcode_47() {
    run(0x47);
}

#[test]
fn opcode_48() {
    run(0x48);
}

#[test]
fn opcode_49() {
    run(0x49);
}

#[test]
fn opcode_4a() {
    run(0x4a);
}

#[test]
fn opcode_4b() {
    run(0x4b);
}

#[test]
fn opcode_4c() {
    run(0x4c);
}

#[test]
fn opcode_4d() {
    run(0x4d);
}

#[test]
fn opcode_4e() {
    run(0x4e);
}

#[test]
fn opcode_4f() {
    run(0x4f);
}

#[test]
fn opcode_50() {
    run(0x50);
}

#[test]
fn opcode_51() {
    run(0x51);
}

#[test]
fn opcode_53() {
    run(0x53);
}

#[test]
fn opcode_54() {
    run(0x54);
}

#[test]
fn opcode_55() {
    run(0x55);
}

#[test]
fn opcode_56() {
    run(0x56);
}

#[test]
fn opcode_57() {
    run(0x57);
}

#[test]
fn opcode_58() {
    run(0x58);
}

#[test]
fn opcode_59() {
    run(0x59);
}

#[test]
fn opcode_5a() {
    run(0x5a);
}

#[test]
fn opcode_5b() {
    run(0x5b);
}

#[test]
fn opcode_5c() {
    run(0x5c);
}

#[test]
fn opcode_5d() {
    run(0x5d);
}

#[test]
fn opcode_5e() {
    run(0x5e);
}

#[test]
fn opcode_5f() {
    run(0x5f);
}

#[test]
fn opcode_60() {
    run(0x60);
}

#[test]
fn opcode_61() {
    run(0x61);
}

#[test]
fn opcode_63() {
    run(0x63);
}

#[test]
fn opcode_64() {
    run(0x64);
}

#[test]
fn opcode_65() {
    run(0x65);
}

#[test]
fn opcode_66() {
    run(0x66);
}

#[test]
fn opcode_67() {
    run(0x67);
}

#[test]
fn opcode_68() {
    run(0x68);
}

#[test]
fn opcode_69() {
    run(0x69);
}

#[test]
fn opcode_6a() {
    run(0x6a);
}

#[test]
fn opcode_6c() {
    run(0x6c);
}

#[test]
fn opcode_6d() {
    run(0x6d);
}

#[test]
fn opcode_6e() {
    run(0x6e);
}

#[test]
fn opcode_6f() {
    run(0x6f);
}

#[test]
fn opcode_70() {
    run(0x70);
}

#[test]
fn opcode_71() {
    run(0x71);
}

#[test]
fn opcode_73() {
    run(0x73);
}

#[test]
fn opcode_74() {
    run(0x74);
}

#[test]
fn opcode_75() {
    run(0x75);
}

#[test]
fn opcode_76() {
    run(0x76);
}

#[test]
fn opcode_77() {
    run(0x77);
}

#[test]
fn opcode_78() {
    run(0x78);
}

#[test]
fn opcode_79() {
    run(0x79);
}

#[test]
fn opcode_7a() {
    run(0x7a);
}

#[test]
fn opcode_7b() {
    run(0x7b);
}

#[test]
fn opcode_7c() {
    run(0x7c);
}

#[test]
fn opcode_7d() {
    run(0x7d);
}

#[test]
fn opcode_7e() {
    run(0x7e);
}

#[test]
fn opcode_7f() {
    run(0x7f);
}

#[test]
fn opcode_80() {
    run(0x80);
}

#[test]
fn opcode_81() {
    run(0x81);
}

#[test]
fn opcode_82() {
    run(0x82);
}

#[test]
fn opcode_83() {
    run(0x83);
}

#[test]
fn opcode_84() {
    run(0x84);
}

#[test]
fn opcode_85() {
    run(0x85);
}

#[test]
fn opcode_86() {
    run(0x86);
}

#[test]
fn opcode_87() {
    run(0x87);
}

#[test]
fn opcode_88() {
    run(0x88);
}

#[test]
fn opcode_89() {
    run(0x89);
}

#[test]
fn opcode_8a() {
    run(0x8a);
}

#[test]
fn opcode_8c() {
    run(0x8c);
}

#[test]
fn opcode_8d() {
    run(0x8d);
}

#[test]
fn opcode_8e() {
    run(0x8e);
}

#[test]
fn opcode_8f() {
    run(0x8f);
}

#[test]
fn opcode_90() {
    run(0x90);
}

#[test]
fn opcode_91() {
    run(0x91);
}

#[test]
fn opcode_94() {
    run(0x94);
}

#[test]
fn opcode_95() {
    run(0x95);
}

#[test]
fn opcode_96() {
    run(0x96);
}

#[test]
fn opcode_97() {
    run(0x97);
}

#[test]
fn opcode_98() {
    run(0x98);
}

#[test]
fn opcode_99() {
    run(0x99);
}

#[test]
fn opcode_9a() {
    run(0x9a);
}

#[test]
fn opcode_9d() {
    run(0x9d);
}

#[test]
fn opcode_a0() {
    run(0xa0);
}

#[test]
fn opcode_a1() {
    run(0xa1);
}

#[test]
fn opcode_a2() {
    run(0xa2);
}

#[test]
fn opcode_a3() {
    run(0xa3);
}

#[test]
fn opcode_a4() {
    run(0xa4);
}

#[test]
fn opcode_a5() {
    run(0xa5);
}

#[test]
fn opcode_a6() {
    run(0xa6);
}

#[test]
fn opcode_a7() {
    run(0xa7);
}

#[test]
fn opcode_a8() {
    run(0xa8);
}

#[test]
fn opcode_a9() {
    run(0xa9);
}

#[test]
fn opcode_aa() {
    run(0xaa);
}

#[test]
fn opcode_ac() {
    run(0xac);
}

#[test]
fn opcode_ad() {
    run(0xad);
}

#[test]
fn opcode_ae() {
    run(0xae);
}

#[test]
fn opcode_af() {
    run(0xaf);
}

#[test]
fn opcode_b0() {
    run(0xb0);
}

#[test]
fn opcode_b1() {
    run(0xb1);
}

#[test]
fn opcode_b3() {
    run(0xb3);
}

#[test]
fn opcode_b4() {
    run(0xb4);
}

#[test]
fn opcode_b5() {
    run(0xb5);
}

#[test]
fn opcode_b6() {
    run(0xb6);
}

#[test]
fn opcode_b7() {
    run(0xb7);
}

#[test]
fn opcode_b8() {
    run(0xb8);
}

#[test]
fn opcode_b9() {
    run(0xb9);
}

#[test]
fn opcode_ba() {
    run(0xba);
}

#[test]
fn opcode_bc() {
    run(0xbc);
}

#[test]
fn opcode_bd() {
    run(0xbd);
}

#[test]
fn opcode_be() {
    run(0xbe);
}

#[test]
fn opcode_bf() {
    run(0xbf);
}

#[test]
fn opcode_c0() {
    run(0xc0);
}

#[test]
fn opcode_c1() {
    run(0xc1);
}

#[test]
fn opcode_c2() {
    run(0xc2);
}

#[test]
fn opcode_c3() {
    run(0xc3);
}

#[test]
fn opcode_c4() {
    run(0xc4);
}

#[test]
fn opcode_c5() {
    run(0xc5);
}

#[test]
fn opcode_c6() {
    run(0xc6);
}

#[test]
fn opcode_c7() {
    run(0xc7);
}

#[test]
fn opcode_c8() {
    run(0xc8);
}

#[test]
fn opcode_c9() {
    run(0xc9);
}

#[test]
fn opcode_ca() {
    run(0xca);
}

#[test]
fn opcode_cb() {
    run(0xcb);
}

#[test]
fn opcode_cc() {
    run(0xcc);
}

#[test]
fn opcode_cd() {
    run(0xcd);
}

#[test]
fn opcode_ce() {
    run(0xce);
}

#[test]
fn opcode_cf() {
    run(0xcf);
}

#[test]
fn opcode_d0() {
    run(0xd0);
}

#[test]
fn opcode_d1() {
    run(0xd1);
}

#[test]
fn opcode_d3() {
    run(0xd3);
}

#[test]
fn opcode_d4() {
    run(0xd4);
}

#[test]
fn opcode_d5() {
    run(0xd5);
}

#[test]
fn opcode_d6() {
    run(0xd6);
}

#[test]
fn opcode_d7() {
    run(0xd7);
}

#[test]
fn opcode_d8() {
    run(0xd8);
}

#[test]
fn opcode_d9() {
    run(0xd9);
}

#[test]
fn opcode_da() {
    run(0xda);
}

#[test]
fn opcode_db() {
    run(0xdb);
}

#[test]
fn opcode_dc() {
    run(0xdc);
}

#[test]
fn opcode_dd() {
    run(0xdd);
}

#[test]
fn opcode_de() {
    run(0xde);
}

#[test]
fn opcode_df() {
    run(0xdf);
}

#[test]
fn opcode_e0() {
    run(0xe0);
}

#[test]
fn opcode_e1() {
    run(0xe1);
}

#[test]
fn opcode_e2() {
    run(0xe2);
}

#[test]
fn opcode_e3() {
    run(0xe3);
}

#[test]
fn opcode_e4() {
    run(0xe4);
}

#[test]
fn opcode_e5() {
    run(0xe5);
}

#[test]
fn opcode_e6() {
    run(0xe6);
}

#[test]
fn opcode_e7() {
    run(0xe7);
}

#[test]
fn opcode_e8() {
    run(0xe8);
}

#[test]
fn opcode_e9() {
    run(0xe9);
}

#[test]
fn opcode_ea() {
    run(0xea);
}

#[test]
fn opcode_eb() {
    run(0xeb);
}

#[test]
fn opcode_ec() {
    run(0xec);
}

#[test]
fn opcode_ed() {
    run(0xed);
}

#[test]
fn opcode_ee() {
    run(0xee);
}

#[test]
fn opcode_ef() {
    run(0xef);
}

#[test]
fn opcode_f0() {
    run(0xf0);
}

#[test]
fn opcode_f1() {
    run(0xf1);
}

#[test]
fn opcode_f3() {
    run(0xf3);
}

#[test]
fn opcode_f4() {
    run(0xf4);
}

#[test]
fn opcode_f5() {
    run(0xf5);
}

#[test]
fn opcode_f6() {
    run(0xf6);
}

#[test]
fn opcode_f7() {
    run(0xf7);
}

#[test]
fn opcode_f8() {
    run(0xf8);
}

#[test]
fn opcode_f9() {
    run(0xf9);
}

#[test]
fn opcode_fa() {
    run(0xfa);
}

#[test]
fn opcode_fb() {
    run(0xfb);
}

#[test]
fn opcode_fc() {
    run(0xfc);
}

#[test]
fn opcode_fd() {
    run(0xfd);
}

#[test]
fn opcode_fe() {
    run(0xfe);
}

#[test]
fn opcode_ff() {
    run(0xff);
}
