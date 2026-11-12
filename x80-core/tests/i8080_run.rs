//! `I8080::run` and `I8080::step`: the budget (decision-2), the boundary
//! sequence and exit precedence (decision-5 items 4, 5 and 8) and the
//! unsupported exit for undocumented opcodes (decision-3, decision-5 item 6).
//!
//! Expected values are literals. The only instruction that executes is NOP:
//! 00000000, one byte, 4 states (98-153B p. 4-14). The undocumented opcodes
//! are the complement of the 98-153B Ch. 4 entries (pp. 4-4 to 4-14), worked
//! out by hand.
//!
//! Interrupt acceptance is not implemented yet (bead x80-ecy.14), so no test
//! here depends on whether an eligible interrupt is accepted.

mod support;

use support::{
    assert_unsupported, is_fetch_unsupported, sample_latches, sample_registers, Event, RecordingBus,
};
use x80_core::i8080::{Latches, I8080};
use x80_core::{Budget, Exit, MemoryAccess, Report};

const UNDOCUMENTED: [u8; 12] = [
    0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0xCB, 0xD9, 0xDD, 0xED, 0xFD,
];

const NOP: u8 = 0x00;

const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

/// The trace of one boundary that fetches `opcode` at `pc`, with both input
/// lines low.
fn boundary(pc: u16, opcode: u8) -> [Event; 3] {
    [
        Event::StopPoll(false),
        Event::InterruptPoll(false),
        Event::Read {
            address: pc,
            kind: MemoryAccess::OpcodeFetch,
            value: opcode,
        },
    ]
}

/// A CPU at `pc` with the sample registers and the given latches.
fn cpu_at(pc: u16, latches: Latches) -> I8080 {
    I8080::new(sample_registers(pc), latches)
}

const RUNNING: Latches = Latches {
    interrupt_enable: false,
    ei_delay: false,
    halted: false,
};

const HALTED: Latches = Latches {
    interrupt_enable: false,
    ei_delay: false,
    halted: true,
};

// Undocumented opcodes.

#[test]
fn every_undocumented_opcode_is_unsupported_and_changes_nothing() {
    for opcode in UNDOCUMENTED {
        assert_unsupported(opcode, 0x1234);
        assert_unsupported(opcode, 0xFFFF);
        assert_unsupported(opcode, 0x0000);
    }
}

#[test]
fn a_repeated_call_returns_the_same_unsupported_exit() {
    let mut bus = RecordingBus::new();
    bus.load(0x4000, &[0xDD]);
    let mut cpu = cpu_at(0x4000, sample_latches());

    let first = cpu.run(&mut bus, UNLIMITED);
    let second = cpu.run(&mut bus, UNLIMITED);

    assert_eq!(first, second);
    assert!(is_fetch_unsupported(first.exit, 0x4000, 0xDD));
    assert_eq!((first.steps, first.cycles), (0, 0));
    assert_eq!(*cpu.registers(), sample_registers(0x4000));
    assert_eq!(*cpu.latches(), sample_latches());
    assert_eq!(
        bus.take_log(),
        [boundary(0x4000, 0xDD), boundary(0x4000, 0xDD)].concat()
    );
}

#[test]
fn unsupported_after_progress_reports_the_earlier_instructions() {
    let mut bus = RecordingBus::new();
    bus.load(0x0100, &[NOP, NOP, 0xCB]);
    let mut cpu = cpu_at(0x0100, RUNNING);

    let report = cpu.run(&mut bus, UNLIMITED);

    assert_eq!((report.steps, report.cycles), (2, 8));
    assert!(is_fetch_unsupported(report.exit, 0x0102, 0xCB));
    assert_eq!(*cpu.registers(), sample_registers(0x0102));
    assert_eq!(
        bus.take_log(),
        [
            boundary(0x0100, NOP),
            boundary(0x0101, NOP),
            boundary(0x0102, 0xCB)
        ]
        .concat()
    );
}

// Budget (decision-2).

/// Runs `cpu` over `bus` with `budget` and checks a `Budget` exit with no
/// progress, no state change and an empty trace.
fn assert_budget_exit_without_access(mut cpu: I8080, mut bus: RecordingBus, budget: Budget) {
    let registers = *cpu.registers();
    let latches = *cpu.latches();
    let report = cpu.run(&mut bus, budget);
    assert_eq!(
        report,
        Report {
            steps: 0,
            cycles: 0,
            exit: Exit::Budget,
        }
    );
    assert_eq!(*cpu.registers(), registers);
    assert_eq!(*cpu.latches(), latches);
    assert_eq!(bus.take_log(), Vec::<Event>::new());
}

#[test]
fn a_zero_step_or_cycle_limit_executes_nothing_and_polls_nothing() {
    for budget in [
        Budget {
            steps: 0,
            cycles: 100,
        },
        Budget {
            steps: 100,
            cycles: 0,
        },
        Budget {
            steps: 0,
            cycles: 0,
        },
    ] {
        assert_budget_exit_without_access(cpu_at(0x0100, RUNNING), RecordingBus::new(), budget);
    }
}

#[test]
fn the_step_limit_holds() {
    let mut bus = RecordingBus::new();
    let mut cpu = cpu_at(0x0100, RUNNING);

    let report = cpu.run(
        &mut bus,
        Budget {
            steps: 3,
            cycles: u64::MAX,
        },
    );

    assert_eq!(
        report,
        Report {
            steps: 3,
            cycles: 12,
            exit: Exit::Budget,
        }
    );
    assert_eq!(cpu.registers().pc, 0x0103);
    // No poll after the third instruction: the limit is checked first.
    assert_eq!(
        bus.take_log(),
        [
            boundary(0x0100, NOP),
            boundary(0x0101, NOP),
            boundary(0x0102, NOP)
        ]
        .concat()
    );
}

#[test]
fn the_cycle_limit_is_checked_at_boundaries_and_may_be_overshot() {
    // (cycle limit, NOPs executed, cycles reported): an instruction starts
    // while fewer cycles than the limit have been spent, and completes.
    for (limit, steps, cycles) in [
        (1, 1, 4),
        (4, 1, 4),
        (5, 2, 8),
        (6, 2, 8),
        (8, 2, 8),
        (9, 3, 12),
    ] {
        let mut bus = RecordingBus::new();
        let mut cpu = cpu_at(0x0200, RUNNING);

        let report = cpu.run(
            &mut bus,
            Budget {
                steps: u64::MAX,
                cycles: limit,
            },
        );

        assert_eq!(
            report,
            Report {
                steps,
                cycles,
                exit: Exit::Budget,
            },
            "cycle limit {limit}"
        );
        assert_eq!(cpu.registers().pc, 0x0200 + steps as u16);
        assert_eq!(bus.log.len(), 3 * steps as usize, "cycle limit {limit}");
    }
}

// Exit precedence (decision-5 item 5, worked pairs reachable without
// interrupt acceptance).

#[test]
fn a_zero_budget_wins_over_stop_interrupt_halt_and_unsupported() {
    let zero = Budget {
        steps: 0,
        cycles: 0,
    };

    let mut stop = RecordingBus::new();
    stop.stop = true;
    assert_budget_exit_without_access(cpu_at(0x0100, RUNNING), stop, zero);

    let mut interrupt = RecordingBus::new();
    interrupt.interrupt = true;
    let enabled = Latches {
        interrupt_enable: true,
        ..RUNNING
    };
    assert_budget_exit_without_access(cpu_at(0x0100, enabled), interrupt, zero);

    // The helper checks that the latches, halt included, are unchanged: the
    // halt stays visible in `latches()`.
    assert_budget_exit_without_access(cpu_at(0x0100, HALTED), RecordingBus::new(), zero);

    let mut undocumented = RecordingBus::new();
    undocumented.load(0x0100, &[0xCB]);
    assert_budget_exit_without_access(cpu_at(0x0100, RUNNING), undocumented, zero);
}

/// Runs with an unlimited budget and checks `exit` with no progress, no
/// state change and a trace of the two polls only.
fn assert_exit_after_polls(mut cpu: I8080, mut bus: RecordingBus, exit: Exit) {
    let registers = *cpu.registers();
    let latches = *cpu.latches();
    let (stop, interrupt) = (bus.stop, bus.interrupt);

    let report = cpu.run(&mut bus, UNLIMITED);

    assert_eq!(
        report,
        Report {
            steps: 0,
            cycles: 0,
            exit,
        }
    );
    assert_eq!(*cpu.registers(), registers);
    assert_eq!(*cpu.latches(), latches);
    assert_eq!(
        bus.take_log(),
        [Event::StopPoll(stop), Event::InterruptPoll(interrupt)]
    );
}

#[test]
fn a_stop_polls_both_lines_and_fetches_nothing() {
    let mut bus = RecordingBus::new();
    bus.stop = true;
    assert_exit_after_polls(cpu_at(0x0100, RUNNING), bus, Exit::Stop);
}

#[test]
fn a_stop_wins_over_halt() {
    let mut bus = RecordingBus::new();
    bus.stop = true;
    assert_exit_after_polls(cpu_at(0x0100, HALTED), bus, Exit::Stop);
}

#[test]
fn a_stop_wins_over_an_undocumented_opcode() {
    let mut bus = RecordingBus::new();
    bus.stop = true;
    bus.load(0x0100, &[0xED]);
    assert_exit_after_polls(cpu_at(0x0100, RUNNING), bus, Exit::Stop);
}

#[test]
fn a_stop_wins_over_an_eligible_interrupt() {
    // INT high, interrupts enabled, no EI delay: eligible (decision-5 item 4).
    let mut bus = RecordingBus::new();
    bus.stop = true;
    bus.interrupt = true;
    let enabled = Latches {
        interrupt_enable: true,
        ..RUNNING
    };
    assert_exit_after_polls(cpu_at(0x0100, enabled), bus, Exit::Stop);
}

#[test]
fn the_inputs_are_polled_again_on_the_next_call_after_a_stop() {
    let mut bus = RecordingBus::new();
    bus.stop = true;
    let mut cpu = cpu_at(0x0100, RUNNING);
    assert_eq!(cpu.run(&mut bus, UNLIMITED).exit, Exit::Stop);
    assert_eq!(
        bus.take_log(),
        [Event::StopPoll(true), Event::InterruptPoll(false)]
    );

    bus.stop = false;
    let report = cpu.step(&mut bus);

    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget,
        }
    );
    assert_eq!(bus.take_log(), boundary(0x0100, NOP));
}

#[test]
fn a_halted_cpu_returns_at_once_without_fetching() {
    assert_exit_after_polls(cpu_at(0x0100, HALTED), RecordingBus::new(), Exit::Halt);
}

#[test]
fn halt_wins_over_an_undocumented_opcode() {
    let mut bus = RecordingBus::new();
    bus.load(0x0100, &[0xFD]);
    assert_exit_after_polls(cpu_at(0x0100, HALTED), bus, Exit::Halt);
}

#[test]
fn a_halted_cpu_with_int_high_and_interrupts_disabled_does_not_spin() {
    let mut bus = RecordingBus::new();
    bus.interrupt = true;
    assert_exit_after_polls(cpu_at(0x0100, HALTED), bus, Exit::Halt);
}

// `step` (decision-5 item 8).

#[test]
fn step_executes_at_most_one_instruction() {
    let mut bus = RecordingBus::new();
    let mut cpu = cpu_at(0x0300, RUNNING);

    let report = cpu.step(&mut bus);

    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget,
        }
    );
    assert_eq!(cpu.registers().pc, 0x0301);
    assert_eq!(bus.take_log(), boundary(0x0300, NOP));
}

#[test]
fn step_reports_halt_and_unsupported_without_progress() {
    assert_exit_after_polls(cpu_at(0x0300, HALTED), RecordingBus::new(), Exit::Halt);

    let mut bus = RecordingBus::new();
    bus.load(0x0300, &[0x38]);
    let mut cpu = cpu_at(0x0300, RUNNING);
    let report = cpu.step(&mut bus);
    assert_eq!((report.steps, report.cycles), (0, 0));
    assert!(is_fetch_unsupported(report.exit, 0x0300, 0x38));
    assert_eq!(bus.take_log(), boundary(0x0300, 0x38));
}

// Determinism (plan §2.5, decision-2 consequences).

/// Runs a fixed sequence of calls over NOPs and an undocumented opcode and
/// returns every report, the trace and the final state.
fn scripted_session() -> (Vec<Report>, Vec<Event>, I8080) {
    let mut bus = RecordingBus::new();
    bus.load(0xFFFD, &[NOP, NOP, NOP, NOP, NOP, 0xD9]);
    let mut cpu = cpu_at(0xFFFD, RUNNING);
    let first = cpu.step(&mut bus);
    let second = cpu.run(
        &mut bus,
        Budget {
            steps: u64::MAX,
            cycles: 6,
        },
    );
    bus.stop = true;
    let third = cpu.run(&mut bus, UNLIMITED);
    bus.stop = false;
    let fourth = cpu.run(&mut bus, UNLIMITED);
    let fifth = cpu.run(&mut bus, UNLIMITED);
    (
        vec![first, second, third, fourth, fifth],
        bus.take_log(),
        cpu,
    )
}

#[test]
fn identical_inputs_give_identical_reports_traces_and_state() {
    let first = scripted_session();
    let second = scripted_session();
    assert_eq!(first, second);

    let (reports, log, cpu) = first;
    assert_eq!(reports.len(), 5);
    assert_eq!(
        reports[..4],
        [
            Report {
                steps: 1,
                cycles: 4,
                exit: Exit::Budget,
            },
            Report {
                steps: 2,
                cycles: 8,
                exit: Exit::Budget,
            },
            Report {
                steps: 0,
                cycles: 0,
                exit: Exit::Stop,
            },
            Report {
                steps: 2,
                cycles: 8,
                exit: reports[3].exit,
            },
        ]
    );
    assert!(is_fetch_unsupported(reports[3].exit, 0x0002, 0xD9));
    assert_eq!(reports[4].exit, reports[3].exit);
    assert_eq!(cpu.registers().pc, 0x0002);
    assert_eq!(log.len(), 3 + 2 * 3 + 2 + 3 * 3 + 3);
}
