use core::cell::Cell;

use bitflags::bitflags;

use crate::bus::Bus;

// Vectors
const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const IRQ_VECTOR: u16 = 0xfffe;

// Addressing Modes
const ABSOLUTE: u8 = 0;
const ABSOLUTE_X: u8 = 1;
const ABSOLUTE_Y: u8 = 2;
const ACCUMULATOR: u8 = 3;
const IMMEDIATE: u8 = 4;
const IMPLIED: u8 = 5;
const INDEXED_INDIRECT: u8 = 6;
const INDIRECT: u8 = 7;
const INDIRECT_INDEXED: u8 = 8;
const ZERO_PAGE: u8 = 9;
const ZERO_PAGE_X: u8 = 10;
const ZERO_PAGE_Y: u8 = 11;

const STACK_BASE: u16 = 0x0100;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Status: u8 {
        const C = 1;
        const Z = 1 << 1;
        const I = 1 << 2;
        const D = 1 << 3;
        const B = 1 << 4;
        const U = 1 << 5;
        const V = 1 << 6;
        const N = 1 << 7;
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::U | Status::I
    }
}

pub enum InterruptKind {
    Brk,
    Irq,
    Nmi,
    Reset,
}

// #[derive(Clone, Copy, Debug)]
// struct InterruptLine(Option<u8>);

// impl InterruptLine {
//     fn new() -> Self {
//         Self(None)
//     }

//     pub fn start(&mut self) {
//         self.0 = Some(1);
//     }

//     fn tick(self) -> Self {
//         match self.0 {
//             Some(c) => {
//                 if c > 0 {
//                     Self(Some(c - 1))
//                 } else {
//                     self
//                 }
//             }
//             None => self,
//         }
//     }

//     fn finish(self) -> Self {
//         Self(None)
//     }

//     fn is_ready(&self) -> bool {
//         match self.0 {
//             Some(c) => c == 0,
//             None => false,
//         }
//     }
// }

pub struct Cpu<B> {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub s: u8,
    pub p: Status,
    interrupt_kind: InterruptKind,
    // irq: Cell<InterruptLine>,
    // nmi: Cell<InterruptLine>,
    cycles: Cell<u64>,
    pub bus: B,
}

impl<B> Cpu<B>
where
    B: Bus,
{
    /// Constructs a new `Cpu`.
    pub fn new(bus: B) -> Cpu<B> {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xfd,
            p: Status::default(),
            interrupt_kind: InterruptKind::Brk,
            // irq: Cell::new(InterruptLine::new()),
            // nmi: Cell::new(InterruptLine::new()),
            cycles: Cell::new(0),
            bus,
        }
    }

    /// Returns the current cycle count.
    pub fn cycles(&self) -> u64 {
        self.cycles.get()
    }

    /// Reads a byte from memory.
    fn read_byte(&self, address: u16) -> u8 {
        self.cycles.set(self.cycles.get() + 1);
        self.bus.read(address)
        // self.nmi.set(self.nmi.get().tick());
        // self.irq.set(self.irq.get().tick());
    }

    /// Reads a word from memory.
    fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address);
        let high = self.read_byte(address + 1);
        (high as u16) << 8 | low as u16
    }

    /// Reads a word from memory without handling page boundary crosses.
    ///
    /// There's a hardware bug where the low byte wraps without incrementing
    /// the high byte, i.e., it wraps around in the same page. This behavior
    /// only affects indirect addressing modes.
    fn read_word_bugged(&self, address: u16) -> u16 {
        let low = self.read_byte(address);
        let high = self.read_byte(
            (address & 0xff00) | (address as u8).wrapping_add(1) as u16,
        );
        (high as u16) << 8 | low as u16
    }

    /// Writes a byte to memory.
    fn write_byte(&mut self, address: u16, data: u8) {
        self.cycles.set(self.cycles.get() + 1);
        self.bus.write(address, data);

        // self.nmi.set(self.nmi.get().tick());
        // self.irq.set(self.irq.get().tick());

        // match inter {
        //     Some((is_irq, is_nmi)) => {
        //         if is_nmi {
        //             self.nmi.get_mut().start();
        //         }
        //         if is_irq {
        //             self.irq.get_mut().start();
        //         }
        //     }
        //     None => (),
        // }
    }

    /// Reads the byte addressed by the PC and increments the PC.
    fn consume_byte(&mut self) -> u8 {
        let data = self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        data
    }

    /// Reads the word addressed by the PC and increments the PC by two.
    fn consume_word(&mut self) -> u16 {
        let data = self.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        data
    }

    /// Returns the byte at the top of the stack.
    fn peek(&self) -> u8 {
        self.read_byte(STACK_BASE + self.s.wrapping_add(1) as u16)
    }

    /// Pushes a byte onto the stack.
    fn push(&mut self, data: u8) {
        self.write_byte(STACK_BASE + self.s as u16, data);
        self.s = self.s.wrapping_sub(1);
    }

    /// Pops a byte off the top of the stack.
    fn pop(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read_byte(STACK_BASE + self.s as u16)
    }

    /// Runs the startup sequence.
    pub fn reset(&mut self) {
        // There are things happening on the address and data bus in the
        // first five cycles, but I don't think they matter. Incrementing the
        // cycle count seems to be sufficient.
        self.cycles.set(5);

        // Turn on the interrupt disable bit? Yes

        // TODO: The CPU should be in read-only state during the reset
        // sequence. Any writes during the reset sequence should become reads.
        self.pc = self.read_word(RESET_VECTOR);
    }

    /// Executes the next instruction.
    pub fn step(&mut self) {
        // TODO: Check for interrupts. Interrupts are (always?) handled before
        // an instruction starts. To process an interrupt before the next
        // instruction, then it must occur before the last cycle of the current
        // instruction (there are exceptions to this, but it's fine for now).
        // I'll need to add some kind of timer to see if an interrupt needs to
        // be handled.
        // NMI alway stakes priority over IRQ, and reset has priority over
        // NMI
        // println!("{:04X}, {:?}", self.pc, self.irq);
        // let opcode = if self.nmi.get_mut().is_ready() {
        //     // println!("NMI triggered at {:04X}", self.pc);
        //     self.interrupt_kind = InterruptKind::Nmi;
        //     self.nmi.set(self.nmi.get().finish());
        //     self.read_byte(self.pc);
        //     self.read_byte(self.pc);

        //     0x00
        // } else if self.irq.get_mut().is_ready() && !self.p.contains(Status::I)
        // {
        //     // println!("IRQ triggered at {:04X}", self.pc);
        //     self.interrupt_kind = InterruptKind::Irq;
        //     self.irq.set(self.irq.get().finish());
        //     self.read_byte(self.pc);
        //     self.read_byte(self.pc);
        //     0x00
        // } else {
        //     self.consume_byte()
        // };

        let opcode = self.consume_byte();

        match opcode {
            0x69 => self.adc::<IMMEDIATE>(),
            0x65 => self.adc::<ZERO_PAGE>(),
            0x75 => self.adc::<ZERO_PAGE_X>(),
            0x6d => self.adc::<ABSOLUTE>(),
            0x7d => self.adc::<ABSOLUTE_X>(),
            0x79 => self.adc::<ABSOLUTE_Y>(),
            0x61 => self.adc::<INDEXED_INDIRECT>(),
            0x71 => self.adc::<INDIRECT_INDEXED>(),

            0x29 => self.and::<IMMEDIATE>(),
            0x25 => self.and::<ZERO_PAGE>(),
            0x35 => self.and::<ZERO_PAGE_X>(),
            0x2d => self.and::<ABSOLUTE>(),
            0x3d => self.and::<ABSOLUTE_X>(),
            0x39 => self.and::<ABSOLUTE_Y>(),
            0x21 => self.and::<INDEXED_INDIRECT>(),
            0x31 => self.and::<INDIRECT_INDEXED>(),

            0x0a => self.asl::<ACCUMULATOR>(),
            0x06 => self.asl::<ZERO_PAGE>(),
            0x16 => self.asl::<ZERO_PAGE_X>(),
            0x0e => self.asl::<ABSOLUTE>(),
            0x1e => self.asl::<ABSOLUTE_X>(),

            0x90 => self.bcc(),

            0xb0 => self.bcs(),

            0xf0 => self.beq(),

            0x24 => self.bit::<ZERO_PAGE>(),
            0x2c => self.bit::<ABSOLUTE>(),

            0x30 => self.bmi(),

            0xd0 => self.bne(),

            0x10 => self.bpl(),

            0x00 => self.brk(),

            0x50 => self.bvc(),

            0x70 => self.bvs(),

            0x18 => self.clc(),

            0xd8 => self.cld(),

            0x58 => self.cli(),

            0xb8 => self.clv(),

            0xc9 => self.cmp::<IMMEDIATE>(),
            0xc5 => self.cmp::<ZERO_PAGE>(),
            0xd5 => self.cmp::<ZERO_PAGE_X>(),
            0xcd => self.cmp::<ABSOLUTE>(),
            0xdd => self.cmp::<ABSOLUTE_X>(),
            0xd9 => self.cmp::<ABSOLUTE_Y>(),
            0xc1 => self.cmp::<INDEXED_INDIRECT>(),
            0xd1 => self.cmp::<INDIRECT_INDEXED>(),

            0xe0 => self.cpx::<IMMEDIATE>(),
            0xe4 => self.cpx::<ZERO_PAGE>(),
            0xec => self.cpx::<ABSOLUTE>(),

            0xc0 => self.cpy::<IMMEDIATE>(),
            0xc4 => self.cpy::<ZERO_PAGE>(),
            0xcc => self.cpy::<ABSOLUTE>(),

            0xc7 => self.dcp::<ZERO_PAGE>(),
            0xd7 => self.dcp::<ZERO_PAGE_X>(),
            0xcf => self.dcp::<ABSOLUTE>(),
            0xdf => self.dcp::<ABSOLUTE_X>(),
            0xdb => self.dcp::<ABSOLUTE_Y>(),
            0xc3 => self.dcp::<INDEXED_INDIRECT>(),
            0xd3 => self.dcp::<INDIRECT_INDEXED>(),

            0xc6 => self.dec::<ZERO_PAGE>(),
            0xd6 => self.dec::<ZERO_PAGE_X>(),
            0xce => self.dec::<ABSOLUTE>(),
            0xde => self.dec::<ABSOLUTE_X>(),

            0xca => self.dex(),

            0x88 => self.dey(),

            0x49 => self.eor::<IMMEDIATE>(),
            0x45 => self.eor::<ZERO_PAGE>(),
            0x55 => self.eor::<ZERO_PAGE_X>(),
            0x4d => self.eor::<ABSOLUTE>(),
            0x5d => self.eor::<ABSOLUTE_X>(),
            0x59 => self.eor::<ABSOLUTE_Y>(),
            0x41 => self.eor::<INDEXED_INDIRECT>(),
            0x51 => self.eor::<INDIRECT_INDEXED>(),

            0xe6 => self.inc::<ZERO_PAGE>(),
            0xf6 => self.inc::<ZERO_PAGE_X>(),
            0xee => self.inc::<ABSOLUTE>(),
            0xfe => self.inc::<ABSOLUTE_X>(),

            0xe8 => self.inx(),

            0xc8 => self.iny(),

            0xe7 => self.isb::<ZERO_PAGE>(),
            0xf7 => self.isb::<ZERO_PAGE_X>(),
            0xef => self.isb::<ABSOLUTE>(),
            0xff => self.isb::<ABSOLUTE_X>(),
            0xfb => self.isb::<ABSOLUTE_Y>(),
            0xe3 => self.isb::<INDEXED_INDIRECT>(),
            0xf3 => self.isb::<INDIRECT_INDEXED>(),

            0x4c => self.jmp::<ABSOLUTE>(),
            0x6c => self.jmp::<INDIRECT>(),

            0x20 => self.jsr(),

            0xa7 => self.lax::<ZERO_PAGE>(),
            0xb7 => self.lax::<ZERO_PAGE_Y>(),
            0xaf => self.lax::<ABSOLUTE>(),
            0xbf => self.lax::<ABSOLUTE_Y>(),
            0xa3 => self.lax::<INDEXED_INDIRECT>(),
            0xb3 => self.lax::<INDIRECT_INDEXED>(),

            0xa9 => self.lda::<IMMEDIATE>(),
            0xa5 => self.lda::<ZERO_PAGE>(),
            0xb5 => self.lda::<ZERO_PAGE_X>(),
            0xad => self.lda::<ABSOLUTE>(),
            0xbd => self.lda::<ABSOLUTE_X>(),
            0xb9 => self.lda::<ABSOLUTE_Y>(),
            0xa1 => self.lda::<INDEXED_INDIRECT>(),
            0xb1 => self.lda::<INDIRECT_INDEXED>(),

            0xa2 => self.ldx::<IMMEDIATE>(),
            0xa6 => self.ldx::<ZERO_PAGE>(),
            0xb6 => self.ldx::<ZERO_PAGE_Y>(),
            0xae => self.ldx::<ABSOLUTE>(),
            0xbe => self.ldx::<ABSOLUTE_Y>(),

            0xa0 => self.ldy::<IMMEDIATE>(),
            0xa4 => self.ldy::<ZERO_PAGE>(),
            0xb4 => self.ldy::<ZERO_PAGE_X>(),
            0xac => self.ldy::<ABSOLUTE>(),
            0xbc => self.ldy::<ABSOLUTE_X>(),

            0x4a => self.lsr::<ACCUMULATOR>(),
            0x46 => self.lsr::<ZERO_PAGE>(),
            0x56 => self.lsr::<ZERO_PAGE_X>(),
            0x4e => self.lsr::<ABSOLUTE>(),
            0x5e => self.lsr::<ABSOLUTE_X>(),

            0x1a => self.nop::<IMPLIED>(),
            0x3a => self.nop::<IMPLIED>(),
            0x5a => self.nop::<IMPLIED>(),
            0x7a => self.nop::<IMPLIED>(),
            0xda => self.nop::<IMPLIED>(),
            0xea => self.nop::<IMPLIED>(),
            0xfa => self.nop::<IMPLIED>(),
            0x80 => self.nop::<IMMEDIATE>(),
            0x82 => self.nop::<IMMEDIATE>(),
            0x89 => self.nop::<IMMEDIATE>(),
            0xc2 => self.nop::<IMMEDIATE>(),
            0xe2 => self.nop::<IMMEDIATE>(),
            0x04 => self.nop::<ZERO_PAGE>(),
            0x44 => self.nop::<ZERO_PAGE>(),
            0x64 => self.nop::<ZERO_PAGE>(),
            0x14 => self.nop::<ZERO_PAGE_X>(),
            0x34 => self.nop::<ZERO_PAGE_X>(),
            0x54 => self.nop::<ZERO_PAGE_X>(),
            0x74 => self.nop::<ZERO_PAGE_X>(),
            0xd4 => self.nop::<ZERO_PAGE_X>(),
            0xf4 => self.nop::<ZERO_PAGE_X>(),
            0x0c => self.nop::<ABSOLUTE>(),
            0x1c => self.nop::<ABSOLUTE_X>(),
            0x3c => self.nop::<ABSOLUTE_X>(),
            0x5c => self.nop::<ABSOLUTE_X>(),
            0x7c => self.nop::<ABSOLUTE_X>(),
            0xdc => self.nop::<ABSOLUTE_X>(),
            0xfc => self.nop::<ABSOLUTE_X>(),

            0x09 => self.ora::<IMMEDIATE>(),
            0x05 => self.ora::<ZERO_PAGE>(),
            0x15 => self.ora::<ZERO_PAGE_X>(),
            0x0d => self.ora::<ABSOLUTE>(),
            0x1d => self.ora::<ABSOLUTE_X>(),
            0x19 => self.ora::<ABSOLUTE_Y>(),
            0x01 => self.ora::<INDEXED_INDIRECT>(),
            0x11 => self.ora::<INDIRECT_INDEXED>(),

            0x48 => self.pha(),

            0x08 => self.php(),

            0x68 => self.pla(),

            0x28 => self.plp(),

            0x27 => self.rla::<ZERO_PAGE>(),
            0x37 => self.rla::<ZERO_PAGE_X>(),
            0x2f => self.rla::<ABSOLUTE>(),
            0x3f => self.rla::<ABSOLUTE_X>(),
            0x3b => self.rla::<ABSOLUTE_Y>(),
            0x23 => self.rla::<INDEXED_INDIRECT>(),
            0x33 => self.rla::<INDIRECT_INDEXED>(),

            0x2a => self.rol::<ACCUMULATOR>(),
            0x26 => self.rol::<ZERO_PAGE>(),
            0x36 => self.rol::<ZERO_PAGE_X>(),
            0x2e => self.rol::<ABSOLUTE>(),
            0x3e => self.rol::<ABSOLUTE_X>(),

            0x6a => self.ror::<ACCUMULATOR>(),
            0x66 => self.ror::<ZERO_PAGE>(),
            0x76 => self.ror::<ZERO_PAGE_X>(),
            0x6e => self.ror::<ABSOLUTE>(),
            0x7e => self.ror::<ABSOLUTE_X>(),

            0x67 => self.rra::<ZERO_PAGE>(),
            0x77 => self.rra::<ZERO_PAGE_X>(),
            0x6f => self.rra::<ABSOLUTE>(),
            0x7f => self.rra::<ABSOLUTE_X>(),
            0x7b => self.rra::<ABSOLUTE_Y>(),
            0x63 => self.rra::<INDEXED_INDIRECT>(),
            0x73 => self.rra::<INDIRECT_INDEXED>(),

            0x40 => self.rti(),

            0x60 => self.rts(),

            0x87 => self.sax::<ZERO_PAGE>(),
            0x97 => self.sax::<ZERO_PAGE_Y>(),
            0x8f => self.sax::<ABSOLUTE>(),
            0x83 => self.sax::<INDEXED_INDIRECT>(),

            0xe9 => self.sbc::<IMMEDIATE>(),
            0xeb => self.sbc::<IMMEDIATE>(),
            0xe5 => self.sbc::<ZERO_PAGE>(),
            0xf5 => self.sbc::<ZERO_PAGE_X>(),
            0xed => self.sbc::<ABSOLUTE>(),
            0xfd => self.sbc::<ABSOLUTE_X>(),
            0xf9 => self.sbc::<ABSOLUTE_Y>(),
            0xe1 => self.sbc::<INDEXED_INDIRECT>(),
            0xf1 => self.sbc::<INDIRECT_INDEXED>(),

            0x38 => self.sec(),

            0xf8 => self.sed(),

            0x78 => self.sei(),

            0x07 => self.slo::<ZERO_PAGE>(),
            0x17 => self.slo::<ZERO_PAGE_X>(),
            0x0f => self.slo::<ABSOLUTE>(),
            0x1f => self.slo::<ABSOLUTE_X>(),
            0x1b => self.slo::<ABSOLUTE_Y>(),
            0x03 => self.slo::<INDEXED_INDIRECT>(),
            0x13 => self.slo::<INDIRECT_INDEXED>(),

            0x47 => self.sre::<ZERO_PAGE>(),
            0x57 => self.sre::<ZERO_PAGE_X>(),
            0x4f => self.sre::<ABSOLUTE>(),
            0x5f => self.sre::<ABSOLUTE_X>(),
            0x5b => self.sre::<ABSOLUTE_Y>(),
            0x43 => self.sre::<INDEXED_INDIRECT>(),
            0x53 => self.sre::<INDIRECT_INDEXED>(),

            0x85 => self.sta::<ZERO_PAGE>(),
            0x95 => self.sta::<ZERO_PAGE_X>(),
            0x8d => self.sta::<ABSOLUTE>(),
            0x9d => self.sta::<ABSOLUTE_X>(),
            0x99 => self.sta::<ABSOLUTE_Y>(),
            0x81 => self.sta::<INDEXED_INDIRECT>(),
            0x91 => self.sta::<INDIRECT_INDEXED>(),

            0x86 => self.stx::<ZERO_PAGE>(),
            0x96 => self.stx::<ZERO_PAGE_Y>(),
            0x8e => self.stx::<ABSOLUTE>(),

            0x84 => self.sty::<ZERO_PAGE>(),
            0x94 => self.sty::<ZERO_PAGE_X>(),
            0x8c => self.sty::<ABSOLUTE>(),

            0xaa => self.tax(),

            0xa8 => self.tay(),

            0xba => self.tsx(),

            0x8a => self.txa(),

            0x9a => self.txs(),

            0x98 => self.tya(),

            _ => unreachable!("unexpected opcode {}", opcode),
        };
    }

    // fn interrupt(&mut self) {
    //     self.consume_next_byte();
    //     self.consume_next_byte();

    //     self.push((self.pc >> 8) as u8);
    //     self.push(self.pc as u8);
    //     self.push(self.p.bits());

    //     self.pc = self.read_word(INTERRUPT_REQUEST_VECTOR);
    //     self.p.set(Flag::I, true);
    // }

    fn effective_address<const M: u8, const W: bool>(&mut self) -> u16 {
        match M {
            ABSOLUTE => self.consume_word(),
            ABSOLUTE_X | ABSOLUTE_Y => {
                let index = if M == ABSOLUTE_X { self.x } else { self.y };

                let (low, did_cross_page) =
                    self.consume_byte().overflowing_add(index);
                let high = self.consume_byte();

                let effective_address =
                    (high.wrapping_add(did_cross_page as u8) as u16) << 8
                        | (low as u16);

                // If the effective address is invalid, i.e., it crosses a page
                // boundary, then it takes an extra cycle to fix it. For write
                // instructions, the processor always reads from the effective
                // address (that is, it always takes an extra cycle) since it
                // can't undo a write to an invalid address.
                if did_cross_page || W {
                    self.read_byte(effective_address);
                }

                effective_address
            }
            IMMEDIATE => {
                let effective_address = self.pc;
                self.pc = self.pc.wrapping_add(1);
                effective_address
            }
            INDIRECT => {
                let ptr = self.consume_word();
                self.read_word_bugged(ptr)
            }
            INDEXED_INDIRECT => {
                let ptr = self.consume_byte();
                self.read_byte(ptr as u16);
                self.read_word_bugged(ptr.wrapping_add(self.x) as u16)
            }
            INDIRECT_INDEXED => {
                let ptr = self.consume_byte();

                let (low, did_cross_page) =
                    self.read_byte(ptr as u16).overflowing_add(self.y);
                let high = self.read_byte(ptr.wrapping_add(1) as u16);

                let effective_address =
                    (high.wrapping_add(did_cross_page as u8) as u16) << 8
                        | (low as u16);

                // Write instructions always read from the effective address.
                // See the AbsoluteX/AbsoluteY branch for details.
                if did_cross_page || W {
                    self.read_byte(effective_address);
                }

                effective_address
            }
            ZERO_PAGE => self.consume_byte() as u16,
            ZERO_PAGE_X | ZERO_PAGE_Y => {
                let index = if M == ZERO_PAGE_X { self.x } else { self.y };

                let address = self.consume_byte();
                self.read_byte(address as u16);

                address.wrapping_add(index) as u16
            }
            // TODO: Convert M to a string for the panic message
            _ => unreachable!(
                "can't compute an effective address with addressing mode {:?}",
                M
            ),
        }
    }
}

enum InstructionName {
    Asl,
    Slo,
    Dcp,
    Dec,
    Inc,
    Isb,
    Lsr,
    Sre,
    Rol,
    Rla,
    Ror,
    Rra,
}

// Instruction helpers
impl<B> Cpu<B>
where
    B: Bus,
{
    fn add(&mut self, value: u8) {
        let a = self.a;
        let result = (self.a as u16)
            .wrapping_add(value as u16)
            .wrapping_add(self.p.contains(Status::C) as u16);
        self.a = result as u8;

        self.p.set(Status::C, result > 0xff);
        self.p.set(Status::Z, self.a == 0);
        self.p
            .set(Status::V, ((a ^ self.a) & (value ^ self.a) & 0x80) != 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn branch(&mut self, condition: bool) {
        let offset = self.consume_byte() as i8 as u16;
        if condition {
            self.read_byte(self.pc);

            let old_pc = self.pc;
            // TODO: Use overflowing add here?
            self.pc = self.pc.wrapping_add(offset);

            if old_pc & 0xff00 != self.pc & 0xff00 {
                self.read_byte(self.pc);
            }
        }
    }

    /// Subtracts `value` from `register` and updates flags based on the result.
    fn compare(&mut self, register: u8, value: u8) {
        let result = register.wrapping_sub(value);

        self.p.set(Status::C, register >= value);
        self.p.set(Status::Z, result == 0);
        self.p.set(Status::N, result & 0x80 != 0);
    }

    fn modify(&mut self, name: InstructionName, value: u8) -> u8 {
        let (result, carry) = match name {
            InstructionName::Asl | InstructionName::Slo => {
                (value.wrapping_shl(1), value & 0x80 != 0)
            }
            InstructionName::Dcp | InstructionName::Dec => {
                (value.wrapping_sub(1), self.p.contains(Status::C))
            }
            InstructionName::Inc | InstructionName::Isb => {
                (value.wrapping_add(1), self.p.contains(Status::C))
            }
            InstructionName::Lsr | InstructionName::Sre => {
                (value.wrapping_shr(1), value & 0x01 != 0)
            }
            InstructionName::Rol | InstructionName::Rla => (
                value.wrapping_shl(1) | self.p.contains(Status::C) as u8,
                value & 0x80 != 0,
            ),
            InstructionName::Ror | InstructionName::Rra => (
                (self.p.contains(Status::C) as u8) << 7
                    | value.wrapping_shr(1),
                value & 0x01 != 0,
            ),
        };

        self.p.set(Status::C, carry);
        self.p.set(Status::Z, result == 0);
        self.p.set(Status::N, result & 0x80 != 0);

        result
    }

    fn read_modify_write<const M: u8>(&mut self, name: InstructionName) -> u8 {
        // TODO: This simplifies all RMW instructions, but the performance is worse
        // since there are multiple matches. It might be worth investigating.
        match M {
            ACCUMULATOR => {
                self.read_byte(self.pc);
                self.a = self.modify(name, self.a);
                self.a
            }
            _ => {
                // Treat it as a write instruction while fetching the effective
                // address to get the cycle count right.
                let effective_address = self.effective_address::<M, true>();
                let value = self.read_byte(effective_address);

                // Read-Modify-Write instructions have an extra write since it
                // takes an extra cycle to modify the value.
                self.write_byte(effective_address, value);

                let result = self.modify(name, value);

                self.write_byte(effective_address, result);

                result
            }
        }
    }
}

// Instructions
impl<B> Cpu<B>
where
    B: Bus,
{
    fn adc<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        let value = self.read_byte(effective_address);
        self.add(value);
    }

    fn and<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        self.a &= self.read_byte(effective_address);

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn asl<const M: u8>(&mut self) {
        self.read_modify_write::<M>(InstructionName::Asl);
    }

    fn bcc(&mut self) {
        self.branch(!self.p.contains(Status::C));
    }

    fn bcs(&mut self) {
        self.branch(self.p.contains(Status::C));
    }

    fn beq(&mut self) {
        self.branch(self.p.contains(Status::Z));
    }

    fn bit<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        let value = self.read_byte(effective_address);

        self.p.set(Status::Z, self.a & value == 0);
        self.p.set(Status::V, value & Status::V.bits() != 0);
        self.p.set(Status::N, value & Status::N.bits() != 0);
    }

    fn bmi(&mut self) {
        self.branch(self.p.contains(Status::N));
    }

    fn bne(&mut self) {
        self.branch(!self.p.contains(Status::Z));
    }

    fn bpl(&mut self) {
        self.branch(!self.p.contains(Status::N));
    }

    fn brk(&mut self) {
        // TODO: Should this consume?
        self.read_byte(self.pc);

        if !matches!(
            self.interrupt_kind,
            InterruptKind::Irq | InterruptKind::Nmi
        ) {
            self.pc += 1;
        }

        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        self.push(
            (if matches!(self.interrupt_kind, InterruptKind::Brk) {
                self.p | Status::B
            } else {
                self.p
            })
            .bits(),
        );

        self.pc = self.read_word(match self.interrupt_kind {
            InterruptKind::Brk | InterruptKind::Irq => IRQ_VECTOR,
            InterruptKind::Nmi => NMI_VECTOR,
            InterruptKind::Reset => RESET_VECTOR,
        });
        self.interrupt_kind = InterruptKind::Brk;
        self.p.insert(Status::I);
    }

    fn bvc(&mut self) {
        self.branch(!self.p.contains(Status::V));
    }

    fn bvs(&mut self) {
        self.branch(self.p.contains(Status::V));
    }

    fn clc(&mut self) {
        self.read_byte(self.pc);
        self.p.remove(Status::C);
    }

    fn cld(&mut self) {
        self.read_byte(self.pc);
        self.p.remove(Status::D);
    }

    fn cli(&mut self) {
        self.read_byte(self.pc);
        self.p.remove(Status::I);
    }

    fn clv(&mut self) {
        self.read_byte(self.pc);
        self.p.remove(Status::V);
    }

    fn cmp<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        let value = self.read_byte(effective_address);
        self.compare(self.a, value);
    }

    fn cpx<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        let value = self.read_byte(effective_address);
        self.compare(self.x, value);
    }

    fn cpy<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        let value = self.read_byte(effective_address);
        self.compare(self.y, value);
    }

    fn dcp<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M>(InstructionName::Dcp);
        self.compare(self.a, result);
    }

    fn dec<const M: u8>(&mut self) {
        self.read_modify_write::<M>(InstructionName::Dec);
    }

    fn dex(&mut self) {
        self.read_byte(self.pc);
        self.x = self.x.wrapping_sub(1);

        self.p.set(Status::Z, self.x == 0);
        self.p.set(Status::N, self.x & 0x80 != 0);
    }

    fn dey(&mut self) {
        self.read_byte(self.pc);
        self.y = self.y.wrapping_sub(1);

        self.p.set(Status::Z, self.y == 0);
        self.p.set(Status::N, self.y & 0x80 != 0);
    }

    fn eor<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        self.a ^= self.read_byte(effective_address);

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn inc<const M: u8>(&mut self) {
        self.read_modify_write::<M>(InstructionName::Inc);
    }

    fn inx(&mut self) {
        self.read_byte(self.pc);
        self.x = self.x.wrapping_add(1);

        self.p.set(Status::Z, self.x == 0);
        self.p.set(Status::N, self.x & 0x80 != 0);
    }

    fn iny(&mut self) {
        self.read_byte(self.pc);
        self.y = self.y.wrapping_add(1);

        self.p.set(Status::Z, self.y == 0);
        self.p.set(Status::N, self.y & 0x80 != 0);
    }

    fn isb<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M>(InstructionName::Isb);
        self.add(result ^ 0xff);
    }

    fn jmp<const M: u8>(&mut self) {
        self.pc = self.effective_address::<M, false>();
    }

    fn jsr(&mut self) {
        let pcl = self.consume_byte();
        self.peek();
        self.push((self.pc >> 8) as u8);
        self.push(self.pc as u8);
        let pch = self.consume_byte();
        self.pc = (pch as u16) << 8 | pcl as u16;
    }

    fn lax<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        let value = self.read_byte(effective_address);
        self.a = value;
        self.x = value;

        self.p.set(Status::Z, value == 0);
        self.p.set(Status::N, value & 0x80 != 0);
    }

    fn lda<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        self.a = self.read_byte(effective_address);

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn ldx<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        self.x = self.read_byte(effective_address);

        self.p.set(Status::Z, self.x == 0);
        self.p.set(Status::N, self.x & 0x80 != 0);
    }

    fn ldy<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        self.y = self.read_byte(effective_address);

        self.p.set(Status::Z, self.y == 0);
        self.p.set(Status::N, self.y & 0x80 != 0);
    }

    fn lsr<const M: u8>(&mut self) {
        self.read_modify_write::<M>(InstructionName::Lsr);
    }

    fn nop<const M: u8>(&mut self) {
        if M == IMPLIED {
            self.read_byte(self.pc);
        } else {
            self.effective_address::<M, false>();
            self.read_byte(self.pc);
        }
    }

    fn ora<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        self.a |= self.read_byte(effective_address);

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn pha(&mut self) {
        self.read_byte(self.pc);
        self.push(self.a);
    }

    fn php(&mut self) {
        self.read_byte(self.pc);
        self.push((self.p | Status::B | Status::U).bits());
    }

    fn pla(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        self.a = self.pop();

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn plp(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        self.p = (Status::from_bits_truncate(self.pop())
            & !(Status::B | Status::U))
            | (self.p & (Status::B | Status::U));
    }

    fn rla<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M>(InstructionName::Rla);
        self.a &= result;

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn rol<const M: u8>(&mut self) {
        self.read_modify_write::<M>(InstructionName::Rol);
    }

    fn ror<const M: u8>(&mut self) {
        self.read_modify_write::<M>(InstructionName::Ror);
    }

    fn rra<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M>(InstructionName::Rra);
        self.add(result);
    }

    fn rti(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        self.p = (Status::from_bits_truncate(self.pop())
            & !(Status::B | Status::U))
            | (self.p & (Status::B | Status::U));
        let pcl = self.pop();
        let pch = self.pop();
        self.pc = (pch as u16) << 8 | pcl as u16;
    }

    fn rts(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        let pcl = self.pop();
        let pch = self.pop();
        self.pc = (pch as u16) << 8 | pcl as u16;
        self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
    }

    fn sax<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, true>();

        self.write_byte(effective_address, self.a & self.x);
    }

    fn sbc<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        // If we reformulate the subtraction as addition, then we can use the
        // same logic for ADC and SBC. All we need to do is make our value from
        // memory negative, i.e., invert it.
        let value = self.read_byte(effective_address) ^ 0xff;
        self.add(value);
    }

    fn sec(&mut self) {
        self.read_byte(self.pc);
        self.p.insert(Status::C);
    }

    fn sed(&mut self) {
        self.read_byte(self.pc);
        self.p.insert(Status::D);
    }

    fn sei(&mut self) {
        self.read_byte(self.pc);
        self.p.insert(Status::I);
    }

    fn slo<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M>(InstructionName::Slo);
        self.a |= result;

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn sre<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M>(InstructionName::Sre);
        self.a ^= result;

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn sta<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, true>();

        self.write_byte(effective_address, self.a);
    }

    fn stx<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, true>();

        self.write_byte(effective_address, self.x);
    }

    fn sty<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, true>();

        self.write_byte(effective_address, self.y);
    }

    fn tax(&mut self) {
        self.read_byte(self.pc);
        self.x = self.a;

        self.p.set(Status::Z, self.x == 0);
        self.p.set(Status::N, self.x & 0x80 != 0);
    }

    fn tay(&mut self) {
        self.read_byte(self.pc);
        self.y = self.a;

        self.p.set(Status::Z, self.y == 0);
        self.p.set(Status::N, self.y & 0x80 != 0);
    }

    fn tsx(&mut self) {
        self.read_byte(self.pc);
        self.x = self.s;

        self.p.set(Status::Z, self.x == 0);
        self.p.set(Status::N, self.x & 0x80 != 0);
    }

    fn txa(&mut self) {
        self.read_byte(self.pc);
        self.a = self.x;

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn txs(&mut self) {
        self.read_byte(self.pc);
        self.s = self.x;
    }

    fn tya(&mut self) {
        self.read_byte(self.pc);
        self.a = self.y;

        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }
}
