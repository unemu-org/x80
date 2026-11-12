//! The 8080 Stack, I/O and Machine Control Group (98-153B pp. 4-13 and 4-14).
//!
//! Design: decision-5 items 3, 7 and 8; plan §2.2 (registers and flags).
//! Expected values are literals or the test-side flag reference below, derived
//! from the cited manuals independently of crate helpers. Cited pages were
//! checked as images; source revisions and hashes are pinned in PROVENANCE.md.

mod support;

use support::{sample_registers, Event, RecordingBus};
use x80_core::i8080::{Flags, Latches, Registers, I8080};
use x80_core::{Budget, Exit, MemoryAccess, Report};

/// Interrupts enabled, no EI delay pending, not halted.
const LATCHES: Latches = Latches {
    interrupt_enable: true,
    ei_delay: false,
    halted: false,
};

fn nop_fetch(pc: u16) -> [Event; 3] {
    [
        Event::StopPoll(false),
        Event::InterruptPoll(false),
        Event::Read {
            address: pc,
            kind: MemoryAccess::OpcodeFetch,
            value: 0x00,
        },
    ]
}

#[test]
fn nop_takes_four_states_and_only_advances_pc() {
    // 98-153B p. 4-14: NOP is 00000000, "Cycles: 1, States: 4"; "No operation
    // is performed. The registers and flags are unaffected."
    let mut bus = RecordingBus::new();
    bus.load(0x1233, &[0x5A, 0x00, 0xA5]);
    let memory = bus.memory.clone();
    let mut cpu = I8080::new(sample_registers(0x1234), LATCHES);

    let report = cpu.step(&mut bus);

    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget,
        }
    );
    assert_eq!(*cpu.registers(), sample_registers(0x1235));
    assert_eq!(*cpu.latches(), LATCHES);
    assert!(bus.memory == memory, "memory changed");
    assert_eq!(bus.take_log(), nop_fetch(0x1234));
}

#[test]
fn nop_wraps_pc_from_ffff_to_0000() {
    // PC is a 16-bit register (98-153B p. 2-2). Memory is all zero, so 0x0000
    // also holds a NOP.
    let mut bus = RecordingBus::new();
    let mut cpu = I8080::new(sample_registers(0xFFFF), LATCHES);

    let report = cpu.run(
        &mut bus,
        Budget {
            steps: 2,
            cycles: u64::MAX,
        },
    );

    assert_eq!(
        report,
        Report {
            steps: 2,
            cycles: 8,
            exit: Exit::Budget,
        }
    );
    assert_eq!(*cpu.registers(), sample_registers(0x0001));
    assert_eq!(
        bus.take_log(),
        [nop_fetch(0xFFFF), nop_fetch(0x0000)].concat()
    );
}

// These helpers describe the expected trace, not CPU semantics. Opcode fetch
// and polls follow decision-5 items 3-5; Stack follows 98-153B p. 2-20 notes
// 15-16. Port events are Bus::input/output; those methods have no kind argument.
fn fetch(pc: u16, opcode: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::InterruptPoll(false),
        Event::Read {
            address: pc,
            kind: MemoryAccess::OpcodeFetch,
            value: opcode,
        },
    ]
}

fn stack_read(address: u16, value: u8) -> Event {
    Event::Read {
        address,
        kind: MemoryAccess::Stack,
        value,
    }
}

fn stack_write(address: u16, value: u8) -> Event {
    Event::Write {
        address,
        kind: MemoryAccess::Stack,
        value,
    }
}

fn assert_completed(cpu: &I8080, report: Report, expected: Registers, states: u64) {
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: states,
            exit: Exit::Budget
        }
    );
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(*cpu.latches(), LATCHES);
}

// Test-side transcription of 98-153B Ch. 4 p. 4-13 POP PSW transfer list:
// only bits 7, 6, 4, 2, 0 have flag destinations. Second witness:
// 9800301-04 Ch. 3 pp. 3-43, 3-45 (bits 1, 3, 5 are filler).
fn reference_flags(byte: u8) -> Flags {
    Flags {
        sign: byte & 0x80 != 0,
        zero: byte & 0x40 != 0,
        auxiliary_carry: byte & 0x10 != 0,
        parity: byte & 0x04 != 0,
        carry: byte & 0x01 != 0,
    }
}

#[test]
fn push_each_pair_writes_high_then_low_in_eleven_states() {
    // 98-153B Ch. 4 p. 4-13 PUSH rp: B/C, D/E, H/L, flags unchanged,
    // SP -= 2, 11 states. Ch. 2 pp. 2-18, 2-19: M2 writes rh at SP-1,
    // M3 writes rl at SP-2, both stack writes (p. 2-20 note 16).
    // Second witness: 9800301-04 Ch. 3 p. 3-44 PUSH.
    // PC/SP wrap is derived from their 16-bit width (98-153B p. 2-2).
    for (opcode, high, low) in [(0xC5, 0xB2, 0xC3), (0xD5, 0xD4, 0xE5), (0xE5, 0xF6, 0x07)] {
        for (pc, next_pc, sp, high_at, low_at) in [
            (0x1234, 0x1235, 0x8F90, 0x8F8F, 0x8F8E),
            (0xFFFF, 0x0000, 0x0001, 0x0000, 0xFFFF),
            (0x1234, 0x1235, 0x0000, 0xFFFF, 0xFFFE),
        ] {
            for flags in [reference_flags(0x00), reference_flags(0xD5)] {
                let mut bus = RecordingBus::new();
                bus.load(pc, &[opcode]);
                let mut memory = bus.memory.clone();
                memory[usize::from(high_at)] = high;
                memory[usize::from(low_at)] = low;
                let registers = Registers {
                    sp,
                    flags,
                    ..sample_registers(pc)
                };
                let mut cpu = I8080::new(registers, LATCHES);
                let report = cpu.step(&mut bus);
                assert_completed(
                    &cpu,
                    report,
                    Registers {
                        pc: next_pc,
                        sp: low_at,
                        ..registers
                    },
                    11,
                );
                assert_eq!(bus.memory, memory);
                let mut trace = fetch(pc, opcode);
                trace.extend([stack_write(high_at, high), stack_write(low_at, low)]);
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn pop_each_pair_reads_low_then_high_in_ten_states() {
    // 98-153B Ch. 4 p. 4-13 POP rp: rl = [SP], rh = [SP+1], SP += 2,
    // flags unchanged, 10 states. Ch. 2 pp. 2-18, 2-19: M2 then M3 stack
    // reads (p. 2-20 note 15). Second witness: 9800301-04 Ch. 3 p. 3-43.
    for opcode in [0xC1, 0xD1, 0xE1] {
        for (pc, next_pc, sp, high_at, next_sp) in [
            (0x1234, 0x1235, 0x8F90, 0x8F91, 0x8F92),
            (0xFFFF, 0x0000, 0x4000, 0x4001, 0x4002),
            (0x1234, 0x1235, 0xFFFF, 0x0000, 0x0001),
        ] {
            for flags in [reference_flags(0x00), reference_flags(0xD5)] {
                let mut bus = RecordingBus::new();
                bus.load(pc, &[opcode]);
                bus.load(sp, &[0x3C, 0xA5]);
                let memory = bus.memory.clone();
                let registers = Registers {
                    sp,
                    flags,
                    ..sample_registers(pc)
                };
                let mut expected = Registers {
                    pc: next_pc,
                    sp: next_sp,
                    ..registers
                };
                match opcode {
                    0xC1 => {
                        expected.b = 0xA5;
                        expected.c = 0x3C;
                    }
                    0xD1 => {
                        expected.d = 0xA5;
                        expected.e = 0x3C;
                    }
                    0xE1 => {
                        expected.h = 0xA5;
                        expected.l = 0x3C;
                    }
                    _ => unreachable!(),
                }
                let mut cpu = I8080::new(registers, LATCHES);
                let report = cpu.step(&mut bus);
                assert_completed(&cpu, report, expected, 10);
                assert_eq!(bus.memory, memory);
                let mut trace = fetch(pc, opcode);
                trace.extend([stack_read(sp, 0x3C), stack_read(high_at, 0xA5)]);
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn push_pop_psw_round_trip_uses_literal_flag_images() {
    // 98-153B Ch. 4 p. 4-13 FLAG WORD and PUSH/POP PSW: fixed bits 1=1,
    // 3=5=0; A high, flags low; 11/10 states. Stack order: Ch. 2 pp. 2-18
    // to 2-20. Second witness: 9800301-04 Ch. 3 pp. 3-43, 3-45.
    for image in [0x02, 0xD7, 0x93, 0x46, 0x82, 0x42, 0x12, 0x06, 0x03] {
        let registers = Registers {
            flags: reference_flags(image),
            ..sample_registers(0x1234)
        };
        let mut bus = RecordingBus::new();
        bus.load(0x1234, &[0xF5, 0xF1]);
        let mut memory = bus.memory.clone();
        memory[0x8F8F] = 0xA1;
        memory[0x8F8E] = image;
        let mut cpu = I8080::new(registers, LATCHES);
        let pushed = cpu.step(&mut bus);
        assert_completed(
            &cpu,
            pushed,
            Registers {
                pc: 0x1235,
                sp: 0x8F8E,
                ..registers
            },
            11,
        );
        assert_eq!(bus.memory, memory);
        let mut trace = fetch(0x1234, 0xF5);
        trace.extend([stack_write(0x8F8F, 0xA1), stack_write(0x8F8E, image)]);
        assert_eq!(bus.take_log(), trace);

        // Disturb every flag and A so POP must restore, rather than preserve.
        cpu.registers_mut().a = 0x5E;
        cpu.registers_mut().flags = reference_flags(!image);
        let popped = cpu.step(&mut bus);
        assert_completed(
            &cpu,
            popped,
            Registers {
                pc: 0x1236,
                ..registers
            },
            10,
        );
        assert_eq!(bus.memory, memory);
        let mut trace = fetch(0x1235, 0xF1);
        trace.extend([stack_read(0x8F8E, image), stack_read(0x8F8F, 0xA1)]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn pop_psw_arbitrary_bytes_drop_filler_bits_before_repush() {
    // 98-153B Ch. 4 p. 4-13 POP PSW loads only bits 0,2,4,6,7; PUSH PSW
    // forces 1,3,5 to 1,0,0. Second witness: 9800301-04 Ch. 3 pp. 3-43,
    // 3-45. Every possible stack flag byte, with all flags initially opposite.
    // Trace from 98-153B Ch. 2 pp. 2-18 to 2-20; SP crosses the 16-bit edge.
    for byte in 0..=255 {
        let mut bus = RecordingBus::new();
        bus.load(0x2345, &[0xF1, 0xF5]);
        bus.load(0xFFFF, &[byte, 0x5A]);
        let mut memory = bus.memory.clone();
        let registers = Registers {
            sp: 0xFFFF,
            flags: reference_flags(!byte),
            ..sample_registers(0x2345)
        };
        let mut cpu = I8080::new(registers, LATCHES);
        let expected = Registers {
            a: 0x5A,
            flags: reference_flags(byte),
            sp: 0x0001,
            pc: 0x2346,
            ..registers
        };
        let report = cpu.step(&mut bus);
        assert_completed(&cpu, report, expected, 10);
        assert_eq!(bus.memory, memory);
        let mut trace = fetch(0x2345, 0xF1);
        trace.extend([stack_read(0xFFFF, byte), stack_read(0x0000, 0x5A)]);
        assert_eq!(bus.take_log(), trace);

        // Independent reference: preserve the five flag positions, set bit 1.
        let normalized = (byte & 0xD5) | 0x02;
        let report = cpu.step(&mut bus);
        assert_completed(
            &cpu,
            report,
            Registers {
                pc: 0x2347,
                sp: 0xFFFF,
                ..expected
            },
            11,
        );
        memory[0xFFFF] = normalized;
        assert_eq!(bus.memory, memory);
        let mut trace = fetch(0x2346, 0xF5);
        trace.extend([stack_write(0x0000, 0x5A), stack_write(0xFFFF, normalized)]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn xthl_reads_both_bytes_then_writes_high_before_low_in_eighteen_states() {
    // 98-153B Ch. 4 p. 4-14 XTHL: exchange L/[SP], H/[SP+1], no flags,
    // 18 states. Ch. 2 pp. 2-18, 2-19: M2 read low, M3 read high, M4
    // write H at SP+1, M5 write L at SP; p. 2-20 notes 15-16: Stack.
    // Second witness: 9800301-04 Ch. 3 p. 3-69, including its literal example.
    for (pc, next_pc, sp, high_at) in [
        (0x1234, 0x1235, 0x10AD, 0x10AE),
        (0xFFFF, 0x0000, 0x4000, 0x4001),
        (0x1234, 0x1235, 0xFFFF, 0x0000),
    ] {
        for flags in [reference_flags(0x00), reference_flags(0xD5)] {
            let mut bus = RecordingBus::new();
            bus.load(pc, &[0xE3]);
            bus.load(sp, &[0xF0, 0x0D]);
            let mut memory = bus.memory.clone();
            memory[usize::from(sp)] = 0x3C;
            memory[usize::from(high_at)] = 0x0B;
            let registers = Registers {
                h: 0x0B,
                l: 0x3C,
                sp,
                flags,
                ..sample_registers(pc)
            };
            let mut cpu = I8080::new(registers, LATCHES);
            let report = cpu.step(&mut bus);
            assert_completed(
                &cpu,
                report,
                Registers {
                    h: 0x0D,
                    l: 0xF0,
                    pc: next_pc,
                    ..registers
                },
                18,
            );
            assert_eq!(bus.memory, memory);
            let mut trace = fetch(pc, 0xE3);
            trace.extend([
                stack_read(sp, 0xF0),
                stack_read(high_at, 0x0D),
                stack_write(high_at, 0x0B),
                stack_write(sp, 0x3C),
            ]);
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn sphl_copies_hl_without_memory_access_in_five_states() {
    // 98-153B Ch. 4 p. 4-14 SPHL: SP = HL, flags unchanged, 5 states;
    // Ch. 2 p. 2-16 SPHL: only M1, no M2. Second witness: 9800301-04
    // Ch. 3 p. 3-61, including the literal 50FF example.
    for (pc, next_pc) in [(0x1234, 0x1235), (0xFFFF, 0x0000)] {
        for flags in [reference_flags(0x00), reference_flags(0xD5)] {
            let registers = Registers {
                h: 0x50,
                l: 0xFF,
                flags,
                ..sample_registers(pc)
            };
            let mut cpu = I8080::new(registers, LATCHES);
            let mut bus = RecordingBus::new();
            bus.load(pc, &[0xF9]);
            let memory = bus.memory.clone();
            let report = cpu.step(&mut bus);
            assert_completed(
                &cpu,
                report,
                Registers {
                    sp: 0x50FF,
                    pc: next_pc,
                    ..registers
                },
                5,
            );
            assert_eq!(bus.memory, memory);
            assert_eq!(bus.take_log(), fetch(pc, 0xF9));
        }
    }
}

#[test]
fn in_out_fetch_port_before_io_and_preserve_flags_in_ten_states() {
    // 98-153B Ch. 4 p. 4-14 IN/OUT: port is byte 2; IN changes only A,
    // OUT sends A, no flags, 10 states each. Ch. 2 p. 2-9 Figs. 2-6, 2-7
    // and pp. 2-18, 2-19: M1 opcode, M2 byte 2, M3 input/output. OperandFetch
    // is decision-5 item 3's instruction-stream distinction. Port calls have
    // no MemoryAccess argument. Second witness: 9800301-04 Ch. 3 pp. 3-24,
    // 3-25 (IN), p. 3-41 (OUT).
    for opcode in [0xDB, 0xD3] {
        for port in [0x00, 0x7E, 0xFF] {
            for (pc, operand_at, next_pc) in [(0x1234, 0x1235, 0x1236), (0xFFFF, 0x0000, 0x0001)] {
                for flags in [reference_flags(0x00), reference_flags(0xD5)] {
                    let mut bus = RecordingBus::new();
                    bus.load(pc, &[opcode, port]);
                    bus.inputs.push_back(0x5A);
                    let memory = bus.memory.clone();
                    let registers = Registers {
                        flags,
                        ..sample_registers(pc)
                    };
                    let mut expected = Registers {
                        pc: next_pc,
                        ..registers
                    };
                    let io = if opcode == 0xDB {
                        expected.a = 0x5A;
                        Event::Input { port, value: 0x5A }
                    } else {
                        Event::Output { port, value: 0xA1 }
                    };
                    let mut cpu = I8080::new(registers, LATCHES);
                    let report = cpu.step(&mut bus);
                    assert_completed(&cpu, report, expected, 10);
                    assert_eq!(bus.memory, memory);
                    let mut trace = fetch(pc, opcode);
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
