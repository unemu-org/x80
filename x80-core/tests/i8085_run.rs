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

const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

fn cpu_at(pc: u16) -> I8085 {
    I8085::new(
        Registers {
            a: 0x91,
            flags: Flags {
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
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert!(cpu.latches().trap_pending);
    assert!(cpu.latches().rst7_5_pending);
    assert!(cpu.latches().last_trap);
    assert!(cpu.latches().last_rst7_5);
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
fn halt_follows_all_polls_and_declined_interrupts_without_fetch() {
    // Acceptance is a stub even for enabled, unmasked sources. Each priority
    // candidate is exercised; no acknowledge, stack access or wake occurs.
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
        bus.load(0x1000, &[0x08]);
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 0,
                cycles: 0,
                exit: Exit::Halt
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
        assert_eq!(bus.take_log(), expected_polls);
        assert_eq!(cpu.registers(), cpu_at(0x1000).registers());
        assert!(cpu.latches().halted);
        assert!(cpu.latches().interrupt_enable);
    }
}

fn assert_fetch_unsupported(exit: Exit, pc: u16, opcode: u8) {
    let Exit::Unsupported(unsupported) = exit else {
        panic!("expected Unsupported, got {exit:?}")
    };
    assert_eq!(
        (unsupported.pc, unsupported.opcode, unsupported.acknowledge),
        (pc, opcode, false)
    );
}

fn unsupported_byte_has_no_instruction_effects(opcode: u8) {
    let mut cpu = cpu_at(0x1234);
    cpu.latches_mut().ei_delay = true;
    cpu.latches_mut().rst7_5_pending = true;
    cpu.latches_mut().sod = true;
    let before = cpu.clone();
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[opcode, 0xAA, 0xBB]);
    let memory = bus.memory.clone();
    for _ in 0..2 {
        let report = cpu.step(&mut bus);
        assert_eq!((report.steps, report.cycles), (0, 0));
        assert_fetch_unsupported(report.exit, 0x1234, opcode);
        assert_eq!(cpu, before, "{opcode:02X}");
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
                    address: 0x1234,
                    kind: MemoryAccess::OpcodeFetch,
                    value: opcode
                }
            ]
        );
    }
}

#[test]
fn ten_undocumented_bytes_are_unsupported_without_state_change() {
    // Table 5-2 p. 5-20 and 9800301-04 p. A-5's literal dashes.
    for opcode in [0x08, 0x10, 0x18, 0x28, 0x38, 0xCB, 0xD9, 0xDD, 0xED, 0xFD] {
        unsupported_byte_has_no_instruction_effects(opcode);
    }
}

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
fn unsupported_reports_earlier_nop_progress() {
    let mut cpu = cpu_at(0x2000);
    cpu.latches_mut().ei_delay = true;
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0x00, 0x08]);
    bus.trap_levels.extend([false, true]);
    bus.rst7_5_levels.extend([false, true]);
    let report = cpu.run(&mut bus, UNLIMITED);
    assert_eq!((report.steps, report.cycles), (1, 4));
    assert_fetch_unsupported(report.exit, 0x2001, 0x08);
    assert_eq!(cpu.registers().pc, 0x2001);
    assert!(!cpu.latches().ei_delay);
    assert!(!cpu.latches().trap_pending);
    assert!(!cpu.latches().rst7_5_pending);
    assert!(!cpu.latches().last_trap);
    assert!(!cpu.latches().last_rst7_5);
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
            Event::Read {
                address: 0x2001,
                kind: MemoryAccess::OpcodeFetch,
                value: 0x08
            },
        ]
    );
}

#[test]
fn unsupported_preserves_latches_then_held_edges_are_sampled() {
    // Control/decision-5 item 6 preserve the pre-sampling boundary state.
    let mut cpu = cpu_at(0x3000);
    let before = cpu.clone();
    let mut bus = RecordingBus::new();
    bus.load(0x3000, &[0x08]);
    bus.trap = true;
    bus.rst7_5 = true;
    let report = cpu.step(&mut bus);
    assert_eq!((report.steps, report.cycles), (0, 0));
    assert_fetch_unsupported(report.exit, 0x3000, 0x08);
    assert_eq!(cpu.latches(), before.latches());
    assert_eq!(cpu, before);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(true),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false),
            Event::Read {
                address: 0x3000,
                kind: MemoryAccess::OpcodeFetch,
                value: 0x08
            }
        ]
    );
    bus.load(0x3000, &[0x00]);
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert!(cpu.latches().trap_pending);
    assert!(cpu.latches().rst7_5_pending);
    assert!(cpu.latches().last_trap);
    assert!(cpu.latches().last_rst7_5);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(true),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(false),
            Event::Read {
                address: 0x3000,
                kind: MemoryAccess::OpcodeFetch,
                value: 0x00
            }
        ]
    );
}

#[test]
fn stop_and_unsupported_preserve_pending_and_history_when_lines_fall() {
    // Roll back cancellation, re-arm and history as well as new rising edges.
    for stop in [true, false] {
        let mut cpu = cpu_at(0x3000);
        cpu.latches_mut().trap_pending = true;
        cpu.latches_mut().trap_armed = false;
        cpu.latches_mut().last_trap = true;
        cpu.latches_mut().rst7_5_pending = true;
        cpu.latches_mut().last_rst7_5 = true;
        let before = cpu.clone();
        let mut bus = RecordingBus::new();
        bus.load(0x3000, &[0x08]);
        bus.stop = stop;
        let report = cpu.step(&mut bus);
        if stop {
            assert_eq!(report.exit, Exit::Stop);
        } else {
            assert_fetch_unsupported(report.exit, 0x3000, 0x08);
        }
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
fn pulses_only_at_stop_or_unsupported_are_not_retained() {
    // Decision-5 item 6 / Control: the embedder must keep the pulse high
    // until a boundary that retains sampling (decision-12 rule 2).
    for stop in [true, false] {
        let mut cpu = cpu_at(0x3000);
        let before = *cpu.latches();
        let mut bus = RecordingBus::new();
        bus.load(0x3000, &[0x08]);
        bus.stop = stop;
        bus.trap = true;
        bus.rst7_5 = true;
        let report = cpu.step(&mut bus);
        if stop {
            assert_eq!(report.exit, Exit::Stop);
        } else {
            assert_fetch_unsupported(report.exit, 0x3000, 0x08);
        }
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
fn trap_pending_survives_inhibited_low_boundary_then_high_stays_pending() {
    // Same rules 2/6 sequence, but high at the next eligible boundary.
    // The acceptance seam declines, so pending remains observable.
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
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert!(cpu.latches().trap_pending);
    assert!(cpu.latches().trap_armed);
    assert!(cpu.latches().last_trap);
    assert!(!cpu.latches().ei_delay);
    assert!(cpu.latches().interrupt_enable);
}

#[test]
fn trap_held_high_does_not_rearm_after_an_embedder_supplied_acceptance_state() {
    // Decision-12 rule 2; 231718-001 p. 16-15: recognition requires low
    // then high after acceptance. Simulate that acceptance's sampler state
    // through the public latch view; the seam itself still never accepts.
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
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    assert!(cpu.latches().trap_pending);
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
fn declined_eligible_intr_falls_through_to_nop_without_acknowledging() {
    let mut cpu = cpu_at(0x5000);
    cpu.latches_mut().interrupt_enable = true;
    let mut bus = RecordingBus::new();
    bus.intr = true;
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert!(cpu.latches().interrupt_enable);
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
                kind: MemoryAccess::OpcodeFetch,
                value: 0
            }
        ]
    );
}
