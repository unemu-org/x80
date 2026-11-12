//! 8080 whole-program replay and budget splitting (plan §2.5, decision-2;
//! decision-5 items 4/5/8, decision-6).
//!
//! This is a composition witness, using instructions independently witnessed
//! in the group tests. Encodings, behavior and totals: Intel 98-153B
//! (September 1975), Ch. 4 pp. 4-4..4-14 and summary p. 4-15; second witness
//! Intel 9800301-04 (May 1981), Ch. 3 corresponding instruction entries and
//! opcode sequence p. A-5. DAA: pp. 4-8 / 3-19. Interrupt RST: 98-153B
//! Ch. 2 Fig. 2-8 p. 2-11, machine-cycle table pp. 2-18/2-19; 9800301-04
//! Ch. 3 RST pp. 3-54/3-55. Expected totals and final values are independent
//! literals; this test does not consult the implementation's opcode data.

mod support;

use support::{sample_registers, Event, RecordingBus};
use x80_core::i8080::{Flags, Latches, Registers, I8080};
use x80_core::{Budget, Exit, MemoryAccess, Report};

const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

fn fixture() -> (I8080, RecordingBus) {
    let cpu = I8080::new(sample_registers(0x0100), Latches::default());
    let mut bus = RecordingBus::new();
    bus.load(
        0x0100,
        &[
            0x31, 0x00, 0x90, // 0100 LXI SP,9000
            0x21, 0x00, 0x20, // 0103 LXI H,2000
            0x36, 0x09, // 0106 MVI M,09
            0x3E, 0x09, // 0108 MVI A,09
            0x86, // 010A ADD M
            0x27, // 010B DAA: 09 + 09 -> decimal 18
            0x47, // 010C MOV B,A
            0xE6, 0x0F, // 010D ANI 0F
            0xEE, 0x03, // 010F XRI 03
            0xF6, 0x10, // 0111 ORI 10
            0xC5, // 0113 PUSH B
            0xDB, 0x22, // 0114 IN 22
            0xD3, 0x23, // 0116 OUT 23
            0xD1, // 0118 POP D
            0x0E, 0x03, // 0119 MVI C,03
            0x0D, // 011B DCR C
            0xC2, 0x1B, 0x01, // 011C JNZ 011B: taken twice, then untaken
            0xCD, 0x50, 0x01, // 011F CALL 0150
            0xFB, // 0122 EI
            0x00, // 0123 NOP: completes the EI delay
            0x32, 0x01, 0x20, // 0124 STA 2001: interrupted before fetching
            0x76, // 0127 HLT
        ],
    );
    bus.load(0x0150, &[0xAF, 0xC8]); // XRA A; RZ
    bus.load(
        0x0038,
        &[
            0xF5, // PUSH PSW
            0x3E, 0x42, // MVI A,42
            0xD3, 0x24, // OUT 24
            0xF1, // POP PSW
            0xC9, // RET
        ],
    );
    bus.inputs.push_back(0x96);
    bus.acknowledges.push_back(0xFF); // RST 7
                                      // The first 24 instructions complete before EI. INT stays high during
                                      // EI, its following NOP and acceptance, then falls. This script is indexed
                                      // by boundary polls, independent of calls to run (decision-5 item 4).
    bus.interrupt_levels.extend([false; 24]);
    bus.interrupt_levels.extend([true, true, true]);
    (cpu, bus)
}

#[derive(Debug, PartialEq, Eq)]
struct Session {
    cpu: I8080,
    memory: Vec<u8>,
    trace: Vec<Event>,
    reports: Vec<Report>,
    steps: u64,
    cycles: u64,
}

fn session(budget: Budget) -> Session {
    let (mut cpu, mut bus) = fixture();
    let mut reports = Vec::new();
    let (mut steps, mut cycles) = (0, 0);
    // Even a one-step slice needs only 35 calls, including the final Halt
    // boundary. Bound the harness so a broken branch/interrupt cannot hang it.
    for _ in 0..100 {
        let report = cpu.run(&mut bus, budget);
        steps += report.steps;
        cycles += report.cycles; // Actual states, including overshoot; never the limit.
        reports.push(report);
        match report.exit {
            Exit::Halt => {
                assert!(bus.inputs.is_empty());
                assert!(bus.acknowledges.is_empty());
                assert!(bus.interrupt_levels.is_empty());
                return Session {
                    cpu,
                    memory: bus.memory,
                    trace: bus.log,
                    reports,
                    steps,
                    cycles,
                };
            }
            Exit::Budget => {
                assert!(report.steps > 0);
                assert!(report.steps == budget.steps || report.cycles >= budget.cycles);
            }
            other => panic!("unexpected program exit: {other:?}"),
        }
    }
    panic!("program did not halt within 100 calls")
}

fn assert_program_completed(result: &Session) {
    // 34 instructions, including the acknowledged RST and its five-instruction
    // handler; sum of their p. 4-15 totals is 286. The final halt poll adds none.
    assert_eq!((result.steps, result.cycles), (34, 286));
    assert_eq!(result.reports.last().unwrap().exit, Exit::Halt);
    assert_eq!(
        *result.cpu.registers(),
        Registers {
            a: 0,
            b: 0x18,
            c: 0,
            d: 0x18,
            e: 0xC3,
            h: 0x20,
            l: 0,
            sp: 0x9000,
            pc: 0x0128,
            flags: Flags {
                sign: false,
                zero: true,
                auxiliary_carry: false,
                parity: true,
                carry: false,
            },
        }
    );
    assert_eq!(
        *result.cpu.latches(),
        Latches {
            interrupt_enable: false,
            ei_delay: false,
            halted: true,
        }
    );
    assert_eq!(&result.memory[0x2000..0x2002], &[0x09, 0]);
    assert_eq!(&result.memory[0x8FFC..0x9000], &[0x46, 0, 0x24, 0x01]);
    let io: Vec<_> = result
        .trace
        .iter()
        .copied()
        .filter(|event| matches!(event, Event::Input { .. } | Event::Output { .. }))
        .collect();
    assert_eq!(
        io,
        [
            Event::Input {
                port: 0x22,
                value: 0x96
            },
            Event::Output {
                port: 0x23,
                value: 0x96
            },
            Event::Output {
                port: 0x24,
                value: 0x42
            },
        ]
    );
    let acknowledges: Vec<_> = result
        .trace
        .iter()
        .copied()
        .filter(|event| {
            matches!(
                event,
                Event::Read {
                    kind: MemoryAccess::InterruptAcknowledge,
                    ..
                }
            )
        })
        .collect();
    assert_eq!(
        acknowledges,
        [Event::Read {
            address: 0x0124,
            kind: MemoryAccess::InterruptAcknowledge,
            value: 0xFF,
        }]
    );
    assert_eq!(
        result
            .trace
            .iter()
            .filter(|event| matches!(event, Event::StopPoll(false)))
            .count(),
        35
    );
    assert_eq!(
        result
            .trace
            .iter()
            .filter(|event| matches!(event, Event::InterruptPoll(_)))
            .count(),
        35
    );
}

#[test]
fn mixed_program_replays_state_trace_exits_and_counters_identically() {
    let budget = Budget {
        steps: 7,
        cycles: 53,
    };
    let first = session(budget);
    let second = session(budget);
    assert_program_completed(&first);
    // Includes all memory, all reports (exits and counters), registers,
    // latches and every bus call, including both kinds of poll.
    assert_eq!(first, second);
}

fn assert_same_execution(whole: &Session, budget: Budget) {
    let split = session(budget);
    assert_eq!(split.cpu, whole.cpu, "budget {budget:?}");
    assert_eq!(split.memory, whole.memory, "budget {budget:?}");
    assert_eq!(split.trace, whole.trace, "budget {budget:?}");
    assert_eq!(
        (split.steps, split.cycles),
        (whole.steps, whole.cycles),
        "budget {budget:?}"
    );
    assert_eq!(split.reports.last().unwrap().exit, Exit::Halt);
}

#[test]
fn step_and_cycle_splits_preserve_the_entire_execution() {
    let whole = session(UNLIMITED);
    assert_program_completed(&whole);
    assert_eq!(whole.reports.len(), 1);
    // Repeated one-step slices, every larger step slice through the terminal
    // boundary, and every cycle slice through the complete program's total.
    for steps in 1..=35 {
        assert_same_execution(&whole, Budget { steps, ..UNLIMITED });
    }
    for cycles in 1..=287 {
        assert_same_execution(
            &whole,
            Budget {
                cycles,
                ..UNLIMITED
            },
        );
    }
    // Both limits active: either can stop a slice, including around the
    // interrupt and conditional timings.
    for steps in [2, 3, 7, 13, 26, 27, 33, 34] {
        for cycles in [1, 4, 5, 10, 11, 17, 53, 100, 285, 286, 287] {
            assert_same_execution(&whole, Budget { steps, cycles });
        }
    }
    let overshot = session(Budget {
        cycles: 1,
        ..UNLIMITED
    });
    assert_eq!(
        (overshot.reports[0].steps, overshot.reports[0].cycles),
        (1, 10)
    );
    assert_eq!((overshot.steps, overshot.cycles), (34, 286));
}
