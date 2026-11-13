//! 16-bit load witnesses: decisions 13 rules 2–5, 14 rules 1/5, 15 rules 5–6.
//! Primary: Zilog UM008011-0816 rev. 11 (August 2016), entries pp. 99–122
//! (PDF = printed + 14). Second transcription: 03-0029-01 (1977),
//! Table 7.0-2 p. 45 (PDF 51). Both hashes match PROVENANCE.md §1;
//! all entry pages and the second table were read as 150 dpi images.
//! Agreement is documentation evidence, not a hardware measurement.
//!
//! Each test failed before implementation with Unsupported, zero steps/cycles.
//! Expected values never call execution, register-pair or flag-image helpers.
//! PC, SP and nn+1 wraps are derived from 16-bit widths (UM CPU Register p. 2).
//! Poll order, access kinds and per-M1 R updates are decision-15 API mappings.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ARG: MemoryAccess = MemoryAccess::OperandFetch;
const DATA: MemoryAccess = MemoryAccess::Data;
const STACK: MemoryAccess = MemoryAccess::Stack;

// Independently enumerate all six documented flag positions; no F helpers.
fn flags(n: u8) -> Flags {
    Flags {
        carry: n & 1 != 0,
        subtract: n & 2 != 0,
        parity_overflow: n & 4 != 0,
        half_carry: n & 8 != 0,
        zero: n & 16 != 0,
        sign: n & 32 != 0,
    }
}

fn registers(n: u8) -> Registers {
    Registers {
        a: 0x69,
        flags: flags(n),
        b: 0x2A,
        c: 0x4C,
        d: 0x87,
        e: 0x16,
        h: 0xB3,
        l: 0xE8,
        a_alternate: 0xA1,
        flags_alternate: flags(21),
        b_alternate: 0xB2,
        c_alternate: 0xC3,
        d_alternate: 0xD4,
        e_alternate: 0xE5,
        h_alternate: 0xF6,
        l_alternate: 0x07,
        ix: 0x8091,
        iy: 0x92A3,
        sp: 0x9AAF,
        pc: 0x2000,
        i: 0xF8,
        r: 0xFF,
    }
}

fn latches() -> Latches {
    Latches {
        iff1: true,
        iff2: false,
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

fn write(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Write {
        address,
        kind,
        value,
    }
}

fn fetch(pc: u16, code: &[u8]) -> Vec<Event> {
    let mut trace = vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ];
    for (&address, &value) in [pc, pc.wrapping_add(1)].iter().zip(code) {
        trace.push(read(address, OP, value));
    }
    trace
}

fn complete(cpu: &Z80, report: Report, expected: Registers, cycles: u64) {
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles,
            exit: Exit::Budget
        }
    );
    // Whole state checks all unchanged registers and every changed/unchanged flag.
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(
        *cpu.latches(),
        Latches {
            ei_delay: false,
            parity_note_pending: false,
            ..latches()
        }
    );
}

#[derive(Clone, Copy)]
enum Pair {
    Bc,
    De,
    Hl,
    Sp,
    Ix,
    Iy,
}

// Destination selection is a literal test-case field, independent of opcode bits.
fn loaded(r: &mut Registers, pair: Pair) {
    match pair {
        Pair::Bc => {
            r.b = 0xD2;
            r.c = 0x35;
        }
        Pair::De => {
            r.d = 0xD2;
            r.e = 0x35;
        }
        Pair::Hl => {
            r.h = 0xD2;
            r.l = 0x35;
        }
        Pair::Sp => r.sp = 0xD235,
        Pair::Ix => r.ix = 0xD235,
        Pair::Iy => r.iy = 0xD235,
    }
}

#[test]
fn ld_dd_nn_low_operand_first_with_derived_pc_wrap() {
    // UM p. 99: BC/DE/HL/SP, 10 states, no flags. TM p. 45 agrees.
    for (opcode, pair) in [
        (0x01, Pair::Bc),
        (0x11, Pair::De),
        (0x21, Pair::Hl),
        (0x31, Pair::Sp),
    ] {
        for n in 0..64 {
            let before = Registers {
                pc: 0xFFFE,
                ..registers(n)
            };
            let mut expected = Registers {
                pc: 1,
                r: 0x80,
                ..before
            };
            loaded(&mut expected, pair);
            let mut cpu = Z80::new(before, latches());
            let mut bus = RecordingBus::new();
            bus.load(0xFFFE, &[opcode, 0x35, 0xD2]);
            let memory = bus.memory.clone();
            let mut trace = fetch(0xFFFE, &[opcode]);
            trace.extend([read(0xFFFF, ARG, 0x35), read(0, ARG, 0xD2)]);
            let report = cpu.step(&mut bus);
            complete(&cpu, report, expected, 10);
            assert_eq!(bus.memory, memory);
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn ld_ix_iy_nn_two_m1s_and_derived_pc_wrap() {
    // UM pp. 100–101: DD/FD 21, 14 states, no flags; TM p. 45.
    for (prefix, pair) in [(0xDD, Pair::Ix), (0xFD, Pair::Iy)] {
        for n in 0..64 {
            let before = Registers {
                pc: 0xFFFE,
                ..registers(n)
            };
            let mut expected = Registers {
                pc: 2,
                r: 0x81,
                ..before
            };
            loaded(&mut expected, pair);
            let mut cpu = Z80::new(before, latches());
            let mut bus = RecordingBus::new();
            bus.load(0xFFFE, &[prefix, 0x21, 0x35, 0xD2]);
            let memory = bus.memory.clone();
            let mut trace = fetch(0xFFFE, &[prefix, 0x21]);
            trace.extend([read(0, ARG, 0x35), read(1, ARG, 0xD2)]);
            let report = cpu.step(&mut bus);
            complete(&cpu, report, expected, 14);
            assert_eq!(bus.memory, memory);
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn ld_pairs_from_memory_including_ed_6b_with_derived_address_wrap() {
    // UM pp. 102–106; TM p. 45: low at nn, high at nn+1; no flags.
    // ED 6B is explicit on p. 103 despite compact Table 7's omission
    // (decision-14 rule 1 and existing PROVENANCE row).
    for (code, pair, arg_at, next_pc, next_r, cycles) in [
        (&[0x2A][..], Pair::Hl, 0x2001, 0x2003, 0x80, 16),
        (&[0xED, 0x4B][..], Pair::Bc, 0x2002, 0x2004, 0x81, 20),
        (&[0xED, 0x5B][..], Pair::De, 0x2002, 0x2004, 0x81, 20),
        (&[0xED, 0x6B][..], Pair::Hl, 0x2002, 0x2004, 0x81, 20),
        (&[0xED, 0x7B][..], Pair::Sp, 0x2002, 0x2004, 0x81, 20),
        (&[0xDD, 0x2A][..], Pair::Ix, 0x2002, 0x2004, 0x81, 20),
        (&[0xFD, 0x2A][..], Pair::Iy, 0x2002, 0x2004, 0x81, 20),
    ] {
        for (address, lo, hi, high_at) in [(0x4567, 0x67, 0x45, 0x4568), (0xFFFF, 0xFF, 0xFF, 0)] {
            for n in 0..64 {
                let before = registers(n);
                let mut expected = Registers {
                    pc: next_pc,
                    r: next_r,
                    ..before
                };
                loaded(&mut expected, pair);
                let mut cpu = Z80::new(before, latches());
                let mut bus = RecordingBus::new();
                bus.load(0x2000, code);
                bus.load(arg_at, &[lo, hi]);
                bus.load(address, &[0x35, 0xD2]);
                let memory = bus.memory.clone();
                let mut trace = fetch(0x2000, code);
                trace.extend([
                    read(arg_at, ARG, lo),
                    read(arg_at + 1, ARG, hi),
                    read(address, DATA, 0x35),
                    read(high_at, DATA, 0xD2),
                ]);
                let report = cpu.step(&mut bus);
                complete(&cpu, report, expected, cycles);
                assert_eq!(bus.memory, memory);
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn ld_memory_from_pairs_including_ed_63_with_derived_address_wrap() {
    // UM pp. 107–111; TM p. 45: low then high, no flags. ED 63's
    // explicit HL field on p. 108 prevails over compact Table 7's omission.
    for (code, low, high, arg_at, next_pc, next_r, cycles) in [
        (&[0x22][..], 0xE8, 0xB3, 0x2001, 0x2003, 0x80, 16),
        (&[0xED, 0x43][..], 0x4C, 0x2A, 0x2002, 0x2004, 0x81, 20),
        (&[0xED, 0x53][..], 0x16, 0x87, 0x2002, 0x2004, 0x81, 20),
        (&[0xED, 0x63][..], 0xE8, 0xB3, 0x2002, 0x2004, 0x81, 20),
        (&[0xED, 0x73][..], 0xAF, 0x9A, 0x2002, 0x2004, 0x81, 20),
        (&[0xDD, 0x22][..], 0x91, 0x80, 0x2002, 0x2004, 0x81, 20),
        (&[0xFD, 0x22][..], 0xA3, 0x92, 0x2002, 0x2004, 0x81, 20),
    ] {
        for (address, lo, hi, high_at) in [(0x4567, 0x67, 0x45, 0x4568), (0xFFFF, 0xFF, 0xFF, 0)] {
            for n in 0..64 {
                let before = registers(n);
                let expected = Registers {
                    pc: next_pc,
                    r: next_r,
                    ..before
                };
                let mut cpu = Z80::new(before, latches());
                let mut bus = RecordingBus::new();
                bus.load(0x2000, code);
                bus.load(arg_at, &[lo, hi]);
                let mut memory = bus.memory.clone();
                memory[usize::from(address)] = low;
                memory[usize::from(high_at)] = high;
                let mut trace = fetch(0x2000, code);
                trace.extend([
                    read(arg_at, ARG, lo),
                    read(arg_at + 1, ARG, hi),
                    write(address, DATA, low),
                    write(high_at, DATA, high),
                ]);
                let report = cpu.step(&mut bus);
                complete(&cpu, report, expected, cycles);
                assert_eq!(bus.memory, memory);
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn ld_sp_hl_ix_iy_changes_only_sp_and_fetch_bookkeeping() {
    // UM pp. 112–114; TM p. 45: 6/10/10 states, no flags or stack access.
    for (code, sp, next_pc, next_r, cycles) in [
        (&[0xF9][..], 0xB3E8, 0x2001, 0x80, 6),
        (&[0xDD, 0xF9][..], 0x8091, 0x2002, 0x81, 10),
        (&[0xFD, 0xF9][..], 0x92A3, 0x2002, 0x81, 10),
    ] {
        for n in 0..64 {
            let before = registers(n);
            let expected = Registers {
                sp,
                pc: next_pc,
                r: next_r,
                ..before
            };
            let mut cpu = Z80::new(before, latches());
            let mut bus = RecordingBus::new();
            bus.load(0x2000, code);
            let memory = bus.memory.clone();
            let report = cpu.step(&mut bus);
            complete(&cpu, report, expected, cycles);
            assert_eq!(bus.memory, memory);
            assert_eq!(bus.take_log(), fetch(0x2000, code));
        }
    }
}

#[test]
fn push_pairs_high_then_low_with_derived_sp_wrap() {
    // UM pp. 115–118 explicitly order decrement/high, decrement/low;
    // TM p. 45: 11/15 states, no flags. Entry encodings pin the existing
    // compact PUSH misprint resolution (PROVENANCE §1).
    for (code, low, high, next_pc, next_r, cycles) in [
        (&[0xC5][..], 0x4C, 0x2A, 0x2001, 0x80, 11),
        (&[0xD5][..], 0x16, 0x87, 0x2001, 0x80, 11),
        (&[0xE5][..], 0xE8, 0xB3, 0x2001, 0x80, 11),
        (&[0xDD, 0xE5][..], 0x91, 0x80, 0x2002, 0x81, 15),
        (&[0xFD, 0xE5][..], 0xA3, 0x92, 0x2002, 0x81, 15),
    ] {
        for (sp, high_at, low_at) in [
            (0x9AAF, 0x9AAE, 0x9AAD),
            (0, 0xFFFF, 0xFFFE),
            (1, 0, 0xFFFF),
        ] {
            for n in 0..64 {
                let before = Registers { sp, ..registers(n) };
                let expected = Registers {
                    sp: low_at,
                    pc: next_pc,
                    r: next_r,
                    ..before
                };
                let mut cpu = Z80::new(before, latches());
                let mut bus = RecordingBus::new();
                bus.load(0x2000, code);
                let mut memory = bus.memory.clone();
                memory[usize::from(high_at)] = high;
                memory[usize::from(low_at)] = low;
                let mut trace = fetch(0x2000, code);
                trace.extend([write(high_at, STACK, high), write(low_at, STACK, low)]);
                let report = cpu.step(&mut bus);
                complete(&cpu, report, expected, cycles);
                assert_eq!(bus.memory, memory);
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn pop_pairs_low_then_high_including_iy_operation_line_resolution() {
    // UM pp. 119–122 descriptions: low, increment, high, increment;
    // TM p. 45: 10/14 states. UM p. 122's operation line prints SP – X1;
    // its description/example and TM explicitly use SP+1. Both re-read at
    // 200 dpi with enlarged crops; follow the agreeing description/table.
    for (code, pair, next_pc, next_r, cycles) in [
        (&[0xC1][..], Pair::Bc, 0x2001, 0x80, 10),
        (&[0xD1][..], Pair::De, 0x2001, 0x80, 10),
        (&[0xE1][..], Pair::Hl, 0x2001, 0x80, 10),
        (&[0xDD, 0xE1][..], Pair::Ix, 0x2002, 0x81, 14),
        (&[0xFD, 0xE1][..], Pair::Iy, 0x2002, 0x81, 14),
    ] {
        for (sp, high_at, next_sp) in [
            (0x9AAD, 0x9AAE, 0x9AAF),
            (0xFFFF, 0, 1),
            (0xFFFE, 0xFFFF, 0),
        ] {
            for n in 0..64 {
                let before = Registers { sp, ..registers(n) };
                let mut expected = Registers {
                    sp: next_sp,
                    pc: next_pc,
                    r: next_r,
                    ..before
                };
                loaded(&mut expected, pair);
                let mut cpu = Z80::new(before, latches());
                let mut bus = RecordingBus::new();
                bus.load(0x2000, code);
                bus.load(sp, &[0x35, 0xD2]);
                let memory = bus.memory.clone();
                let mut trace = fetch(0x2000, code);
                trace.extend([read(sp, STACK, 0x35), read(high_at, STACK, 0xD2)]);
                let report = cpu.step(&mut bus);
                complete(&cpu, report, expected, cycles);
                assert_eq!(bus.memory, memory);
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn pop_and_push_af_pin_six_flags_and_zero_unused_bits_library_choice() {
    // UM POP qq pp. 119–120, PUSH qq pp. 115–116; TM p. 45 names AF.
    // UM Tables 21/22 pp. 65–66 place the six flags and call bits 5/3 unused.
    // F is the low byte, A the high byte. Decision-14 rule 5 fixes F bits
    // 5/3 at zero; this UNMASKED witness is a library contract, not silicon.
    // Try every input image and its complement as the initial flags, so
    // each of the six flags is witnessed both loaded and cleared.
    for image in 0u8..=255 {
        let expected_flags = Flags {
            sign: image & 0x80 != 0,
            zero: image & 0x40 != 0,
            half_carry: image & 0x10 != 0,
            parity_overflow: image & 4 != 0,
            subtract: image & 2 != 0,
            carry: image & 1 != 0,
        };
        let before = Registers {
            flags: Flags {
                sign: !expected_flags.sign,
                zero: !expected_flags.zero,
                half_carry: !expected_flags.half_carry,
                parity_overflow: !expected_flags.parity_overflow,
                subtract: !expected_flags.subtract,
                carry: !expected_flags.carry,
            },
            sp: 0xFFFF,
            ..registers(0)
        };
        let mut expected = Registers {
            a: 0xD2,
            flags: expected_flags,
            sp: 1,
            pc: 0x2001,
            r: 0x80,
            ..before
        };
        let mut cpu = Z80::new(before, latches());
        let mut bus = RecordingBus::new();
        bus.load(0x2000, &[0xF1, 0xF5]);
        bus.load(0xFFFF, &[image, 0xD2]);
        let mut memory = bus.memory.clone();
        let mut trace = fetch(0x2000, &[0xF1]);
        trace.extend([read(0xFFFF, STACK, image), read(0, STACK, 0xD2)]);
        let report = cpu.step(&mut bus);
        complete(&cpu, report, expected, 10);
        assert_eq!(bus.memory, memory);
        assert_eq!(bus.take_log(), trace);
        assert_eq!(cpu.registers().flags.to_byte(), image & 0xD7);

        expected.pc = 0x2002;
        expected.r = 0x81;
        expected.sp = 0xFFFF;
        memory[0xFFFF] = image & 0xD7;
        let mut trace = fetch(0x2001, &[0xF5]);
        trace.extend([write(0, STACK, 0xD2), write(0xFFFF, STACK, image & 0xD7)]);
        let report = cpu.step(&mut bus);
        complete(&cpu, report, expected, 11);
        assert_eq!(bus.memory, memory);
        assert_eq!(bus.take_log(), trace);
    }
}
