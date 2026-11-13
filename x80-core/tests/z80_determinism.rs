//! Composition/replay witness: decisions 2, 13–15; no map/ALU expectations.
//!
//! Primary: UM008011-0816 rev. 11 (August 2016), Tables 6/7 pp. 42/45
//! (loads, reconciled by entries pp. 74–87,99–108), LD I,A p. 96,
//! LDIR pp. 132–133, NOP/HALT/DI/EI/IM pp. 180–186, RLC pp. 213–219,
//! RETI/RETN pp. 288/290, IN/OUT pp. 295–297/306–307. Second witness:
//! 03-0029-01 (1977), Tables 7.0-1/2/3/5/7/10/11 pp. 44–46,48,50,53–54.
//! Interrupts: UM Figs 9/10 pp. 13–14, Table 1 pp. 18–19, CPU Response
//! pp. 19–20; TM §4 p. 18, §8 pp. 55–57. R: UM p. 3; TM §§2/3 pp. 4/7.
//! All encoded values checked on rendered 150 dpi pages (PDF +14 / +6).
//! NMI's 11 and IM 1's 13 are derived under decision-15 rules 8/9;
//! IM 2's 19 is stated. LDIR's per-iteration P/V follows UM p. 133 and
//! the existing PROVENANCE §1 resolution, not TM's completed-result zero.
//! Poll order, one iteration/step, access kinds and prompt Halt are API
//! choices; indexed-CB fourth-byte OperandFetch remains unverified (rule 5).
//! Decision-20 §4 now supplies full flags; this sequence retains its
//! literal boundary states. MEASURED historically on base 50af936: all
//! three tests pass before staging removal. These are regression witnesses.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, MemoryAccess, Report};

const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};
const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ARG: MemoryAccess = MemoryAccess::OperandFetch;
const DATA: MemoryAccess = MemoryAccess::Data;
const STACK: MemoryAccess = MemoryAccess::Stack;
const ACK: MemoryAccess = MemoryAccess::InterruptAcknowledge;

// Literal boundary PC, complete cost, number of M1 stream bytes, stream.
// Empty streams are interrupt acceptances, with their reads pinned below.
// These rows are independent of the fixture's program and the crate's map.
const RETIREMENTS: [(u16, u64, usize, &[u8]); 40] = [
    (0x0100, 10, 1, &[0x31, 0x00, 0x90]),       // 0
    (0x0103, 14, 2, &[0xDD, 0x21, 0x00, 0x30]), // 1
    (0x0107, 14, 2, &[0xFD, 0x21, 0x00, 0x31]), // 2
    (0x010B, 19, 2, &[0xDD, 0x36, 0x01, 0x81]), // 3
    (0x010F, 19, 2, &[0xFD, 0x7E, 0xFF]),       // 4
    (0x0112, 8, 2, &[0xCB, 0x07]),              // 5
    (0x0114, 23, 2, &[0xDD, 0xCB, 0x01, 0x06]), // 6
    (0x0118, 10, 1, &[0x21, 0x01, 0x30]),       // 7
    (0x011B, 10, 1, &[0x11, 0x00, 0x32]),       // 8
    (0x011E, 10, 1, &[0x01, 0x02, 0x00]),       // 9
    (0x0121, 21, 2, &[0xED, 0xB0]),             // 10
    (0x0121, 11, 0, &[]),                       // 11
    (0x0066, 14, 2, &[0xED, 0x45]),             // 12
    (0x0121, 16, 2, &[0xED, 0xB0]),             // 13
    (0x0123, 10, 1, &[0x01, 0x34, 0x12]),       // 14
    (0x0126, 12, 2, &[0xED, 0x78]),             // 15
    (0x0128, 12, 2, &[0xED, 0x79]),             // 16
    (0x012A, 11, 1, &[0xD3, 0x56]),             // 17
    (0x012C, 11, 1, &[0xDB, 0x78]),             // 18
    (0x012E, 8, 2, &[0xED, 0x56]),              // 19
    (0x0130, 4, 1, &[0xFB]),                    // 20
    (0x0131, 4, 1, &[0x00]),                    // 21
    (0x0132, 4, 1, &[0x76]),                    // 22
    (0x0133, 13, 0, &[]),                       // 23
    (0x0038, 4, 1, &[0xFB]),                    // 24
    (0x0039, 4, 1, &[0x00]),                    // 25
    (0x003A, 14, 2, &[0xED, 0x4D]),             // 26
    (0x0133, 4, 1, &[0xF3]),                    // 27
    (0x0134, 8, 2, &[0xED, 0x5E]),              // 28
    (0x0136, 7, 1, &[0x3E, 0x40]),              // 29
    (0x0138, 9, 2, &[0xED, 0x47]),              // 30
    (0x013A, 4, 1, &[0xFB]),                    // 31
    (0x013B, 4, 1, &[0x00]),                    // 32
    (0x013C, 4, 1, &[0x76]),                    // 33
    (0x013D, 19, 0, &[]),                       // 34
    (0x0200, 7, 1, &[0x3E, 0xA5]),              // 35
    (0x0202, 12, 2, &[0xED, 0x79]),             // 36
    (0x0204, 14, 2, &[0xED, 0x4D]),             // 37
    (0x013D, 4, 1, &[0xF3]),                    // 38
    (0x013E, 4, 1, &[0x76]),                    // 39
];

fn fixture() -> (Z80, RecordingBus) {
    let cpu = Z80::new(
        Registers {
            pc: 0x0100,
            r: 0xF0,
            ..Registers::default()
        },
        Latches::default(),
    );
    let mut bus = RecordingBus::new();
    bus.load(
        0x0100,
        &[
            0x31, 0x00, 0x90, // 0100 LD SP,9000
            0xDD, 0x21, 0x00, 0x30, // 0103 LD IX,3000
            0xFD, 0x21, 0x00, 0x31, // 0107 LD IY,3100
            0xDD, 0x36, 0x01, 0x81, // 010B LD (IX+1),81
            0xFD, 0x7E, 0xFF, // 010F LD A,(IY-1)
            0xCB, 0x07, // 0112 RLC A: 42 -> 84
            0xDD, 0xCB, 0x01, 0x06, // 0114 RLC (IX+1): 81 -> 03
            0x21, 0x01, 0x30, // 0118 LD HL,3001
            0x11, 0x00, 0x32, // 011B LD DE,3200
            0x01, 0x02, 0x00, // 011E LD BC,2
            0xED, 0xB0, // 0121 LDIR: NMI between its two iterations
            0x01, 0x34, 0x12, // 0123 LD BC,1234
            0xED, 0x78, // 0126 IN A,(C): port 1234 -> 96
            0xED, 0x79, // 0128 OUT (C),A: port 1234 <- 96
            0xD3, 0x56, // 012A OUT (56),A: port 9656 <- 96
            0xDB, 0x78, // 012C IN A,(78): port 9678 -> 55
            0xED, 0x56, // 012E IM 1
            0xFB, 0x00, 0x76, // 0130 EI; NOP; HALT; INT wakes it
            0xF3, // 0133 DI
            0xED, 0x5E, // 0134 IM 2
            0x3E, 0x40, // 0136 LD A,40
            0xED, 0x47, // 0138 LD I,A
            0xFB, 0x00, 0x76, // 013A EI; NOP; HALT; IM 2 wakes it
            0xF3, 0x76, // 013D DI; HALT: final prompt Halt
        ],
    );
    bus.load(0x0066, &[0xED, 0x45]); // RETN to the rewound LDIR
    bus.load(0x0038, &[0xFB, 0x00, 0xED, 0x4D]); // EI; NOP; RETI
    bus.load(0x0200, &[0x3E, 0xA5, 0xED, 0x79, 0xED, 0x4D]); // LD A,A5; OUT(C),A; RETI
    bus.load(0x4020, &[0x00, 0x02]); // IM 2 pointer -> 0200
    bus.load(0x3002, &[0x5A]);
    bus.load(0x30FF, &[0x42]);
    bus.inputs.extend([0x96, 0x55]);
    bus.acknowledges.extend([0xA7, 0x20]); // IM 1 ignores A7; IM 2 uses even 20
    for boundary in 0..=40 {
        // NMI held across RETN and the resumed LDIR, then deasserted.
        bus.nmi_levels.push_back(matches!(boundary, 11..=13));
        // Assert at EI and the delayed NOP, lower for HALT, raise to wake;
        // also assert while disabled after DI and at final Halt's poll.
        bus.int_levels.push_back(matches!(
            boundary,
            20 | 21 | 23 | 28 | 31 | 32 | 34 | 39 | 40
        ));
    }
    (cpu, bus)
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

fn write(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Write {
        address,
        kind,
        value,
    }
}

fn expected_polls(boundary: usize) -> [Event; 3] {
    [
        Event::StopPoll(false),
        Event::NmiPoll(matches!(boundary, 11..=13)),
        Event::IntPoll(matches!(
            boundary,
            20 | 21 | 23 | 28 | 31 | 32 | 34 | 39 | 40
        )),
    ]
}

fn expected_trace() -> Vec<Event> {
    let mut trace = Vec::new();
    for (boundary, &(pc, _, m1s, bytes)) in RETIREMENTS.iter().enumerate() {
        trace.extend(expected_polls(boundary));
        for (offset, &value) in bytes.iter().enumerate() {
            trace.push(read(
                pc + offset as u16,
                if offset < m1s { OP } else { ARG },
                value,
            ));
        }
        // Literal data/stack/port accesses after the instruction stream;
        // the interrupt entries include their discarded read/acknowledge.
        match boundary {
            3 => trace.push(write(0x3001, DATA, 0x81)),
            4 => trace.push(read(0x30FF, DATA, 0x42)),
            6 => trace.extend([read(0x3001, DATA, 0x81), write(0x3001, DATA, 0x03)]),
            10 => trace.extend([read(0x3001, DATA, 0x03), write(0x3200, DATA, 0x03)]),
            11 => trace.extend([
                read(0x0121, OP, 0xED),
                write(0x8FFF, STACK, 0x01),
                write(0x8FFE, STACK, 0x21),
            ]),
            12 => trace.extend([read(0x8FFE, STACK, 0x21), read(0x8FFF, STACK, 0x01)]),
            13 => trace.extend([read(0x3002, DATA, 0x5A), write(0x3201, DATA, 0x5A)]),
            15 => trace.push(Event::Input16 {
                port: 0x1234,
                value: 0x96,
            }),
            16 => trace.push(Event::Output16 {
                port: 0x1234,
                value: 0x96,
            }),
            17 => trace.push(Event::Output16 {
                port: 0x9656,
                value: 0x96,
            }),
            18 => trace.push(Event::Input16 {
                port: 0x9678,
                value: 0x55,
            }),
            23 => trace.extend([
                read(0x0133, ACK, 0xA7),
                write(0x8FFF, STACK, 0x01),
                write(0x8FFE, STACK, 0x33),
            ]),
            26 => trace.extend([read(0x8FFE, STACK, 0x33), read(0x8FFF, STACK, 0x01)]),
            34 => trace.extend([
                read(0x013D, ACK, 0x20),
                write(0x8FFF, STACK, 0x01),
                write(0x8FFE, STACK, 0x3D),
                read(0x4020, DATA, 0x00),
                read(0x4021, DATA, 0x02),
            ]),
            36 => trace.push(Event::Output16 {
                port: 0x1234,
                value: 0xA5,
            }),
            37 => trace.extend([read(0x8FFE, STACK, 0x3D), read(0x8FFF, STACK, 0x01)]),
            _ => {}
        }
    }
    trace.extend(expected_polls(40)); // Prompt Halt polls, no read/retirement.
    trace
}

#[derive(Debug, PartialEq, Eq)]
struct Session {
    cpu: Z80,
    memory: Vec<u8>,
    trace: Vec<Event>,
    reports: Vec<Report>,
    steps: u64,
    cycles: u64,
}

fn session(budgets: &[Budget], stepping: bool) -> Session {
    assert!(!budgets.is_empty());
    let (mut cpu, mut bus) = fixture();
    let (mut steps, mut cycles) = (0, 0);
    let mut reports = Vec::new();
    for call in 0..100 {
        let budget = budgets[call % budgets.len()];
        let report = if stepping {
            cpu.step(&mut bus)
        } else {
            cpu.run(&mut bus, budget)
        };
        steps += report.steps;
        cycles += report.cycles;
        reports.push(report);
        match report.exit {
            Exit::Halt => {
                assert!(bus.inputs.is_empty() && bus.acknowledges.is_empty());
                assert!(bus.nmi_levels.is_empty() && bus.int_levels.is_empty());
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
                if !stepping {
                    assert!(report.steps == budget.steps || report.cycles >= budget.cycles);
                }
            }
            other => panic!("unexpected program exit: {other:?}"),
        }
    }
    panic!("program did not halt within 100 calls")
}

fn assert_completed(result: &Session) {
    assert_eq!((result.steps, result.cycles), (40, 410));
    assert_eq!(RETIREMENTS.iter().map(|row| row.1).sum::<u64>(), 410);
    assert_eq!(result.reports.last().unwrap().exit, Exit::Halt);
    assert_eq!(
        *result.cpu.registers(),
        Registers {
            a: 0xA5,
            flags: Flags {
                sign: true,
                zero: false,
                y: false,
                x: false,
                half_carry: false,
                parity_overflow: true,
                subtract: false,
                carry: true
            },
            b: 0x12,
            c: 0x34,
            d: 0x32,
            e: 0x02,
            h: 0x30,
            l: 0x03,
            ix: 0x3000,
            iy: 0x3100,
            sp: 0x9000,
            pc: 0x013F,
            i: 0x40,
            r: 0xA9, // Derived: F0 low-seven-bit wrap + 57 M1s, bit 7 kept.
            ..Registers::default()
        }
    );
    assert_eq!(
        *result.cpu.latches(),
        Latches {
            wz: 0x013D,
            interrupt_mode: InterruptMode::Mode2,
            halted: true,
            ..Latches::default()
        }
    );
    let (_, mut expected) = fixture();
    expected.load(0x3001, &[0x03]);
    expected.load(0x3200, &[0x03, 0x5A]);
    expected.load(0x8FFE, &[0x3D, 0x01]);
    assert_eq!(result.memory, expected.memory);
    assert_eq!(result.trace, expected_trace());
}

#[test]
fn literal_boundaries_pin_prefixes_block_resume_interrupts_and_halt_wakeup() {
    let (mut cpu, mut bus) = fixture();
    for (boundary, (pc, states, _, _)) in RETIREMENTS.into_iter().enumerate() {
        assert_eq!(cpu.registers().pc, pc, "boundary {boundary}");
        let report = cpu.step(&mut bus);
        assert_eq!(
            report,
            Report {
                steps: 1,
                cycles: states,
                exit: Exit::Budget
            },
            "boundary {boundary}"
        );
        match boundary {
            5 => assert_eq!(cpu.registers().a, 0x84),
            10 => {
                assert_eq!((cpu.registers().b, cpu.registers().c), (0, 1));
                assert!(cpu.registers().flags.parity_overflow);
            }
            11 => {
                assert_eq!(cpu.registers().pc, 0x0066);
                assert!(!cpu.latches().iff1 && !cpu.latches().iff2);
            }
            13 => {
                assert_eq!((cpu.registers().b, cpu.registers().c), (0, 0));
                assert!(!cpu.registers().flags.parity_overflow);
            }
            20 | 24 | 31 => {
                assert!(cpu.latches().iff1 && cpu.latches().iff2 && cpu.latches().ei_delay)
            }
            22 | 33 | 39 => assert!(cpu.latches().halted),
            23 | 34 => assert!(!cpu.latches().halted && !cpu.latches().iff1 && !cpu.latches().iff2),
            27 | 38 => {
                assert!(!cpu.latches().iff1 && !cpu.latches().iff2 && !cpu.latches().ei_delay)
            }
            _ => {}
        }
    }
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 0,
            cycles: 0,
            exit: Exit::Halt
        }
    );
    assert_eq!(bus.log, expected_trace());
}

#[test]
fn identical_scripts_replay_reports_state_and_all_bus_calls() {
    let budgets = [
        Budget {
            steps: 7,
            cycles: 53,
        },
        Budget {
            steps: 3,
            cycles: 29,
        },
    ];
    let first = session(&budgets, false);
    let second = session(&budgets, false);
    assert_completed(&first);
    assert_eq!(first, second);
}

fn assert_same_execution(whole: &Session, budgets: &[Budget], stepping: bool) {
    let split = session(budgets, stepping);
    assert_eq!(split.cpu, whole.cpu, "{budgets:?}");
    assert_eq!(split.memory, whole.memory, "{budgets:?}");
    assert_eq!(split.trace, whole.trace, "{budgets:?}");
    let polls = |trace: &[Event]| {
        trace
            .iter()
            .copied()
            .filter(|event| {
                matches!(
                    event,
                    Event::StopPoll(_) | Event::NmiPoll(_) | Event::IntPoll(_)
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(polls(&split.trace), polls(&whole.trace), "{budgets:?}");
    assert_eq!(
        (split.steps, split.cycles),
        (whole.steps, whole.cycles),
        "{budgets:?}"
    );
    assert_eq!(split.reports.last().unwrap().exit, Exit::Halt);
}

#[test]
fn unlimited_step_and_cycle_budget_splits_have_identical_state_trace_and_polls() {
    let whole = session(&[UNLIMITED], false);
    assert_completed(&whole);
    assert_eq!(whole.reports.len(), 1);
    assert_same_execution(&whole, &[UNLIMITED], true);
    for steps in [1, 2, 3, 7, 11, 23, 34, 39, 40, 41] {
        assert_same_execution(&whole, &[Budget { steps, ..UNLIMITED }], false);
    }
    for cycles in [
        1, 4, 8, 11, 13, 16, 19, 20, 21, 23, 37, 53, 97, 409, 410, 411,
    ] {
        assert_same_execution(
            &whole,
            &[Budget {
                cycles,
                ..UNLIMITED
            }],
            false,
        );
    }
    let variable = [1, 23, 7, 19, 53].map(|cycles| Budget {
        cycles,
        ..UNLIMITED
    });
    assert_same_execution(&whole, &variable, false);
    let overshot = session(
        &[Budget {
            cycles: 1,
            ..UNLIMITED
        }],
        false,
    );
    assert_eq!(
        (overshot.reports[0].steps, overshot.reports[0].cycles),
        (1, 10)
    );
    assert_eq!(overshot.reports.last().unwrap().steps, 0);
}
