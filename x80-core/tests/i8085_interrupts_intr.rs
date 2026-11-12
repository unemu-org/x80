//! INTR witnesses: decision-12 rules 1, 3, 5-7; decision-5 items 4-6/8.
//!
//! Intel 205775-002 (January 1983), §2.3.4 pp. 2-13/2-14 (PDF 36/37),
//! Figs 2-17/2-18: INA at unchanged PC, CALL low/high bytes, then stack
//! high/low writes. Derived totals: RST 6+3+3=12; CALL 6+3+3+3+3=18.
//! §5.6.4 pp. 5-13/5-14 and Table 5-1 p. 5-19 agree on those totals;
//! Table 5-2 p. 5-20 pins CD and the eight RST bytes (not cycle sequences).
//! Second witness: Intel 9800301-04 (May 1981), CALL pp. 3-8/3-9,
//! RST pp. 3-54/3-55 (8085 values in parentheses). Intel 231718-001
//! (September 1987), handbook pp. 16-12/16-13/16-15, independently pins
//! PC inhibition, RST/CALL injection, priority, levels and enable clearing.
//! All cited values checked on rendered pages; hashes match PROVENANCE.md §1.
//! Bus kinds, polls and Unsupported restoration are API choices.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Budget, Exit, MemoryAccess, Report, Unsupported};

fn registers() -> Registers {
    Registers {
        a: 0x95,
        b: 0x21,
        c: 0x43,
        d: 0x65,
        e: 0x87,
        h: 0xA9,
        l: 0xCB,
        flags: Flags {
            sign: true,
            zero: false,
            auxiliary_carry: true,
            parity: false,
            carry: true,
        },
        pc: 0x23FF,
        sp: 0x8100,
    }
}

fn enabled() -> Latches {
    // Masked RST 7.5 pending and saved TRAP/RIM state must survive INTR.
    Latches {
        interrupt_enable: true,
        rst7_5_pending: true,
        trap_saved_interrupt_enable: true,
        trap_rim_pending: true,
        sod: true,
        ..Latches::default()
    }
}

fn polls(trap: bool, rst7_5: bool, rst6_5: bool, rst5_5: bool, intr: bool) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::TrapPoll(trap),
        Event::Rst7_5Poll(rst7_5),
        Event::Rst6_5Poll(rst6_5),
        Event::Rst5_5Poll(rst5_5),
        Event::IntrPoll(intr),
    ]
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
        kind: MemoryAccess::Stack,
        value,
    }
}

fn report(steps: u64, cycles: u64) -> Report {
    Report {
        steps,
        cycles,
        exit: Exit::Budget,
    }
}

#[test]
fn rst_acknowledge_literal_trace_pushes_boundary_pc_in_twelve_states() {
    let mut cpu = I8085::new(registers(), enabled());
    let mut bus = RecordingBus::new();
    bus.intr = true;
    bus.acknowledges.push_back(0xEF);
    bus.load(0x23FF, &[0x08]); // An ordinary fetch would be Unsupported.
    let mut memory = bus.memory.clone();
    memory[0x80FF] = 0x23;
    memory[0x80FE] = 0xFF;
    assert_eq!(cpu.step(&mut bus), report(1, 12));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0028,
            sp: 0x80FE,
            ..registers()
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            interrupt_enable: false,
            ..enabled()
        }
    );
    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(true),
            read(0x23FF, MemoryAccess::InterruptAcknowledge, 0xEF),
            write(0x80FF, 0x23),
            write(0x80FE, 0xFF),
        ]
    );
}

#[test]
fn call_acknowledge_literal_trace_reads_three_bytes_at_unchanged_pc() {
    let mut cpu = I8085::new(registers(), enabled());
    let mut bus = RecordingBus::new();
    bus.intr = true;
    bus.acknowledges.extend([0xCD, 0xB6, 0x4D]);
    bus.load(0x23FF, &[0x08, 0xAA, 0x55]);
    let mut memory = bus.memory.clone();
    memory[0x80FF] = 0x23;
    memory[0x80FE] = 0xFF;
    assert_eq!(cpu.step(&mut bus), report(1, 18));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x4DB6,
            sp: 0x80FE,
            ..registers()
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            interrupt_enable: false,
            ..enabled()
        }
    );
    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(true),
            read(0x23FF, MemoryAccess::InterruptAcknowledge, 0xCD),
            read(0x23FF, MemoryAccess::InterruptAcknowledge, 0xB6),
            read(0x23FF, MemoryAccess::InterruptAcknowledge, 0x4D),
            write(0x80FF, 0x23),
            write(0x80FE, 0xFF),
        ]
    );
}

#[test]
fn all_eight_acknowledged_restarts_use_literal_vectors() {
    // 205775-002 p. 5-14; 9800301-04 pp. 3-54/3-55. No flags change.
    for (opcode, vector) in [
        (0xC7, 0x0000),
        (0xCF, 0x0008),
        (0xD7, 0x0010),
        (0xDF, 0x0018),
        (0xE7, 0x0020),
        (0xEF, 0x0028),
        (0xF7, 0x0030),
        (0xFF, 0x0038),
    ] {
        let mut cpu = I8085::new(registers(), enabled());
        let mut bus = RecordingBus::new();
        bus.intr = true;
        bus.acknowledges.push_back(opcode);
        assert_eq!(cpu.step(&mut bus), report(1, 12));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: vector,
                sp: 0x80FE,
                ..registers()
            }
        );
        let mut trace = polls(false, false, false, false, true);
        trace.extend([
            read(0x23FF, MemoryAccess::InterruptAcknowledge, opcode),
            write(0x80FF, 0x23),
            write(0x80FE, 0xFF),
        ]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn every_other_acknowledge_including_ei_di_restores_state_and_repeats() {
    // Decision-12 rule 5 deliberately supports only RST/CALL. EI/DI are
    // prohibited by 205775-002 p. 5-17. Restoration is decision-5 item 6's
    // stated departure from hardware, including the pre-sampling latches.
    for opcode in 0..=255 {
        if [0xC7, 0xCF, 0xD7, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF, 0xCD].contains(&opcode) {
            continue;
        }
        for halted in [false, true] {
            let mut cpu = I8085::new(
                registers(),
                Latches {
                    halted,
                    trap_pending: true,
                    trap_armed: false,
                    last_trap: true,
                    rst7_5_pending: false,
                    ..enabled()
                },
            );
            let before = cpu.clone();
            let mut bus = RecordingBus::new();
            bus.intr = true;
            bus.rst7_5 = true; // Masked rising edge; low TRAP cancels pending.
            bus.acknowledges.extend([opcode, opcode, 0xAA]);
            bus.load(0x23FF, &[0x5A, 0xA5]);
            let memory = bus.memory.clone();
            let mut trace = polls(false, true, false, false, true);
            trace.push(read(0x23FF, MemoryAccess::InterruptAcknowledge, opcode));
            let first = cpu.step(&mut bus);
            assert_eq!((first.steps, first.cycles), (0, 0));
            assert!(matches!(first.exit, Exit::Unsupported(Unsupported {
                pc: 0x23FF, opcode: byte, acknowledge: true, ..
            }) if byte == opcode));
            assert_eq!(cpu, before);
            assert_eq!(bus.memory, memory);
            assert_eq!(bus.take_log(), trace);
            assert_eq!(cpu.step(&mut bus), first);
            assert_eq!(cpu, before);
            assert_eq!(bus.memory, memory);
            assert_eq!(bus.take_log(), trace);
            assert_eq!(bus.acknowledges.iter().copied().collect::<Vec<_>>(), [0xAA]);
        }
    }
}

#[test]
fn intr_loses_to_each_eligible_higher_line_without_acknowledging() {
    // 231718-001 Table 2 p. 16-13; decision-12 rules 3-5: each higher
    // eligible line is accepted at its vector, pushing the boundary PC
    // without acknowledging INTR or consuming its queued byte.
    for (source, vector) in [(0, 0x0024), (1, 0x003C), (2, 0x0034), (3, 0x002C)] {
        let mut cpu = I8085::new(
            registers(),
            Latches {
                interrupt_enable: true,
                mask7_5: false,
                mask6_5: false,
                mask5_5: false,
                ..Latches::default()
            },
        );
        let mut bus = RecordingBus::new();
        bus.trap = source == 0;
        bus.rst7_5 = source == 1;
        bus.rst6_5 = source == 2;
        bus.rst5_5 = source == 3;
        bus.intr = true;
        bus.acknowledges.push_back(0xC7);
        let mut memory = bus.memory.clone();
        memory[0x80FF] = 0x23;
        memory[0x80FE] = 0xFF;
        assert_eq!(cpu.step(&mut bus), report(1, 12));
        let mut trace = polls(source == 0, source == 1, source == 2, source == 3, true);
        trace.extend([write(0x80FF, 0x23), write(0x80FE, 0xFF)]);
        assert_eq!(bus.take_log(), trace);
        assert_eq!(bus.acknowledges.iter().copied().collect::<Vec<_>>(), [0xC7]);
        assert_eq!(bus.memory, memory);
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: vector,
                sp: 0x80FE,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                interrupt_enable: false,
                mask7_5: false,
                mask6_5: false,
                mask5_5: false,
                trap_armed: source != 0,
                trap_saved_interrupt_enable: source == 0,
                trap_rim_pending: source == 0,
                last_trap: source == 0,
                last_rst7_5: source == 1,
                ..Latches::default()
            }
        );
    }
}

#[test]
fn intr_requires_high_level_enable_and_expired_delay() {
    // 205775-002 §2.3.4 p. 2-13, EI p. 5-17; 231718-001 Table 2
    // p. 16-13; 9800301-04 p. 3-23. No eligible interrupt means no INA.
    for (intr, interrupt_enable, ei_delay) in [
        (false, true, false),
        (true, false, false),
        (true, false, true),
        (true, true, true),
    ] {
        let initial = Latches {
            interrupt_enable,
            ei_delay,
            ..enabled()
        };
        let mut cpu = I8085::new(registers(), initial);
        let mut bus = RecordingBus::new();
        bus.intr = intr;
        let memory = bus.memory.clone();
        assert_eq!(cpu.step(&mut bus), report(1, 4));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 0x2400,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                ei_delay: false,
                ..initial
            }
        );
        assert_eq!(bus.memory, memory);
        let mut trace = polls(false, false, false, false, intr);
        trace.push(read(0x23FF, MemoryAccess::OpcodeFetch, 0));
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn held_intr_waits_through_ei_and_hlt_then_wakes_and_pushes_after_hlt() {
    // 205775-002 pp. 5-17/2-16: EI delay, HLT, valid interrupt wake.
    // 9800301-04 pp. 3-23/3-24 explicitly gives EI;HLT and next PC.
    for (bytes, target, states) in [
        (vec![0xCF], 0x0008, 12),
        (vec![0xCD, 0xB6, 0x4D], 0x4DB6, 18),
    ] {
        let mut cpu = I8085::new(registers(), Latches::default());
        let mut bus = RecordingBus::new();
        bus.intr = true;
        bus.load(0x23FF, &[0xFB, 0x76, 0x08]);
        bus.acknowledges.extend(bytes.iter().copied());
        for (pc, opcode, steps, cycles, delay, halted) in [
            (0x23FF, 0xFB, 1, 4, true, false),
            (0x2400, 0x76, 1, 5, false, true),
        ] {
            assert_eq!(cpu.step(&mut bus), report(steps, cycles));
            assert_eq!(
                *cpu.latches(),
                Latches {
                    interrupt_enable: true,
                    ei_delay: delay,
                    halted,
                    ..Latches::default()
                }
            );
            let mut trace = polls(false, false, false, false, true);
            trace.push(read(pc, MemoryAccess::OpcodeFetch, opcode));
            assert_eq!(bus.take_log(), trace);
        }
        assert_eq!(cpu.registers().pc, 0x2401);
        let mut memory = bus.memory.clone();
        memory[0x80FF] = 0x24;
        memory[0x80FE] = 0x01;
        assert_eq!(cpu.step(&mut bus), report(1, states));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: target,
                sp: 0x80FE,
                ..registers()
            }
        );
        assert_eq!(*cpu.latches(), Latches::default());
        assert_eq!(bus.memory, memory);
        let mut trace = polls(false, false, false, false, true);
        trace.extend(
            bytes
                .into_iter()
                .map(|byte| read(0x2401, MemoryAccess::InterruptAcknowledge, byte)),
        );
        trace.extend([write(0x80FF, 0x24), write(0x80FE, 0x01)]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn accepted_intr_overshoots_cycle_budget_and_disables_held_level() {
    // Decision-5 items 5/8: acceptance counts one step and completes even
    // across a cycle limit. 205775-002 p. 2-13 clears INTE on acceptance.
    for (bytes, target, states) in [
        (vec![0xFF], 0x0038, 12),
        (vec![0xCD, 0xB6, 0x4D], 0x4DB6, 18),
    ] {
        let mut cpu = I8085::new(registers(), enabled());
        let mut bus = RecordingBus::new();
        bus.intr = true;
        bus.acknowledges.extend(bytes.iter().copied());
        assert_eq!(
            cpu.run(
                &mut bus,
                Budget {
                    steps: 100,
                    cycles: 1
                }
            ),
            report(1, states)
        );
        let mut trace = polls(false, false, false, false, true);
        trace.extend(
            bytes
                .into_iter()
                .map(|byte| read(0x23FF, MemoryAccess::InterruptAcknowledge, byte)),
        );
        trace.extend([write(0x80FF, 0x23), write(0x80FE, 0xFF)]);
        assert_eq!(bus.take_log(), trace); // Reached budget makes no extra poll.
        assert_eq!(cpu.step(&mut bus), report(1, 4));
        let mut trace = polls(false, false, false, false, true);
        trace.push(read(target, MemoryAccess::OpcodeFetch, 0));
        assert_eq!(bus.take_log(), trace); // Still high, now disabled: no INA.
    }
}
