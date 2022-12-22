use bitflags::bitflags;

use crate::{Bus, Pins};

const NMI_VECTOR: u16 = 0xfffa;
const RESET_VECTOR: u16 = 0xfffc;
const IRQ_VECTOR: u16 = 0xfffe;

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

const ASL: u8 = 0;
const DCP: u8 = 1;
const DEC: u8 = 2;
const INC: u8 = 3;
const ISB: u8 = 4;
const LSR: u8 = 5;
const RLA: u8 = 6;
const ROL: u8 = 7;
const ROR: u8 = 8;
const RRA: u8 = 9;
const SLO: u8 = 10;
const SRE: u8 = 11;

const BRK: u8 = 0;
const IRQ: u8 = 1;
const NMI: u8 = 2;
const RST: u8 = 3;

const STACK_BASE: u16 = 0x0100;

bitflags! {
    /// The status register bitflags.
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

/// A MOS 6502 CPU.
pub struct Cpu<B> {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub s: u8,
    pub p: Status,
    pub pins: Pins,
    pub cycles: u64,

    prev_irq: bool,
    irq: bool,
    prev_nmi: bool,
    prev_need_nmi: bool,
    need_nmi: bool,
    rst: bool,

    pub bus: B,
}

impl<B> Cpu<B>
where
    B: Bus,
{
    const OPCODE_LUT: [fn(&mut Cpu<B>); 256] = [
        Cpu::brk::<BRK>,
        Cpu::ora::<INDEXED_INDIRECT>,
        Cpu::jam,
        Cpu::slo::<INDEXED_INDIRECT>,
        Cpu::nop::<ZERO_PAGE>,
        Cpu::ora::<ZERO_PAGE>,
        Cpu::asl::<ZERO_PAGE>,
        Cpu::slo::<ZERO_PAGE>,
        Cpu::php,
        Cpu::ora::<IMMEDIATE>,
        Cpu::asl::<ACCUMULATOR>,
        Cpu::anc::<IMMEDIATE>,
        Cpu::nop::<ABSOLUTE>,
        Cpu::ora::<ABSOLUTE>,
        Cpu::asl::<ABSOLUTE>,
        Cpu::slo::<ABSOLUTE>,
        Cpu::bpl,
        Cpu::ora::<INDIRECT_INDEXED>,
        Cpu::jam,
        Cpu::slo::<INDIRECT_INDEXED>,
        Cpu::nop::<ZERO_PAGE_X>,
        Cpu::ora::<ZERO_PAGE_X>,
        Cpu::asl::<ZERO_PAGE_X>,
        Cpu::slo::<ZERO_PAGE_X>,
        Cpu::clc,
        Cpu::ora::<ABSOLUTE_Y>,
        Cpu::nop::<IMPLIED>,
        Cpu::slo::<ABSOLUTE_Y>,
        Cpu::nop::<ABSOLUTE_X>,
        Cpu::ora::<ABSOLUTE_X>,
        Cpu::asl::<ABSOLUTE_X>,
        Cpu::slo::<ABSOLUTE_X>,
        Cpu::jsr,
        Cpu::and::<INDEXED_INDIRECT>,
        Cpu::jam,
        Cpu::rla::<INDEXED_INDIRECT>,
        Cpu::bit::<ZERO_PAGE>,
        Cpu::and::<ZERO_PAGE>,
        Cpu::rol::<ZERO_PAGE>,
        Cpu::rla::<ZERO_PAGE>,
        Cpu::plp,
        Cpu::and::<IMMEDIATE>,
        Cpu::rol::<ACCUMULATOR>,
        Cpu::anc::<IMMEDIATE>,
        Cpu::bit::<ABSOLUTE>,
        Cpu::and::<ABSOLUTE>,
        Cpu::rol::<ABSOLUTE>,
        Cpu::rla::<ABSOLUTE>,
        Cpu::bmi,
        Cpu::and::<INDIRECT_INDEXED>,
        Cpu::jam,
        Cpu::rla::<INDIRECT_INDEXED>,
        Cpu::nop::<ZERO_PAGE_X>,
        Cpu::and::<ZERO_PAGE_X>,
        Cpu::rol::<ZERO_PAGE_X>,
        Cpu::rla::<ZERO_PAGE_X>,
        Cpu::sec,
        Cpu::and::<ABSOLUTE_Y>,
        Cpu::nop::<IMPLIED>,
        Cpu::rla::<ABSOLUTE_Y>,
        Cpu::nop::<ABSOLUTE_X>,
        Cpu::and::<ABSOLUTE_X>,
        Cpu::rol::<ABSOLUTE_X>,
        Cpu::rla::<ABSOLUTE_X>,
        Cpu::rti,
        Cpu::eor::<INDEXED_INDIRECT>,
        Cpu::jam,
        Cpu::sre::<INDEXED_INDIRECT>,
        Cpu::nop::<ZERO_PAGE>,
        Cpu::eor::<ZERO_PAGE>,
        Cpu::lsr::<ZERO_PAGE>,
        Cpu::sre::<ZERO_PAGE>,
        Cpu::pha,
        Cpu::eor::<IMMEDIATE>,
        Cpu::lsr::<ACCUMULATOR>,
        Cpu::alr::<IMMEDIATE>,
        Cpu::jmp::<ABSOLUTE>,
        Cpu::eor::<ABSOLUTE>,
        Cpu::lsr::<ABSOLUTE>,
        Cpu::sre::<ABSOLUTE>,
        Cpu::bvc,
        Cpu::eor::<INDIRECT_INDEXED>,
        Cpu::jam,
        Cpu::sre::<INDIRECT_INDEXED>,
        Cpu::nop::<ZERO_PAGE_X>,
        Cpu::eor::<ZERO_PAGE_X>,
        Cpu::lsr::<ZERO_PAGE_X>,
        Cpu::sre::<ZERO_PAGE_X>,
        Cpu::cli,
        Cpu::eor::<ABSOLUTE_Y>,
        Cpu::nop::<IMPLIED>,
        Cpu::sre::<ABSOLUTE_Y>,
        Cpu::nop::<ABSOLUTE_X>,
        Cpu::eor::<ABSOLUTE_X>,
        Cpu::lsr::<ABSOLUTE_X>,
        Cpu::sre::<ABSOLUTE_X>,
        Cpu::rts,
        Cpu::adc::<INDEXED_INDIRECT>,
        Cpu::jam,
        Cpu::rra::<INDEXED_INDIRECT>,
        Cpu::nop::<ZERO_PAGE>,
        Cpu::adc::<ZERO_PAGE>,
        Cpu::ror::<ZERO_PAGE>,
        Cpu::rra::<ZERO_PAGE>,
        Cpu::pla,
        Cpu::adc::<IMMEDIATE>,
        Cpu::ror::<ACCUMULATOR>,
        Cpu::arr::<IMMEDIATE>,
        Cpu::jmp::<INDIRECT>,
        Cpu::adc::<ABSOLUTE>,
        Cpu::ror::<ABSOLUTE>,
        Cpu::rra::<ABSOLUTE>,
        Cpu::bvs,
        Cpu::adc::<INDIRECT_INDEXED>,
        Cpu::jam,
        Cpu::rra::<INDIRECT_INDEXED>,
        Cpu::nop::<ZERO_PAGE_X>,
        Cpu::adc::<ZERO_PAGE_X>,
        Cpu::ror::<ZERO_PAGE_X>,
        Cpu::rra::<ZERO_PAGE_X>,
        Cpu::sei,
        Cpu::adc::<ABSOLUTE_Y>,
        Cpu::nop::<IMPLIED>,
        Cpu::rra::<ABSOLUTE_Y>,
        Cpu::nop::<ABSOLUTE_X>,
        Cpu::adc::<ABSOLUTE_X>,
        Cpu::ror::<ABSOLUTE_X>,
        Cpu::rra::<ABSOLUTE_X>,
        Cpu::nop::<IMMEDIATE>,
        Cpu::sta::<INDEXED_INDIRECT>,
        Cpu::nop::<IMMEDIATE>,
        Cpu::sax::<INDEXED_INDIRECT>,
        Cpu::sty::<ZERO_PAGE>,
        Cpu::sta::<ZERO_PAGE>,
        Cpu::stx::<ZERO_PAGE>,
        Cpu::sax::<ZERO_PAGE>,
        Cpu::dey,
        Cpu::nop::<IMMEDIATE>,
        Cpu::txa,
        Cpu::ane::<IMMEDIATE>,
        Cpu::sty::<ABSOLUTE>,
        Cpu::sta::<ABSOLUTE>,
        Cpu::stx::<ABSOLUTE>,
        Cpu::sax::<ABSOLUTE>,
        Cpu::bcc,
        Cpu::sta::<INDIRECT_INDEXED>,
        Cpu::jam,
        Cpu::sha::<ABSOLUTE_Y>,
        Cpu::sty::<ZERO_PAGE_X>,
        Cpu::sta::<ZERO_PAGE_X>,
        Cpu::stx::<ZERO_PAGE_Y>,
        Cpu::sax::<ZERO_PAGE_Y>,
        Cpu::tya,
        Cpu::sta::<ABSOLUTE_Y>,
        Cpu::txs,
        Cpu::tas::<ABSOLUTE_Y>,
        Cpu::shy::<ABSOLUTE_X>,
        Cpu::sta::<ABSOLUTE_X>,
        Cpu::shx::<ABSOLUTE_Y>,
        Cpu::sha::<INDIRECT_INDEXED>,
        Cpu::ldy::<IMMEDIATE>,
        Cpu::lda::<INDEXED_INDIRECT>,
        Cpu::ldx::<IMMEDIATE>,
        Cpu::lax::<INDEXED_INDIRECT>,
        Cpu::ldy::<ZERO_PAGE>,
        Cpu::lda::<ZERO_PAGE>,
        Cpu::ldx::<ZERO_PAGE>,
        Cpu::lax::<ZERO_PAGE>,
        Cpu::tay,
        Cpu::lda::<IMMEDIATE>,
        Cpu::tax,
        Cpu::lxa::<IMMEDIATE>,
        Cpu::ldy::<ABSOLUTE>,
        Cpu::lda::<ABSOLUTE>,
        Cpu::ldx::<ABSOLUTE>,
        Cpu::lax::<ABSOLUTE>,
        Cpu::bcs,
        Cpu::lda::<INDIRECT_INDEXED>,
        Cpu::jam,
        Cpu::lax::<INDIRECT_INDEXED>,
        Cpu::ldy::<ZERO_PAGE_X>,
        Cpu::lda::<ZERO_PAGE_X>,
        Cpu::ldx::<ZERO_PAGE_Y>,
        Cpu::lax::<ZERO_PAGE_Y>,
        Cpu::clv,
        Cpu::lda::<ABSOLUTE_Y>,
        Cpu::tsx,
        Cpu::las::<ABSOLUTE_Y>,
        Cpu::ldy::<ABSOLUTE_X>,
        Cpu::lda::<ABSOLUTE_X>,
        Cpu::ldx::<ABSOLUTE_Y>,
        Cpu::lax::<ABSOLUTE_Y>,
        Cpu::cpy::<IMMEDIATE>,
        Cpu::cmp::<INDEXED_INDIRECT>,
        Cpu::nop::<IMMEDIATE>,
        Cpu::dcp::<INDEXED_INDIRECT>,
        Cpu::cpy::<ZERO_PAGE>,
        Cpu::cmp::<ZERO_PAGE>,
        Cpu::dec::<ZERO_PAGE>,
        Cpu::dcp::<ZERO_PAGE>,
        Cpu::iny,
        Cpu::cmp::<IMMEDIATE>,
        Cpu::dex,
        Cpu::sbx::<IMMEDIATE>,
        Cpu::cpy::<ABSOLUTE>,
        Cpu::cmp::<ABSOLUTE>,
        Cpu::dec::<ABSOLUTE>,
        Cpu::dcp::<ABSOLUTE>,
        Cpu::bne,
        Cpu::cmp::<INDIRECT_INDEXED>,
        Cpu::jam,
        Cpu::dcp::<INDIRECT_INDEXED>,
        Cpu::nop::<ZERO_PAGE_X>,
        Cpu::cmp::<ZERO_PAGE_X>,
        Cpu::dec::<ZERO_PAGE_X>,
        Cpu::dcp::<ZERO_PAGE_X>,
        Cpu::cld,
        Cpu::cmp::<ABSOLUTE_Y>,
        Cpu::nop::<IMPLIED>,
        Cpu::dcp::<ABSOLUTE_Y>,
        Cpu::nop::<ABSOLUTE_X>,
        Cpu::cmp::<ABSOLUTE_X>,
        Cpu::dec::<ABSOLUTE_X>,
        Cpu::dcp::<ABSOLUTE_X>,
        Cpu::cpx::<IMMEDIATE>,
        Cpu::sbc::<INDEXED_INDIRECT>,
        Cpu::nop::<IMMEDIATE>,
        Cpu::isb::<INDEXED_INDIRECT>,
        Cpu::cpx::<ZERO_PAGE>,
        Cpu::sbc::<ZERO_PAGE>,
        Cpu::inc::<ZERO_PAGE>,
        Cpu::isb::<ZERO_PAGE>,
        Cpu::inx,
        Cpu::sbc::<IMMEDIATE>,
        Cpu::nop::<IMPLIED>,
        Cpu::sbc::<IMMEDIATE>,
        Cpu::cpx::<ABSOLUTE>,
        Cpu::sbc::<ABSOLUTE>,
        Cpu::inc::<ABSOLUTE>,
        Cpu::isb::<ABSOLUTE>,
        Cpu::beq,
        Cpu::sbc::<INDIRECT_INDEXED>,
        Cpu::jam,
        Cpu::isb::<INDIRECT_INDEXED>,
        Cpu::nop::<ZERO_PAGE_X>,
        Cpu::sbc::<ZERO_PAGE_X>,
        Cpu::inc::<ZERO_PAGE_X>,
        Cpu::isb::<ZERO_PAGE_X>,
        Cpu::sed,
        Cpu::sbc::<ABSOLUTE_Y>,
        Cpu::nop::<IMPLIED>,
        Cpu::isb::<ABSOLUTE_Y>,
        Cpu::nop::<ABSOLUTE_X>,
        Cpu::sbc::<ABSOLUTE_X>,
        Cpu::inc::<ABSOLUTE_X>,
        Cpu::isb::<ABSOLUTE_X>,
    ];

    /// Constructs a new `Cpu` in a power-up state.
    pub fn new(bus: B) -> Cpu<B> {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xfd,
            p: Status::default(),
            pins: Pins::default(),
            cycles: 0,
            prev_irq: false,
            irq: false,
            prev_nmi: false,
            prev_need_nmi: false,
            need_nmi: false,
            rst: true,
            bus,
        }
    }

    /// Executes the next instruction.
    pub fn step(&mut self) {
        if self.rst || self.prev_need_nmi || self.prev_irq {
            let brk_fn = if self.rst {
                // TODO: Reset CPU struct fields?
                self.rst = false;
                Cpu::brk::<RST>
            } else if self.prev_need_nmi {
                self.need_nmi = false;
                Cpu::brk::<NMI>
            } else {
                Cpu::brk::<IRQ>
            };

            self.read_byte(self.pc);
            (brk_fn)(self);
        } else {
            let opcode = self.consume_byte();
            (Cpu::OPCODE_LUT[opcode as usize])(self);
        }
    }

    fn read_byte(&mut self, address: u16) -> u8 {
        self.cycles += 1;

        self.pins.address = address;
        self.pins.rw = true;
        self.bus.tick(&mut self.pins);

        self.poll_interrupts();

        self.pins.data
    }

    fn read_word(&mut self, address: u16) -> u16 {
        let low = self.read_byte(address);
        let high = self.read_byte(address.wrapping_add(1));
        (high as u16) << 8 | low as u16
    }

    fn read_word_bugged(&mut self, address: u16) -> u16 {
        let low = self.read_byte(address);
        // Indirect addressing modes are affected by a hardware bug where reads
        // that would cross a page instead wrap around in the same page.
        let high = self.read_byte(
            (address & 0xff00) | (address as u8).wrapping_add(1) as u16,
        );
        (high as u16) << 8 | low as u16
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        self.cycles += 1;

        self.pins.address = address;
        self.pins.data = data;
        self.pins.rw = false;
        self.bus.tick(&mut self.pins);

        self.poll_interrupts();
    }

    fn consume_byte(&mut self) -> u8 {
        let data = self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        data
    }

    fn consume_word(&mut self) -> u16 {
        let data = self.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        data
    }

    fn peek(&mut self) -> u8 {
        self.read_byte(STACK_BASE + self.s as u16)
    }

    fn push(&mut self, data: u8) {
        self.write_byte(STACK_BASE + self.s as u16, data);
        self.s = self.s.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.read_byte(STACK_BASE + self.s as u16)
    }

    fn poll_interrupts(&mut self) {
        // We need to track the previous status of the interrupt pins because
        // their statuses at the end of the second-to-last cycle determine if
        // the next instruction will be an interrupt.
        self.prev_irq = self.irq;
        self.irq = self.pins.irq && !self.p.contains(Status::I);

        self.prev_need_nmi = self.need_nmi;

        // An NMI is raised if the NMI pin goes from inactive during one cycle
        // to active during the next. The NMI stays "raised" until it's
        // handled.
        if !self.prev_nmi && self.pins.nmi {
            self.need_nmi = true;
        }
        self.prev_nmi = self.pins.nmi;

        if !self.rst && self.pins.rst {
            self.rst = self.pins.rst;
        }
    }

    fn set_a(&mut self, value: u8) {
        self.a = value;
        self.p.set(Status::Z, self.a == 0);
        self.p.set(Status::N, self.a & 0x80 != 0);
    }

    fn set_x(&mut self, value: u8) {
        self.x = value;
        self.p.set(Status::Z, self.x == 0);
        self.p.set(Status::N, self.x & 0x80 != 0);
    }

    fn set_y(&mut self, value: u8) {
        self.y = value;
        self.p.set(Status::Z, self.y == 0);
        self.p.set(Status::N, self.y & 0x80 != 0);
    }
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
            self.pc = self.pc.wrapping_add(offset);

            if old_pc & 0xff00 != self.pc & 0xff00 {
                self.read_byte(
                    (old_pc & 0xff00)
                        | (old_pc as u8).wrapping_add(offset as u8) as u16,
                );
            }
        }
    }

    fn compare(&mut self, register: u8, value: u8) {
        let result = register.wrapping_sub(value);

        self.p.set(Status::C, register >= value);
        self.p.set(Status::Z, result == 0);
        self.p.set(Status::N, result & 0x80 != 0);
    }

    fn modify<const I: u8>(&mut self, value: u8) -> u8 {
        let (result, carry) = match I {
            ASL | SLO => (value.wrapping_shl(1), value & 0x80 != 0),
            DCP | DEC => (value.wrapping_sub(1), self.p.contains(Status::C)),
            INC | ISB => (value.wrapping_add(1), self.p.contains(Status::C)),
            LSR | SRE => (value.wrapping_shr(1), value & 0x01 != 0),
            ROL | RLA => (
                value.wrapping_shl(1) | self.p.contains(Status::C) as u8,
                value & 0x80 != 0,
            ),
            ROR | RRA => (
                (self.p.contains(Status::C) as u8) << 7
                    | value.wrapping_shr(1),
                value & 0x01 != 0,
            ),
            _ => unreachable!("unexpected RMW instruction: {}", I),
        };

        self.p.set(Status::C, carry);
        self.p.set(Status::Z, result == 0);
        self.p.set(Status::N, result & 0x80 != 0);

        result
    }

    fn read_modify_write<const M: u8, const I: u8>(&mut self) -> u8 {
        if M == ACCUMULATOR {
            self.read_byte(self.pc);
            self.a = self.modify::<I>(self.a);
            self.a
        } else {
            // Treat it as a write instruction while fetching the effective
            // address to get the cycle count right.
            let effective_address = self.effective_address::<M, true>();
            let value = self.read_byte(effective_address);

            // Read-Modify-Write instructions have an extra write since it
            // takes an extra cycle to modify the value.
            self.write_byte(effective_address, value);

            let result = self.modify::<I>(value);

            self.write_byte(effective_address, result);

            result
        }
    }

    fn effective_address<const M: u8, const W: bool>(&mut self) -> u16 {
        match M {
            ABSOLUTE => self.consume_word(),
            ABSOLUTE_X | ABSOLUTE_Y => {
                let index = if M == ABSOLUTE_X { self.x } else { self.y };

                let (low, page_cross) =
                    self.consume_byte().overflowing_add(index);
                let high = self.consume_byte();

                let effective_address =
                    (high.wrapping_add(page_cross as u8) as u16) << 8
                        | (low as u16);

                // If the effective address is invalid, i.e., it crossed a
                // page, then it takes an extra read cycle to fix it. Write
                // instructions always have the extra read since they can't
                // undo a write to an invalid address.
                if page_cross || W {
                    self.read_byte((high as u16) << 8 | low as u16);
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

                // If the effective address is invalid, i.e., it crossed a
                // page, then it takes an extra read cycle to fix it. Write
                // instructions always have the extra read since they can't
                // undo a write to an invalid address.
                if did_cross_page || W {
                    self.read_byte((high as u16) << 8 | low as u16);
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
            _ => unreachable!("unexpected addressing mode: {}", M),
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

    fn anc<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();
        let operand = self.read_byte(effective_address);
        self.set_a(self.a & operand);

        self.p.set(Status::C, self.a & 0x80 != 0);
    }

    fn and<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();
        let operand = self.read_byte(effective_address);
        self.set_a(self.a & operand);
    }

    fn alr<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        self.a &= self.read_byte(effective_address);
        let carry = self.a & 0x01 != 0;
        self.set_a(self.a.wrapping_shr(1));

        self.p.set(Status::C, carry);
    }

    fn ane<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        // Treat ANE as a NOP since it's unstable.
        self.read_byte(effective_address);
    }

    fn arr<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        self.a &= self.read_byte(effective_address);
        self.set_a(
            (self.p.contains(Status::C) as u8) << 7 | self.a.wrapping_shr(1),
        );

        // TODO: Explain how the carry and overflow flag are set.
        self.p.set(Status::C, self.a & 0x40 != 0);
        self.p.set(
            Status::V,
            ((self.p.contains(Status::C) as u8) ^ ((self.a >> 5) & 0x01)) != 0,
        );
    }

    fn asl<const M: u8>(&mut self) {
        self.read_modify_write::<M, ASL>();
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

    fn brk<const I: u8>(&mut self) {
        self.read_byte(self.pc);
        if I == BRK {
            self.pc += 1;
        }

        if I == RST {
            self.peek();
            self.s = self.s.wrapping_sub(1);
            self.peek();
            self.s = self.s.wrapping_sub(1);
            self.peek();
            self.s = self.s.wrapping_sub(1);
        } else {
            self.push((self.pc >> 8) as u8);
            self.push(self.pc as u8);
            let p = if I == BRK { self.p | Status::B } else { self.p };
            self.push(p.bits());
        }

        // TODO: Implement interrupt hijacking.
        // TODO: Should NMI not set the I flag?
        self.p.insert(Status::I);
        let vector = match I {
            BRK | IRQ => IRQ_VECTOR,
            NMI => NMI_VECTOR,
            RST => RESET_VECTOR,
            _ => unreachable!("unexpected interrupt type: {}", I),
        };
        self.pc = self.read_word(vector);
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
        let result = self.read_modify_write::<M, DCP>();
        self.compare(self.a, result);
    }

    fn dec<const M: u8>(&mut self) {
        self.read_modify_write::<M, DEC>();
    }

    fn dex(&mut self) {
        self.read_byte(self.pc);
        self.set_x(self.x.wrapping_sub(1));
    }

    fn dey(&mut self) {
        self.read_byte(self.pc);
        self.set_y(self.y.wrapping_sub(1));
    }

    fn eor<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();
        let operand = self.read_byte(effective_address);
        self.set_a(self.a ^ operand);
    }

    fn inc<const M: u8>(&mut self) {
        self.read_modify_write::<M, INC>();
    }

    fn inx(&mut self) {
        self.read_byte(self.pc);
        self.set_x(self.x.wrapping_add(1));
    }

    fn iny(&mut self) {
        self.read_byte(self.pc);
        self.set_y(self.y.wrapping_add(1));
    }

    fn isb<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M, ISB>();
        self.add(result ^ 0xff);
    }

    fn jam(&mut self) {
        // Treat JAM as a one byte NOP.
        self.read_byte(self.pc);
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

    fn las<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        self.a = self.read_byte(effective_address) & self.s;
        self.set_x(self.a);
        self.s = self.a;
    }

    fn lax<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        let value = self.read_byte(effective_address);
        self.a = value;
        self.set_x(value);
    }

    fn lda<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();
        let operand = self.read_byte(effective_address);
        self.set_a(operand);
    }

    fn ldx<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();
        let operand = self.read_byte(effective_address);
        self.set_x(operand);
    }

    fn ldy<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();
        let operand = self.read_byte(effective_address);
        self.set_y(operand);
    }

    fn lsr<const M: u8>(&mut self) {
        self.read_modify_write::<M, LSR>();
    }

    fn lxa<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        // This instruction should perform a bitwise AND between a constant and
        // the operand before storing the result. The constant is unreliable
        // though. To remove uncertainty, we have the constant always be 0xff,
        // removing the need for the bitwise AND.
        self.a = self.read_byte(effective_address);
        self.set_x(self.a);
    }

    fn nop<const M: u8>(&mut self) {
        if M == IMPLIED {
            self.read_byte(self.pc);
        } else {
            let effective_address = self.effective_address::<M, false>();
            self.read_byte(effective_address);
        }
    }

    fn ora<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();
        let operand = self.read_byte(effective_address);
        self.set_a(self.a | operand);
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
        let value = self.pop();
        self.set_a(value);
    }

    fn plp(&mut self) {
        self.read_byte(self.pc);
        self.peek();
        self.p = (Status::from_bits_truncate(self.pop())
            & !(Status::B | Status::U))
            | (self.p & (Status::B | Status::U));
    }

    fn rla<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M, RLA>();
        self.set_a(self.a & result);
    }

    fn rol<const M: u8>(&mut self) {
        self.read_modify_write::<M, ROL>();
    }

    fn ror<const M: u8>(&mut self) {
        self.read_modify_write::<M, ROR>();
    }

    fn rra<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M, RRA>();
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

        // If we reformulate subtraction as addition, then we can use the same
        // logic for ADC and SBC. All we need to do is make our value from
        // memory negative, i.e., invert it.
        let value = self.read_byte(effective_address) ^ 0xff;
        self.add(value);
    }

    fn sbx<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, false>();

        let value = self.read_byte(effective_address);
        let carry = (self.a & self.x) >= value;
        self.set_x((self.a & self.x).wrapping_sub(value));

        self.p.set(Status::C, carry);
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

    fn sha<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, true>();

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.a & self.x & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.a as u16 & self.x as u16 & (high_byte.wrapping_add(1)))
                << 8)
                | low_byte,
            value,
        );
    }

    fn shx<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, true>();

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.x & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.x as u16 & (high_byte.wrapping_add(1))) << 8) | low_byte,
            value,
        );
    }

    fn shy<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, true>();

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.y & (high_byte as u8).wrapping_add(1);

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.y as u16 & (high_byte.wrapping_add(1))) << 8) | low_byte,
            value,
        );
    }

    fn slo<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M, SLO>();
        self.set_a(self.a | result);
    }

    fn sre<const M: u8>(&mut self) {
        let result = self.read_modify_write::<M, SRE>();
        self.set_a(self.a ^ result);
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

    fn tas<const M: u8>(&mut self) {
        let effective_address = self.effective_address::<M, true>();

        let high_byte = (effective_address & 0xff00) >> 8;
        let low_byte = effective_address & 0x00ff;
        let value = self.a & self.x & (high_byte as u8).wrapping_add(1);
        self.s = self.a & self.x;

        // https://forums.nesdev.org/viewtopic.php?f=3&t=3831&start=30
        self.write_byte(
            ((self.a as u16 & self.x as u16 & (high_byte.wrapping_add(1)))
                << 8)
                | low_byte,
            value,
        );
    }

    fn tax(&mut self) {
        self.read_byte(self.pc);
        self.set_x(self.a);
    }

    fn tay(&mut self) {
        self.read_byte(self.pc);
        self.set_y(self.a);
    }

    fn tsx(&mut self) {
        self.read_byte(self.pc);
        self.set_x(self.s);
    }

    fn txa(&mut self) {
        self.read_byte(self.pc);
        self.set_a(self.x);
    }

    fn txs(&mut self) {
        self.read_byte(self.pc);
        self.s = self.x;
    }

    fn tya(&mut self) {
        self.read_byte(self.pc);
        self.set_a(self.y);
    }
}
