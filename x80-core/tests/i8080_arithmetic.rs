//! Arithmetic witnesses through the public step API (decision-5 items 3/8,
//! plan §2.2, decision-6). Sources are pinned in PROVENANCE.md §1 and all cited
//! pages were checked as images. No expected value uses a crate ALU helper.

mod arithmetic_reference;
mod support;

use arithmetic_reference::arithmetic;
use support::{sample_latches, sample_registers, Event, RecordingBus};
use x80_core::i8080::{Flags, Registers, I8080};
use x80_core::{Exit, MemoryAccess, Report};

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

// Register codes: 98-153B p. 4-3, "DDD, SSS". M is addressed by HL.
fn put(registers: &mut Registers, code: u8, value: u8) {
    match code {
        0 => registers.b = value,
        1 => registers.c = value,
        2 => registers.d = value,
        3 => registers.e = value,
        4 => registers.h = value,
        5 => registers.l = value,
        7 => registers.a = value,
        _ => panic!("memory is not a register"),
    }
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

#[test]
fn literal_boundaries_guard_reference_and_cpu() {
    // 98-153B pp. 4-2/4-3: even parity includes zero; AC is carry across
    // bit 3, CY is carry/borrow. Arithmetic definitions pp. 4-6/4-7.
    // Tuples: subtract, A, operand, carry/borrow input, result, S/Z/AC/P/CY.
    let vectors = [
        (
            false,
            0x0F,
            0x01,
            false,
            0x10,
            flags(false, false, true, false, false),
        ),
        (
            false,
            0x0E,
            0x01,
            false,
            0x0F,
            flags(false, false, false, true, false),
        ),
        (
            false,
            0xFF,
            0x00,
            true,
            0x00,
            flags(false, true, true, true, true),
        ),
        (
            false,
            0x7F,
            0x00,
            true,
            0x80,
            flags(true, false, true, false, false),
        ),
        (
            true,
            0x10,
            0x01,
            false,
            0x0F,
            flags(false, false, false, true, false),
        ),
        (
            true,
            0x00,
            0x01,
            false,
            0xFF,
            flags(true, false, false, true, true),
        ),
        (
            true,
            0x01,
            0x01,
            false,
            0x00,
            flags(false, true, true, true, false),
        ),
        (
            true,
            0x00,
            0x00,
            false,
            0x00,
            flags(false, true, true, true, false),
        ),
        (
            true,
            0x00,
            0xFF,
            true,
            0x00,
            flags(false, true, false, true, true),
        ),
        (
            true,
            0x10,
            0x0F,
            true,
            0x00,
            flags(false, true, false, true, false),
        ),
        // 9800301-04 p. 3-64: literal SUB A example, 3E - 3E = 0, AC=1.
        (
            true,
            0x3E,
            0x3E,
            false,
            0x00,
            flags(false, true, true, true, false),
        ),
        // 9800301-04 p. 3-57: literal SBB B example, 4 - 2 - 1 = 1, AC=1.
        (
            true,
            0x04,
            0x02,
            true,
            0x01,
            flags(false, false, true, false, false),
        ),
    ];
    let mut bus = RecordingBus::new();
    for (subtract, a, operand, carry, value, expected_flags) in vectors {
        assert_eq!(
            arithmetic(a, operand, carry, subtract),
            (value, expected_flags)
        );
        // ACI/SBI, 7 states: 98-153B pp. 4-6/4-7.
        let opcode = if subtract { 0xDE } else { 0xCE };
        let mut registers = sample_registers(0x1234);
        registers.a = a;
        registers.flags.carry = carry;
        let mut cpu = I8080::new(registers, sample_latches());
        bus.load(0x1234, &[opcode, operand]);
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 7,
                exit: Exit::Budget
            }
        );
        assert_eq!(
            (cpu.registers().a, cpu.registers().flags),
            (value, expected_flags)
        );
        bus.log.clear();
    }
}

#[test]
fn exhaustive_add_adc_sub_sbb_every_encoding_and_trace() {
    // 98-153B p. 4-6: ADD/ADC register 4, memory/immediate 7 states;
    // SUB register 4. p. 4-7: SUB memory/SUI 7; SBB register 4,
    // memory/SBI 7. M1 opcode read, M2 HL/PC read: pp. 2-16/2-17,
    // p. 2-20 notes 1/6/9. OperandFetch is decision-5 item 3.
    // Each independent source permits every (A, operand, CY). A as source
    // necessarily equals the initial A, and is tested separately below.
    let mut bus = RecordingBus::new();
    for (base, immediate, subtract, with_input) in [
        (0x80, 0xC6, false, false),
        (0x88, 0xCE, false, true),
        (0x90, 0xD6, true, false),
        (0x98, 0xDE, true, true),
    ] {
        for a in 0..=255u8 {
            for operand in 0..=255u8 {
                for carry in [false, true] {
                    let (value, expected_flags) =
                        arithmetic(a, operand, with_input && carry, subtract);
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
                        // Both polarities ensure every old non-CY flag is overwritten.
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
                        let states = if source == 6 || source == 8 { 7 } else { 4 };
                        assert_eq!(
                            cpu.step(&mut bus),
                            Report {
                                steps: 1,
                                cycles: states,
                                exit: Exit::Budget
                            },
                            "{opcode:02x}"
                        );
                        let mut expected = before;
                        expected.a = value;
                        expected.flags = expected_flags;
                        expected.pc = if source == 8 { 1 } else { 0 };
                        assert_eq!(
                            *cpu.registers(),
                            expected,
                            "opcode={opcode:02x} A={a:02x} operand={operand:02x} CY={carry}"
                        );
                        assert_eq!(*cpu.latches(), sample_latches());
                        let prefix = boundary(0xFFFF, opcode);
                        assert_eq!(&bus.log[..3], &prefix);
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
}

#[test]
fn exhaustive_inr_dcr_every_encoding_preserves_carry_and_traces() {
    // 98-153B p. 4-7 INR r/M and DCR r; p. 4-8 DCR M:
    // 5 states for registers, 10 for M; CY unchanged. pp. 2-16/2-17,
    // p. 2-20 notes 6/7: M reads HL then writes HL, both Data (decision-5 §3).
    let mut bus = RecordingBus::new();
    for (base, subtract) in [(0x04, false), (0x05, true)] {
        for destination in 0..8 {
            let opcode = base + destination * 8;
            for operand in 0..=255u8 {
                for carry in [false, true] {
                    let mut before = sample_registers(0xFFFF);
                    before.flags = flags(carry, carry, carry, carry, carry);
                    if destination != 6 {
                        put(&mut before, destination, operand);
                    }
                    let address = (u16::from(before.h) << 8) | u16::from(before.l);
                    bus.memory[0xFFFF] = opcode;
                    if destination == 6 {
                        bus.memory[usize::from(address)] = operand;
                    }
                    let mut cpu = I8080::new(before, sample_latches());
                    let (value, mut expected_flags) = arithmetic(operand, 1, false, subtract);
                    expected_flags.carry = carry;
                    let mut expected = before;
                    expected.pc = 0;
                    expected.flags = expected_flags;
                    if destination != 6 {
                        put(&mut expected, destination, value);
                    }
                    let states = if destination == 6 { 10 } else { 5 };
                    assert_eq!(
                        cpu.step(&mut bus),
                        Report {
                            steps: 1,
                            cycles: states,
                            exit: Exit::Budget
                        }
                    );
                    assert_eq!(
                        *cpu.registers(),
                        expected,
                        "opcode={opcode:02x} operand={operand:02x} CY={carry}"
                    );
                    assert_eq!(*cpu.latches(), sample_latches());
                    assert_eq!(&bus.log[..3], &boundary(0xFFFF, opcode));
                    if destination == 6 {
                        assert_eq!(
                            &bus.log[3..],
                            &[
                                read(address, MemoryAccess::Data, operand),
                                Event::Write {
                                    address,
                                    kind: MemoryAccess::Data,
                                    value
                                },
                            ]
                        );
                        assert_eq!(bus.memory[usize::from(address)], value);
                    } else {
                        assert_eq!(bus.log.len(), 3);
                    }
                    bus.log.clear();
                }
            }
        }
    }
}

fn put_pair(registers: &mut Registers, code: u8, value: u16) {
    // Pair selection/byte order: 98-153B p. 4-3. Deliberately no crate helpers.
    let [high, low] = value.to_be_bytes();
    match code {
        0 => {
            registers.b = high;
            registers.c = low;
        }
        1 => {
            registers.d = high;
            registers.e = low;
        }
        2 => {
            registers.h = high;
            registers.l = low;
        }
        3 => registers.sp = value,
        _ => panic!("invalid pair"),
    }
}

#[test]
fn inx_dcx_all_pairs_wrap_and_preserve_all_flags() {
    // 98-153B p. 4-8 INX/DCX: 5 states; no flags. 16-bit pair width p. 4-3.
    // pp. 2-16/2-17: no memory cycle beyond the opcode fetch.
    let mut bus = RecordingBus::new();
    for (base, cases) in [
        (
            0x03,
            [
                (0x0000, 0x0001),
                (0x00FF, 0x0100),
                (0x7FFF, 0x8000),
                (0xFFFF, 0x0000),
            ],
        ),
        (
            0x0B,
            [
                (0x0000, 0xFFFF),
                (0x0100, 0x00FF),
                (0x8000, 0x7FFF),
                (0xFFFF, 0xFFFE),
            ],
        ),
    ] {
        for pair in 0..4 {
            let opcode = base + pair * 16;
            for (value, result) in cases {
                for set in [false, true] {
                    let mut before = sample_registers(0xFFFF);
                    before.flags = flags(set, set, set, set, set);
                    put_pair(&mut before, pair, value);
                    let mut expected = before;
                    expected.pc = 0;
                    put_pair(&mut expected, pair, result);
                    let mut cpu = I8080::new(before, sample_latches());
                    bus.memory[0xFFFF] = opcode;
                    assert_eq!(
                        cpu.step(&mut bus),
                        Report {
                            steps: 1,
                            cycles: 5,
                            exit: Exit::Budget
                        }
                    );
                    assert_eq!(*cpu.registers(), expected);
                    assert_eq!(*cpu.latches(), sample_latches());
                    assert_eq!(bus.log, boundary(0xFFFF, opcode));
                    bus.log.clear();
                }
            }
        }
    }
}

#[test]
fn dad_all_pairs_changes_only_hl_and_carry() {
    // 98-153B p. 4-8 DAD: 10 states, only CY changed. pp. 2-16/2-17 and
    // p. 2-20 note 8: M2/M3 are internal; they do not reference memory.
    let mut bus = RecordingBus::new();
    for pair in 0..4 {
        for (hl, operand, result, carry) in [
            (0x0000, 0x0000, 0x0000, false),
            (0x00FF, 0x0001, 0x0100, false),
            (0x7FFF, 0x0001, 0x8000, false),
            (0xFFFF, 0x0001, 0x0000, true),
            (0xFFFF, 0xFFFF, 0xFFFE, true),
            (0x8000, 0x8000, 0x0000, true),
        ] {
            // DAD H has identical input operands.
            if pair == 2 && hl != operand {
                continue;
            }
            for set in [false, true] {
                let opcode = 0x09 + pair * 16;
                let mut before = sample_registers(0xFFFF);
                before.flags = flags(set, set, set, set, set);
                put_pair(&mut before, 2, hl);
                put_pair(&mut before, pair, operand);
                let mut expected = before;
                expected.pc = 0;
                expected.flags.carry = carry;
                put_pair(&mut expected, 2, result);
                let mut cpu = I8080::new(before, sample_latches());
                bus.memory[0xFFFF] = opcode;
                assert_eq!(
                    cpu.step(&mut bus),
                    Report {
                        steps: 1,
                        cycles: 10,
                        exit: Exit::Budget
                    }
                );
                assert_eq!(*cpu.registers(), expected);
                assert_eq!(*cpu.latches(), sample_latches());
                assert_eq!(bus.log, boundary(0xFFFF, opcode));
                bus.log.clear();
            }
        }
    }
}

#[test]
fn memory_operand_can_alias_the_fetched_opcode() {
    // 98-153B pp. 4-6 to 4-8; pp. 2-16/2-17: M1 fetch precedes
    // M2's HL read, and INR/DCR M writes the same address in M3.
    // Literal arithmetic includes the opcode byte as the data operand.
    let cases = [
        (0x86, 0x96, flags(true, false, false, true, false), None, 7),
        (0x8E, 0x9F, flags(true, false, false, true, false), None, 7),
        (0x96, 0x7A, flags(false, false, false, false, true), None, 7),
        (0x9E, 0x71, flags(false, false, false, true, true), None, 7),
        (
            0x34,
            0x10,
            flags(false, false, false, true, true),
            Some(0x35),
            10,
        ),
        (
            0x35,
            0x10,
            flags(false, false, true, false, true),
            Some(0x34),
            10,
        ),
    ];
    for (opcode, result, expected_flags, write, states) in cases {
        let mut bus = RecordingBus::new();
        bus.memory[0xFFFF] = opcode;
        let mut before = sample_registers(0xFFFF);
        before.a = 0x10;
        before.h = 0xFF;
        before.l = 0xFF;
        let mut expected = before;
        expected.pc = 0;
        expected.a = result;
        expected.flags = expected_flags;
        let mut cpu = I8080::new(before, sample_latches());
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: states,
                exit: Exit::Budget
            }
        );
        assert_eq!(*cpu.registers(), expected);
        let mut trace = boundary(0xFFFF, opcode).to_vec();
        trace.push(read(0xFFFF, MemoryAccess::Data, opcode));
        if let Some(value) = write {
            trace.push(Event::Write {
                address: 0xFFFF,
                kind: MemoryAccess::Data,
                value,
            });
        }
        assert_eq!(bus.log, trace);
        assert_eq!(bus.memory[0xFFFF], write.unwrap_or(opcode));
    }
}
