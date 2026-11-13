//! Composition witness: plan §2.5, decision-2, decision-5 items 5-8,
//! decision-12 rules 1-8. No expectation uses the crate's map or ALU.
//!
//! Intel 205775-002 (January 1983), instruction entries: LXI/STA p. 5-5,
//! MVI p. 5-4, ADD M p. 5-6, ANI/XRA p. 5-10, CALL p. 5-13,
//! RET/RST p. 5-14, PUSH/POP p. 5-15, IN/OUT p. 5-16,
//! EI/NOP/HLT/RIM p. 5-17, SIM p. 5-18. Table 5-1 p. 5-19 gives
//! states and machine-cycle sequences; Table 5-2 p. 5-20 pins opcodes.
//! Second witness: Intel 9800301-04 (May 1981), App. A pp. A-1..A-6
//! (8085 columns); RIM p. 3-49, SIM p. 3-60. PUSH's 12 states follow
//! 205775-002 p. 5-15/Table 5-1 and 9800301-04 p. A-2; the latter's
//! p. 3-44 instead prints 13 (PROVENANCE.md §1).
//!
//! Interrupt lines/vectors: 205775-002 p. 2-5; INTR's unchanged PC,
//! acknowledge then high/low stack writes: Figs 2-17/2-18 pp. 2-13/2-14.
//! Vectored acceptance: Fig. 2-19 p. 2-15, bus-idle then stack writes,
//! no read. Acceptance's 12 states are derived: 6+3+3, replacing RST's
//! fetch with INA/BI (RST p. 5-14; 9800301-04 p. A-2).
//! TRAP edge/re-arm and saved enable: Intel 231718-001 (September 1987),
//! handbook p. 16-15; RIM/SIM also witnessed there. Page images checked.
//! Boundary sampling, EI/DI inhibition and SIM notification timing retain
//! decision-12's stated API choices/unverified reading.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Budget, Exit, MemoryAccess, Report};

const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

// Boundary PC and actual states, transcribed from the program and the cited
// pages, including acceptance at 10/23/31. The final Halt poll retires nothing.
const RETIREMENTS: [(u16, u64); 36] = [
    (0x0100, 10),
    (0x0103, 10),
    (0x0106, 10),
    (0x0108, 7),
    (0x010A, 7),
    (0x010B, 7),
    (0x010D, 12),
    (0x010E, 10),
    (0x010F, 10),
    (0x0111, 10),
    (0x0113, 12),
    (0x0024, 4),
    (0x0025, 13),
    (0x0028, 4),
    (0x0029, 4),
    (0x002A, 10),
    (0x0113, 18),
    (0x0150, 4),
    (0x0151, 10),
    (0x0116, 4),
    (0x0117, 13),
    (0x011A, 7),
    (0x011C, 4),
    (0x011D, 12),
    (0x003C, 7),
    (0x003E, 4),
    (0x003F, 4),
    (0x0040, 4),
    (0x0041, 10),
    (0x011D, 4),
    (0x011E, 13),
    (0x0121, 12),
    (0x0008, 7),
    (0x000A, 10),
    (0x000C, 10),
    (0x0121, 5),
];

fn fixture() -> (I8085, RecordingBus) {
    let cpu = I8085::new(
        Registers {
            pc: 0x0100,
            b: 0x12,
            c: 0x34,
            ..Registers::default()
        },
        Latches {
            interrupt_enable: true,
            ..Latches::default()
        },
    );
    let mut bus = RecordingBus::new();
    bus.load(
        0x0100,
        &[
            0x31, 0x00, 0x90, // 0100 LXI SP,9000
            0x21, 0x00, 0x20, // 0103 LXI H,2000
            0x36, 0x09, // 0106 MVI M,09
            0x3E, 0x03, // 0108 MVI A,03
            0x86, // 010A ADD M -> 0C
            0xE6, 0x0F, // 010B ANI 0F
            0xC5, // 010D PUSH B
            0xD1, // 010E POP D
            0xDB, 0x22, // 010F IN 22
            0xD3, 0x23, // 0111 OUT 23
            0xCD, 0x50, 0x01, // 0113 CALL 0150 (TRAP before fetch)
            0x20, // 0116 RIM -> 5F
            0x32, 0x01, 0x20, // 0117 STA 2001
            0x3E, 0xCB, // 011A MVI A,CB
            0x30, // 011C SIM: unmask 7.5, keep 6.5/5.5 masked, SOD high
            0x20, // 011D RIM -> AB (RST 7.5 before fetch)
            0x32, 0x03, 0x20, // 011E STA 2003
            0x76, // 0121 HLT (INTR before fetch)
        ],
    );
    bus.load(0x0150, &[0xAF, 0xC9]); // XRA A; RET
    bus.load(
        0x0024,
        &[
            0x20, // RIM -> FF: uses the enable saved by TRAP
            0x32, 0x02, 0x20, // STA 2002
            0xFB, 0x00, 0xC9, // EI; NOP; RET
        ],
    );
    bus.load(
        0x003C,
        &[
            0x3E, 0x40, // MVI A,40
            0x30, // SIM: SOD low, masks unchanged
            0xFB, 0x00, 0xC9, // EI; NOP; RET
        ],
    );
    bus.load(0x0008, &[0x3E, 0x55, 0xD3, 0x24, 0xC9]); // MVI A,55; OUT 24; RET
    bus.inputs.push_back(0x96);
    bus.acknowledges.push_back(0xCF); // INTR supplies RST 1
    bus.sid_levels.extend([true, false, true]);
    // Scripts count calls, never run slices. A single early 7.5 edge is
    // latched while masked, then stays low through SIM and acceptance.
    // TRAP has a sampled low before and after its rising edge. The two level
    // lines stay masked; RIM's extra reads deliberately differ from the
    // boundary reads (decision-12 rule 8).
    for boundary in 0..=36 {
        bus.trap_levels.push_back(boundary == 10);
        bus.rst7_5_levels.push_back(boundary == 1);
        bus.intr_levels.push_back(boundary == 31);
        bus.rst6_5_levels.push_back(true);
        bus.rst5_5_levels.push_back(true);
        match boundary {
            11 => {
                bus.rst6_5_levels.push_back(true);
                bus.rst5_5_levels.push_back(true);
            }
            19 => {
                bus.rst6_5_levels.push_back(false);
                bus.rst5_5_levels.push_back(true);
            }
            29 => {
                bus.rst6_5_levels.push_back(true);
                bus.rst5_5_levels.push_back(false);
            }
            _ => {}
        }
    }
    (cpu, bus)
}

#[derive(Debug, PartialEq, Eq)]
struct Session {
    cpu: I8085,
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
    for _ in 0..100 {
        let report = cpu.run(&mut bus, budget);
        steps += report.steps;
        cycles += report.cycles; // Actual states, including any overshoot.
        reports.push(report);
        match report.exit {
            Exit::Halt => {
                assert!(bus.inputs.is_empty());
                assert!(bus.acknowledges.is_empty());
                assert!(bus.trap_levels.is_empty());
                assert!(bus.rst7_5_levels.is_empty());
                assert!(bus.rst6_5_levels.is_empty());
                assert!(bus.rst5_5_levels.is_empty());
                assert!(bus.intr_levels.is_empty());
                assert!(bus.sid_levels.is_empty());
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
    assert_eq!((result.steps, result.cycles), (36, 302));
    assert_eq!(
        RETIREMENTS.iter().map(|(_, states)| states).sum::<u64>(),
        302
    );
    assert_eq!(result.reports.last().unwrap().exit, Exit::Halt);
    assert_eq!(
        *result.cpu.registers(),
        Registers {
            a: 0x55,
            b: 0x12,
            c: 0x34,
            d: 0x12,
            e: 0x34,
            h: 0x20,
            l: 0,
            sp: 0x9000,
            pc: 0x0122,
            flags: Flags {
                overflow: false,
                k: false,
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
            mask5_5: true,
            mask6_5: true,
            mask7_5: false,
            rst7_5_pending: false,
            trap_pending: false,
            trap_armed: true,
            trap_saved_interrupt_enable: true,
            trap_rim_pending: false,
            sod: false,
            last_trap: false,
            last_rst7_5: false,
        }
    );
    assert_eq!(&result.memory[0x2000..0x2004], &[0x09, 0x5F, 0xFF, 0xAB]);
    assert_eq!(&result.memory[0x8FFE..0x9000], &[0x21, 0x01]);

    let io: Vec<_> = result
        .trace
        .iter()
        .copied()
        .filter(|event| {
            matches!(
                event,
                Event::Input { .. } | Event::Output { .. } | Event::SodChanged(_)
            )
        })
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
            Event::SodChanged(true),
            Event::SodChanged(false),
            Event::Output {
                port: 0x24,
                value: 0x55
            },
        ]
    );

    // Pin every boundary's six polls and fetch/acceptance. In particular the
    // two internally vectored entries have no opcode/acknowledge read, and
    // INTR acknowledges the unchanged PC before high/low Stack writes.
    let starts: Vec<_> = result
        .trace
        .iter()
        .enumerate()
        .filter_map(|(at, event)| matches!(event, Event::StopPoll(_)).then_some(at))
        .collect();
    assert_eq!(starts.len(), 37);
    for (boundary, &start) in starts.iter().enumerate() {
        let end = starts
            .get(boundary + 1)
            .copied()
            .unwrap_or(result.trace.len());
        let events = &result.trace[start..end];
        assert_eq!(
            &events[..6],
            &[
                Event::StopPoll(false),
                Event::TrapPoll(boundary == 10),
                Event::Rst7_5Poll(boundary == 1),
                Event::Rst6_5Poll(true),
                Event::Rst5_5Poll(true),
                Event::IntrPoll(boundary == 31),
            ]
        );
        if boundary == 36 {
            assert_eq!(events.len(), 6);
            continue;
        }
        let pc = RETIREMENTS[boundary].0;
        if matches!(boundary, 10 | 23 | 31) {
            let mut expected = Vec::new();
            if boundary == 31 {
                expected.push(Event::Read {
                    address: 0x0121,
                    kind: MemoryAccess::InterruptAcknowledge,
                    value: 0xCF,
                });
            }
            expected.extend([
                Event::Write {
                    address: 0x8FFF,
                    kind: MemoryAccess::Stack,
                    value: 0x01,
                },
                Event::Write {
                    address: 0x8FFE,
                    kind: MemoryAccess::Stack,
                    value: match boundary {
                        10 => 0x13,
                        23 => 0x1D,
                        _ => 0x21,
                    },
                },
            ]);
            assert_eq!(&events[6..], expected);
        } else {
            assert!(matches!(events[6], Event::Read {
                address, kind: MemoryAccess::OpcodeFetch, ..
            } if address == pc));
        }
        if let Some((sid, rst6_5, rst5_5)) = match boundary {
            11 => Some((true, true, true)),
            19 => Some((false, false, true)),
            29 => Some((true, true, false)),
            _ => None,
        } {
            assert_eq!(
                &events[7..],
                &[
                    Event::SidPoll(sid),
                    Event::Rst6_5Poll(rst6_5),
                    Event::Rst5_5Poll(rst5_5),
                ]
            );
        }
    }
    assert_eq!(
        result
            .trace
            .iter()
            .filter(|event| matches!(
                event,
                Event::Read {
                    kind: MemoryAccess::InterruptAcknowledge,
                    ..
                }
            ))
            .count(),
        1
    );
}

#[test]
fn scripted_boundaries_pin_interrupts_masks_and_rim_values() {
    let (mut cpu, mut bus) = fixture();
    for (boundary, (pc, states)) in RETIREMENTS.into_iter().enumerate() {
        assert_eq!(cpu.registers().pc, pc, "boundary {boundary}");
        let report = cpu.step(&mut bus);
        assert_eq!(
            (report.exit, report.steps, report.cycles),
            (Exit::Budget, 1, states)
        );
        match boundary {
            1 => assert!(cpu.latches().mask7_5 && cpu.latches().rst7_5_pending),
            4 | 5 => assert_eq!(cpu.registers().a, 0x0C),
            10 => {
                assert_eq!(cpu.registers().pc, 0x0024);
                assert!(!cpu.latches().interrupt_enable);
                assert!(cpu.latches().trap_rim_pending);
            }
            11 => {
                assert_eq!(cpu.registers().a, 0xFF);
                assert!(!cpu.latches().trap_rim_pending);
            }
            19 => assert_eq!(cpu.registers().a, 0x5F),
            22 => {
                assert!(!cpu.latches().mask7_5);
                assert!(cpu.latches().rst7_5_pending && cpu.latches().sod);
                assert!(cpu.latches().mask6_5 && cpu.latches().mask5_5);
            }
            23 => {
                assert_eq!(cpu.registers().pc, 0x003C);
                assert!(!cpu.latches().rst7_5_pending);
            }
            25 => assert!(!cpu.latches().sod),
            29 => assert_eq!(cpu.registers().a, 0xAB),
            31 => {
                assert_eq!(cpu.registers().pc, 0x0008);
                assert!(!cpu.latches().interrupt_enable);
            }
            _ => {}
        }
    }
    let halted = cpu.step(&mut bus);
    assert_eq!(
        (halted.exit, halted.steps, halted.cycles),
        (Exit::Halt, 0, 0)
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
    assert_eq!(first, second);
}

fn assert_same_execution(whole: &Session, budget: Budget) {
    let split = session(budget);
    assert_eq!(split.cpu, whole.cpu, "{budget:?}");
    assert_eq!(split.memory, whole.memory, "{budget:?}");
    assert_eq!(split.trace, whole.trace, "{budget:?}");
    assert_eq!(
        (split.steps, split.cycles),
        (whole.steps, whole.cycles),
        "{budget:?}"
    );
    assert_eq!(split.reports.last().unwrap().exit, Exit::Halt);
}

#[test]
fn step_and_cycle_splits_preserve_the_entire_execution() {
    let whole = session(UNLIMITED);
    assert_program_completed(&whole);
    assert_eq!(whole.reports.len(), 1);
    for steps in 1..=37 {
        assert_same_execution(&whole, Budget { steps, ..UNLIMITED });
    }
    for cycles in 1..=303 {
        assert_same_execution(
            &whole,
            Budget {
                cycles,
                ..UNLIMITED
            },
        );
    }
    for steps in [2, 3, 7, 11, 23, 31, 35, 36] {
        for cycles in [1, 4, 5, 7, 10, 12, 13, 18, 53, 301, 302, 303] {
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
    assert_eq!(overshot.reports.last().unwrap().steps, 0);
}
