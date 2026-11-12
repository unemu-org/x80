//! 8085 Logical Group witnesses through step (plan §2.2; decision-10;
//! decision-12). Intel 205775-002 (January 1983), §5.5 p. 5-3, §5.6.3
//! pp. 5-9 to 5-12; Intel 9800301-04 (May 1981), pages cited below.
//! Hashes match PROVENANCE.md §1; cited values checked on rendered pages.
//! Expectations use no crate ALU, register-pair or flag-image helpers.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Exit, MemoryAccess, Report};

#[derive(Clone, Copy)]
enum Operation {
    And,
    Xor,
    Or,
    Compare,
}

// Boolean truth tables: 205775-002 pp. 5-9 to 5-11; 9800301-04 p. 3-6.
// Compare: 205775-002 pp. 5-3/5-6/5-11; 9800301-04 pp. 3-12/3-13.
// Independently add the complemented operand, retaining the bit-3 carry
// (AC) and testing unsigned less-than (CY), unlike alu::subtract's signed
// byte and nibble differences. S/Z/P: 205775-002 p. 5-3, independently
// witnessed by 9800301-04 pp. 1-10/1-11; parity toggles for each set bit.
fn reference(operation: Operation, a: u8, operand: u8) -> (u8, Flags) {
    let compare = matches!(operation, Operation::Compare);
    let value = if compare {
        ((u16::from(a) + u16::from(255 - operand) + 1) % 256) as u8
    } else {
        let mut value = 0;
        for bit in 0..8 {
            let place = 2u8.pow(bit);
            let left = (a / place) % 2 == 1;
            let right = (operand / place) % 2 == 1;
            let set = match operation {
                Operation::And => left && right,
                Operation::Xor => left != right,
                Operation::Or => left || right,
                Operation::Compare => unreachable!(),
            };
            if set {
                value += place;
            }
        }
        value
    };
    let mut even = true;
    let mut remaining = value;
    for _ in 0..8 {
        if remaining % 2 == 1 {
            even = !even;
        }
        remaining /= 2;
    }
    let auxiliary = match operation {
        // 205775-002 p. 5-10; 9800301-04 p. 1-12: always ON on 8085.
        Operation::And => true,
        Operation::Compare => u16::from(a % 16) + u16::from(15 - operand % 16) + 1 > 15,
        Operation::Xor | Operation::Or => false,
    };
    (
        if compare { a } else { value },
        flags(
            value >= 128,
            value == 0,
            auxiliary,
            even,
            compare && a < operand,
        ),
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

fn registers(pc: u16) -> Registers {
    Registers {
        a: 0x91,
        flags: flags(true, false, true, false, true),
        b: 0x12,
        c: 0x34,
        d: 0x56,
        e: 0x78,
        h: 0x9A,
        l: 0xBC,
        sp: 0xDEF0,
        pc,
    }
}

// Deliberately mixed latches; only EI delay expires on completion
// (decision-12 rule 6). The group must not touch the other fields.
fn latches() -> Latches {
    Latches {
        interrupt_enable: true,
        ei_delay: true,
        mask6_5: false,
        rst7_5_pending: true,
        trap_saved_interrupt_enable: true,
        trap_rim_pending: true,
        sod: true,
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

fn boundary(pc: u16, opcode: u8) -> [Event; 7] {
    [
        Event::StopPoll(false),
        Event::TrapPoll(false),
        Event::Rst7_5Poll(false),
        Event::Rst6_5Poll(false),
        Event::Rst5_5Poll(false),
        Event::IntrPoll(false),
        read(pc, MemoryAccess::OpcodeFetch, opcode),
    ]
}

fn completed(cpu: &I8085, report: Report, expected: Registers, states: u64) {
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: states,
            exit: Exit::Budget
        }
    );
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(
        *cpu.latches(),
        Latches {
            ei_delay: false,
            ..latches()
        }
    );
}

// 205775-002 §5.2 p. 5-1 register codes; Table 5-1 p. 5-19 adds M=110.
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

// Every A x operand, every independent source, and both old flag polarities.
// A-source necessarily aliases A. Exact access order and 4/7 states:
// 205775-002 Table 5-1 p. 5-19 (F / F R), Ch. 2 pp. 2-7 to 2-10;
// Table 5-2 p. 5-20 independently pins the byte assignments.
// Access kinds and boundary polls: decision-5 item 3, decision-12 rule 1.
// DERIVED: PC wrap follows its 16-bit width (205775-002 p. 2-2), not an
// explicit wrap statement. Immediate byte at FFFF+1 is read at 0000.
fn exhaustive(operation: Operation, base: u8, immediate: u8) {
    let mut bus = RecordingBus::new();
    for a in 0..=255u8 {
        for operand in 0..=255u8 {
            let (value, expected_flags) = reference(operation, a, operand);
            for old in [false, true] {
                for source in 0..=8 {
                    if source == 7 && operand != a {
                        continue;
                    }
                    let opcode = if source == 8 {
                        immediate
                    } else {
                        base + source
                    };
                    let mut before = registers(0xFFFF);
                    before.a = a;
                    before.flags = flags(old, old, old, old, old);
                    if source < 6 || source == 7 {
                        put(&mut before, source, operand);
                    }
                    let address = u16::from(before.h) * 256 + u16::from(before.l);
                    bus.load(0xFFFF, &[opcode, operand]);
                    if source == 6 {
                        bus.memory[usize::from(address)] = operand;
                    }
                    let mut cpu = I8085::new(before, latches());
                    let expected = Registers {
                        a: value,
                        flags: expected_flags,
                        pc: u16::from(source == 8),
                        ..before
                    };
                    let states = if source == 6 || source == 8 { 7 } else { 4 };
                    let report = cpu.step(&mut bus);
                    completed(&cpu, report, expected, states);
                    assert_eq!(&bus.log[..7], &boundary(0xFFFF, opcode));
                    match source {
                        6 => {
                            assert_eq!(&bus.log[7..], &[read(address, MemoryAccess::Data, operand)])
                        }
                        8 => assert_eq!(
                            &bus.log[7..],
                            &[read(0, MemoryAccess::OperandFetch, operand)]
                        ),
                        _ => assert_eq!(bus.log.len(), 7),
                    }
                    bus.log.clear();
                }
            }
        }
    }
}

#[test]
fn ana_ani_exhaustive_every_encoding_and_trace() {
    // 205775-002 §5.6.3 pp. 5-9/5-10: AND, CY=0, AC=1, 4/7/7 states.
    // Second witness: 9800301-04 pp. 1-12, 3-6 to 3-8.
    exhaustive(Operation::And, 0xA0, 0xE6);
}

#[test]
fn xra_xri_exhaustive_every_encoding_and_trace() {
    // 205775-002 p. 5-10: XOR, CY=AC=0, 4/7/7 states.
    // Second witness: 9800301-04 pp. 3-66 to 3-68.
    exhaustive(Operation::Xor, 0xA8, 0xEE);
}

#[test]
fn ora_ori_exhaustive_every_encoding_and_trace() {
    // 205775-002 pp. 5-10/5-11: OR, CY=AC=0, 4/7/7 states.
    // Second witness: 9800301-04 pp. 3-38 to 3-40.
    exhaustive(Operation::Or, 0xB0, 0xF6);
}

#[test]
fn cmp_cpi_exhaustive_preserve_a_every_encoding_and_trace() {
    // 205775-002 p. 5-11: subtraction flags, A unchanged, Z iff equal,
    // CY iff A < operand; 4/7/7 states. 9800301-04 pp. 3-12/3-13, 3-16/3-17.
    exhaustive(Operation::Compare, 0xB8, 0xFE);
}

#[test]
fn and_sets_ac_when_8080_bit3_rule_would_clear_it() {
    // 205775-002 p. 5-10; 9800301-04 p. 1-12 explicitly distinguishes
    // 8085's always-set AC from 8080's OR of operand bit 3. A=01, operand=02
    // have both bit 3 clear: 8080 would give AC=0; 8085 must give AC=1.
    for opcode in [0xA0, 0xA6, 0xE6] {
        let before = Registers {
            a: 0x01,
            b: 0x02,
            ..registers(0x1234)
        };
        let expected_flags = flags(false, true, true, true, false);
        assert_eq!(reference(Operation::And, 0x01, 0x02), (0, expected_flags));
        let mut bus = RecordingBus::new();
        bus.load(0x1234, &[opcode, 0x02]);
        bus.memory[0x9ABC] = 0x02;
        let memory = bus.memory.clone();
        let mut cpu = I8085::new(before, latches());
        let report = cpu.step(&mut bus);
        completed(
            &cpu,
            report,
            Registers {
                a: 0,
                flags: expected_flags,
                pc: if opcode == 0xE6 { 0x1236 } else { 0x1235 },
                ..before
            },
            if opcode == 0xA0 { 4 } else { 7 },
        );
        let mut trace = boundary(0x1234, opcode).to_vec();
        match opcode {
            0xA6 => trace.push(read(0x9ABC, MemoryAccess::Data, 0x02)),
            0xE6 => trace.push(read(0x1235, MemoryAccess::OperandFetch, 0x02)),
            _ => {}
        }
        assert_eq!(bus.log, trace);
        assert_eq!(bus.memory, memory);
    }
}

#[test]
fn literal_vectors_guard_reference_and_cpu() {
    // Re-derived from 205775-002 pp. 5-3, 5-6, 5-10/5-11. Each tuple is
    // opcode, operation, A, operand, unchanged/result A, (S,Z,AC,P,CY).
    // Second witnesses: 9800301-04 pp. 1-10 to 1-12 and per-op pages above.
    let vectors = [
        (
            0xE6,
            Operation::And,
            0x80,
            0x81,
            0x80,
            flags(true, false, true, false, false),
        ),
        (
            0xE6,
            Operation::And,
            0xFF,
            0xFF,
            0xFF,
            flags(true, false, true, true, false),
        ),
        (
            0xEE,
            Operation::Xor,
            0xFF,
            0xFF,
            0x00,
            flags(false, true, false, true, false),
        ),
        (
            0xEE,
            Operation::Xor,
            0x55,
            0xD4,
            0x81,
            flags(true, false, false, true, false),
        ),
        (
            0xF6,
            Operation::Or,
            0x80,
            0x01,
            0x81,
            flags(true, false, false, true, false),
        ),
        (
            0xF6,
            Operation::Or,
            0x00,
            0x00,
            0x00,
            flags(false, true, false, true, false),
        ),
        (
            0xFE,
            Operation::Compare,
            0x00,
            0x01,
            0x00,
            flags(true, false, false, true, true),
        ),
        (
            0xFE,
            Operation::Compare,
            0x20,
            0x01,
            0x20,
            flags(false, false, false, false, false),
        ),
        (
            0xFE,
            Operation::Compare,
            0x01,
            0x20,
            0x01,
            flags(true, false, true, true, true),
        ),
        (
            0xFE,
            Operation::Compare,
            0x80,
            0x80,
            0x80,
            flags(false, true, true, true, false),
        ),
    ];
    for (opcode, operation, a, operand, value, expected_flags) in vectors {
        assert_eq!(reference(operation, a, operand), (value, expected_flags));
        let before = Registers {
            a,
            ..registers(0x1234)
        };
        let mut cpu = I8085::new(before, latches());
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
        let mut trace = boundary(0x1234, opcode).to_vec();
        trace.push(read(0x1235, MemoryAccess::OperandFetch, operand));
        assert_eq!(bus.log, trace);
        assert_eq!(bus.memory, memory);
    }
}

#[test]
fn rotates_exhaustive_values_carry_and_preserved_flags() {
    // 205775-002 RLC p. 5-11, RRC/RAL/RAR p. 5-12: 4 states, CY only.
    // 9800301-04 RLC p. 3-49, RRC pp. 3-53/3-54, RAL pp. 3-45/3-46,
    // RAR p. 3-46. Quotient/remainder reference, every A and all 32 flags.
    // DERIVED PC wrap as in exhaustive. Only F (Table 5-1 p. 5-19).
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
                    ..registers(0xFFFF)
                };
                let (value, carry) = match opcode {
                    0x07 => ((a % 128) * 2 + a / 128, a >= 128),
                    0x0F => (a / 2 + (a % 2) * 128, a % 2 == 1),
                    0x17 => ((a % 128) * 2 + u8::from(old.carry), a >= 128),
                    0x1F => (a / 2 + u8::from(old.carry) * 128, a % 2 == 1),
                    _ => unreachable!(),
                };
                bus.load(0xFFFF, &[opcode]);
                let mut cpu = I8085::new(before, latches());
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
    // 205775-002 p. 5-12; 9800301-04 CMA p. 3-11, CMC p. 3-12,
    // STC p. 3-63. CMA complements A, no flags; CMC/STC change only CY.
    // 4 states, only F (Table 5-1 p. 5-19). DERIVED PC wrap as above.
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
                    ..registers(0xFFFF)
                };
                let mut expected = Registers { pc: 0, ..before };
                match opcode {
                    0x2F => expected.a = 255 - a,
                    0x3F => expected.flags.carry = !old.carry,
                    0x37 => expected.flags.carry = true,
                    _ => unreachable!(),
                }
                bus.load(0xFFFF, &[opcode]);
                let mut cpu = I8085::new(before, latches());
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
    // 205775-002 pp. 5-10/5-11; second witnesses are per-op pages above.
    // M reads HL after opcode fetch; immediate reads byte 2, regardless of
    // HL. Table 5-1 p. 5-19 F R; Ch. 2 pp. 2-7 to 2-10. No writes.
    // DERIVED PC wrap, including operand fetch at 0000 from PC=FFFF.
    for (base, immediate, operation) in [
        (0xA0, 0xE6, Operation::And),
        (0xA8, 0xEE, Operation::Xor),
        (0xB0, 0xF6, Operation::Or),
        (0xB8, 0xFE, Operation::Compare),
    ] {
        for address in [0xFFFFu16, 0] {
            for is_immediate in [false, true] {
                let opcode = if is_immediate { immediate } else { base + 6 };
                let before = Registers {
                    a: 0x5A,
                    h: (address / 256) as u8,
                    l: (address % 256) as u8,
                    ..registers(0xFFFF)
                };
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[opcode, 0x96]);
                let memory = bus.memory.clone();
                let (at, kind) = if is_immediate {
                    (0, MemoryAccess::OperandFetch)
                } else {
                    (address, MemoryAccess::Data)
                };
                let operand = if at == 0xFFFF { opcode } else { 0x96 };
                let (value, expected_flags) = reference(operation, 0x5A, operand);
                let mut cpu = I8085::new(before, latches());
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
                let mut trace = boundary(0xFFFF, opcode).to_vec();
                trace.push(read(at, kind, operand));
                assert_eq!(bus.log, trace);
                assert_eq!(bus.memory, memory);
            }
        }
    }
}
