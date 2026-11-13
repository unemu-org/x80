//! 8085 machine-control witnesses: decision-12 rules 6/7/9, decision-11.
//! Intel 205775-002 (January 1983), pp. 5-17, 5-19; Intel 9800301-04
//! (May 1981), pp. 3-22 to 3-24 and App. A p. A-4. Cited page images and
//! source hashes checked (PROVENANCE.md §1). All expected state is test-side.
//! Table 5-1 p. 5-19 gives F for EI/DI/NOP, FB for HLT; §2.3.5 p. 2-16
//! describes halt. Bus idle has no Bus call; all four fetch only the opcode.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Budget, Exit, MemoryAccess, Report};

const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

fn registers(pc: u16) -> Registers {
    Registers {
        pc,
        sp: 0x8E13,
        a: 0x73,
        b: 0xB9,
        c: 0xC2,
        d: 0xD4,
        e: 0xE7,
        h: 0x56,
        l: 0x19,
        flags: Flags {
            overflow: false,
            k: false,
            sign: true,
            zero: false,
            auxiliary_carry: true,
            parity: false,
            carry: true,
        },
    }
}

fn polls(
    stop: bool,
    trap: bool,
    rst7_5: bool,
    rst6_5: bool,
    rst5_5: bool,
    intr: bool,
) -> Vec<Event> {
    vec![
        Event::StopPoll(stop),
        Event::TrapPoll(trap),
        Event::Rst7_5Poll(rst7_5),
        Event::Rst6_5Poll(rst6_5),
        Event::Rst5_5Poll(rst5_5),
        Event::IntrPoll(intr),
    ]
}

fn fetch(pc: u16, opcode: u8) -> Vec<Event> {
    let mut trace = polls(false, false, false, false, false, false);
    trace.push(Event::Read {
        address: pc,
        kind: MemoryAccess::OpcodeFetch,
        value: opcode,
    });
    trace
}

fn report(steps: u64, cycles: u64, exit: Exit) -> Report {
    Report {
        steps,
        cycles,
        exit,
    }
}

#[test]
fn nop_four_states_preserves_state_with_derived_pc_wrap() {
    // 205775-002 p. 5-17: no operation/flag changes, four states;
    // 9800301-04 App. A p. A-4 agrees. PC wrap is derived from its
    // 16-bit width (205775-002 p. 2-2); sequential execution p. 5-3.
    for (pc, next_pc) in [(0x2000, 0x2001), (0xFFFF, 0)] {
        for enabled in [false, true] {
            let latches = Latches {
                interrupt_enable: enabled,
                ..Latches::default()
            };
            let mut cpu = I8085::new(registers(pc), latches);
            let mut bus = RecordingBus::new();
            let memory = bus.memory.clone();
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(*cpu.registers(), registers(next_pc));
            assert_eq!(*cpu.latches(), latches);
            assert_eq!(bus.memory, memory);
            assert_eq!(bus.take_log(), fetch(pc, 0));
        }
    }
}

#[test]
fn ei_arms_delay_until_next_completed_instruction_across_steps() {
    // 205775-002 p. 5-17; 9800301-04 p. 3-23: enable following the
    // next instruction, four states/no flags. Observe after each step.
    for initially_enabled in [false, true] {
        let mut cpu = I8085::new(
            registers(0x2000),
            Latches {
                interrupt_enable: initially_enabled,
                ..Latches::default()
            },
        );
        let mut bus = RecordingBus::new();
        bus.load(0x2000, &[0xFB, 0, 0]);
        let memory = bus.memory.clone();
        for (pc, opcode, next_pc, delay) in [
            (0x2000, 0xFB, 0x2001, true),
            (0x2001, 0, 0x2002, false),
            (0x2002, 0, 0x2003, false),
        ] {
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(*cpu.registers(), registers(next_pc));
            assert_eq!(
                *cpu.latches(),
                Latches {
                    interrupt_enable: true,
                    ei_delay: delay,
                    ..Latches::default()
                }
            );
            assert_eq!(bus.take_log(), fetch(pc, opcode));
        }
        assert_eq!(bus.memory, memory);
    }
}

#[test]
fn di_immediate_disable_and_next_boundary_inhibition_unverified() {
    // 205775-002 p. 5-17, 9800301-04 pp. 3-22/3-23: immediate DI,
    // four states/no flags. Decision-12 rule 6: unverified interpretation
    // of "not recognized during" suppresses acceptance at the next boundary.
    // TRAP included; pending survives an inhibited low sample (rule 2).
    // The asserted lines cannot preempt the instruction after DI.
    for initially_enabled in [false, true] {
        for initial_delay in [false, true] {
            let mut cpu = I8085::new(
                registers(0x2000),
                Latches {
                    interrupt_enable: initially_enabled,
                    ei_delay: initial_delay,
                    ..Latches::default()
                },
            );
            let mut bus = RecordingBus::new();
            bus.load(0x2000, &[0xF3, 0, 0]);
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(*cpu.registers(), registers(0x2001));
            assert_eq!(
                *cpu.latches(),
                Latches {
                    ei_delay: true,
                    ..Latches::default()
                }
            );
            assert_eq!(bus.take_log(), fetch(0x2000, 0xF3));

            // Fresh asserted lines at the boundary immediately following DI.
            bus.trap = true;
            bus.rst7_5 = true;
            bus.rst6_5 = true;
            bus.rst5_5 = true;
            bus.intr = true;
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(*cpu.registers(), registers(0x2002));
            assert_eq!(
                *cpu.latches(),
                Latches {
                    trap_pending: true,
                    rst7_5_pending: true,
                    last_trap: true,
                    last_rst7_5: true,
                    ..Latches::default()
                }
            );
            let mut trace = polls(false, true, true, true, true, true);
            trace.push(Event::Read {
                address: 0x2001,
                kind: MemoryAccess::OpcodeFetch,
                value: 0,
            });
            assert_eq!(bus.take_log(), trace);
        }
    }

    // Make the inhibition independently observable through cancellation:
    // a low sample cancels pending TRAP only when eligible, not after DI.
    // Start inhibited so the high TRAP does not preempt DI itself; DI
    // re-arms the delay, preserving pending at the following low boundary.
    let mut cpu = I8085::new(
        registers(0x2000),
        Latches {
            ei_delay: true,
            ..Latches::default()
        },
    );
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0xF3, 0, 0]);
    bus.trap = true;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert!(cpu.latches().trap_pending);
    assert!(cpu.latches().ei_delay);
    bus.take_log();
    bus.trap = false;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert!(cpu.latches().trap_pending);
    assert!(!cpu.latches().ei_delay);
    assert!(cpu.latches().trap_armed);
    assert_eq!(bus.take_log(), fetch(0x2001, 0));
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert!(!cpu.latches().trap_pending);
    assert_eq!(bus.take_log(), fetch(0x2002, 0));
}

#[test]
fn ei_di_ei_ei_and_di_ei_edges_are_unverified_rearming_models() {
    // Neither 205775-002 p. 5-17 nor 9800301-04 pp. 3-22/3-23
    // discusses these pairs. Decision-12 rule 6: each EI/DI re-arms;
    // DI clears enable immediately, EI sets it with a pending delay.
    for (first, second, enabled_first, enabled_second) in [
        (0xFB, 0xF3, true, false),
        (0xFB, 0xFB, true, true),
        (0xF3, 0xFB, false, true),
    ] {
        let mut cpu = I8085::new(registers(0x2000), Latches::default());
        let mut bus = RecordingBus::new();
        bus.load(0x2000, &[first, second, 0]);
        let memory = bus.memory.clone();
        for (pc, opcode, next_pc, enabled, delay) in [
            (0x2000, first, 0x2001, enabled_first, true),
            (0x2001, second, 0x2002, enabled_second, true),
            (0x2002, 0, 0x2003, enabled_second, false),
        ] {
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(*cpu.registers(), registers(next_pc));
            assert_eq!(
                *cpu.latches(),
                Latches {
                    interrupt_enable: enabled,
                    ei_delay: delay,
                    ..Latches::default()
                }
            );
            assert_eq!(bus.take_log(), fetch(pc, opcode));
        }
        assert_eq!(bus.memory, memory);
    }
}

#[test]
fn ei_di_delay_survives_budget_stop_and_halt_exits() {
    // Decision-12 rule 6 counts completed instructions; rule 10 and
    // decision-5 preserve state at Stop. Neither advances delay.
    // Decision-18 removes fetched refusals; RSTV ordinary retirement has
    // its independent witness in i8085_undocumented.rs.
    for opcode in [0xFB, 0xF3] {
        let mut cpu = I8085::new(registers(0x2000), Latches::default());
        let mut bus = RecordingBus::new();
        bus.load(0x2000, &[opcode, 0x08]);
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert_eq!(bus.take_log(), fetch(0x2000, opcode));
        let before = cpu.clone();
        assert_eq!(
            cpu.run(
                &mut bus,
                Budget {
                    steps: 0,
                    cycles: 100
                }
            ),
            report(0, 0, Exit::Budget)
        );
        assert_eq!(cpu, before);
        assert!(bus.take_log().is_empty());
        bus.stop = true;
        assert_eq!(cpu.run(&mut bus, UNLIMITED), report(0, 0, Exit::Stop));
        assert_eq!(cpu, before);
        assert_eq!(
            bus.take_log(),
            polls(true, false, false, false, false, false)
        );
        bus.stop = false;
        // Decision-18: 08 now executes; omit the obsolete refusal probe.
        cpu.latches_mut().halted = true;
        let halted = cpu.clone();
        assert_eq!(cpu.run(&mut bus, UNLIMITED), report(0, 0, Exit::Halt));
        assert_eq!(cpu, halted);
        assert_eq!(
            bus.take_log(),
            polls(false, false, false, false, false, false)
        );
        cpu.latches_mut().halted = false;
        bus.load(0x2001, &[0]);
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert!(!cpu.latches().ei_delay);
        assert_eq!(bus.take_log(), fetch(0x2001, 0));
    }
}

#[test]
fn hlt_five_states_then_halt_without_fetch_with_derived_pc_wrap() {
    // 205775-002 p. 5-17 and Table 5-1: five states, FB; p. 2-16:
    // stopped until reset/valid interrupt. 9800301-04 p. 3-24: five
    // on 8085, registers/flags unchanged, PC at next instruction.
    // Wrap is derived (p. 2-2); Budget before Halt is decision-12 rule 3.
    for (pc, next_pc) in [(0x2000, 0x2001), (0xFFFF, 0)] {
        for enabled in [false, true] {
            for delay in [false, true] {
                let mut cpu = I8085::new(
                    registers(pc),
                    Latches {
                        interrupt_enable: enabled,
                        ei_delay: delay,
                        ..Latches::default()
                    },
                );
                let mut bus = RecordingBus::new();
                bus.load(pc, &[0x76, 0x08]);
                let memory = bus.memory.clone();
                assert_eq!(cpu.step(&mut bus), report(1, 5, Exit::Budget));
                assert_eq!(*cpu.registers(), registers(next_pc));
                assert_eq!(
                    *cpu.latches(),
                    Latches {
                        interrupt_enable: enabled,
                        halted: true,
                        ..Latches::default()
                    }
                );
                assert_eq!(bus.take_log(), fetch(pc, 0x76));
                let halted = cpu.clone();
                for _ in 0..2 {
                    assert_eq!(cpu.run(&mut bus, UNLIMITED), report(0, 0, Exit::Halt));
                    assert_eq!(cpu, halted);
                    assert_eq!(
                        bus.take_log(),
                        polls(false, false, false, false, false, false)
                    );
                }
                assert_eq!(bus.memory, memory);
            }
        }
    }
}

#[test]
fn ei_hlt_nine_states_and_reset_restarts_at_zero() {
    // 9800301-04 pp. 3-23/3-24 explicitly gives EI;HLT; primary
    // 205775-002 p. 5-17: 4+5 states. Halt exit/reset: p. 2-16,
    // §2.3.7 p. 2-18; second witnesses 9800301-04 p. 3-24 and
    // 231718-001 Table 1 p. 16-13 (RESET clears PC/enable).
    // Registers/SP/flags preserved by library choice, not hardware;
    // clearing internal delay on reset is unverified (rule 9).
    let mut cpu = I8085::new(registers(0x2000), Latches::default());
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0xFB, 0x76, 0x08]);
    let memory = bus.memory.clone();
    assert_eq!(cpu.run(&mut bus, UNLIMITED), report(2, 9, Exit::Halt));
    assert_eq!(*cpu.registers(), registers(0x2002));
    assert_eq!(
        *cpu.latches(),
        Latches {
            interrupt_enable: true,
            halted: true,
            ..Latches::default()
        }
    );
    let mut trace = fetch(0x2000, 0xFB);
    trace.extend(fetch(0x2001, 0x76));
    trace.extend(polls(false, false, false, false, false, false));
    assert_eq!(bus.take_log(), trace);
    // Also exercise reset during the unverified delay, with halt set.
    cpu.latches_mut().ei_delay = true;
    cpu.reset();
    assert_eq!(*cpu.registers(), registers(0));
    assert_eq!(*cpu.latches(), Latches::default());
    assert!(bus.take_log().is_empty());
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(*cpu.registers(), registers(1));
    assert_eq!(*cpu.latches(), Latches::default());
    assert_eq!(bus.take_log(), fetch(0, 0));
    assert_eq!(bus.memory, memory);
}
