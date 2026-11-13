//! 8085 DAA witnesses: plan §2.2; decisions 10/12.
//! Intel 205775-002 (January 1983), §5.6.2 p. 5-9 (PDF 94), §5.5
//! p. 5-3 (PDF 88); Intel 9800301-04 (May 1981) p. 3-19 (PDF 73),
//! pp. 6-12 to 6-14 (PDF 178-180). Checked against rendered images.
//! Carry retention/step-1 propagation are derived; PROVENANCE.md §1.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Exit, MemoryAccess, Report};

fn flags(s: bool, z: bool, ac: bool, p: bool, cy: bool) -> Flags {
    Flags {
        overflow: false,
        k: false,
        sign: s,
        zero: z,
        auxiliary_carry: ac,
        parity: p,
        carry: cy,
    }
}

// Independent digit arithmetic transcription, rather than whole-byte
// additions: 205775-002 p. 5-9 says adjust low, then high "now greater
// than 9"; 9800301-04 p. 3-19's 9BH -> A1H -> 01H keeps low-step AC.
fn manual_rule(a: u8, cy: bool, ac: bool) -> (u8, Flags) {
    let mut units = u16::from(a % 16);
    let mut tens = u16::from(a / 16);
    if units >= 10 || ac {
        units += 6;
    }
    let auxiliary = units >= 16;
    tens += units / 16;
    units %= 16;
    let low_overflow = tens >= 16;
    tens %= 16;
    // Derived: retain incoming CY and propagate step 1's bit-7 carry
    // into step 2. §5.5 p. 5-3 read per adjustment; decimal addition
    // requires this carry for higher bytes (9800301-04 pp. 6-12 to 6-14).
    if tens >= 10 || cy || low_overflow {
        tens += 6;
    }
    let carry = cy || low_overflow || tens >= 16;
    let value = (16 * (tens % 16) + units) as u8;
    let ones = (0..8).filter(|bit| value & (1 << bit) != 0).count();
    (
        value,
        flags(value >= 128, value == 0, auxiliary, ones % 2 == 0, carry),
    )
}

fn boundary(pc: u16, opcode: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::TrapPoll(false),
        Event::Rst7_5Poll(false),
        Event::Rst6_5Poll(false),
        Event::Rst5_5Poll(false),
        Event::IntrPoll(false),
        Event::Read {
            address: pc,
            kind: MemoryAccess::OpcodeFetch,
            value: opcode,
        },
    ]
}

fn check(a: u8, cy: bool, ac: bool, pc: u16, next_pc: u16, expected: (u8, Flags)) {
    let before = Registers {
        a,
        b: 0x12,
        c: 0x34,
        d: 0x56,
        e: 0x78,
        h: 0x9A,
        l: 0xBC,
        sp: 0xDEF0,
        pc,
        flags: flags(
            !expected.1.sign,
            !expected.1.zero,
            ac,
            !expected.1.parity,
            cy,
        ),
    };
    let initial_latches = Latches {
        interrupt_enable: true,
        ei_delay: true,
        sod: true,
        ..Latches::default()
    };
    let mut cpu = I8085::new(before, initial_latches);
    let mut bus = RecordingBus::new();
    bus.memory[usize::from(pc)] = 0x27;
    let memory = bus.memory.clone();
    // Both adjustment/no-adjustment paths: one F, 4 states, 1 byte.
    // 205775-002 p. 5-9 / Table 5-1 p. 5-19; 9800301-04 p. 3-19.
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    // Mechanical decision-18 migration: keep all five original flag
    // expectations; V/K are independently witnessed in i8085_vk_flags.rs.
    let mut actual = *cpu.registers();
    actual.flags.overflow = false;
    actual.flags.k = false;
    assert_eq!(
        actual,
        Registers {
            a: expected.0,
            flags: expected.1,
            pc: next_pc,
            ..before
        },
        "A={a:02X}, CY={cy}, AC={ac}"
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            ei_delay: false,
            ..initial_latches
        }
    );
    assert_eq!(bus.memory, memory);
    // OF reads PC (Ch. 2 pp. 2-9/2-12); no further memory cycle.
    assert_eq!(bus.log, boundary(pc, 0x27));
}

#[test]
fn exhaustive_daa_all_1024_accumulator_carry_auxiliary_inputs() {
    let mut cases = 0;
    for a in 0..=255 {
        for cy in [false, true] {
            for ac in [false, true] {
                check(a, cy, ac, 0x4000, 0x4001, manual_rule(a, cy, ac));
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 1024);
}

#[test]
fn literal_manual_examples_and_adjustment_paths() {
    // 205775-002 p. 5-9 prints the rule, no numeric example. The literal
    // worked example is in the second witness, 9800301-04 p. 3-19:
    // 93H + 08H -> 9BH -> A1H -> 01H with CY=AC=1.
    // Other S/Z/P/AC values follow 205775-002 §5.5 p. 5-3.
    for (a, cy, ac, value, expected_flags) in [
        (
            0x9B,
            false,
            false,
            0x01,
            flags(false, false, true, false, true),
        ),
        // Second witness p. 1-11: 19H + 1 -> 1AH -> 20H.
        (
            0x1A,
            false,
            false,
            0x20,
            flags(false, false, true, false, false),
        ),
        // Second witness p. 6-13: 85H + 36H -> BBH -> 21H, CY=1.
        (
            0xBB,
            false,
            false,
            0x21,
            flags(false, false, true, true, true),
        ),
        // Second witness p. 6-14: 29H + 49H + CY -> 73H, AC=1 -> 79H.
        (
            0x73,
            false,
            true,
            0x79,
            flags(false, false, false, false, false),
        ),
        // Re-derived rule boundaries: neither, low only, high only, both.
        (
            0x00,
            false,
            false,
            0x00,
            flags(false, true, false, true, false),
        ),
        (
            0x09,
            false,
            false,
            0x09,
            flags(false, false, false, true, false),
        ),
        (
            0x0A,
            false,
            false,
            0x10,
            flags(false, false, true, false, false),
        ),
        (
            0xA0,
            false,
            false,
            0x00,
            flags(false, true, false, true, true),
        ),
        (
            0x9A,
            false,
            false,
            0x00,
            flags(false, true, true, true, true),
        ),
    ] {
        assert_eq!(manual_rule(a, cy, ac), (value, expected_flags));
        check(a, cy, ac, 0x4000, 0x4001, (value, expected_flags));
    }
}

#[test]
fn derived_carry_retention_and_low_step_carry() {
    // Derived, NOT stated by p. 5-9: §5.5 p. 5-3's CY rule read per
    // adjustment, retaining incoming carry. 9800301-04 pp. 6-12 to 6-14
    // require decimal carry for the next byte; 99+99 yields binary 32H,
    // CY=AC=1 and decimal 98H,CY=1. 95+65 yields FAH -> 00H -> 60H.
    for (a, cy, ac, value, expected_flags) in [
        (
            0x32,
            true,
            true,
            0x98,
            flags(true, false, false, false, true),
        ),
        (
            0xFA,
            false,
            false,
            0x60,
            flags(false, false, true, true, true),
        ),
        (
            0x00,
            true,
            false,
            0x60,
            flags(false, false, false, true, true),
        ),
    ] {
        assert_eq!(manual_rule(a, cy, ac), (value, expected_flags));
        check(a, cy, ac, 0x4000, 0x4001, (value, expected_flags));
    }
}

#[test]
fn derived_all_decimal_sums_with_carry_use_add_then_daa() {
    // Independent decimal-integer witness for the purpose stated in
    // 205775-002 p. 5-9, 9800301-04 pp. 6-12 to 6-14. Includes carry
    // retention and step-1 propagation: derived rule, not hardware data.
    let mut bus = RecordingBus::new();
    bus.load(0x4000, &[0xCE, 0, 0x27]); // ACI data; DAA
    for left in 0u16..100 {
        for right in 0u16..100 {
            for cy in [false, true] {
                let packed_left = ((left / 10) * 16 + left % 10) as u8;
                bus.memory[0x4001] = ((right / 10) * 16 + right % 10) as u8;
                let mut cpu = I8085::new(
                    Registers {
                        a: packed_left,
                        pc: 0x4000,
                        flags: flags(false, false, false, false, cy),
                        ..Registers::default()
                    },
                    Latches::default(),
                );
                bus.log.clear();
                assert_eq!(
                    cpu.step(&mut bus),
                    Report {
                        steps: 1,
                        cycles: 7,
                        exit: Exit::Budget
                    }
                );
                assert_eq!(
                    cpu.step(&mut bus),
                    Report {
                        steps: 1,
                        cycles: 4,
                        exit: Exit::Budget
                    }
                );
                let sum = left + right + u16::from(cy);
                let digits = sum % 100;
                assert_eq!(cpu.registers().a, ((digits / 10) * 16 + digits % 10) as u8);
                assert_eq!(cpu.registers().flags.carry, sum >= 100);
                assert_eq!(cpu.registers().pc, 0x4003);
            }
        }
    }
}

#[test]
fn derived_daa_pc_wrap() {
    // Derived from PC's 16-bit width, not an explicit DAA page statement.
    check(
        0x9B,
        false,
        false,
        0xFFFF,
        0,
        (0x01, flags(false, false, true, false, true)),
    );
}
