//! Decision-20 §4 replaces zero Y/X and former unknown preservation here.
//! H class rules: Young v0.91 §§4.1–4.3 pp. 15–17 / §§8.1–8.11 pp. 26–35;
//! Rak z80test v1.2a corresponding load/ALU/BIT/block-I/O groups, pinned in
//! PROVENANCE §1. SCF/CCF stay interim L until §6; parity-window rules stay L.
//! Documented Z80 I/O witnesses: decisions 13–15.
//!
//! Primary: UM008011-0816, rev. 11 (August 2016), Input and Output Group,
//! pp. 295–316 (PDF = printed + 14). Second witness: 03-0029-01 (1977),
//! Table 7.0-11 p. 54 (PDF 60); IN H reset also 03-0002-01 (January 1978),
//! p. 255 (PDF 265). Values read from rendered page images, not OCR.
//! Polls, access kinds, R increments and iteration retirement follow
//! decision-15 rules 1, 4–6, 11. Decision-20 §4 block flags are compared
//! as full bytes; source and evidence limits are given above.
//! Fail-before: all positive witnesses return Unsupported on base 5f5f6e8,
//! before any operand, Data or port access. Decision-20 §1 ED70/71
//! admission is witnessed separately in z80_undocumented_ed.rs.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, MemoryAccess, Report};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ARG: MemoryAccess = MemoryAccess::OperandFetch;
const DATA: MemoryAccess = MemoryAccess::Data;

fn registers() -> Registers {
    Registers {
        a: 0xA1,
        flags: Flags::from_byte(0xD7),
        b: 0xB2,
        c: 0xC3,
        d: 0xD4,
        e: 0xE5,
        h: 0xF6,
        l: 0x07,
        a_alternate: 0x18,
        flags_alternate: Flags::from_byte(0x42),
        b_alternate: 0x29,
        c_alternate: 0x3A,
        d_alternate: 0x4B,
        e_alternate: 0x5C,
        h_alternate: 0x6D,
        l_alternate: 0x7E,
        ix: 0x8091,
        iy: 0x92A3,
        sp: 0xDEF0,
        pc: 0x2000,
        i: 0xF8,
        r: 0xFF,
    }
}

fn latches() -> Latches {
    Latches {
        iff1: true,
        iff2: true,
        interrupt_mode: InterruptMode::Mode2,
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

fn write(address: u16, value: u8) -> Event {
    Event::Write {
        address,
        kind: DATA,
        value,
    }
}

fn fetch(pc: u16, opcode: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
        read(pc, OP, opcode),
    ]
}

fn ed_fetch(pc: u16, key_at: u16, opcode: u8) -> Vec<Event> {
    let mut trace = fetch(pc, 0xED);
    trace.push(read(key_at, OP, opcode));
    trace
}

fn retired(steps: u64, cycles: u64) -> Report {
    Report {
        steps,
        cycles,
        exit: Exit::Budget,
    }
}

fn complete(cpu: &Z80, report: Report, expected: Registers, cycles: u64, wz: u16) {
    assert_eq!(report, retired(1, cycles));
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(*cpu.latches(), Latches { wz, ..latches() });
}

#[test]
fn immediate_input_uses_old_a_for_the_full_port_and_preserves_flags() {
    // UM IN A,(n) p. 295; TM Table 7.0-11 p. 54: 11 states, no flags.
    // PC wrap is derived from its 16-bit width (UM Registers p. 3).
    for (pc, operand_at, next_pc) in [
        (0x2000, 0x2001, 0x2002),
        (0xFFFE, 0xFFFF, 0),
        (0xFFFF, 0, 1),
    ] {
        for (a, n, port, value) in [(0x23, 0x01, 0x2301_u16, 0x7B), (0x80, 0xFF, 0x80FF, 0)] {
            for flags in [0x00, 0xD7] {
                let before = Registers {
                    a,
                    pc,
                    flags: Flags::from_byte(flags),
                    ..registers()
                };
                let mut cpu = Z80::new(before, latches());
                let mut bus = RecordingBus::new();
                bus.load(pc, &[0xDB, n]);
                bus.inputs.push_back(value);
                let memory = bus.memory.clone();
                let report = cpu.step(&mut bus);
                complete(
                    &cpu,
                    report,
                    Registers {
                        a: value,
                        pc: next_pc,
                        r: 0x80,
                        ..before
                    },
                    11,
                    port.wrapping_add(1),
                );
                assert_eq!(bus.memory, memory);
                let mut trace = fetch(pc, 0xDB);
                trace.extend([read(operand_at, ARG, n), Event::Input16 { port, value }]);
                assert_eq!(bus.take_log(), trace);
                assert!(bus.inputs.is_empty());
            }
        }
    }
}

#[test]
fn immediate_output_places_a_on_both_data_and_high_address_bits() {
    // UM OUT (n),A p. 306; TM Table 7.0-11 p. 54: 11 states, no flags.
    // The FFFFh instruction-stream wrap is derived from 16-bit PC.
    for flags in [0x00, 0xD7] {
        let before = Registers {
            a: 0x23,
            pc: 0xFFFF,
            flags: Flags::from_byte(flags),
            ..registers()
        };
        let mut cpu = Z80::new(before, latches());
        let mut bus = RecordingBus::new();
        bus.load(0xFFFF, &[0xD3, 0x01]);
        let memory = bus.memory.clone();
        let report = cpu.step(&mut bus);
        complete(
            &cpu,
            report,
            Registers {
                pc: 1,
                r: 0x80,
                ..before
            },
            11,
            0x2302,
        );
        assert_eq!(bus.memory, memory);
        let mut trace = fetch(0xFFFF, 0xD3);
        trace.extend([
            read(0, ARG, 0x01),
            Event::Output16 {
                port: 0x2301,
                value: 0x23,
            },
        ]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn every_register_input_samples_old_bc_and_sets_sz_parity_clearing_hn() {
    // UM IN r,(C) pp. 296–297; TM Table 7.0-11 p. 54. TM marks H
    // affected without an equation; primary p. 297 and ALPM p. 255 say
    // reset (PROVENANCE §1, IN H notation). Includes IN B and IN C so
    // replacement of an address register cannot change the sampled port.
    for opcode in [0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x78] {
        for (value, result_flags) in [
            (0x00, 0x44),
            (0x80, 0x80),
            (0x81, 0x84),
            (0x7F, 0x00),
            (0xFF, 0x84),
            (0x01, 0x00),
            (0x7B, 0x04),
        ] {
            for (initial_flags, carry) in [(0x00, 0), (0xD7, 1)] {
                let before = Registers {
                    flags: Flags::from_byte(initial_flags),
                    ..registers()
                };
                let mut expected = Registers {
                    pc: 0x2002,
                    r: 0x81,
                    flags: Flags::from_byte(result_flags | carry | (value & 0x28)),
                    ..before
                };
                match opcode {
                    0x40 => expected.b = value,
                    0x48 => expected.c = value,
                    0x50 => expected.d = value,
                    0x58 => expected.e = value,
                    0x60 => expected.h = value,
                    0x68 => expected.l = value,
                    0x78 => expected.a = value,
                    _ => unreachable!(),
                }
                let mut cpu = Z80::new(before, latches());
                let mut bus = RecordingBus::new();
                bus.load(0x2000, &[0xED, opcode]);
                bus.inputs.push_back(value);
                let memory = bus.memory.clone();
                let report = cpu.step(&mut bus);
                complete(&cpu, report, expected, 12, 0xB2C4);
                assert_eq!(bus.memory, memory);
                let mut trace = ed_fetch(0x2000, 0x2001, opcode);
                trace.push(Event::Input16 {
                    port: 0xB2C3,
                    value,
                });
                assert_eq!(bus.take_log(), trace);
                assert!(bus.inputs.is_empty());
            }
        }
    }
}

#[test]
fn every_register_output_uses_bc_and_preserves_all_registers_and_flags() {
    // UM OUT (C),r pp. 307–308; TM Table 7.0-11 p. 54: 12 states.
    for (opcode, value) in [
        (0x41, 0xB2),
        (0x49, 0xC3),
        (0x51, 0xD4),
        (0x59, 0xE5),
        (0x61, 0xF6),
        (0x69, 0x07),
        (0x79, 0xA1),
    ] {
        for flags in [0x00, 0xD7] {
            let before = Registers {
                flags: Flags::from_byte(flags),
                ..registers()
            };
            let mut cpu = Z80::new(before, latches());
            let mut bus = RecordingBus::new();
            bus.load(0x2000, &[0xED, opcode]);
            let memory = bus.memory.clone();
            let report = cpu.step(&mut bus);
            complete(
                &cpu,
                report,
                Registers {
                    pc: 0x2002,
                    r: 0x81,
                    ..before
                },
                12,
                0xB2C4,
            );
            assert_eq!(bus.memory, memory);
            let mut trace = ed_fetch(0x2000, 0x2001, opcode);
            trace.push(Event::Output16 {
                port: 0xB2C3,
                value,
            });
            assert_eq!(bus.take_log(), trace);
        }
    }
}

// Literal entry cases: opcode, input/output, decrement HL, repeating.
// UM INI p. 298; INIR pp. 300–301; IND p. 302; INDR pp. 304–305;
// OUTI p. 309; OTIR pp. 311–312; OUTD p. 313; OTDR pp. 315–316.
// All independently cross-read against TM Table 7.0-11 p. 54.
const BLOCKS: [(u8, bool, bool, bool); 8] = [
    (0xA2, true, false, false),
    (0xB2, true, false, true),
    (0xAA, true, true, false),
    (0xBA, true, true, true),
    (0xA3, false, false, false),
    (0xB3, false, false, true),
    (0xAB, false, true, false),
    (0xBB, false, true, true),
];

#[test]
fn block_io_hardware_results_ports_order_wraps_and_both_repeat_paths() {
    // Ports differ across the decrement: input uses old B, output new B
    // (decision-15 rule 4; each UM entry cited above). B=0 -> FF and HL
    // FFFF -> 0000 / 0000 -> FFFF are derived from 8-/16-bit widths and
    // the entry's decrement/increment, not additional silicon claims.
    // Decision-20 §4 H: Young §4.3 pp. 16–17 / Rak block-I/O groups;
    // full-byte flags replace the former 43h mask and preservation.
    for (opcode, input, decrement, repeat) in BLOCKS {
        for (b, next_b, input_port, output_port, repeat_pc, repeat_cycles) in [
            (2, 1, 0x02C3_u16, 0x01C3_u16, 0x2000, 21),
            (1, 0, 0x01C3, 0x00C3, 0x2002, 16),
            (0, 0xFF, 0x00C3, 0xFFC3, 0x2000, 21),
        ] {
            for initial_flags in [0x00, 0xFF] {
                let (h, l, at, next_h, next_l) = if decrement {
                    (0, 0, 0, 0xFF, 0xFF)
                } else {
                    (0xFF, 0xFF, 0xFFFF, 0, 0)
                };
                let before = Registers {
                    b,
                    h,
                    l,
                    flags: Flags::from_byte(initial_flags),
                    ..registers()
                };
                let mut cpu = Z80::new(before, latches());
                let mut bus = RecordingBus::new();
                bus.load(0x2000, &[0xED, opcode]);
                bus.memory[usize::from(at)] = 0x69;
                if input {
                    bus.inputs.push_back(0xA5);
                }
                let mut memory = bus.memory.clone();
                if input {
                    memory[usize::from(at)] = 0xA5;
                }
                let report = cpu.step(&mut bus);
                // Literal §4 results for input A5+C2/C4 or output 69+FF/00.
                let image = match (input, decrement, b) {
                    (true, false, 2) => 0x17,
                    (true, false, 1) => 0x53,
                    (true, false, _) => 0xBB,
                    (true, true, 2) => 0x17,
                    (true, true, 1) => 0x53,
                    (true, true, _) => 0xBB,
                    (false, false, 2) => 0x04,
                    (false, false, 1) => 0x40,
                    (false, false, _) => 0xA8,
                    (false, true, 2) => 0x11,
                    (false, true, 1) => 0x55,
                    (false, true, _) => 0xBD,
                };
                assert_eq!(cpu.registers().flags.to_byte(), image);
                complete(
                    &cpu,
                    report,
                    Registers {
                        b: next_b,
                        h: next_h,
                        l: next_l,
                        pc: if repeat { repeat_pc } else { 0x2002 },
                        r: 0x81,
                        flags: Flags::from_byte(image),
                        ..before
                    },
                    if repeat { repeat_cycles } else { 16 },
                    if input && repeat && next_b != 0 {
                        0x2001
                    } else {
                        (if input { input_port } else { output_port }).wrapping_add(if decrement {
                            u16::MAX
                        } else {
                            1
                        })
                    },
                );
                assert_eq!(bus.memory, memory);
                let mut trace = ed_fetch(0x2000, 0x2001, opcode);
                if input {
                    trace.extend([
                        Event::Input16 {
                            port: input_port,
                            value: 0xA5,
                        },
                        write(at, 0xA5),
                    ]);
                } else {
                    trace.extend([
                        read(at, DATA, 0x69),
                        Event::Output16 {
                            port: output_port,
                            value: 0x69,
                        },
                    ]);
                }
                assert_eq!(bus.take_log(), trace);
                assert!(bus.inputs.is_empty());
            }
        }
    }
}

#[test]
fn block_io_replaces_former_unknown_flags_independently_of_initial_image() {
    // Decision-20 §4 H: Young §4.3 pp. 16–17 / Rak block-I/O groups.
    // b=0, input=80, C=C3: k=144/142; output=80+08/06=88/86.
    for (opcode, input, down, _) in BLOCKS {
        for initial_flags in [0x00, 0x04, 0x10, 0x14, 0x80, 0x84, 0x90, 0x94, 0xFF] {
            let before = Registers {
                b: 1,
                flags: Flags::from_byte(initial_flags),
                ..registers()
            };
            let mut cpu = Z80::new(before, latches());
            let mut bus = RecordingBus::new();
            bus.load(0x2000, &[0xED, opcode]);
            if input {
                bus.inputs.push_back(0x80);
            } else {
                bus.memory[usize::from(before.hl())] = 0x80;
            }
            assert_eq!(cpu.step(&mut bus), retired(1, 16));
            let expected = match (input, down) {
                (true, false) => 0x53,
                (true, true) => 0x53,
                (false, false) => 0x46,
                (false, true) => 0x46,
            };
            assert_eq!(cpu.registers().flags.to_byte(), expected);
        }
    }
}

#[test]
fn all_repeating_forms_resume_after_a_cycle_budget_splits_the_block() {
    // UM INIR pp. 300–301; INDR pp. 304–305; OTIR pp. 311–312;
    // OTDR pp. 315–316; TM p. 54. Each iteration refetches both bytes,
    // retires once, and rewinds PC only while B != 0 (decision-15 rule 11).
    // PC FFFF -> 0001 -> FFFF on repetition is derived 16-bit wrapping.
    // 2*21=42 overshoots a 22-cycle slice; the final iteration costs 16.
    for (opcode, input, addresses, values, h2, l2, h3, l3, ports) in [
        (
            0xB2,
            true,
            [0x1000, 0x1001, 0x1002],
            [0x51, 0xA9, 0x03],
            0x10,
            0x02,
            0x10,
            0x03,
            [0x0307, 0x0207, 0x0107],
        ),
        (
            0xBA,
            true,
            [0x1000, 0x0FFF, 0x0FFE],
            [0x51, 0xA9, 0x03],
            0x0F,
            0xFE,
            0x0F,
            0xFD,
            [0x0307, 0x0207, 0x0107],
        ),
        (
            0xB3,
            false,
            [0x1000, 0x1001, 0x1002],
            [0x51, 0xA9, 0x03],
            0x10,
            0x02,
            0x10,
            0x03,
            [0x0207, 0x0107, 0x0007],
        ),
        (
            0xBB,
            false,
            [0x1000, 0x0FFF, 0x0FFE],
            [0x03, 0xA9, 0x51],
            0x0F,
            0xFE,
            0x0F,
            0xFD,
            [0x0207, 0x0107, 0x0007],
        ),
    ] {
        let before = Registers {
            b: 3,
            c: 7,
            h: 0x10,
            l: 0,
            pc: 0xFFFF,
            ..registers()
        };
        let mut cpu = Z80::new(before, latches());
        let mut whole_cpu = cpu.clone();
        let mut bus = RecordingBus::new();
        let mut whole_bus = RecordingBus::new();
        for target in [&mut bus, &mut whole_bus] {
            target.load(0xFFFF, &[0xED, opcode]);
            if input {
                target.inputs.extend(values);
            } else {
                for (address, value) in addresses.into_iter().zip(values) {
                    target.memory[usize::from(address)] = value;
                }
            }
        }
        let mut memory = bus.memory.clone();
        let mut trace = Vec::new();
        for (address, (port, value)) in addresses
            .into_iter()
            .zip(ports.into_iter().zip(values))
            .take(2)
        {
            trace.extend(ed_fetch(0xFFFF, 0, opcode));
            if input {
                trace.extend([Event::Input16 { port, value }, write(address, value)]);
                memory[usize::from(address)] = value;
            } else {
                trace.extend([read(address, DATA, value), Event::Output16 { port, value }]);
            }
        }
        assert_eq!(
            cpu.run(
                &mut bus,
                Budget {
                    steps: u64::MAX,
                    cycles: 22
                }
            ),
            retired(2, 42)
        );
        assert_eq!(
            *cpu.registers(),
            Registers {
                b: 1,
                h: h2,
                l: l2,
                r: 0x83,
                flags: Flags::from_byte(match opcode {
                    0xB2 => 0x06,
                    0xBA => 0x06,
                    0xB3 => 0x02,
                    _ => 0x17,
                }), // §4 ordinary flags until §6.
                ..before
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: match opcode {
                    0xB2 | 0xBA => 0,
                    0xB3 => 0x0108,
                    _ => 0x0106,
                },
                ..latches()
            }
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(bus.take_log(), trace);
        let mut final_trace = ed_fetch(0xFFFF, 0, opcode);
        let [_, _, address] = addresses;
        let [_, _, port] = ports;
        let [_, _, value] = values;
        if input {
            final_trace.extend([Event::Input16 { port, value }, write(address, value)]);
            memory[usize::from(address)] = value;
        } else {
            final_trace.extend([read(address, DATA, value), Event::Output16 { port, value }]);
        }
        assert_eq!(
            cpu.run(
                &mut bus,
                Budget {
                    steps: u64::MAX,
                    cycles: 1
                }
            ),
            retired(1, 16)
        );
        assert_eq!(
            *cpu.registers(),
            Registers {
                b: 0,
                h: h3,
                l: l3,
                pc: 1,
                r: 0x85,
                flags: Flags::from_byte(match opcode {
                    0xB2 => 0x44,
                    0xBA => 0x40,
                    0xB3 => 0x44,
                    _ => 0x55,
                }), // Decision-20 §4.
                ..before
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: match opcode {
                    0xB2 => 0x0108,
                    0xBA => 0x0106,
                    0xB3 => 0x0008,
                    _ => 0x0006,
                },
                ..latches()
            }
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(bus.take_log(), final_trace);
        assert!(bus.inputs.is_empty());
        assert_eq!(
            whole_cpu.run(
                &mut whole_bus,
                Budget {
                    steps: 3,
                    cycles: u64::MAX
                }
            ),
            retired(3, 58)
        );
        assert_eq!(whole_cpu, cpu);
        assert_eq!(whole_bus.memory, memory);
        trace.extend(final_trace);
        assert_eq!(whole_bus.take_log(), trace);
        assert!(whole_bus.inputs.is_empty());
    }
}

// Decision-20 §1 replaces ED70/71 refusals with A/H flags-only input
// (TM Table 7.0-11 p. 54; UM pp. 296–297) and H preservation/L zero
// output (Young v0.91 §3.4 p. 13 / §8.11 p. 35; Rak .out_c_0).
// Literal port/state/flag witnesses: z80_undocumented_ed.rs.
