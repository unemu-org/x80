//! 8085 arithmetic witnesses: plan §2.2; decisions 10 and 12.
//! Intel 205775-002 (January 1983) §5.6.2 pp. 5-6 to 5-9 (PDF 91-94),
//! §5.5 p. 5-3 (PDF 88). Independent Intel 9800301-04 (May 1981) pages
//! are cited per test. All cited values checked on rendered page images.
//! Bus sequences: Table 5-1 p. 5-19 (PDF 104), NOT the opcode-only
//! Table 5-2 p. 5-20; Ch. 2 pp. 2-8 to 2-12 (PDF 31-35).

mod support;

// The existing reference uses crate::i8085 in ALU unit tests, and this
// import supplies the same type path here without sharing crate helpers.
use x80_core::i8085;
#[path = "i8085_arithmetic_reference/mod.rs"]
mod reference;

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

// Mechanical decision-18 migration: compare every original register and
// five-flag expectation; V/K have separate literals in i8085_vk_flags.rs.
fn documented(mut registers: Registers) -> Registers {
    registers.flags.overflow = false;
    registers.flags.k = false;
    registers
}

fn sample() -> Registers {
    Registers {
        a: 0x83,
        b: 0x17,
        c: 0x29,
        d: 0x3B,
        e: 0x4D,
        h: 0x65,
        l: 0x79,
        sp: 0x8BAD,
        pc: 0x2000,
        flags: flags(true, false, true, false, true),
    }
}

fn latches() -> Latches {
    Latches {
        interrupt_enable: true,
        ei_delay: true,
        sod: true,
        ..Latches::default()
    }
}

fn completed() -> Latches {
    Latches {
        ei_delay: false,
        ..latches()
    }
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

fn boundary(pc: u16, opcode: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::TrapPoll(false),
        Event::Rst7_5Poll(false),
        Event::Rst6_5Poll(false),
        Event::Rst5_5Poll(false),
        Event::IntrPoll(false),
        read(pc, MemoryAccess::OpcodeFetch, opcode),
    ]
}

fn step(cpu: &mut I8085, bus: &mut RecordingBus, states: u64) {
    assert_eq!(
        cpu.step(bus),
        Report {
            steps: 1,
            cycles: states,
            exit: Exit::Budget
        }
    );
    assert_eq!(*cpu.latches(), completed());
}

// Test-owned register access; no production pair or flag-image helpers.
fn register(r: &Registers, code: u8) -> u8 {
    [r.b, r.c, r.d, r.e, r.h, r.l, 0, r.a][usize::from(code)]
}

fn set_register(r: &mut Registers, code: u8, value: u8) {
    match code {
        0 => r.b = value,
        1 => r.c = value,
        2 => r.d = value,
        3 => r.e = value,
        4 => r.h = value,
        5 => r.l = value,
        7 => r.a = value,
        _ => panic!("memory is not a register"),
    }
}

fn set_pair(r: &mut Registers, code: u8, value: u16) {
    let high = (value / 256) as u8;
    let low = (value % 256) as u8;
    match code {
        0 => {
            r.b = high;
            r.c = low;
        }
        1 => {
            r.d = high;
            r.e = low;
        }
        2 => {
            r.h = high;
            r.l = low;
        }
        3 => r.sp = value,
        _ => panic!("invalid pair"),
    }
}

#[test]
fn exhaustive_add_adc_sub_sbb_register_memory_and_immediate() {
    // Primary: ADD/ADI/ADC p. 5-6, ADC/ACI/SUB/SUI/SBB p. 5-7,
    // SBB/SBI p. 5-8. Second witness: 9800301-04 pp. 3-2 to 3-5,
    // 3-56 to 3-58, 3-63 to 3-65 (PDF = 54 + printed page).
    // Each operation: all 256 A x 256 operand x both initial CY, through
    // B, M, and immediate. The reference propagates carries bit by bit.
    let mut bus = RecordingBus::new();
    let mut cases = 0;
    for (base, immediate, subtract, uses_carry) in [
        (0x80, 0xC6, false, false),
        (0x88, 0xCE, false, true),
        (0x90, 0xD6, true, false),
        (0x98, 0xDE, true, true),
    ] {
        for (opcode, length, states) in [(base, 1, 4), (base + 6, 1, 7), (immediate, 2, 7)] {
            bus.memory[0x2000] = opcode;
            for a in 0..=255 {
                for operand in 0..=255 {
                    for cy in [false, true] {
                        let (value, expected_flags) =
                            reference::arithmetic(a, operand, cy && uses_carry, subtract);
                        let before = Registers {
                            a,
                            b: operand,
                            flags: flags(
                                !expected_flags.sign,
                                !expected_flags.zero,
                                !expected_flags.auxiliary_carry,
                                !expected_flags.parity,
                                cy,
                            ),
                            ..sample()
                        };
                        bus.memory[0x2001] = operand;
                        bus.memory[0x6579] = operand;
                        bus.log.clear();
                        let mut cpu = I8085::new(before, latches());
                        step(&mut cpu, &mut bus, states);
                        assert_eq!(
                            documented(*cpu.registers()),
                            Registers {
                                a: value,
                                flags: expected_flags,
                                pc: 0x2000 + length,
                                ..before
                            },
                            "opcode={opcode:02X}, A={a:02X}, operand={operand:02X}, CY={cy}"
                        );
                        assert_eq!(bus.memory[0x6579], operand);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 1_572_864);
}

#[test]
fn every_register_source_including_a_is_sampled_before_writeback() {
    // Same instruction citations as exhaustive_add_adc_sub_sbb_... .
    // DDD/SSS: 205775-002 §5.2 p. 5-1 / Table 5-1 p. 5-19;
    // 9800301-04 pp. 3-3/3-4/3-56/3-63. F only, 4 states.
    for (base, subtract, with_carry) in [
        (0x80, false, false),
        (0x88, false, true),
        (0x90, true, false),
        (0x98, true, true),
    ] {
        for source in [0, 1, 2, 3, 4, 5, 7] {
            for cy in [false, true] {
                let before = Registers {
                    flags: flags(false, true, false, true, cy),
                    ..sample()
                };
                let (a, expected_flags) = reference::arithmetic(
                    before.a,
                    register(&before, source),
                    cy && with_carry,
                    subtract,
                );
                let opcode = base + source;
                let mut bus = RecordingBus::new();
                bus.memory[0x2000] = opcode;
                let memory = bus.memory.clone();
                let mut cpu = I8085::new(before, latches());
                step(&mut cpu, &mut bus, 4);
                assert_eq!(
                    documented(*cpu.registers()),
                    Registers {
                        a,
                        flags: expected_flags,
                        pc: 0x2001,
                        ..before
                    }
                );
                assert_eq!(bus.log, boundary(0x2000, opcode));
                assert_eq!(bus.memory, memory);
            }
        }
    }
}

#[test]
fn literal_memory_and_immediate_results_and_exact_access_kinds() {
    // Re-derived from 205775-002 §5.5 p. 5-3 and arithmetic entries
    // pp. 5-6 to 5-8; second witness instruction pages as above.
    // A=0FH, operand=01H, initial CY=1: plain add/sub ignore that input.
    // Table 5-1: F R, 7 states. MR: Ch. 2 pp. 2-10/2-12.
    for (memory_op, immediate_op, value, expected_flags) in [
        (0x86, 0xC6, 0x10, flags(false, false, true, false, false)),
        (0x8E, 0xCE, 0x11, flags(false, false, true, true, false)),
        (0x96, 0xD6, 0x0E, flags(false, false, true, false, false)),
        (0x9E, 0xDE, 0x0D, flags(false, false, true, false, false)),
    ] {
        for (opcode, address, kind, next_pc) in [
            (memory_op, 0x6579, MemoryAccess::Data, 0x2001),
            (immediate_op, 0x2001, MemoryAccess::OperandFetch, 0x2002),
        ] {
            let mut bus = RecordingBus::new();
            bus.memory[0x2000] = opcode;
            bus.memory[usize::from(address)] = 1;
            let memory = bus.memory.clone();
            let before = Registers {
                a: 0x0F,
                ..sample()
            };
            let mut cpu = I8085::new(before, latches());
            step(&mut cpu, &mut bus, 7);
            assert_eq!(
                documented(*cpu.registers()),
                Registers {
                    a: value,
                    flags: expected_flags,
                    pc: next_pc,
                    ..before
                }
            );
            let mut trace = boundary(0x2000, opcode);
            trace.push(read(address, kind, 1));
            assert_eq!(bus.log, trace);
            assert_eq!(bus.memory, memory);
        }
    }
}

#[test]
fn inr_dcr_all_bytes_all_destinations_preserve_carry() {
    // 205775-002 p. 5-8: Z/S/P/AC only; 9800301-04 pp. 3-20/3-21
    // DCR and 3-25 INR. AC: §5.5 p. 5-3 includes decrements; p. 5-6
    // specifies two's complement, hence AC = no low-nibble borrow for
    // DCR. Second witness Ch. 1 pp. 1-11/1-12. Byte wrap is derived.
    let mut bus = RecordingBus::new();
    for (base, decrement) in [(0x04, false), (0x05, true)] {
        for destination in 0..8 {
            let opcode = base + destination * 8;
            for operand in 0..=255 {
                for cy in [false, true] {
                    let (value, mut expected_flags) =
                        reference::arithmetic(operand, 1, false, decrement);
                    expected_flags.carry = cy;
                    let mut before = sample();
                    before.flags = flags(
                        !expected_flags.sign,
                        !expected_flags.zero,
                        !expected_flags.auxiliary_carry,
                        !expected_flags.parity,
                        cy,
                    );
                    if destination != 6 {
                        set_register(&mut before, destination, operand);
                    }
                    bus.memory[0x2000] = opcode;
                    bus.memory[0x6579] = operand;
                    let mut expected = before;
                    expected.pc = 0x2001;
                    expected.flags = expected_flags;
                    let mut trace = boundary(0x2000, opcode);
                    let states = if destination == 6 {
                        // Table 5-1 F R W, Ch. 2 §§2.3.2/2.3.3.
                        trace.push(read(0x6579, MemoryAccess::Data, operand));
                        trace.push(Event::Write {
                            address: 0x6579,
                            kind: MemoryAccess::Data,
                            value,
                        });
                        10
                    } else {
                        set_register(&mut expected, destination, value);
                        4
                    };
                    bus.log.clear();
                    let mut cpu = I8085::new(before, latches());
                    step(&mut cpu, &mut bus, states);
                    assert_eq!(
                        documented(*cpu.registers()),
                        expected,
                        "opcode={opcode:02X}, byte={operand:02X}, CY={cy}"
                    );
                    assert_eq!(bus.log, trace);
                    assert_eq!(
                        bus.memory[0x6579],
                        if destination == 6 { value } else { operand }
                    );
                }
            }
        }
    }
}

#[test]
fn literal_inr_dcr_auxiliary_carry_boundaries() {
    // Same 8085 citations as inr_dcr_all_bytes... . In particular DCR
    // 10H -> 0FH clears AC, while 11H -> 10H sets it (complemented add).
    // FFH -> 00H / 00H -> FFH: derived 8-bit wrap witnesses.
    for (opcode, operand, value, expected_flags) in [
        (0x04, 0x0F, 0x10, flags(false, false, true, false, true)),
        (0x04, 0xFF, 0x00, flags(false, true, true, true, true)),
        (0x05, 0x10, 0x0F, flags(false, false, false, true, true)),
        (0x05, 0x11, 0x10, flags(false, false, true, false, true)),
        (0x05, 0x00, 0xFF, flags(true, false, false, true, true)),
    ] {
        let before = Registers {
            b: operand,
            ..sample()
        };
        let mut bus = RecordingBus::new();
        bus.memory[0x2000] = opcode;
        let mut cpu = I8085::new(before, latches());
        step(&mut cpu, &mut bus, 4);
        assert_eq!(
            documented(*cpu.registers()),
            Registers {
                b: value,
                flags: expected_flags,
                pc: 0x2001,
                ..before
            }
        );
        assert_eq!(bus.log, boundary(0x2000, opcode));
    }
}

#[test]
fn inx_dcx_pairs_six_states_no_flags_and_derived_wrap() {
    // 205775-002 p. 5-9; 9800301-04 pp. 3-22/3-26. No flags,
    // S fetch only (Table 5-1); Ch. 2 pp. 2-9/2-10 Fig. 2-13.
    // FFFF/0000 wraps are derived from 16-bit width; the second witness
    // also explicitly gives INX SP FFFFH -> 0000H on p. 3-26.
    for (base, input, output) in [
        (0x03, 0x12FF, 0x1300),
        (0x0B, 0x9800, 0x97FF),
        (0x03, 0xFFFF, 0x0000),
        (0x0B, 0x0000, 0xFFFF),
    ] {
        for pair in 0..4 {
            for bits in 0..32 {
                let mut before = sample();
                before.flags = flags(
                    bits & 1 != 0,
                    bits & 2 != 0,
                    bits & 4 != 0,
                    bits & 8 != 0,
                    bits & 16 != 0,
                );
                set_pair(&mut before, pair, input);
                let mut expected = before;
                set_pair(&mut expected, pair, output);
                expected.pc = 0x2001;
                let opcode = base + pair * 16;
                let mut bus = RecordingBus::new();
                bus.memory[0x2000] = opcode;
                let memory = bus.memory.clone();
                let mut cpu = I8085::new(before, latches());
                step(&mut cpu, &mut bus, 6);
                assert_eq!(documented(*cpu.registers()), expected);
                assert_eq!(bus.log, boundary(0x2000, opcode));
                assert_eq!(bus.memory, memory);
            }
        }
    }
}

#[test]
fn dad_all_pairs_only_carry_ten_states_no_bus_idle_calls_and_derived_wrap() {
    // 205775-002 p. 5-9; 9800301-04 p. 3-20. Table 5-1 F B B:
    // B means bus idle, not memory read (Ch. 2 Fig. 2-10 p. 2-8 and
    // Fig. 2-12 p. 2-9; 231718-001 Tables 3/4 handbook p. 16-20).
    // Carry and no-carry paths both take 10 states. Wrapped HL is derived.
    for pair in 0..4 {
        let cases: &[(u16, u16, u16, bool)] = if pair == 2 {
            &[
                (0x1234, 0x1234, 0x2468, false),
                (0x8000, 0x8000, 0, true),
                (0xFFFF, 0xFFFF, 0xFFFE, true),
            ]
        } else {
            &[
                (0x1234, 0x0102, 0x1336, false),
                (0xFFFF, 1, 0, true),
                (0x8000, 0x8001, 1, true),
            ]
        };
        for &(hl, operand, result, carry) in cases {
            for bits in 0..32 {
                let mut before = sample();
                before.flags = flags(
                    bits & 1 != 0,
                    bits & 2 != 0,
                    bits & 4 != 0,
                    bits & 8 != 0,
                    bits & 16 != 0,
                );
                set_pair(&mut before, 2, hl);
                set_pair(&mut before, pair, operand);
                let mut expected = before;
                set_pair(&mut expected, 2, result);
                expected.pc = 0x2001;
                expected.flags.carry = carry;
                let opcode = 0x09 + pair * 16;
                let mut bus = RecordingBus::new();
                bus.memory[0x2000] = opcode;
                let memory = bus.memory.clone();
                let mut cpu = I8085::new(before, latches());
                step(&mut cpu, &mut bus, 10);
                assert_eq!(documented(*cpu.registers()), expected);
                assert_eq!(bus.log, boundary(0x2000, opcode));
                assert_eq!(bus.memory, memory);
            }
        }
    }
}

#[test]
fn derived_pc_wrap_immediates_and_memory_aliasing_opcode() {
    // Derived PC wrap, not stated on the instruction pages. Immediate
    // byte 2 at 0000H follows opcode at FFFFH (§5.4 p. 5-3).
    for (opcode, value, expected_flags) in [
        (0xC6, 0x20, flags(false, false, true, false, false)),
        (0xCE, 0x21, flags(false, false, true, true, false)),
        (0xD6, 0xFE, flags(true, false, true, false, true)),
        (0xDE, 0xFD, flags(true, false, true, false, true)),
    ] {
        let before = Registers {
            a: 0x0F,
            pc: 0xFFFF,
            ..sample()
        };
        let mut bus = RecordingBus::new();
        bus.load(0xFFFF, &[opcode, 0x11]);
        let mut cpu = I8085::new(before, latches());
        step(&mut cpu, &mut bus, 7);
        assert_eq!(
            documented(*cpu.registers()),
            Registers {
                a: value,
                flags: expected_flags,
                pc: 1,
                ..before
            }
        );
        let mut trace = boundary(0xFFFF, opcode);
        trace.push(read(0, MemoryAccess::OperandFetch, 0x11));
        assert_eq!(bus.log, trace);
    }
    // Fetch before data read; INR/DCR write only after the read. HL and
    // PC alias at FFFFH. Table 5-1 F R / F R W, Ch. 2 pp. 2-8 to 2-12.
    for (opcode, value, expected_flags, write, states) in [
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
    ] {
        let before = Registers {
            a: 0x10,
            h: 0xFF,
            l: 0xFF,
            pc: 0xFFFF,
            ..sample()
        };
        let mut bus = RecordingBus::new();
        bus.memory[0xFFFF] = opcode;
        let mut cpu = I8085::new(before, latches());
        step(&mut cpu, &mut bus, states);
        assert_eq!(
            documented(*cpu.registers()),
            Registers {
                a: value,
                flags: expected_flags,
                pc: 0,
                ..before
            }
        );
        let mut trace = boundary(0xFFFF, opcode);
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
