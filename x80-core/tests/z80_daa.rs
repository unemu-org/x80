//! DAA witnesses: decisions 13–15. Zilog UM008011-0816 rev. 11
//! (August 2016), pp. 173–174 (PDF +14); 03-0029-01 (1977),
//! Table 7.0-5 p. 48 (PDF 54), section 6 p. 39 (PDF 45).
//! Read as rendered 150 dpi images. All execution tests fail before landing with
//! Unsupported for 27h and zero retired steps/cycles.
//!
//! The literal table covers 684 of 2048 A/N/H/C inputs. A separate
//! witness pins a handful of the other 1364 inputs as a library extension
//! accepted by the maintainer (2026-11-13): the primary documents only the table cases.
//! H-after is a DERIVED READING: use the adjustment's bit-4 carry/borrow
//! (TM section 6's H definition). UM p. 174 refers to p. 173, which has
//! no H-after column. This is a library rule for table cases too, not an
//! explicit DAA statement. See PROVENANCE section 1.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

// LITERAL UM p. 173 columns: operation (N), C-before, upper digit range,
// H-before, lower digit range, byte added, C-after. Retain both printed
// anomalies (9–0 and 6–7). No production helpers supply these expectations.
type Row = (bool, bool, (u8, u8), bool, (u8, u8), u8, bool);
const MANUAL_TABLE: [Row; 13] = [
    (false, false, (0x9, 0x0), false, (0x0, 0x9), 0x00, false),
    (false, false, (0x0, 0x8), false, (0xA, 0xF), 0x06, false),
    (false, false, (0x0, 0x9), true, (0x0, 0x3), 0x06, false),
    (false, false, (0xA, 0xF), false, (0x0, 0x9), 0x60, true),
    (false, false, (0x9, 0xF), false, (0xA, 0xF), 0x66, true),
    (false, false, (0xA, 0xF), true, (0x0, 0x3), 0x66, true),
    (false, true, (0x0, 0x2), false, (0x0, 0x9), 0x60, true),
    (false, true, (0x0, 0x2), false, (0xA, 0xF), 0x66, true),
    (false, true, (0x0, 0x3), true, (0x0, 0x3), 0x66, true),
    (true, false, (0x0, 0x9), false, (0x0, 0x9), 0x00, false),
    (true, false, (0x0, 0x8), true, (0x6, 0xF), 0xFA, false),
    (true, true, (0x7, 0xF), false, (0x0, 0x9), 0xA0, true),
    (true, true, (0x6, 0x7), true, (0x6, 0xF), 0x9A, true),
];

fn table_reading(a: u8, n: bool, h: bool, c: bool) -> Option<(u8, bool)> {
    let mut found = None;
    for &(subtract, carry, upper, half, lower, added, carry_after) in &MANUAL_TABLE {
        // The printed first range is descending 9–0. Retain its endpoints
        // above, interpreting the inclusive range in either direction.
        // The final row stays literally 6–7; do not widen it to 6–F.
        if (n, h, c) == (subtract, half, carry)
            && (upper.0.min(upper.1)..=upper.0.max(upper.1)).contains(&(a / 16))
            && (lower.0..=lower.1).contains(&(a % 16))
        {
            assert!(found.is_none(), "overlapping manual rows");
            found = Some((added, carry_after));
        }
    }
    found
}

fn expected(a: u8, n: bool, added: u8, carry: bool) -> (u8, u8) {
    // Literal table gives a BYTE ADDED even for subtraction (FA/A0/9A).
    let result = ((u16::from(a) + u16::from(added)) % 256) as u8;
    // DERIVED H-after reading: addition carries if the units sum >=16;
    // subtraction borrows if units < magnitude of the low correction.
    // This arithmetic test deliberately differs from production's XOR.
    let half = if n {
        let magnitude = (256 - u16::from(added)) % 256;
        u16::from(a % 16) < magnitude % 16
    } else {
        u16::from(a % 16) + u16::from(added % 16) >= 16
    };
    let mut even = true;
    let mut bits = result;
    for _ in 0..8 {
        if !bits.is_multiple_of(2) {
            even = !even;
        }
        bits /= 2;
    }
    let f = if result >= 128 { 0x80 } else { 0 }
        | if result == 0 { 0x40 } else { 0 }
        | if half { 0x10 } else { 0 }
        | if even { 0x04 } else { 0 }
        | if n { 0x02 } else { 0 }
        | u8::from(carry);
    (result, f)
}

fn check(a: u8, before_f: u8, result: u8, result_f: u8) {
    let mut cpu = Z80::new(
        Registers {
            a,
            flags: Flags::from_byte(before_f),
            b: 0x12,
            c: 0x34,
            d: 0x56,
            e: 0x78,
            h: 0x9A,
            l: 0xBC,
            a_alternate: 0xDE,
            flags_alternate: Flags::from_byte(0x95),
            b_alternate: 0xF0,
            c_alternate: 0x13,
            d_alternate: 0x25,
            e_alternate: 0x37,
            h_alternate: 0x49,
            l_alternate: 0x5B,
            ix: 0x6789,
            iy: 0xABCD,
            sp: 0xDEF0,
            pc: 0xFFFF,
            i: 0xF8,
            r: 0xFF,
        },
        Latches {
            q: false,
            wz: 0,
            iff1: true,
            iff2: false,
            interrupt_mode: InterruptMode::Mode1,
            ei_delay: true,
            halted: false,
            nmi_asserted: true,
            parity_note_pending: true,
        },
    );
    // Decision-20 §4 H: Young §4.7 p. 18 / §8.5 p. 30; Rak .daa.
    // Only Y/X change; retain the accepted correction and H expectations.
    let result_f = result_f | (result & 0x28);
    let mut after = cpu.clone();
    after.registers_mut().a = result;
    after.registers_mut().flags = Flags::from_byte(result_f);
    // DERIVED wrap from 16-bit PC (UM p. 3); R accounting: rule 6.
    after.registers_mut().pc = 0;
    after.registers_mut().r = 0x80;
    after.latches_mut().q = true;
    after.latches_mut().ei_delay = false;
    after.latches_mut().nmi_asserted = false;
    after.latches_mut().parity_note_pending = false;
    let mut bus = RecordingBus::new();
    bus.load(0xFFFF, &[0x27]);
    let memory = bus.memory.clone();
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert_eq!(cpu, after, "A={a:02X}, F={before_f:02X}");
    assert_eq!(cpu.registers().flags.to_byte(), result_f);

    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            Event::Read {
                address: 0xFFFF,
                kind: MemoryAccess::OpcodeFetch,
                value: 0x27
            },
        ]
    );
}

#[test]
fn exhaustive_input_domain_against_literal_manual_rows() {
    let mut table_cases = 0;
    let mut extension_cases = 0;
    for a in 0..=255 {
        for n in [false, true] {
            for h in [false, true] {
                for c in [false, true] {
                    let Some((added, carry)) = table_reading(a, n, h, c) else {
                        extension_cases += 1;
                        continue; // Never silently fill a gap in the primary's table.
                    };
                    table_cases += 1;
                    let (result, flags) = expected(a, n, added, carry);
                    // Old S/Z/P-V are never inputs: use both uniform values.
                    for old_szpv in [0x00, 0xC4] {
                        let before =
                            old_szpv | (u8::from(n) * 2) | (u8::from(h) * 16) | u8::from(c);
                        check(a, before, result, flags);
                    }
                }
            }
        }
    }
    assert_eq!((table_cases, extension_cases), (684, 1364));
}

#[test]
fn literal_table_anchors_and_manual_example() {
    // One literal result per row, plus UM p. 174's 3C -> 42 example.
    // H uses the labelled derived reading. Documented flags are literal; decision-20 §4 adds adjusted-A Y/X.
    for (a, before, result, flags) in [
        (0x00, 0x00, 0x00, 0x44),
        (0x0A, 0x00, 0x10, 0x10),
        (0x10, 0x10, 0x16, 0x00),
        (0xA0, 0x00, 0x00, 0x45),
        (0x9A, 0x00, 0x00, 0x55),
        (0xA0, 0x10, 0x06, 0x05),
        (0x00, 0x01, 0x60, 0x05),
        (0x0A, 0x01, 0x70, 0x11),
        (0x00, 0x11, 0x66, 0x05),
        (0x00, 0x02, 0x00, 0x46),
        (0x06, 0x12, 0x00, 0x46),
        (0x70, 0x03, 0x10, 0x03),
        (0x66, 0x13, 0x00, 0x47),
        (0x3C, 0x00, 0x42, 0x14),
    ] {
        assert!(table_reading(a, before & 2 != 0, before & 16 != 0, before & 1 != 0).is_some());
        check(a, before, result, flags);
    }
}

#[test]
fn library_extension_outside_manual_table() {
    // LIBRARY EXTENSION (maintainer-accepted 2026-11-13; undocumented by the sources): choose 06h if input low digit >9 or H;
    // choose 60h if input A>99h or C. Add when N=0, subtract when N=1.
    // C reports selection of 60h; H is adjustment bit-4 carry/borrow.
    // Literal expectations below follow this rule as written, not silicon.
    // E.g. N=1,A=9A,H=C=0 selects 66 and yields 34,C=1; A=00,H=1
    // subtracts 06 to FA with a low-digit borrow (H=1).
    for (a, before, result, flags) in [
        (0x04, 0x10, 0x0A, 0x04),
        (0x0A, 0x10, 0x10, 0x10),
        (0x40, 0x01, 0xA0, 0x85),
        (0x00, 0x12, 0xFA, 0x96),
        (0x0A, 0x02, 0x04, 0x02),
        (0x9A, 0x02, 0x34, 0x03),
        (0xA0, 0x02, 0x40, 0x03),
        (0x80, 0x13, 0x1A, 0x13),
        (0xFF, 0x13, 0x99, 0x87),
        (0x00, 0x03, 0xA0, 0x87),
    ] {
        assert!(table_reading(a, before & 2 != 0, before & 16 != 0, before & 1 != 0).is_none());
        for old_szpv in [0x00, 0xC4] {
            check(a, before | old_szpv, result, flags);
        }
    }
}
