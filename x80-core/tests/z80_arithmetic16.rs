//! 16-bit arithmetic witnesses: decisions 13 rules 2–5, 14 rules 5–6,
//! and 15 rules 1/5–6. Primary: Zilog UM008011-0816 rev. 11 (August 2016),
//! pp. 188–203 (PDF = printed + 14). Second transcription: 03-0029-01
//! (1977), Table 7.0-6 p. 49 (PDF 55). Hashes match PROVENANCE.md §1;
//! all cited instruction pages and the second table were read as images.
//! Existing H-definedness and indexed ADD example disagreements were also
//! checked in enlarged 200 dpi crops. No new disagreement is introduced.
//!
//! MEASURED on base b11628b before implementation: all 12 tests failed with
//! Unsupported and zero steps/cycles; afterwards all 5,632 cases passed.
//! Expectations are literal arithmetic/flag transcriptions, independent of ALU
//! helpers and opcode tables. All eight flag bits are compared (decision-20 §4): H is defined by the
//! primary even though TM prints unknown (PROVENANCE §1; decision-14 rule 6).
//! H Y/X: Young v0.91 §8.6 p. 31 / Rak pair-arithmetic groups.
//! Every case checks the entire register/latch state, unchanged memory, whole
//! bus trace, PC, R and Report. PC/pair wraps are derived from 16-bit widths
//! (UM CPU Register / Registers pp. 2–3); poll/fetch/R mapping is decision-15.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

// Independent transcription of the documented F positions (UM pp. 65–66).
fn flags(byte: u8) -> Flags {
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

fn flag_images() -> impl Iterator<Item = u8> {
    0..=0xFF
}

fn sample(f: u8) -> Registers {
    Registers {
        a: 0x69,
        flags: flags(f),
        b: 0x00,
        c: 0x01,
        d: 0x70,
        e: 0x01,
        h: 0x8F,
        l: 0xFF,
        a_alternate: 0xA1,
        flags_alternate: flags(0x95),
        b_alternate: 0xB2,
        c_alternate: 0xC3,
        d_alternate: 0xD4,
        e_alternate: 0xE5,
        h_alternate: 0xF6,
        l_alternate: 0x07,
        ix: 0x2468,
        iy: 0x3579,
        sp: 0x1000,
        pc: 0xFFFF,
        i: 0xF8,
        r: 0xFF,
    }
}

fn latches() -> Latches {
    Latches {
        wz: 0,
        iff1: true,
        iff2: false,
        interrupt_mode: InterruptMode::Mode2,
        ei_delay: true,
        parity_note_pending: true,
        nmi_asserted: true,
        halted: false,
    }
}

#[derive(Clone, Copy, Debug)]
enum Pair {
    Bc,
    De,
    Hl,
    Sp,
    Ix,
    Iy,
}

// Pair is supplied by each literal test row, never decoded from an opcode.
// No implementation pair helpers are used to form expected registers.
fn put(r: &mut Registers, pair: Pair, value: u16) {
    let high = (value >> 8) as u8;
    let low = value as u8;
    match pair {
        Pair::Bc => {
            r.b = high;
            r.c = low;
        }
        Pair::De => {
            r.d = high;
            r.e = low;
        }
        Pair::Hl => {
            r.h = high;
            r.l = low;
        }
        Pair::Sp => r.sp = value,
        Pair::Ix => r.ix = value,
        Pair::Iy => r.iy = value,
    }
}

fn witness(code: &[u8], before: Registers, mut expected: Registers, f: u8, cycles: u64) {
    if let [0xDD | 0xFD, 0x09 | 0x19 | 0x29 | 0x39] = code {
        // For a fixed destination, distinct 16-bit sources give distinct sums
        // modulo 10000h. Guard every indexed ADD fixture against a wrong pair
        // producing the literal expected result (UM pp. 194/196; TM p. 49).
        let pairs = [
            u16::from_be_bytes([before.b, before.c]),
            u16::from_be_bytes([before.d, before.e]),
            u16::from_be_bytes([before.h, before.l]),
            before.sp,
            before.ix,
            before.iy,
        ];
        for (index, pair) in pairs.iter().enumerate() {
            for other in pairs.iter().skip(index + 1) {
                assert_ne!(pair, other, "{code:02X?}: source pairs must differ");
            }
        }
    }
    let mut cpu = Z80::new(before, latches());
    let mut bus = RecordingBus::new();
    bus.load(0xFFFF, code);
    // Sentinels would expose an accidental operand/data/stack read or write.
    bus.load(0x1000, &[0xA5, 0x5A]);
    bus.load(0x8FFF, &[0x96, 0x69]);
    let memory = bus.memory.clone();
    let (pc, r) = match code.len() {
        1 => (0x0000, 0x80),
        2 => (0x0001, 0x81),
        _ => panic!("only one/two-byte register instructions in this group"),
    };
    expected.pc = pc;
    expected.r = r;
    // Decision-20 §4 H: Young §8.6 p. 31 / Rak pair arithmetic groups.
    // Pair INC/DEC preserve every bit; arithmetic uses high result byte.
    let arithmetic = code.last().is_some_and(|op| op & 7 == 1 || op & 7 == 2);
    let high = match code[0] {
        0xDD => (expected.ix >> 8) as u8,
        0xFD => (expected.iy >> 8) as u8,
        _ => expected.h,
    };
    let f = if arithmetic {
        (f & 0xD7) | (high & 0x28)
    } else {
        f
    };
    expected.flags = flags(f);
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles,
            exit: Exit::Budget
        },
        "{code:02X?}"
    );
    assert_eq!(*cpu.registers(), expected, "{code:02X?}");
    assert_eq!(cpu.registers().flags.to_byte(), f);
    assert_eq!(
        *cpu.latches(),
        Latches {
            // L exact word: decision-20 §5; literal edge witnesses in z80_wz.
            wz: if arithmetic {
                match code[0] {
                    0xDD => before.ix,
                    0xFD => before.iy,
                    _ => u16::from_be_bytes([before.h, before.l]),
                }
                .wrapping_add(1)
            } else {
                0
            },
            ei_delay: false,
            parity_note_pending: false,
            nmi_asserted: false,
            ..latches()
        }
    );
    assert_eq!(bus.memory, memory);
    let mut trace = vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ];
    // Literal addresses pin PC wrapping and all reads as OpcodeFetch, including
    // the ED/DD/FD prefix and its key. No operand/data/stack/I/O call is allowed.
    for (&address, &value) in [0xFFFF, 0x0000].iter().zip(code) {
        trace.push(Event::Read {
            address,
            kind: MemoryAccess::OpcodeFetch,
            value,
        });
    }
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn add_hl_ss_all_fields_and_all_incoming_flags() {
    // UM pp. 188–189; TM Table 7.0-6 p. 49. S/Z/P-V preserved even when
    // inconsistent with the result, N reset, incoming C ignored; H/C replaced.
    // HL+HL samples the old HL. Wrapped sums are derived at 16 bits.
    for (opcode, result, hc) in [
        (0x09, 0x9000, 0x10), // 8FFF + BC(0001): H, no C.
        (0x19, 0x0000, 0x11), // 8FFF + DE(7001): H and C.
        (0x29, 0x1FFE, 0x11), // 8FFF + old HL(8FFF).
        (0x39, 0x9FFF, 0x00), // 8FFF + SP(1000): neither.
    ] {
        for f in flag_images() {
            let before = sample(f);
            let mut expected = before;
            put(&mut expected, Pair::Hl, result);
            witness(&[opcode], before, expected, (f & 0xC4) | hc, 11);
        }
    }
}

fn indexed_add(prefix: u8, target: Pair) {
    // UM pp. 194–197; TM p. 49. Field 10 is IX for pp and IY for rr;
    // the unselected index and HL contain distinct sentinels and stay intact.
    for (opcode, result, hc) in [
        (0x09, 0x9000, 0x10),
        (0x19, 0x0000, 0x11),
        (0x29, 0x1FFE, 0x11),
        (0x39, 0x9FFF, 0x00),
    ] {
        for f in flag_images() {
            let mut before = sample(f);
            put(&mut before, Pair::Hl, 0x4A6B);
            put(&mut before, target, 0x8FFF);
            let mut expected = before;
            put(&mut expected, target, result);
            witness(&[prefix, opcode], before, expected, (f & 0xC4) | hc, 15);
        }
    }
}

#[test]
fn add_ix_pp_all_fields_and_all_incoming_flags() {
    indexed_add(0xDD, Pair::Ix);
}

#[test]
fn add_iy_rr_all_fields_and_all_incoming_flags() {
    indexed_add(0xFD, Pair::Iy);
}

#[test]
fn adc_hl_ss_all_fields_and_all_incoming_flags() {
    // UM pp. 190–191; TM p. 49. Literal pairs (result,F) for C=0/C=1.
    // P/V is signed overflow, not parity; H at bit 11; C at bit 15.
    for (opcode, without_carry, with_carry) in [
        (0x4A, (0x7FFF, 0x00), (0x8000, 0x94)), // BC=0000
        (0x5A, (0xFFFF, 0x80), (0x0000, 0x51)), // DE=8000
        (0x6A, (0xFFFE, 0x94), (0xFFFF, 0x94)), // old HL=7FFF
        (0x7A, (0x8000, 0x94), (0x8001, 0x94)), // SP=0001
    ] {
        for f in flag_images() {
            let mut before = sample(f);
            put(&mut before, Pair::Bc, 0);
            put(&mut before, Pair::De, 0x8000);
            put(&mut before, Pair::Hl, 0x7FFF);
            put(&mut before, Pair::Sp, 1);
            let (result, output_f) = if f & 1 == 0 {
                without_carry
            } else {
                with_carry
            };
            let mut expected = before;
            put(&mut expected, Pair::Hl, result);
            witness(&[0xED, opcode], before, expected, output_f, 15);
        }
    }
}

#[test]
fn sbc_hl_ss_all_fields_and_all_incoming_flags() {
    // UM pp. 192–193; TM p. 49. C means borrow and N is always set.
    // H borrows from bit 12; self subtraction still consumes incoming C.
    for (opcode, without_borrow, with_borrow) in [
        (0x42, (0x8000, 0x82), (0x7FFF, 0x16)), // BC=0000
        (0x52, (0x8001, 0x93), (0x8000, 0x93)), // DE=FFFF
        (0x62, (0x0000, 0x42), (0xFFFF, 0x93)), // old HL=8000
        (0x72, (0x7FFF, 0x16), (0x7FFE, 0x16)), // SP=0001
    ] {
        for f in flag_images() {
            let mut before = sample(f);
            put(&mut before, Pair::Bc, 0);
            put(&mut before, Pair::De, 0xFFFF);
            put(&mut before, Pair::Hl, 0x8000);
            put(&mut before, Pair::Sp, 1);
            let (result, output_f) = if f & 1 == 0 {
                without_borrow
            } else {
                with_borrow
            };
            let mut expected = before;
            put(&mut expected, Pair::Hl, result);
            witness(&[0xED, opcode], before, expected, output_f, 15);
        }
    }
}

#[test]
fn adc_and_sbc_literal_flag_boundaries_and_manual_examples() {
    // UM ADC pp. 190–191 and SBC pp. 192–193; TM p. 49. Include H/C
    // independently, zero versus low-byte zero, both overflow directions,
    // carry/borrow in the second stage and the primary's printed examples.
    // Values wrapping through 10000h/-1 are derived 16-bit results.
    for (opcode, left, right, carry, result, output_f) in [
        (0x4A, 0xFFFF, 0x0000, true, 0x0000, 0x51),
        (0x4A, 0xFFFF, 0x0001, false, 0x0000, 0x51),
        (0x4A, 0x8000, 0x8000, false, 0x0000, 0x45),
        (0x4A, 0x8000, 0xFFFF, true, 0x8000, 0x91),
        (0x4A, 0x0FFF, 0x0000, true, 0x1000, 0x10),
        (0x4A, 0x00FF, 0x0000, true, 0x0100, 0x00),
        (0x4A, 0x5437, 0x2222, true, 0x765A, 0x00), // p. 191
        (0x42, 0x0000, 0x0000, true, 0xFFFF, 0x93),
        (0x42, 0x0000, 0x0001, false, 0xFFFF, 0x93),
        (0x42, 0x7FFF, 0xFFFF, false, 0x8000, 0x87),
        (0x42, 0x0000, 0xFFFF, true, 0x0000, 0x53),
        (0x42, 0x1000, 0x0000, true, 0x0FFF, 0x12),
        (0x42, 0x0100, 0x0000, true, 0x00FF, 0x02),
        (0x52, 0x9999, 0x1111, true, 0x8887, 0x82), // p. 193
    ] {
        for f in flag_images().filter(|f| (f & 1 != 0) == carry) {
            let mut before = sample(f);
            put(&mut before, Pair::Hl, left);
            // The printed SBC example uses DE; all other rows use BC.
            put(
                &mut before,
                if opcode == 0x52 { Pair::De } else { Pair::Bc },
                right,
            );
            let mut expected = before;
            put(&mut expected, Pair::Hl, result);
            witness(&[0xED, opcode], before, expected, output_f, 15);
        }
    }
}

#[test]
fn add_h_definedness_disagreement_and_manual_examples() {
    // Existing PROVENANCE rows: UM pp. 188/194/196 define H, TM p. 49
    // prints unknown. Check both setting and clearing it from every input.
    // UM ADD HL example p. 189 is 4242+1111=5353.
    for (code, target) in [
        (&[0x09][..], Pair::Hl),
        (&[0xDD, 0x09][..], Pair::Ix),
        (&[0xFD, 0x09][..], Pair::Iy),
    ] {
        for (left, right, result, hc) in [
            (0x0FFF, 0x0001, 0x1000, 0x10),
            (0x9000, 0x7000, 0x0000, 0x01),
            (0x4242, 0x1111, 0x5353, 0x00),
        ] {
            for f in flag_images() {
                let mut before = sample(f);
                put(&mut before, target, left);
                put(&mut before, Pair::Bc, right);
                let mut expected = before;
                put(&mut expected, target, result);
                witness(
                    code,
                    before,
                    expected,
                    (f & 0xC4) | hc,
                    if code.len() == 1 { 11 } else { 15 },
                );
            }
        }
    }
}

#[test]
fn indexed_add_example_misprint_follows_operation_lines() {
    // Existing PROVENANCE row: UM pp. 195/197 print 333h+5555h=8888h.
    // Operation lines pp. 194/196 and TM p. 49 prevail: literal 0333h
    // gives 5888h; corrected 3333h gives 8888h. No new source resolution.
    for (prefix, target) in [(0xDD, Pair::Ix), (0xFD, Pair::Iy)] {
        for (left, result) in [(0x0333, 0x5888), (0x3333, 0x8888)] {
            for f in flag_images() {
                let mut before = sample(f);
                put(&mut before, target, left);
                put(&mut before, Pair::Bc, 0x5555);
                let mut expected = before;
                put(&mut expected, target, result);
                witness(&[prefix, 0x09], before, expected, f & 0xC4, 15);
            }
        }
    }
}

fn unchanged_flags(code: &[u8], pair: Pair, cases: &[(u16, u16)], cycles: u64) {
    for &(before_value, after_value) in cases {
        for f in flag_images() {
            let mut before = sample(f);
            put(&mut before, pair, before_value);
            let mut expected = before;
            put(&mut expected, pair, after_value);
            witness(code, before, expected, f, cycles);
        }
    }
}

#[test]
fn inc_ss_all_fields_preserve_every_flag_and_wrap() {
    // UM p. 198; TM p. 49. Includes byte carry, sign boundary and derived
    // 16-bit wrap; all eight flag bits stay unchanged, including H/N/C.
    for (opcode, pair) in [
        (0x03, Pair::Bc),
        (0x13, Pair::De),
        (0x23, Pair::Hl),
        (0x33, Pair::Sp),
    ] {
        unchanged_flags(
            &[opcode],
            pair,
            &[
                (0x1000, 0x1001),
                (0x00FF, 0x0100),
                (0x7FFF, 0x8000),
                (0xFFFF, 0x0000),
            ],
            6,
        );
    }
}

#[test]
fn dec_ss_all_fields_preserve_every_flag_and_wrap() {
    // UM p. 201; TM p. 49. DEC BC is 0B, not compact Table 13 p. 52's
    // DB misprint (existing PROVENANCE row). Wrap from zero is derived.
    for (opcode, pair) in [
        (0x0B, Pair::Bc),
        (0x1B, Pair::De),
        (0x2B, Pair::Hl),
        (0x3B, Pair::Sp),
    ] {
        unchanged_flags(
            &[opcode],
            pair,
            &[
                (0x1001, 0x1000),
                (0x0100, 0x00FF),
                (0x8000, 0x7FFF),
                (0x0000, 0xFFFF),
            ],
            6,
        );
    }
}

#[test]
fn inc_ix_iy_preserve_every_flag_and_wrap() {
    // UM pp. 199–200; TM p. 49. Printed examples and derived wraps.
    for (prefix, pair, initial, result) in [
        (0xDD, Pair::Ix, 0x3300, 0x3301),
        (0xFD, Pair::Iy, 0x2977, 0x2978),
    ] {
        unchanged_flags(
            &[prefix, 0x23],
            pair,
            &[
                (initial, result),
                (0x00FF, 0x0100),
                (0x7FFF, 0x8000),
                (0xFFFF, 0x0000),
            ],
            10,
        );
    }
}

#[test]
fn dec_ix_iy_preserve_every_flag_and_wrap() {
    // UM pp. 202–203; TM p. 49. Printed examples and derived wraps.
    for (prefix, pair, initial, result) in [
        (0xDD, Pair::Ix, 0x2006, 0x2005),
        (0xFD, Pair::Iy, 0x7649, 0x7648),
    ] {
        unchanged_flags(
            &[prefix, 0x2B],
            pair,
            &[
                (initial, result),
                (0x0100, 0x00FF),
                (0x8000, 0x7FFF),
                (0x0000, 0xFFFF),
            ],
            10,
        );
    }
}
