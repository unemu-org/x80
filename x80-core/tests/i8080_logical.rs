//! Logical Group witnesses through the public step API (decision-5 items 3/8,
//! plan §2.2). Sources: Intel 98-153B (September 1975), Ch. 4 pp. 4-8 to
//! 4-11; Intel 9800301-04 (May 1981), Ch. 1 p. 1-12, Ch. 3 pp. 3-12/3-13.
//! Revisions/hashes are pinned in PROVENANCE.md §1; cited pages were checked
//! as images. Expectations never call crate ALU or register-pair helpers.

mod support;

use support::{sample_latches, sample_registers, settled_latches, Event, RecordingBus};
use x80_core::i8080::{Flags, Registers, I8080};
use x80_core::{Exit, MemoryAccess, Report};

#[derive(Clone, Copy)]
enum Operation {
    And,
    Xor,
    Or,
    Compare,
}

// Independent reference from 98-153B Ch. 4, Logical Group pp. 4-8/4-9:
// construct each output bit from its Boolean truth table, not byte operators.
// CMP/CPI pp. 4-9/4-10 and 9800301-04 Ch. 3 p. 3-12: subtract internally,
// preserve A, CY reports unsigned less-than. AC (98-153B Ch. 4 p. 4-3) is
// the carry across bit 3 of the two's-complement addition (p. 4-6), equivalent
// to no borrow between the low nibbles. Unlike the crate this uses comparisons
// and a signed difference, not complemented addition.
// PROVISIONAL AND AC: decision-7; unverified x80-ecy.2. 9800301-04 Ch. 1
// p. 1-12 says OR of the operand bit 3s; 98-153B p. 4-9 says ANI clears AC.
fn reference(operation: Operation, a: u8, operand: u8) -> (u8, Flags) {
    let value = if matches!(operation, Operation::Compare) {
        (i16::from(a) - i16::from(operand)).rem_euclid(256) as u8
    } else {
        let mut result = 0;
        for bit in 0..8 {
            let place = 2u8.pow(bit);
            let left = !(a / place).is_multiple_of(2);
            let right = !(operand / place).is_multiple_of(2);
            let set = match operation {
                Operation::And => left && right,
                Operation::Xor => left != right,
                Operation::Or => left || right,
                Operation::Compare => unreachable!(),
            };
            if set {
                result += place;
            }
        }
        result
    };
    // Standard flags: 98-153B Ch. 4, Condition Flags pp. 4-2/4-3.
    // Parity is the modulo-2 sum of the eight bits, independently accumulated.
    let mut parity = true;
    let mut remaining = value;
    for _ in 0..8 {
        if remaining % 2 != 0 {
            parity = !parity;
        }
        remaining /= 2;
    }
    let flags = Flags {
        sign: value >= 128,
        zero: value == 0,
        auxiliary_carry: match operation {
            Operation::And => !(a / 8).is_multiple_of(2) || !(operand / 8).is_multiple_of(2),
            Operation::Compare => a % 16 >= operand % 16,
            Operation::Xor | Operation::Or => false,
        },
        parity,
        carry: matches!(operation, Operation::Compare) && a < operand,
    };
    (
        if matches!(operation, Operation::Compare) {
            a
        } else {
            value
        },
        flags,
    )
}

fn flags(s: bool, z: bool, ac: bool, p: bool, cy: bool) -> Flags {
    Flags {
        sign: s,
        zero: z,
        auxiliary_carry: ac,
        parity: p,
        carry: cy,
    }
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

fn boundary(pc: u16, opcode: u8) -> [Event; 3] {
    [
        Event::StopPoll(false),
        Event::InterruptPoll(false),
        read(pc, MemoryAccess::OpcodeFetch, opcode),
    ]
}

fn completed(cpu: &I8080, report: Report, expected: Registers, states: u64) {
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: states,
            exit: Exit::Budget,
        }
    );
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(*cpu.latches(), settled_latches());
}

// Register codes: 98-153B Ch. 4 p. 4-3, DDD/SSS.
fn put(registers: &mut Registers, source: u8, operand: u8) {
    match source {
        0 => registers.b = operand,
        1 => registers.c = operand,
        2 => registers.d = operand,
        3 => registers.e = operand,
        4 => registers.h = operand,
        5 => registers.l = operand,
        7 => registers.a = operand,
        _ => panic!("not a register source"),
    }
}

// Each independent operand source sees every (A, operand, old CY); source A
// necessarily aliases the input A. Both polarities of every old flag ensure
// flags are replaced. Bus order: 98-153B Ch. 2 pp. 2-16/2-18, p. 2-20
// notes 1/6/9; no extra access for overlapped writeback. OperandFetch is
// decision-5 item 3. PC wrap derives from its 16-bit width (Ch. 2 p. 2-2).
fn exhaustive(operation: Operation, base: u8, immediate: u8) {
    let mut bus = RecordingBus::new();
    for a in 0..=255u8 {
        for operand in 0..=255u8 {
            let (value, expected_flags) = reference(operation, a, operand);
            for carry in [false, true] {
                for source in 0..=8 {
                    if source == 7 && operand != a {
                        continue;
                    }
                    let opcode = if source == 8 {
                        immediate
                    } else {
                        base + source
                    };
                    let mut before = sample_registers(0xFFFF);
                    before.a = a;
                    before.flags = flags(carry, carry, carry, carry, carry);
                    if source < 6 || source == 7 {
                        put(&mut before, source, operand);
                    }
                    let address = (u16::from(before.h) << 8) | u16::from(before.l);
                    bus.load(0xFFFF, &[opcode, operand]);
                    if source == 6 {
                        bus.memory[usize::from(address)] = operand;
                    }
                    let mut cpu = I8080::new(before, sample_latches());
                    let mut expected = before;
                    expected.a = value;
                    expected.flags = expected_flags;
                    expected.pc = if source == 8 { 1 } else { 0 };
                    let states = if source == 6 || source == 8 { 7 } else { 4 };
                    let report = cpu.step(&mut bus);
                    completed(&cpu, report, expected, states);
                    assert_eq!(&bus.log[..3], &boundary(0xFFFF, opcode));
                    match source {
                        6 => {
                            assert_eq!(
                                &bus.log[3..],
                                &[read(address, MemoryAccess::Data, operand)]
                            );
                            assert_eq!(bus.memory[usize::from(address)], operand);
                        }
                        8 => assert_eq!(
                            &bus.log[3..],
                            &[read(0, MemoryAccess::OperandFetch, operand)]
                        ),
                        _ => assert_eq!(bus.log.len(), 3),
                    }
                    bus.log.clear();
                }
            }
        }
    }
}

#[test]
fn provisional_ana_ani_exhaustive_every_encoding_and_trace() {
    // 98-153B Ch. 4 p. 4-8 ANA r/M: 4/7 states, CY cleared; p. 4-9
    // ANI: 7 states. AC is PROVISIONAL (decision-7, unverified x80-ecy.2),
    // following 9800301-04 Ch. 1 p. 1-12 rather than ANI's earlier AC text.
    exhaustive(Operation::And, 0xA0, 0xE6);
}

#[test]
fn xra_xri_exhaustive_every_encoding_and_trace() {
    // 98-153B Ch. 4 p. 4-9: XRA r/M, XRI: 4/7/7 states; CY/AC cleared.
    exhaustive(Operation::Xor, 0xA8, 0xEE);
}

#[test]
fn ora_ori_exhaustive_every_encoding_and_trace() {
    // 98-153B Ch. 4 p. 4-9: ORA r/M, ORI: 4/7/7 states; CY/AC cleared.
    exhaustive(Operation::Or, 0xB0, 0xF6);
}

#[test]
fn cmp_cpi_exhaustive_preserve_a_every_encoding_and_trace() {
    // 98-153B Ch. 4 pp. 4-9/4-10 CMP r/M, CPI: 4/7/7 states;
    // subtraction flags, A unchanged, Z iff equal, CY iff A < operand.
    // Second witness: 9800301-04 Ch. 3 pp. 3-12/3-13.
    exhaustive(Operation::Compare, 0xB8, 0xFE);
}

#[test]
fn literal_cases_guard_reference_and_cpu() {
    // 98-153B Ch. 4 p. 4-9 XOR/OR and pp. 4-9/4-10 comparison, with
    // Condition Flags pp. 4-2/4-3. These literal vectors guard the reference.
    let cases = [
        (
            Operation::Xor,
            0xEE,
            0xA5,
            0xA5,
            0x00,
            flags(false, true, false, true, false),
        ),
        (
            Operation::Xor,
            0xEE,
            0xF0,
            0x71,
            0x81,
            flags(true, false, false, true, false),
        ),
        (
            Operation::Or,
            0xF6,
            0x80,
            0x01,
            0x81,
            flags(true, false, false, true, false),
        ),
        (
            Operation::Or,
            0xF6,
            0x00,
            0x00,
            0x00,
            flags(false, true, false, true, false),
        ),
        (
            Operation::Compare,
            0xFE,
            0x00,
            0x01,
            0x00,
            flags(true, false, false, true, true),
        ),
        (
            Operation::Compare,
            0xFE,
            0x10,
            0x01,
            0x10,
            flags(false, false, false, true, false),
        ),
        (
            Operation::Compare,
            0xFE,
            0x01,
            0x10,
            0x01,
            flags(true, false, true, false, true),
        ),
        (
            Operation::Compare,
            0xFE,
            0x3E,
            0x3E,
            0x3E,
            flags(false, true, true, true, false),
        ),
        // PROVISIONAL decision-7 / x80-ecy.2: 9800301-04 Ch. 1 p. 1-12.
        // Zero result with AC set when either input's bit 3 is set; also
        // nonzero results with AC clear/set. 98-153B pp. 4-8/4-9: AND, CY=0.
        (
            Operation::And,
            0xE6,
            0x08,
            0x00,
            0x00,
            flags(false, true, true, true, false),
        ),
        (
            Operation::And,
            0xE6,
            0x00,
            0x08,
            0x00,
            flags(false, true, true, true, false),
        ),
        (
            Operation::And,
            0xE6,
            0x80,
            0x81,
            0x80,
            flags(true, false, false, false, false),
        ),
        (
            Operation::And,
            0xE6,
            0xFF,
            0xFF,
            0xFF,
            flags(true, false, true, true, false),
        ),
    ];
    for (operation, opcode, a, operand, value, expected_flags) in cases {
        assert_eq!(reference(operation, a, operand), (value, expected_flags));
        let before = Registers {
            a,
            ..sample_registers(0x1234)
        };
        let mut cpu = I8080::new(before, sample_latches());
        let mut bus = RecordingBus::new();
        bus.load(0x1234, &[opcode, operand]);
        let memory = bus.memory.clone();
        let report = cpu.step(&mut bus);
        completed(
            &cpu,
            report,
            Registers {
                a: value,
                flags: expected_flags,
                pc: 0x1236,
                ..before
            },
            7,
        );
        assert_eq!(bus.memory, memory);
        let mut trace = boundary(0x1234, opcode).to_vec();
        trace.push(read(0x1235, MemoryAccess::OperandFetch, operand));
        assert_eq!(bus.log, trace);
    }
}

#[test]
fn rotates_exhaustive_values_carry_and_preserved_flags() {
    // 98-153B Ch. 4 p. 4-10 RLC/RRC/RAL/RAR: 4 states, only CY changes.
    // Reference uses quotient/remainder and multiplication, independent of
    // the crate's shifts/rotates. Every A and all 32 input flag combinations.
    let mut bus = RecordingBus::new();
    for opcode in [0x07, 0x0F, 0x17, 0x1F] {
        for a in 0..=255u8 {
            for pattern in 0..32 {
                let old = flags(
                    pattern & 1 != 0,
                    pattern & 2 != 0,
                    pattern & 4 != 0,
                    pattern & 8 != 0,
                    pattern & 16 != 0,
                );
                let before = Registers {
                    a,
                    flags: old,
                    ..sample_registers(0xFFFF)
                };
                let (value, carry) = match opcode {
                    0x07 => ((a % 128) * 2 + a / 128, a >= 128),
                    0x0F => (a / 2 + (a % 2) * 128, a % 2 != 0),
                    0x17 => ((a % 128) * 2 + u8::from(old.carry), a >= 128),
                    0x1F => (a / 2 + u8::from(old.carry) * 128, a % 2 != 0),
                    _ => unreachable!(),
                };
                bus.load(0xFFFF, &[opcode]);
                let mut cpu = I8080::new(before, sample_latches());
                let report = cpu.step(&mut bus);
                completed(
                    &cpu,
                    report,
                    Registers {
                        a: value,
                        flags: Flags { carry, ..old },
                        pc: 0,
                        ..before
                    },
                    4,
                );
                assert_eq!(bus.log, boundary(0xFFFF, opcode));
                bus.log.clear();
            }
        }
    }
}

#[test]
fn cma_cmc_stc_exhaustive_values_and_flags() {
    // 98-153B Ch. 4 p. 4-10 CMA: complement A, no flags; p. 4-11 CMC/STC:
    // complement/set CY, no other flag or register changes. All take 4 states.
    let mut bus = RecordingBus::new();
    for opcode in [0x2F, 0x3F, 0x37] {
        for a in 0..=255u8 {
            for pattern in 0..32 {
                let old = flags(
                    pattern & 1 != 0,
                    pattern & 2 != 0,
                    pattern & 4 != 0,
                    pattern & 8 != 0,
                    pattern & 16 != 0,
                );
                let before = Registers {
                    a,
                    flags: old,
                    ..sample_registers(0xFFFF)
                };
                let mut expected = Registers { pc: 0, ..before };
                match opcode {
                    0x2F => expected.a = 255 - a,
                    0x3F => expected.flags.carry = !old.carry,
                    0x37 => expected.flags.carry = true,
                    _ => unreachable!(),
                }
                bus.load(0xFFFF, &[opcode]);
                let mut cpu = I8080::new(before, sample_latches());
                let report = cpu.step(&mut bus);
                completed(&cpu, report, expected, 4);
                assert_eq!(bus.log, boundary(0xFFFF, opcode));
                bus.log.clear();
            }
        }
    }
}

#[test]
fn memory_operand_can_alias_opcode_or_immediate_byte() {
    // 98-153B Ch. 4 pp. 4-8 to 4-10: M uses H/L, immediate uses byte 2.
    // Ch. 2 pp. 2-16/2-18: fetch precedes the data read; no writes.
    // AND expectations are PROVISIONAL decision-7 / x80-ecy.2 (9800301-04
    // Ch. 1 p. 1-12), as for the exhaustive witness above.
    for (base, immediate, operation) in [
        (0xA0, 0xE6, Operation::And),
        (0xA8, 0xEE, Operation::Xor),
        (0xB0, 0xF6, Operation::Or),
        (0xB8, 0xFE, Operation::Compare),
    ] {
        for address in [0xFFFF, 0x0000] {
            for is_immediate in [false, true] {
                let opcode = if is_immediate { immediate } else { base + 6 };
                let before = Registers {
                    a: 0x5A,
                    h: (address >> 8) as u8,
                    l: address as u8,
                    ..sample_registers(0xFFFF)
                };
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[opcode, 0x96]);
                let memory = bus.memory.clone();
                let data_at = (u16::from(before.h) << 8) | u16::from(before.l);
                let (operand_at, kind) = if is_immediate {
                    (0, MemoryAccess::OperandFetch)
                } else {
                    (data_at, MemoryAccess::Data)
                };
                let operand = if operand_at == 0xFFFF { opcode } else { 0x96 };
                let (value, expected_flags) = reference(operation, before.a, operand);
                let mut cpu = I8080::new(before, sample_latches());
                let report = cpu.step(&mut bus);
                completed(
                    &cpu,
                    report,
                    Registers {
                        a: value,
                        flags: expected_flags,
                        pc: u16::from(is_immediate),
                        ..before
                    },
                    7,
                );
                assert_eq!(bus.memory, memory);
                let mut trace = boundary(0xFFFF, opcode).to_vec();
                trace.push(read(operand_at, kind, operand));
                assert_eq!(bus.log, trace);
            }
        }
    }
}
