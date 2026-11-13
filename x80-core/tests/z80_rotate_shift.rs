//! Rotate/shift witnesses: decisions 13–15; UM008011-0816 rev. 11 (August
//! 2016), Rotate and Shift Group pp. 204–241 (PDF printed + 14), cross-read
//! with 03-0029-01 (1977), Table 7.0-7 p. 50 (PDF 56). Both pinned hashes
//! match PROVENANCE.md. Values read on rendered 150 dpi page images.
//! Expectations are literal transcriptions/derivations, never executor helpers.
//! Every retiring witness fails before implementation with Unsupported, zero
//! steps/cycles and no Data accesses (LANDED=false at base b11628b).
//! Decision-20 §4 H Y/X: Young v0.91 §8.7 p. 32 / Rak rotate/shift,
//! .rld/.rrd groups; all eight flag bits are compared.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ARG: MemoryAccess = MemoryAccess::OperandFetch;
const DATA: MemoryAccess = MemoryAccess::Data;

// Literal F columns, decoded independently of Flags::from_byte (UM Table 21
// p. 65; decision-14 rule 5). Expected state never calls a core flag helper.
fn literal_flags(byte: u8) -> Flags {
    Flags {
        sign: byte & 0x80 != 0,
        zero: byte & 0x40 != 0,
        y: byte & 0x20 != 0,
        x: byte & 0x08 != 0,
        half_carry: byte & 0x10 != 0,
        parity_overflow: byte & 0x04 != 0,
        subtract: byte & 0x02 != 0,
        carry: byte & 0x01 != 0,
    }
}

fn registers() -> Registers {
    Registers {
        a: 0xA1,
        flags: literal_flags(0xD7),
        b: 0xB2,
        c: 0xC3,
        d: 0xD4,
        e: 0xE5,
        h: 0x90,
        l: 0x12,
        a_alternate: 0x18,
        flags_alternate: literal_flags(0x42),
        b_alternate: 0x29,
        c_alternate: 0x3A,
        d_alternate: 0x4B,
        e_alternate: 0x5C,
        h_alternate: 0x6D,
        l_alternate: 0x7E,
        ix: 0x1234,
        iy: 0x5678,
        sp: 0xDEF0,
        pc: 0xFFFF,
        i: 0xF8,
        r: 0xFF,
    }
}

fn latches() -> Latches {
    Latches {
        iff1: true,
        iff2: true,
        interrupt_mode: InterruptMode::Mode2,
        ei_delay: true,
        parity_note_pending: true,
        ..Latches::default()
    }
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

fn write(address: u16, value: u8) -> Event {
    Event::Write {
        address,
        kind: DATA,
        value,
    }
}

fn polls() -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ]
}

fn completed(cpu: &Z80, report: Report, expected: Registers, cycles: u64, wz: u16) {
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles,
            exit: Exit::Budget
        }
    );
    assert_eq!(*cpu.registers(), expected);
    // Decision-20 §4 H full-byte comparison is included in Registers above.
    assert_eq!(
        *cpu.latches(),
        Latches {
            q: true,
            wz,
            ei_delay: false,
            parity_note_pending: false,
            ..latches()
        }
    );
}

// Register codes: UM pp. 213, 222, 225, 228, 231; TM p. 50.
fn put(r: &mut Registers, code: u8, value: u8) {
    match code {
        0 => r.b = value,
        1 => r.c = value,
        2 => r.d = value,
        3 => r.e = value,
        4 => r.h = value,
        5 => r.l = value,
        7 => r.a = value,
        _ => panic!("not a register operand"),
    }
}

// op, input A, incoming C, result A, outgoing C. S/Z/P-V are preserved
// for every combination, even when the result would imply opposite flags.
// UM RLCA pp. 205–206; RLA pp. 207–208; RRCA pp. 209–210;
// RRA pp. 211–212; TM Table 7.0-7 p. 50.
fn accumulator(opcode: u8, cases: &[(u8, bool, u8, bool)]) {
    for &(input, carry, result, next_carry) in cases {
        for preserved in [0x00, 0x04, 0x40, 0x44, 0x80, 0x84, 0xC0, 0xC4] {
            for hn in [0x00, 0x02, 0x10, 0x12] {
                let mut initial = registers();
                initial.a = input;
                initial.flags = literal_flags(preserved | hn | u8::from(carry));
                let mut expected = initial;
                expected.a = result;
                expected.flags = literal_flags(preserved | u8::from(next_carry) | (result & 0x28));
                // Derived 16-bit PC wrap; R's low seven bits wrap, bit 7 stays
                // set (decision-15 rules 5–6; UM Registers p. 3).
                expected.pc = 0;
                expected.r = 0x80;
                let mut cpu = Z80::new(initial, latches());
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[opcode]);
                let memory = bus.memory.clone();
                let report = cpu.step(&mut bus);
                completed(&cpu, report, expected, 4, 0);
                assert_eq!(bus.memory, memory);
                let mut trace = polls();
                trace.push(read(0xFFFF, OP, opcode));
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn rlca_preserves_szpv_and_copies_bit_seven() {
    accumulator(
        0x07,
        &[
            (0x88, false, 0x11, true),
            (0x40, true, 0x80, false),
            (0, true, 0, false),
        ],
    );
}

#[test]
fn rla_uses_old_carry() {
    accumulator(
        0x17,
        &[
            (0x76, true, 0xED, false),
            (0x80, false, 0, true),
            (0x80, true, 1, true),
        ],
    );
}

#[test]
fn rrca_preserves_szpv_despite_worked_example_bit_four() {
    // UM p. 210 prints 11h -> 98h, C=1. Follow p. 209's description
    // and TM Table 7.0-7 p. 50: 11h -> 88h, C=1. Both sources re-read
    // on enlarged 200 dpi crops; recorded in PROVENANCE.md §1.
    accumulator(
        0x0F,
        &[
            (0x11, false, 0x88, true),
            (0x80, true, 0x40, false),
            (0, true, 0, false),
        ],
    );
}

#[test]
fn rra_uses_old_carry() {
    accumulator(
        0x1F,
        &[
            (0xE1, false, 0x70, true),
            (0x01, true, 0x80, true),
            (0x80, true, 0xC0, false),
        ],
    );
}

// Input, incoming C, result, complete documented F. Each row derives from
// the entry's bit routing and even-parity rule, cross-read with TM p. 50.
// Both initial values of S/Z/H/P-V/N are tried independently of incoming C.
type Case = (u8, bool, u8, u8);

const RLC: &[Case] = &[
    (0x88, false, 0x11, 0x05),
    (0, true, 0, 0x44),
    (0x80, false, 1, 0x01),
    (0x40, true, 0x80, 0x80),
    (0xFF, false, 0xFF, 0x85),
    (1, true, 2, 0),
];
const RRC: &[Case] = &[
    (0x31, false, 0x98, 0x81),
    (0, true, 0, 0x44),
    (1, false, 0x80, 0x81),
    (0x80, true, 0x40, 0),
    (0xFF, false, 0xFF, 0x85),
    (2, true, 1, 0),
];
const RL: &[Case] = &[
    (0x8F, false, 0x1E, 0x05),
    (0, true, 1, 0),
    (0x80, false, 0, 0x45),
    (0x40, false, 0x80, 0x80),
    (0x7F, true, 0xFF, 0x84),
    (0, false, 0, 0x44),
    (0x80, true, 1, 1),
];
const RR: &[Case] = &[
    (0xDD, false, 0x6E, 0x01),
    (0, true, 0x80, 0x80),
    (1, false, 0, 0x45),
    (0x80, true, 0xC0, 0x84),
    (0xFF, false, 0x7F, 0x01),
    (0, false, 0, 0x44),
    (1, true, 0x80, 0x81),
];
const SLA: &[Case] = &[
    (0xB1, false, 0x62, 0x01),
    (0x80, true, 0, 0x45),
    (0x40, false, 0x80, 0x80),
    (0x7F, true, 0xFE, 0x80),
    (0xFF, false, 0xFE, 0x81),
    (0, true, 0, 0x44),
];
const SRA: &[Case] = &[
    (0xB8, true, 0xDC, 0x80),
    (0x81, false, 0xC0, 0x85),
    (1, true, 0, 0x45),
    (0x7F, false, 0x3F, 0x05),
    (0xFF, false, 0xFF, 0x85),
    (0, true, 0, 0x44),
];
const SRL: &[Case] = &[
    (0x8F, false, 0x47, 0x05),
    (0x80, true, 0x40, 0),
    (1, false, 0, 0x45),
    (0x7F, true, 0x3F, 0x05),
    (0xFF, false, 0x7F, 0x01),
    (0, true, 0, 0x44),
];

fn cb(base: u8, cases: &[Case]) {
    for code in 0..8 {
        for &(input, carry, result, flags) in cases {
            for old_flags in [0x00, 0xD6] {
                let opcode = base | code;
                let mut initial = registers();
                initial.flags = literal_flags(old_flags | u8::from(carry));
                if code != 6 {
                    put(&mut initial, code, input);
                }
                let mut expected = initial;
                if code != 6 {
                    put(&mut expected, code, result);
                }
                expected.flags = literal_flags(flags | (result & 0x28));
                expected.pc = 1; // Derived: FFFF + 2 wraps.
                expected.r = 0x81; // Two M1 reads, decision-15 rules 5–6.
                let mut cpu = Z80::new(initial, latches());
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[0xCB, opcode]);
                bus.load(0x9012, &[input]);
                let mut memory = bus.memory.clone();
                let mut trace = polls();
                trace.extend([read(0xFFFF, OP, 0xCB), read(0, OP, opcode)]);
                let cycles = if code == 6 {
                    memory[0x9012] = result;
                    trace.extend([read(0x9012, DATA, input), write(0x9012, result)]);
                    15
                } else {
                    8
                };
                let report = cpu.step(&mut bus);
                completed(&cpu, report, expected, cycles, 0);
                assert_eq!(bus.memory, memory);
                assert_eq!(bus.take_log(), trace, "CB {opcode:02X}");
            }
        }
    }
}

// These cover all 56 CB terminals. RLC L/(HL) and RRC E pin the compact
// misprints (UM Fig. 39 p. 54) using the existing PROVENANCE §1 resolution.
#[test]
fn rlc_all_registers_and_hl() {
    cb(0x00, RLC);
} // UM pp. 213–216.
#[test]
fn rrc_all_registers_and_hl() {
    cb(0x08, RRC);
} // UM pp. 224–226.
#[test]
fn rl_all_registers_and_hl() {
    cb(0x10, RL);
} // UM pp. 221–223.
#[test]
fn rr_all_registers_and_hl() {
    cb(0x18, RR);
} // UM pp. 227–229; p. 228 bit slip.
#[test]
fn sla_all_registers_and_hl() {
    cb(0x20, SLA);
} // UM pp. 230–232.
#[test]
fn sra_all_registers_and_hl() {
    cb(0x28, SRA);
} // UM pp. 233–235.
#[test]
fn srl_all_registers_and_hl() {
    cb(0x38, SRL);
} // UM pp. 236–237.

#[test]
fn indexed_cb_all_operations_both_indexes_signed_displacement_and_wraps() {
    // UM RLC pp. 217–220, RL pp. 221–222, RRC pp. 224–226,
    // RR pp. 227–229, SLA pp. 230–231, SRA pp. 233–234, SRL pp. 236–237;
    // TM p. 50. The existing p. 225 diagram resolution makes FD ... 0E
    // RRC, distinguished here from RR with opposite shifted-out/input bits.
    for (opcode, cases) in [
        (0x06, RLC),
        (0x0E, RRC),
        (0x16, RL),
        (0x1E, RR),
        (0x26, SLA),
        (0x2E, SRA),
        (0x3E, SRL),
    ] {
        for prefix in [0xDD, 0xFD] {
            // Signed d extremes and zero; wraps are DERIVED from 16-bit width.
            for (index, d, address) in [
                (0x0001, 0x80, 0xFF81),
                (0xFFF0, 0x7F, 0x006F),
                (0x9000, 0xFF, 0x8FFF),
                (0x9000, 0, 0x9000),
            ] {
                for &(input, carry, result, flags) in cases {
                    for old_flags in [0x00, 0xD6] {
                        let mut initial = registers();
                        initial.flags = literal_flags(old_flags | u8::from(carry));
                        if prefix == 0xDD {
                            initial.ix = index;
                        } else {
                            initial.iy = index;
                        }
                        let mut expected = initial;
                        expected.flags = literal_flags(flags | (result & 0x28));
                        expected.pc = 3; // Derived: FFFF + 4 wraps.
                        expected.r = 0x81; // d/op aren't M1: decision-15 rules 5–6.
                        let mut cpu = Z80::new(initial, latches());
                        let mut bus = RecordingBus::new();
                        bus.load(0xFFFF, &[prefix, 0xCB, d, opcode]);
                        bus.load(address, &[input]);
                        let mut memory = bus.memory.clone();
                        memory[usize::from(address)] = result;
                        let report = cpu.step(&mut bus);
                        completed(&cpu, report, expected, 23, address);
                        assert_eq!(bus.memory, memory);
                        let mut trace = polls();
                        trace.extend([
                            read(0xFFFF, OP, prefix),
                            read(0, OP, 0xCB),
                            read(1, ARG, d),
                            read(2, ARG, opcode),
                            read(address, DATA, input),
                            write(address, result),
                        ]);
                        assert_eq!(bus.take_log(), trace);
                    }
                }
            }
        }
    }
}

// A, (HL), resulting A, resulting (HL), F without preserved C.
fn nibble(opcode: u8, cases: &[(u8, u8, u8, u8, u8)]) {
    for &(a, input, next_a, result, flags) in cases {
        for carry in [false, true] {
            for old_flags in [0x00, 0xD6] {
                let mut initial = registers();
                initial.a = a;
                initial.flags = literal_flags(old_flags | u8::from(carry));
                let mut expected = initial;
                expected.a = next_a;
                expected.flags = literal_flags(flags | u8::from(carry) | (next_a & 0x28));
                expected.pc = 1;
                expected.r = 0x81;
                let mut cpu = Z80::new(initial, latches());
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[0xED, opcode]);
                bus.load(0x9012, &[input]);
                let mut memory = bus.memory.clone();
                memory[0x9012] = result;
                let report = cpu.step(&mut bus);
                completed(&cpu, report, expected, 18, 0x9013);
                assert_eq!(bus.memory, memory);
                let mut trace = polls();
                trace.extend([
                    read(0xFFFF, OP, 0xED),
                    read(0, OP, opcode),
                    read(0x9012, DATA, input),
                    write(0x9012, result),
                ]);
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn rld_rotates_three_nibbles_flags_from_a_carry_preserved() {
    // UM pp. 238–239 (including literal 7A/31 -> 73/1A example); TM p. 50.
    nibble(
        0x6F,
        &[
            (0x7A, 0x31, 0x73, 0x1A, 0),
            (0xA5, 0xC3, 0xAC, 0x35, 0x84),
            (0x0F, 0x01, 0, 0x1F, 0x44),
            (0x10, 0xFF, 0x1F, 0xF0, 0),
            (0x01, 0, 0, 1, 0x44),
        ],
    );
}

#[test]
fn rrd_rotates_three_nibbles_flags_from_a_carry_preserved() {
    // UM pp. 240–241 (including literal 84/20 -> 80/42 example); TM p. 50.
    nibble(
        0x67,
        &[
            (0x84, 0x20, 0x80, 0x42, 0x80),
            (0xA5, 0xC3, 0xA3, 0x5C, 0x84),
            (0x0F, 0x10, 0, 0xF1, 0x44),
            (0x10, 0xFF, 0x1F, 0x0F, 0),
            (0x01, 0, 0, 0x10, 0x44),
        ],
    );
}

// Decision-20 §3 replaces the former SLL/indexed-copy refusal assertions
// with literal execution/trace witnesses in z80_undocumented_cb.rs. No fetched
// refusal remains in this group; interrupt refusals live in z80_interrupts_int.rs.
