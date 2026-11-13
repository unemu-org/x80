//! Boundary witnesses (decision-5/12). Instruction facts: Intel 205775-002
//! (January 1983) §5.4 p. 5-3, NOP p. 5-17, Tables 5-1/5-2 pp. 5-19/5-20;
//! Intel 9800301-04 (May 1981) pp. A-4/A-5. Edge facts: 205775-002
//! RIM/SIM pp. 5-17/5-18; Intel 231718-001 (September 1987), handbook
//! pp. 16-13 (Table 2), 16-15 (TRAP and Fig. 4). Consecutive boundary
//! sampling and exit precedence are API choices, not pin-cycle timing.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Budget, Exit, MemoryAccess, Report};

fn cpu_at(pc: u16) -> I8085 {
    I8085::new(
        Registers {
            a: 0x91,
            flags: Flags {
                overflow: false,
                k: false,
                sign: true,
                zero: true,
                auxiliary_carry: true,
                parity: true,
                carry: true,
            },
            b: 0x12,
            c: 0x34,
            d: 0x56,
            e: 0x78,
            h: 0x9A,
            l: 0xBC,
            sp: 0xDEF0,
            pc,
        },
        Latches::default(),
    )
}

fn low_polls() -> [Event; 6] {
    [
        Event::StopPoll(false),
        Event::TrapPoll(false),
        Event::Rst7_5Poll(false),
        Event::Rst6_5Poll(false),
        Event::Rst5_5Poll(false),
        Event::IntrPoll(false),
    ]
}

#[test]
fn either_zero_budget_precedes_stop_interrupts_halt_and_sampling() {
    for budget in [
        Budget {
            steps: 0,
            cycles: 100,
        },
        Budget {
            steps: 100,
            cycles: 0,
        },
    ] {
        let mut cpu = cpu_at(0x1000);
        cpu.latches_mut().halted = true;
        cpu.latches_mut().interrupt_enable = true;
        let before = cpu.clone();
        let mut bus = RecordingBus::new();
        bus.stop = true;
        bus.trap = true;
        bus.rst7_5 = true;
        bus.rst6_5 = true;
        bus.rst5_5 = true;
        bus.intr = true;
        bus.load(0x1000, &[0x08]);
        assert_eq!(
            cpu.run(&mut bus, budget),
            Report {
                steps: 0,
                cycles: 0,
                exit: Exit::Budget
            }
        );
        assert_eq!(cpu, before);
        assert!(bus.take_log().is_empty());
    }
}

#[test]
fn stop_polls_every_line_preserves_latches_then_held_edges_are_sampled() {
    let mut cpu = cpu_at(0x1000);
    cpu.latches_mut().halted = true;
    cpu.latches_mut().interrupt_enable = true;
    let before = cpu.clone();
    let mut bus = RecordingBus::new();
    bus.stop = true;
    bus.trap = true;
    bus.rst7_5 = true;
    bus.rst6_5 = true;
    bus.rst5_5 = true;
    bus.intr = true;
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 0,
            cycles: 0,
            exit: Exit::Stop
        }
    );
    assert_eq!(cpu.latches(), before.latches());
    assert_eq!(cpu, before);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(true),
            Event::TrapPoll(true),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(true),
            Event::Rst5_5Poll(true),
            Event::IntrPoll(true)
        ]
    );

    bus.stop = false;
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 12,
            exit: Exit::Budget
        }
    );
    assert!(!cpu.latches().trap_pending);
    assert!(!cpu.latches().trap_armed);
    assert!(!cpu.latches().halted);
    assert!(!cpu.latches().interrupt_enable);
    assert!(cpu.latches().trap_saved_interrupt_enable);
    assert!(cpu.latches().trap_rim_pending);
    assert!(cpu.latches().rst7_5_pending);
    assert!(cpu.latches().last_trap);
    assert!(cpu.latches().last_rst7_5);
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x0024, 0xDEEE));
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(true),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(true),
            Event::Rst5_5Poll(true),
            Event::IntrPoll(true),
            Event::Write {
                address: 0xDEEF,
                kind: MemoryAccess::Stack,
                value: 0x10
            },
            Event::Write {
                address: 0xDEEE,
                kind: MemoryAccess::Stack,
                value: 0x00
            },
        ]
    );
}

#[test]
fn every_eligible_source_wakes_halt_and_none_returns_halt_without_fetch() {
    // Decision-12 rules 4/5/7: enabled, unmasked vectored sources and an
    // eligible INTR (acknowledging RST 0) wake halt; 205775-002 pp. 2-13 to
    // 2-16. With no request, Halt follows the polls and nothing is fetched.
    for source in 0..6 {
        let mut cpu = cpu_at(0x1000);
        cpu.latches_mut().halted = true;
        cpu.latches_mut().interrupt_enable = true;
        cpu.latches_mut().mask7_5 = false;
        cpu.latches_mut().mask6_5 = false;
        cpu.latches_mut().mask5_5 = false;
        let mut bus = RecordingBus::new();
        bus.trap = source == 0;
        bus.rst7_5 = source == 1;
        bus.rst6_5 = source == 2;
        bus.rst5_5 = source == 3;
        bus.intr = source == 4;
        bus.acknowledges.push_back(0xC7);
        bus.load(0x1000, &[0x08]);
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: u64::from(source < 5),
                cycles: if source < 5 { 12 } else { 0 },
                exit: if source < 5 { Exit::Budget } else { Exit::Halt }
            }
        );
        let expected_polls = match source {
            0 => [
                Event::StopPoll(false),
                Event::TrapPoll(true),
                Event::Rst7_5Poll(false),
                Event::Rst6_5Poll(false),
                Event::Rst5_5Poll(false),
                Event::IntrPoll(false),
            ],
            1 => [
                Event::StopPoll(false),
                Event::TrapPoll(false),
                Event::Rst7_5Poll(true),
                Event::Rst6_5Poll(false),
                Event::Rst5_5Poll(false),
                Event::IntrPoll(false),
            ],
            2 => [
                Event::StopPoll(false),
                Event::TrapPoll(false),
                Event::Rst7_5Poll(false),
                Event::Rst6_5Poll(true),
                Event::Rst5_5Poll(false),
                Event::IntrPoll(false),
            ],
            3 => [
                Event::StopPoll(false),
                Event::TrapPoll(false),
                Event::Rst7_5Poll(false),
                Event::Rst6_5Poll(false),
                Event::Rst5_5Poll(true),
                Event::IntrPoll(false),
            ],
            4 => [
                Event::StopPoll(false),
                Event::TrapPoll(false),
                Event::Rst7_5Poll(false),
                Event::Rst6_5Poll(false),
                Event::Rst5_5Poll(false),
                Event::IntrPoll(true),
            ],
            _ => low_polls(),
        };
        let mut trace = expected_polls.to_vec();
        let mut expected_registers = *cpu_at(0x1000).registers();
        if source < 5 {
            if source == 4 {
                trace.push(Event::Read {
                    address: 0x1000,
                    kind: MemoryAccess::InterruptAcknowledge,
                    value: 0xC7,
                });
            }
            expected_registers.pc = [0x0024, 0x003C, 0x0034, 0x002C, 0x0000][source];
            expected_registers.sp = 0xDEEE;
            trace.extend([
                Event::Write {
                    address: 0xDEEF,
                    kind: MemoryAccess::Stack,
                    value: 0x10,
                },
                Event::Write {
                    address: 0xDEEE,
                    kind: MemoryAccess::Stack,
                    value: 0x00,
                },
            ]);
        }
        assert_eq!(bus.take_log(), trace);
        assert_eq!(*cpu.registers(), expected_registers);
        assert_eq!(cpu.latches().halted, source >= 5);
        assert_eq!(cpu.latches().interrupt_enable, source >= 5);
    }
}

// Decision-18 §§2–4 removes all ten fetched-byte refusals. Their old
// no-effects/retry witnesses are replaced by complete-set execution in
// i8085_coverage.rs and literal path/retirement traces in i8085_undocumented.rs.

#[test]
fn nop_is_one_step_four_states_and_only_advances_pc_and_ei_delay() {
    // 205775-002 p. 5-17 NOP, sequential execution p. 5-3;
    // 9800301-04 p. A-4. Delay advances only on completion (rule 6).
    let mut cpu = cpu_at(0xFFFF);
    cpu.latches_mut().ei_delay = true;
    let mut expected = cpu.clone();
    expected.registers_mut().pc = 0;
    expected.latches_mut().ei_delay = false;
    let mut bus = RecordingBus::new();
    let memory = bus.memory.clone();
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert_eq!(cpu, expected);
    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false),
            Event::Read {
                address: 0xFFFF,
                kind: MemoryAccess::OpcodeFetch,
                value: 0x00
            }
        ]
    );
}

#[test]
fn cycle_budget_overshoots_and_reached_budget_does_not_poll_again() {
    let mut cpu = cpu_at(0xFFFE);
    let mut bus = RecordingBus::new();
    // A pending high edge in the third script slot must remain unconsumed.
    bus.trap_levels.extend([false, false, true]);
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: 10,
                cycles: 5
            }
        ),
        Report {
            steps: 2,
            cycles: 8,
            exit: Exit::Budget
        }
    );
    assert_eq!(cpu.registers().pc, 0);
    assert_eq!(bus.trap_levels.iter().copied().collect::<Vec<_>>(), [true]);
    assert!(!cpu.latches().trap_pending);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false),
            Event::Read {
                address: 0xFFFE,
                kind: MemoryAccess::OpcodeFetch,
                value: 0
            },
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false),
            Event::Read {
                address: 0xFFFF,
                kind: MemoryAccess::OpcodeFetch,
                value: 0
            },
        ]
    );
}

#[test]
fn budget_reports_earlier_nop_and_trap_progress() {
    let mut cpu = cpu_at(0x2000);
    cpu.latches_mut().ei_delay = true;
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0x00, 0x08]);
    bus.load(0x0024, &[0x08]);
    bus.trap_levels.extend([false, true]);
    bus.rst7_5_levels.extend([false, true]);
    // Decision-18: 08 is executable; stop at the acceptance boundary by budget.
    let report = cpu.run(
        &mut bus,
        Budget {
            steps: 2,
            cycles: u64::MAX,
        },
    );
    assert_eq!((report.steps, report.cycles), (2, 16));
    assert_eq!(report.exit, Exit::Budget);
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x0024, 0xDEEE));
    assert!(!cpu.latches().ei_delay);
    assert!(!cpu.latches().trap_pending);
    assert!(cpu.latches().rst7_5_pending);
    assert!(cpu.latches().last_trap);
    assert!(cpu.latches().last_rst7_5);
    assert!(!cpu.latches().trap_armed);
    assert!(cpu.latches().trap_rim_pending);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false),
            Event::Read {
                address: 0x2000,
                kind: MemoryAccess::OpcodeFetch,
                value: 0
            },
            Event::StopPoll(false),
            Event::TrapPoll(true),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false),
            Event::Write {
                address: 0xDEEF,
                kind: MemoryAccess::Stack,
                value: 0x20
            },
            Event::Write {
                address: 0xDEEE,
                kind: MemoryAccess::Stack,
                value: 0x01
            },
        ]
    );
}

#[test]
fn unsupported_acknowledge_reports_earlier_nop_progress() {
    // Decision-12 rule 5 / decision-5 item 6: refusal contributes no
    // progress and restores the rejection boundary, retaining the NOP.
    let mut cpu = cpu_at(0x2000);
    cpu.latches_mut().interrupt_enable = true;
    cpu.latches_mut().ei_delay = true;
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0x00]);
    bus.intr = true;
    bus.rst7_5_levels.extend([false, true]);
    // 08 executes when fetched (decision-18), but INTR accepts only RST/CALL.
    bus.acknowledges.push_back(0x08);
    let memory = bus.memory.clone();
    let report = cpu.run(
        &mut bus,
        Budget {
            steps: 10,
            cycles: 100,
        },
    );
    assert_eq!((report.steps, report.cycles), (1, 4));
    let Exit::Unsupported(unsupported) = report.exit else {
        panic!("expected Unsupported, got {:?}", report.exit);
    };
    assert_eq!(unsupported.pc, 0x2001);
    assert_eq!(unsupported.opcode, 0x08);
    assert!(unsupported.acknowledge);
    assert_eq!(unsupported.preceding, [0, 0, 0]);
    assert_eq!(unsupported.preceding_len, 0);
    assert_eq!(
        cpu,
        I8085::new(
            Registers {
                a: 0x91,
                flags: Flags {
                    overflow: false,
                    k: false,
                    sign: true,
                    zero: true,
                    auxiliary_carry: true,
                    parity: true,
                    carry: true,
                },
                b: 0x12,
                c: 0x34,
                d: 0x56,
                e: 0x78,
                h: 0x9A,
                l: 0xBC,
                sp: 0xDEF0,
                pc: 0x2001,
            },
            Latches {
                interrupt_enable: true,
                ei_delay: false,
                halted: false,
                mask5_5: true,
                mask6_5: true,
                mask7_5: true,
                rst7_5_pending: false,
                trap_pending: false,
                trap_armed: true,
                trap_saved_interrupt_enable: false,
                trap_rim_pending: false,
                sod: false,
                last_trap: false,
                last_rst7_5: false,
            },
        )
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
            Event::Read {
                address: 0x2000,
                kind: MemoryAccess::OpcodeFetch,
                value: 0x00,
            },
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(true),
            Event::Read {
                address: 0x2001,
                kind: MemoryAccess::InterruptAcknowledge,
                value: 0x08,
            },
        ]
    );
}

#[test]
fn stop_preserves_pending_and_history_when_lines_fall() {
    // Roll back cancellation, re-arm and history as well as new rising edges.
    // Decision-18 removes the fetched-refusal path; retain Stop restoration.
    {
        let mut cpu = cpu_at(0x3000);
        cpu.latches_mut().trap_pending = true;
        cpu.latches_mut().trap_armed = false;
        cpu.latches_mut().last_trap = true;
        cpu.latches_mut().rst7_5_pending = true;
        cpu.latches_mut().last_rst7_5 = true;
        let before = cpu.clone();
        let mut bus = RecordingBus::new();
        bus.load(0x3000, &[0x08]);
        bus.stop = true;
        let report = cpu.step(&mut bus);
        assert_eq!(report.exit, Exit::Stop);
        assert_eq!((report.steps, report.cycles), (0, 0));
        assert_eq!(cpu.latches(), before.latches());
        assert_eq!(cpu, before);
        bus.stop = false;
        bus.load(0x3000, &[0x00]);
        assert_eq!(cpu.step(&mut bus).exit, Exit::Budget);
        assert!(!cpu.latches().trap_pending);
        assert!(cpu.latches().trap_armed);
        assert!(!cpu.latches().last_trap);
        assert!(cpu.latches().rst7_5_pending);
        assert!(!cpu.latches().last_rst7_5);
    }
}

#[test]
fn pulses_only_at_stop_are_not_retained() {
    // Decision-5 item 6 / Control: the embedder must keep the pulse high
    // until a boundary that retains sampling (decision-12 rule 2).
    // Decision-18 removes the fetched-refusal path; retain Stop restoration.
    {
        let mut cpu = cpu_at(0x3000);
        // Keep TRAP from preempting the boundary being tested.
        cpu.latches_mut().ei_delay = true;
        let before = *cpu.latches();
        let mut bus = RecordingBus::new();
        bus.load(0x3000, &[0x08]);
        bus.stop = true;
        bus.trap = true;
        bus.rst7_5 = true;
        let report = cpu.step(&mut bus);
        assert_eq!(report.exit, Exit::Stop);
        assert_eq!(cpu.latches(), &before);
        bus.stop = false;
        bus.trap = false;
        bus.rst7_5 = false;
        bus.load(0x3000, &[0x00]);
        assert_eq!(cpu.step(&mut bus).exit, Exit::Budget);
        assert!(!cpu.latches().trap_pending);
        assert!(!cpu.latches().rst7_5_pending);
        assert!(!cpu.latches().last_trap);
        assert!(!cpu.latches().last_rst7_5);
    }
}

#[test]
fn trap_pending_survives_inhibited_low_boundary_then_eligible_low_clears_it() {
    // Decision-12 rules 2/6: cancellation waits for eligibility. The
    // inhibited low sample still re-arms and updates history (231718-001
    // p. 16-15, Fig. 4). NOP completion expires the EI/DI delay.
    let mut cpu = cpu_at(0x3000);
    cpu.latches_mut().trap_pending = true;
    cpu.latches_mut().last_trap = true;
    cpu.latches_mut().trap_armed = false;
    cpu.latches_mut().ei_delay = true;
    cpu.latches_mut().interrupt_enable = true;
    let mut bus = RecordingBus::new();
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert!(cpu.latches().trap_pending);
    assert!(cpu.latches().trap_armed);
    assert!(!cpu.latches().last_trap);
    assert!(!cpu.latches().ei_delay);
    assert!(cpu.latches().interrupt_enable);
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert!(!cpu.latches().trap_pending);
    assert!(cpu.latches().trap_armed);
    assert!(!cpu.latches().last_trap);
    assert!(!cpu.latches().ei_delay);
    assert!(cpu.latches().interrupt_enable);
}

#[test]
fn trap_pending_survives_inhibited_low_boundary_then_high_is_accepted() {
    // Same rules 2/6 sequence, but high at the next eligible boundary.
    // Acceptance consumes pending and disarms TRAP (rule 4).
    let mut cpu = cpu_at(0x3000);
    cpu.latches_mut().trap_pending = true;
    cpu.latches_mut().last_trap = true;
    cpu.latches_mut().trap_armed = false;
    cpu.latches_mut().ei_delay = true;
    cpu.latches_mut().interrupt_enable = true;
    let mut bus = RecordingBus::new();
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert!(cpu.latches().trap_pending);
    assert!(cpu.latches().trap_armed);
    assert!(!cpu.latches().last_trap);
    assert!(!cpu.latches().ei_delay);
    assert!(cpu.latches().interrupt_enable);
    bus.trap = true;
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 12,
            exit: Exit::Budget
        }
    );
    assert!(!cpu.latches().trap_pending);
    assert!(!cpu.latches().trap_armed);
    assert!(cpu.latches().last_trap);
    assert!(!cpu.latches().ei_delay);
    assert!(!cpu.latches().interrupt_enable);
    assert!(cpu.latches().trap_saved_interrupt_enable);
    assert!(cpu.latches().trap_rim_pending);
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x0024, 0xDEEE));
}

#[test]
fn trap_held_high_does_not_rearm_after_an_embedder_supplied_acceptance_state() {
    // Decision-12 rule 2; 231718-001 p. 16-15: recognition requires low
    // then high after acceptance. Simulate that acceptance's sampler state
    // through the public latch view; the next fresh edge is accepted.
    let mut cpu = cpu_at(0x3000);
    cpu.latches_mut().trap_armed = false;
    cpu.latches_mut().last_trap = true;
    cpu.latches_mut().halted = true;
    let mut bus = RecordingBus::new();
    bus.trap_levels.extend([true, true, false, true]);
    for _ in 0..2 {
        assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
        assert!(!cpu.latches().trap_armed);
        assert!(!cpu.latches().trap_pending);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::TrapPoll(true),
                Event::Rst7_5Poll(false),
                Event::Rst6_5Poll(false),
                Event::Rst5_5Poll(false),
                Event::IntrPoll(false)
            ]
        );
    }
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert!(cpu.latches().trap_armed);
    assert!(!cpu.latches().trap_pending);
    assert_eq!(bus.take_log(), low_polls());
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 12,
            exit: Exit::Budget
        }
    );
    assert!(!cpu.latches().trap_pending);
    assert!(!cpu.latches().trap_armed);
    assert!(!cpu.latches().halted);
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x0024, 0xDEEE));
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(true),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false),
            Event::Write {
                address: 0xDEEF,
                kind: MemoryAccess::Stack,
                value: 0x30
            },
            Event::Write {
                address: 0xDEEE,
                kind: MemoryAccess::Stack,
                value: 0x00
            },
        ]
    );
}

#[test]
fn halt_samples_rst7_5_edge_even_masked_and_disabled_and_retains_it_on_low() {
    // 205775-002 pp. 5-17/5-18 RIM/SIM: the edge sets it even when masked;
    // low does not clear it. Boundary sampler is decision-12 rule 2.
    let mut cpu = cpu_at(0x4000);
    let mut bus = RecordingBus::new();
    cpu.latches_mut().halted = true;
    bus.rst7_5_levels.extend([true, false]);
    assert!(cpu.latches().mask7_5);
    assert!(!cpu.latches().interrupt_enable);
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert!(cpu.latches().rst7_5_pending);
    assert!(cpu.latches().last_rst7_5);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false)
        ]
    );
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert!(cpu.latches().rst7_5_pending);
    assert!(!cpu.latches().last_rst7_5);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false)
        ]
    );
}

#[test]
fn held_levels_without_edges_do_not_set_trap_or_rst7_5_pending() {
    let mut cpu = cpu_at(0x4000);
    cpu.latches_mut().last_trap = true;
    cpu.latches_mut().last_rst7_5 = true;
    cpu.latches_mut().halted = true;
    let before = cpu.clone();
    let mut bus = RecordingBus::new();
    bus.trap = true;
    bus.rst7_5 = true;
    bus.rst6_5 = true;
    bus.rst5_5 = true;
    bus.intr = true;
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert_eq!(cpu, before);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(true),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(true),
            Event::Rst5_5Poll(true),
            Event::IntrPoll(true)
        ]
    );
}

#[test]
fn cleared_rst7_5_flip_flop_needs_a_new_rising_sample() {
    // SIM/acceptance/reset eventually clear the latch (205775-002 p. 5-18).
    // Here the embedder clears it; a held high level cannot set it again.
    let mut cpu = cpu_at(0x4000);
    cpu.latches_mut().halted = true;
    let mut bus = RecordingBus::new();
    bus.rst7_5_levels.extend([true, true, false, true]);
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert!(cpu.latches().rst7_5_pending);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false)
        ]
    );
    cpu.latches_mut().rst7_5_pending = false;
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert!(!cpu.latches().rst7_5_pending);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false)
        ]
    );
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert!(!cpu.latches().rst7_5_pending);
    assert_eq!(bus.take_log(), low_polls());
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert!(cpu.latches().rst7_5_pending);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false)
        ]
    );
}

#[test]
fn eligible_intr_acknowledges_instead_of_fetching_nop() {
    // Decision-12 rule 5; 205775-002 pp. 2-13/2-14, 5-14: RST 7,
    // unchanged boundary PC saved, derived 6+3+3=12 states.
    let mut cpu = cpu_at(0x5000);
    cpu.latches_mut().interrupt_enable = true;
    let mut bus = RecordingBus::new();
    bus.intr = true;
    bus.acknowledges.push_back(0xFF);
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 12,
            exit: Exit::Budget
        }
    );
    assert!(!cpu.latches().interrupt_enable);
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0038,
            sp: 0xDEEE,
            ..*cpu_at(0x5000).registers()
        }
    );
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(true),
            Event::Read {
                address: 0x5000,
                kind: MemoryAccess::InterruptAcknowledge,
                value: 0xFF
            },
            Event::Write {
                address: 0xDEEF,
                kind: MemoryAccess::Stack,
                value: 0x50,
            },
            Event::Write {
                address: 0xDEEE,
                kind: MemoryAccess::Stack,
                value: 0x00,
            },
        ]
    );
}
