//! 8085 stack and I/O witnesses (decision-10, decision-11, decision-12).
//! Intel 205775-002 (January 1983) §5.6.5 pp. 5-15/5-16; second witness
//! Intel 9800301-04 (May 1981), cited per test. Page images checked and
//! hashes verified against PROVENANCE.md §1. Expected values use no CPU helpers.
//! Table 5-1 p. 5-19 supplies cycle sequences; Table 5-2 pins encodings.
//! Stack/OperandFetch are decision-5 item 3's API kinds; ports have no kind.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Exit, MemoryAccess, Report};

fn registers() -> Registers {
    Registers {
        a: 0x69,
        b: 0x2A,
        c: 0x4C,
        d: 0x87,
        e: 0x16,
        h: 0xB3,
        l: 0xE8,
        sp: 0x9AAF,
        pc: 0x2000,
        flags: flags(21),
    }
}

// Independent enumeration of five flags, ordered as in the p. 5-15 flag
// word from least to most significant. The index has no undefined bits.
fn flags(index: u8) -> Flags {
    Flags {
        carry: index % 2 == 1,
        parity: index / 2 % 2 == 1,
        auxiliary_carry: index / 4 % 2 == 1,
        zero: index / 8 % 2 == 1,
        sign: index / 16 % 2 == 1,
    }
}

fn fetch(opcode: u8, pc: u16) -> Vec<Event> {
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

fn read(address: u16, value: u8) -> Event {
    Event::Read {
        address,
        value,
        kind: MemoryAccess::Stack,
    }
}

fn write(address: u16, value: u8) -> Event {
    Event::Write {
        address,
        value,
        kind: MemoryAccess::Stack,
    }
}

fn complete(cpu: &I8085, report: Report, expected: Registers, states: u64) {
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: states,
            exit: Exit::Budget
        }
    );
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(*cpu.latches(), Latches::default());
}

#[test]
fn push_pairs_high_then_low_twelve_states_with_derived_wrap() {
    // 205775-002 p. 5-15, Table 5-1 p. 5-19: SWW, 12 states.
    // 9800301-04 p. 3-44: high then low, literal 2A4C/9AAF example;
    // timing there says "11 (13 on 8085)"; App. A p. A-2 says 12.
    // Follow primary entry/table, per supervisor ruling. Wrap is derived
    // from 16-bit SP/PC (205775-002 p. 2-2), not explicitly stated.
    for (opcode, high, low) in [(0xC5, 0x2A, 0x4C), (0xD5, 0x87, 0x16), (0xE5, 0xB3, 0xE8)] {
        for (pc, next_pc, sp, high_at, low_at) in [
            (0x2000, 0x2001, 0x9AAF, 0x9AAE, 0x9AAD),
            (0xFFFF, 0, 1, 0, 0xFFFF),
            (0x2000, 0x2001, 0, 0xFFFF, 0xFFFE),
        ] {
            for index in [0, 31] {
                let before = Registers {
                    pc,
                    sp,
                    flags: flags(index),
                    ..registers()
                };
                let mut cpu = I8085::new(before, Latches::default());
                let mut bus = RecordingBus::new();
                bus.load(pc, &[opcode]);
                let mut memory = bus.memory.clone();
                memory[usize::from(high_at)] = high;
                memory[usize::from(low_at)] = low;
                let report = cpu.step(&mut bus);
                complete(
                    &cpu,
                    report,
                    Registers {
                        pc: next_pc,
                        sp: low_at,
                        ..before
                    },
                    12,
                );
                assert_eq!(bus.memory, memory);
                let mut trace = fetch(opcode, pc);
                trace.extend([write(high_at, high), write(low_at, low)]);
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn pop_pairs_low_then_high_ten_states_with_derived_wrap() {
    // 205775-002 p. 5-15: low <- [SP], high <- [SP+1], SP+2,
    // flags unchanged; Table 5-1: FRR. 9800301-04 pp. 3-42/3-43 agrees.
    // SP/PC wrap cases below are derived from 16-bit widths (p. 2-2).
    for opcode in [0xC1, 0xD1, 0xE1] {
        for (pc, next_pc, sp, high_at, next_sp) in [
            (0x2000, 0x2001, 0x9AAD, 0x9AAE, 0x9AAF),
            (0xFFFF, 0, 0x3000, 0x3001, 0x3002),
            (0x2000, 0x2001, 0xFFFF, 0, 1),
            (0x2000, 0x2001, 0xFFFE, 0xFFFF, 0),
        ] {
            for index in [0, 31] {
                let before = Registers {
                    pc,
                    sp,
                    flags: flags(index),
                    ..registers()
                };
                let mut expected = Registers {
                    pc: next_pc,
                    sp: next_sp,
                    ..before
                };
                match opcode {
                    0xC1 => {
                        expected.b = 0xD2;
                        expected.c = 0x35;
                    }
                    0xD1 => {
                        expected.d = 0xD2;
                        expected.e = 0x35;
                    }
                    _ => {
                        expected.h = 0xD2;
                        expected.l = 0x35;
                    }
                }
                let mut cpu = I8085::new(before, Latches::default());
                let mut bus = RecordingBus::new();
                bus.load(pc, &[opcode]);
                bus.load(sp, &[0x35, 0xD2]);
                let memory = bus.memory.clone();
                let report = cpu.step(&mut bus);
                complete(&cpu, report, expected, 10);
                assert_eq!(bus.memory, memory);
                let mut trace = fetch(opcode, pc);
                trace.extend([read(sp, 0x35), read(high_at, 0xD2)]);
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn push_psw_all_flags_mask_hardware_bits_and_pin_library_image() {
    // 205775-002 p. 5-15: A then S Z X AC X P X CY; 12 states,
    // SWW in Table 5-1. 9800301-04 p. 3-45 agrees, including undefined
    // bits 5/3/1 on 8085. Only D5h is a hardware-derived expectation.
    // Separate full-byte check pins decision-11's library choice 0/0/1.
    for index in 0..32 {
        let before = Registers {
            flags: flags(index),
            ..registers()
        };
        let hardware = (index % 2)
            + (index / 2 % 2) * 4
            + (index / 4 % 2) * 16
            + (index / 8 % 2) * 64
            + (index / 16 % 2) * 128;
        let image = hardware + 2; // Library choice, not a hardware fact.
        let mut cpu = I8085::new(before, Latches::default());
        let mut bus = RecordingBus::new();
        bus.load(0x2000, &[0xF5]);
        let mut memory = bus.memory.clone();
        memory[0x9AAE] = 0x69;
        memory[0x9AAD] = image;
        let report = cpu.step(&mut bus);
        complete(
            &cpu,
            report,
            Registers {
                pc: 0x2001,
                sp: 0x9AAD,
                ..before
            },
            12,
        );
        assert_eq!(bus.memory[0x9AAD] & 0xD5, hardware);
        assert_eq!(bus.memory, memory); // Includes the explicitly chosen filler bits.
        let mut trace = fetch(0xF5, 0x2000);
        trace.extend([write(0x9AAE, 0x69), write(0x9AAD, image)]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn pop_psw_discards_all_undefined_bits_with_derived_wrap() {
    // 205775-002 p. 5-16 transfers only bits 0/2/4/6/7, then A,
    // increments SP twice, ten states/FRR. 9800301-04 p. 3-43 agrees.
    // SP crossing FFFF/0000 is derived (205775-002 p. 2-2).
    for byte in 0u8..=255 {
        let expected_flags = Flags {
            carry: byte % 2 == 1,
            parity: byte / 4 % 2 == 1,
            auxiliary_carry: byte / 16 % 2 == 1,
            zero: byte / 64 % 2 == 1,
            sign: byte >= 128,
        };
        let before = Registers {
            sp: 0xFFFF,
            flags: flags(31),
            ..registers()
        };
        let mut cpu = I8085::new(before, Latches::default());
        let mut bus = RecordingBus::new();
        bus.load(0x2000, &[0xF1]);
        bus.load(0xFFFF, &[byte, 0xA6]);
        let memory = bus.memory.clone();
        let expected = Registers {
            a: 0xA6,
            flags: expected_flags,
            sp: 1,
            pc: 0x2001,
            ..before
        };
        let report = cpu.step(&mut bus);
        complete(&cpu, report, expected, 10);
        assert_eq!(bus.memory, memory);
        let mut trace = fetch(0xF1, 0x2000);
        trace.extend([read(0xFFFF, byte), read(0, 0xA6)]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn xthl_sixteen_states_rrww_with_unverified_pair_order_and_derived_wrap() {
    // 205775-002 p. 5-16: exchange L/[SP], H/[SP+1], preserve flags/SP;
    // Table 5-1 p. 5-19: FRRWW, 16 states. 9800301-04 p. 3-69 agrees.
    // Unverified: these sources do not give address order within RR or WW.
    // The model chooses low/high reads then high/low writes. This test pins
    // that choice as well as the cited exchange. Wrap is derived (p. 2-2).
    for (sp, high_at) in [(0x7A2C, 0x7A2D), (0xFFFF, 0)] {
        for index in [0, 31] {
            let before = Registers {
                sp,
                flags: flags(index),
                ..registers()
            };
            let mut cpu = I8085::new(before, Latches::default());
            let mut bus = RecordingBus::new();
            bus.load(0x2000, &[0xE3]);
            bus.load(sp, &[0x41, 0x96]);
            let mut memory = bus.memory.clone();
            memory[usize::from(sp)] = 0xE8;
            memory[usize::from(high_at)] = 0xB3;
            let report = cpu.step(&mut bus);
            complete(
                &cpu,
                report,
                Registers {
                    pc: 0x2001,
                    h: 0x96,
                    l: 0x41,
                    ..before
                },
                16,
            );
            assert_eq!(bus.memory, memory);
            let mut trace = fetch(0xE3, 0x2000);
            trace.extend([
                read(sp, 0x41),
                read(high_at, 0x96),
                write(high_at, 0xB3),
                write(sp, 0xE8),
            ]);
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn sphl_six_states_fetch_only() {
    // 205775-002 p. 5-16: SP <- HL, no flags; Table 5-1: S, six states.
    // 9800301-04 p. 3-61 agrees. No data access: Ch. 2 pp. 2-7 to 2-10.
    for (h, l, sp) in [(0, 0, 0), (0xB3, 0xE8, 0xB3E8), (0xFF, 0xFF, 0xFFFF)] {
        let before = Registers {
            h,
            l,
            ..registers()
        };
        let mut cpu = I8085::new(before, Latches::default());
        let mut bus = RecordingBus::new();
        bus.load(0x2000, &[0xF9]);
        let memory = bus.memory.clone();
        let report = cpu.step(&mut bus);
        complete(
            &cpu,
            report,
            Registers {
                pc: 0x2001,
                sp,
                ..before
            },
            6,
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(bus.take_log(), fetch(0xF9, 0x2000));
    }
}

#[test]
fn in_out_ten_states_operand_then_port_with_derived_pc_wrap() {
    // 205775-002 p. 5-16; 9800301-04 pp. 3-24/3-25, 3-41:
    // port is byte 2, IN loads A, OUT sends A, preserve flags.
    // Table 5-1: FRI/FRO. §2.3.2/§2.3.3 pp. 2-12/2-13: third
    // cycle is I/O using byte 2. PC wrap is derived (p. 2-2).
    for opcode in [0xDB, 0xD3] {
        for port in [0, 0xB7, 255] {
            for (pc, operand_at, next_pc) in [(0x2000, 0x2001, 0x2002), (0xFFFF, 0, 1)] {
                for index in [0, 31] {
                    let before = Registers {
                        pc,
                        flags: flags(index),
                        ..registers()
                    };
                    let mut expected = Registers {
                        pc: next_pc,
                        ..before
                    };
                    let mut bus = RecordingBus::new();
                    bus.load(pc, &[opcode, port]);
                    let io = if opcode == 0xDB {
                        expected.a = 0x92;
                        bus.inputs.push_back(0x92);
                        Event::Input { port, value: 0x92 }
                    } else {
                        Event::Output { port, value: 0x69 }
                    };
                    let memory = bus.memory.clone();
                    let mut cpu = I8085::new(before, Latches::default());
                    let report = cpu.step(&mut bus);
                    complete(&cpu, report, expected, 10);
                    assert_eq!(bus.memory, memory);
                    let mut trace = fetch(opcode, pc);
                    trace.extend([
                        Event::Read {
                            address: operand_at,
                            kind: MemoryAccess::OperandFetch,
                            value: port,
                        },
                        io,
                    ]);
                    assert_eq!(bus.take_log(), trace);
                }
            }
        }
    }
}
