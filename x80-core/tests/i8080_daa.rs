//! DAA witnesses (plan §2.2; decision-5 items 3/8; decision-6).
//! Sources: Intel 98-153B (September 1975) and 9800301-04 (May 1981),
//! pinned in PROVENANCE.md §1. Cited pages were checked as images.
//! Expectations use literals or the independent test-side transcription below.

mod support;

use support::{sample_latches, sample_registers, settled_latches, Event, RecordingBus};
use x80_core::i8080::{Flags, Registers, I8080};
use x80_core::{Exit, MemoryAccess, Report};

fn flags(sign: bool, zero: bool, auxiliary_carry: bool, parity: bool, carry: bool) -> Flags {
    Flags {
        sign,
        zero,
        auxiliary_carry,
        parity,
        carry,
    }
}

// 98-153B Ch. 4 DAA p. 4-8 (PDF 52); 9800301-04 Ch. 3 DAA p. 3-19
// (PDF 73): adjust the low digit first, propagate its carry, then adjust high.
// AC is the low digit's carry, not the high adjustment's: 98-153B Ch. 4
// "Auxiliary Carry" p. 4-3 (PDF 47); 9800301-04 Ch. 1 p. 1-11 (PDF 21)
// and Ch. 3 p. 3-19's 9BH -> A1H -> 01H example (AC remains 1).
fn manual_two_step_rule(a: u8, cy: bool, ac: bool) -> (u8, Flags) {
    let mut low = u16::from(a % 16);
    let mut high = u16::from(a / 16);
    if low > 9 || ac {
        low += 6;
    }
    let auxiliary_carry = low >= 16;
    high += low / 16;
    low %= 16;
    // Maintainer decision (2026-11-12): read 98-153B Ch. 4 "Carry"
    // p. 4-2 (PDF 46) per step of DAA (p. 4-8, PDF 52). Step 1's
    // carry out of bit 7 sets CY before step 2 tests it. Both this and
    // CY retention are derived, not stated; falsifiable by x80-ecy.17.
    // 9800301-04 Ch. 6 p. 6-13 (PDF 179) requires the decimal result
    // and carry for higher bytes; pp. 6-12 to 6-14 require retained CY.
    let step_1_carry = high >= 16;
    high %= 16;
    if high > 9 || cy || step_1_carry {
        high += 6;
    }
    let step_2_carry = high >= 16;
    let carry = cy || step_1_carry || step_2_carry;
    let value = ((high % 16) * 16 + low) as u8;
    // S/Z/P: 98-153B Ch. 4 "Condition Flags" p. 4-2 (PDF 46).
    // Count bits explicitly, independently of the crate's result helper.
    let ones = (0..8).filter(|bit| value & (1 << bit) != 0).count();
    (
        value,
        flags(
            value >= 128,
            value == 0,
            auxiliary_carry,
            ones.is_multiple_of(2),
            carry,
        ),
    )
}

fn check_daa(a: u8, cy: bool, ac: bool, pc: u16, next_pc: u16, expected: (u8, Flags)) {
    let mut bus = RecordingBus::new();
    bus.load(pc.wrapping_sub(1), &[0x5A, 0x27, 0xA5]);
    let memory = bus.memory.clone();
    let before = Registers {
        a,
        flags: flags(
            !expected.1.sign,
            !expected.1.zero,
            ac,
            !expected.1.parity,
            cy,
        ),
        ..sample_registers(pc)
    };
    let mut cpu = I8080::new(before, sample_latches());
    // 98-153B Ch. 4 DAA p. 4-8 (PDF 52): 27H, 1 byte, 1 cycle,
    // 4 states; second witness 9800301-04 Ch. 3 DAA p. 3-19 (PDF 73).
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget,
        },
        "A={a:02X} CY={cy} AC={ac}"
    );
    assert_eq!(
        *cpu.registers(),
        Registers {
            a: expected.0,
            flags: expected.1,
            pc: next_pc,
            ..before
        },
        "A={a:02X} CY={cy} AC={ac}"
    );
    assert_eq!(*cpu.latches(), settled_latches());
    assert_eq!(bus.memory, memory);
    // 98-153B Ch. 2 p. 2-16 (PDF 30), DAA row: M1 only, PC += 1.
    // OpcodeFetch and boundary polls: decision-5 items 3-5/8.
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::InterruptPoll(false),
            Event::Read {
                address: pc,
                kind: MemoryAccess::OpcodeFetch,
                value: 0x27,
            },
        ]
    );
}

#[test]
fn exhaustive_daa_matches_manual_two_step_rule_for_1024_inputs() {
    // All 256 A values x CY x AC, including non-BCD inputs. Rule and flag
    // citations are at manual_two_step_rule; check_daa pins states and trace.
    let mut cases = 0;
    for a in 0..=255 {
        for cy in [false, true] {
            for ac in [false, true] {
                check_daa(a, cy, ac, 0x4567, 0x4568, manual_two_step_rule(a, cy, ac));
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 1024);
}

#[test]
fn literal_bcd_examples_from_9800301_04() {
    // Literal A/CY/AC examples; S/Z/P derived by 98-153B p. 4-2's rules.
    let cases = [
        // 9800301-04 Ch. 1 "Auxiliary Carry Flag" p. 1-11 (PDF 21):
        // 19H + 01H = 1AH; DAA converts 1AH to 20H.
        (
            0x1A,
            false,
            false,
            0x20,
            flags(false, false, true, false, false),
        ),
        // Ch. 3 DAA p. 3-19 (PDF 73): 93H + 08H = 9BH, then A1H,
        // finally 01H with both CY and AC set.
        (
            0x9B,
            false,
            false,
            0x01,
            flags(false, false, true, false, true),
        ),
        // Ch. 6 "Decimal Addition" p. 6-13 (PDF 179):
        // 85H + 36H = BBH, DAA gives 21H with CY set.
        (
            0xBB,
            false,
            false,
            0x21,
            flags(false, false, true, true, true),
        ),
        // Ch. 6 p. 6-14 (PDF 180): 29H + 49H + CY = 73H, AC=1;
        // DAA gives 79H, CY=0. AC clears: no carry from bit 3 (98-153B
        // p. 4-3; 9800301-04 p. 1-11).
        (
            0x73,
            false,
            true,
            0x79,
            flags(false, false, false, false, false),
        ),
    ];
    for (a, cy, ac, value, expected_flags) in cases {
        assert_eq!(manual_two_step_rule(a, cy, ac), (value, expected_flags));
        check_daa(a, cy, ac, 0x4567, 0x4568, (value, expected_flags));
    }
}

#[test]
fn derived_carry_retention_without_new_high_carry() {
    // Maintainer decision (2026-11-12): derived from the documented decimal
    // routine, not stated explicitly; falsifiable by the oracle run
    // (x80-ecy.17). 9800301-04 Ch. 6 pp. 6-12 to 6-14 (PDF 178-180):
    // MADD with DAA after ADC M requires retained CY for 0099 + 0099.
    // Its low-byte binary sum is 32H, CY=1, AC=1; DAA must give 98H,
    // CY=1, so the high-byte ADC receives the decimal hundreds carry.
    // Contrary reading: 98-153B Ch. 4 "Carry" p. 4-2 (PDF 46),
    // "otherwise it is reset". AC=0 follows p. 4-3's bit-3 rule.
    let expected = (0x98, flags(true, false, false, false, true));
    assert_eq!(manual_two_step_rule(0x32, true, true), expected);
    check_daa(0x32, true, true, 0x4567, 0x4568, expected);
}

#[test]
fn derived_step_1_carry_from_95_plus_65() {
    // Derived, not printed: maintainer decision (2026-11-12), plan §2.2;
    // 98-153B Ch. 4 ADD/ADI p. 4-6, Carry p. 4-2, AC p. 4-3,
    // DAA p. 4-8. Per-step CY gives FAH + 6 = 00H, CY=1, then
    // +60H = 60H, CY=1: decimal 95 + 65 = 160. Decimal purpose:
    // 9800301-04 Ch. 6 p. 6-13; falsifiable by x80-ecy.17.
    let mut bus = RecordingBus::new();
    bus.load(0x4567, &[0xC6, 0x65, 0x27]); // ADI 65H; DAA
    let mut cpu = I8080::new(
        Registers {
            a: 0x95,
            ..sample_registers(0x4567)
        },
        sample_latches(),
    );
    assert_eq!(cpu.step(&mut bus).steps, 1);
    assert_eq!(cpu.registers().a, 0xFA);
    assert!(!cpu.registers().flags.auxiliary_carry);
    assert!(!cpu.registers().flags.carry);
    assert_eq!(cpu.step(&mut bus).steps, 1);
    assert_eq!(cpu.registers().a, 0x60);
    assert_eq!(cpu.registers().flags, flags(false, false, true, true, true));
}

#[test]
fn rule_derived_boundaries_take_four_states_and_one_opcode_fetch() {
    // These are derived vectors, not printed examples: 98-153B Ch. 4
    // DAA p. 4-8 (PDF 52), flags pp. 4-2/4-3 (PDF 46/47);
    // second witness 9800301-04 Ch. 3 DAA p. 3-19 (PDF 73).
    // FAH is derived by the per-step CY reading (maintainer decision
    // 2026-11-12), not stated; falsifiable by x80-ecy.17. Its step-1 carry
    // triggers step 2 and is retained (manual_two_step_rule citations).
    let cases = [
        (
            0x00,
            false,
            false,
            0x00,
            flags(false, true, false, true, false),
        ),
        (
            0x99,
            false,
            false,
            0x99,
            flags(true, false, false, true, false),
        ),
        (
            0x9A,
            false,
            false,
            0x00,
            flags(false, true, true, true, true),
        ),
        (
            0xA0,
            false,
            false,
            0x00,
            flags(false, true, false, true, true),
        ),
        (
            0xFA,
            false,
            false,
            0x60,
            flags(false, false, true, true, true),
        ),
        (
            0x04,
            false,
            true,
            0x0A,
            flags(false, false, false, true, false),
        ),
    ];
    // PC wrap follows its 16-bit width (98-153B Ch. 4 p. 4-3, PDF 47)
    // and DAA's M1 PC increment (Ch. 2 p. 2-16, PDF 30).
    for (pc, next_pc) in [(0x4567, 0x4568), (0xFFFF, 0x0000)] {
        for (a, cy, ac, value, expected_flags) in cases {
            assert_eq!(manual_two_step_rule(a, cy, ac), (value, expected_flags));
            check_daa(a, cy, ac, pc, next_pc, (value, expected_flags));
        }
    }
}
