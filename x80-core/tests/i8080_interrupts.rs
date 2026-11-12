//! External interrupt witnesses for decision-5 items 3..8 and decision-2.
//! Intel 98-153B (September 1975), Ch. 2 Interrupt Sequences/Fig. 2-8,
//! p. 2-11 (PDF 25), Halt Sequences p. 2-13 (PDF 27), RST machine-cycle
//! table pp. 2-18/2-19 (PDF 32/33), Stack note 16 p. 2-20 (PDF 34),
//! Ch. 4 EI p. 4-14 (PDF 58), Ch. 5 INTE/INT p. 5-14 (PDF 76).
//! Second witness: Intel 9800301-04 (May 1981), Ch. 3 EI/HLT pp. 3-23/
//! 3-24 (PDF 77/78), RST pp. 3-54/3-55 (PDF 108/109), Ch. 7 pp. 7-2/
//! 7-3 (PDF 186/187). Revisions/hashes pinned in PROVENANCE.md §1;
//! all cited pages checked as images. Expected values are independent literals.

mod support;

use support::{sample_registers, Event, RecordingBus};
use x80_core::i8080::{Latches, I8080};
use x80_core::{Budget, Exit, MemoryAccess, Report, Unsupported};

const ENABLED: Latches = Latches {
    interrupt_enable: true,
    ei_delay: false,
    halted: false,
};
const DISABLED: Latches = Latches {
    interrupt_enable: false,
    ei_delay: false,
    halted: false,
};
const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

fn fetch(pc: u16, opcode: u8, interrupt: bool) -> [Event; 3] {
    [
        Event::StopPoll(false),
        Event::InterruptPoll(interrupt),
        Event::Read {
            address: pc,
            kind: MemoryAccess::OpcodeFetch,
            value: opcode,
        },
    ]
}

#[test]
fn all_eight_acknowledged_restarts_push_boundary_pc_and_take_one_step_eleven_states() {
    // Fig. 2-8: acknowledge at unchanged PC, M1=5, M2=3, M3=3 states;
    // high PC at SP-1, then low at SP-2. RST table pp. 2-18/2-19 and
    // note 16 p. 2-20: both writes are Stack. Ch. 7 p. 7-3 independently
    // demonstrates the unincremented return address; p. 3-55 gives 11 states.
    // decision-5 items 5/8: RST replaces even an undocumented opcode at PC,
    // and step stops before executing the vector's instruction.
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
        for (pc, high, low) in [(0x1234, 0x12, 0x34), (0xFFFF, 0xFF, 0xFF)] {
            let mut bus = RecordingBus::new();
            bus.interrupt = true;
            bus.acknowledges.push_back(opcode);
            bus.load(pc, &[0xDD]);
            bus.load(vector, &[0x3C]); // INR A must not execute in this step.
            let mut memory = bus.memory.clone();
            memory[0x8F8F] = high;
            memory[0x8F8E] = low;
            let mut cpu = I8080::new(sample_registers(pc), ENABLED);

            assert_eq!(
                cpu.step(&mut bus),
                Report {
                    steps: 1,
                    cycles: 11,
                    exit: Exit::Budget,
                }
            );
            let mut expected = sample_registers(vector);
            expected.sp = 0x8F8E;
            assert_eq!(*cpu.registers(), expected);
            assert_eq!(*cpu.latches(), DISABLED);
            assert_eq!(bus.memory, memory);
            assert_eq!(
                bus.take_log(),
                [
                    Event::StopPoll(false),
                    Event::InterruptPoll(true),
                    Event::Read {
                        address: pc,
                        kind: MemoryAccess::InterruptAcknowledge,
                        value: opcode,
                    },
                    Event::Write {
                        address: 0x8F8F,
                        kind: MemoryAccess::Stack,
                        value: high,
                    },
                    Event::Write {
                        address: 0x8F8E,
                        kind: MemoryAccess::Stack,
                        value: low,
                    },
                ]
            );
        }
    }
}

#[test]
fn a_disabled_request_is_polled_but_not_latched() {
    // decision-5 item 4's level contract; 98-153B INT p. 5-14 and
    // 9800301-04 Ch. 7 p. 7-2: disabled requests are not accepted.
    // A pulse while disabled must not survive EI and its following NOP.
    let mut bus = RecordingBus::new();
    bus.interrupt_levels.extend([true, false, false]);
    bus.load(0x1234, &[0xFB, 0x00, 0x00]);
    let memory = bus.memory.clone();
    let mut cpu = I8080::new(sample_registers(0x1234), DISABLED);
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: 3,
                cycles: u64::MAX,
            }
        ),
        Report {
            steps: 3,
            cycles: 12,
            exit: Exit::Budget,
        }
    );
    assert_eq!(*cpu.registers(), sample_registers(0x1237));
    assert_eq!(*cpu.latches(), ENABLED);
    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            fetch(0x1234, 0xFB, true),
            fetch(0x1235, 0x00, false),
            fetch(0x1236, 0x00, false),
        ]
        .concat()
    );
}

#[test]
fn ei_shadow_defers_a_high_level_for_exactly_one_instruction_across_steps() {
    // 98-153B Ch. 4 EI p. 4-14; 9800301-04 Ch. 3 EI p. 3-23:
    // enable following the next instruction, independently of call boundaries.
    let mut bus = RecordingBus::new();
    bus.interrupt = true;
    bus.acknowledges.push_back(0xCF);
    bus.load(0x1234, &[0xFB, 0x00, 0xDD]);
    let mut cpu = I8080::new(sample_registers(0x1234), DISABLED);
    for (pc, opcode, next_pc, ei_delay) in
        [(0x1234, 0xFB, 0x1235, true), (0x1235, 0x00, 0x1236, false)]
    {
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 4,
                exit: Exit::Budget,
            }
        );
        assert_eq!(*cpu.registers(), sample_registers(next_pc));
        assert_eq!(
            *cpu.latches(),
            Latches {
                ei_delay,
                ..ENABLED
            }
        );
        assert_eq!(bus.take_log(), fetch(pc, opcode, true));
    }
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 11,
            exit: Exit::Budget,
        }
    );
    let mut expected = sample_registers(0x0008);
    expected.sp = 0x8F8E;
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(*cpu.latches(), DISABLED);
    assert_eq!(bus.memory[0x8F8F], 0x12);
    assert_eq!(bus.memory[0x8F8E], 0x36);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::InterruptPoll(true),
            Event::Read {
                address: 0x1236,
                kind: MemoryAccess::InterruptAcknowledge,
                value: 0xCF,
            },
            Event::Write {
                address: 0x8F8F,
                kind: MemoryAccess::Stack,
                value: 0x12,
            },
            Event::Write {
                address: 0x8F8E,
                kind: MemoryAccess::Stack,
                value: 0x36,
            },
        ]
    );
}

#[test]
fn int_raised_during_run_is_accepted_at_the_next_boundary_and_inte_stays_clear() {
    // decision-5 items 4/5: a new level is observed each boundary; acceptance
    // replaces that boundary's fetch and run continues. 98-153B Fig. 2-8
    // p. 2-11; INTE p. 5-14; 9800301-04 Ch. 7 pp. 7-2/7-3.
    // Keeping INT high must not acknowledge again with INTE cleared.
    let mut bus = RecordingBus::new();
    bus.interrupt_levels.push_back(false);
    bus.interrupt = true;
    bus.acknowledges.push_back(0xDF);
    bus.load(0x1234, &[0x00, 0xDD]);
    bus.load(0x0018, &[0x76]);
    let mut cpu = I8080::new(sample_registers(0x1234), ENABLED);
    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 3,
            cycles: 22, // NOP 4 + acknowledged RST 11 + HLT 7.
            exit: Exit::Halt,
        }
    );
    let mut expected = sample_registers(0x0019);
    expected.sp = 0x8F8E;
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(
        *cpu.latches(),
        Latches {
            halted: true,
            ..DISABLED
        }
    );
    let mut trace = fetch(0x1234, 0x00, false).to_vec();
    trace.extend([
        Event::StopPoll(false),
        Event::InterruptPoll(true),
        Event::Read {
            address: 0x1235,
            kind: MemoryAccess::InterruptAcknowledge,
            value: 0xDF,
        },
        Event::Write {
            address: 0x8F8F,
            kind: MemoryAccess::Stack,
            value: 0x12,
        },
        Event::Write {
            address: 0x8F8E,
            kind: MemoryAccess::Stack,
            value: 0x35,
        },
    ]);
    trace.extend(fetch(0x0018, 0x76, true));
    trace.extend([Event::StopPoll(false), Event::InterruptPoll(true)]);
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn ei_hlt_with_int_high_wakes_and_continues_in_the_same_run() {
    // 98-153B Ch. 2 Start-up/Halt Sequences p. 2-13 explicitly gives EI;HLT;
    // Ch. 4 EI p. 4-14 delays one instruction. HLT completes that delay.
    // 9800301-04 Ch. 3 pp. 3-23/3-24 confirms EI;HLT and next-sequential PC;
    // Ch. 7 pp. 7-2/7-3 and Fig. 2-8 p. 2-11 preserve that PC on RST.
    let mut bus = RecordingBus::new();
    bus.interrupt = true;
    bus.lower_interrupt_on_acknowledge = true;
    bus.acknowledges.push_back(0xFF);
    bus.load(0x1234, &[0xFB, 0x76, 0xDD]);
    let mut cpu = I8080::new(sample_registers(0x1234), DISABLED);
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: 4,
                cycles: u64::MAX,
            }
        ),
        Report {
            steps: 4,
            cycles: 26, // EI 4 + HLT 7 + acknowledged RST 11 + NOP 4.
            exit: Exit::Budget,
        }
    );
    let mut expected = sample_registers(0x0039);
    expected.sp = 0x8F8E;
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(*cpu.latches(), DISABLED);
    assert!(!bus.interrupt);
    let mut trace = [fetch(0x1234, 0xFB, true), fetch(0x1235, 0x76, true)].concat();
    trace.extend([
        Event::StopPoll(false),
        Event::InterruptPoll(true),
        Event::Read {
            address: 0x1236,
            kind: MemoryAccess::InterruptAcknowledge,
            value: 0xFF,
        },
        Event::Write {
            address: 0x8F8F,
            kind: MemoryAccess::Stack,
            value: 0x12,
        },
        Event::Write {
            address: 0x8F8E,
            kind: MemoryAccess::Stack,
            value: 0x36,
        },
    ]);
    trace.extend(fetch(0x0038, 0x00, false));
    assert_eq!(bus.take_log(), trace);
    assert_eq!(bus.memory[0x8F8F], 0x12);
    assert_eq!(bus.memory[0x8F8E], 0x36);
}

#[test]
fn a_later_interrupt_wakes_halt_and_pushes_the_address_after_hlt() {
    // 98-153B Halt Sequences p. 2-13 and 9800301-04 HLT p. 3-24:
    // halt persists until an enabled interrupt; PC is already after HLT.
    // Fig. 2-8 p. 2-11 and Ch. 7 p. 7-3: push that PC unchanged.
    // The wrap at FFFF derives from the 16-bit PC (98-153B Ch. 2 p. 2-2).
    for (hlt_pc, return_pc, high, low) in
        [(0x1234, 0x1235, 0x12, 0x35), (0xFFFF, 0x0000, 0x00, 0x00)]
    {
        let mut bus = RecordingBus::new();
        bus.load(hlt_pc, &[0x76]);
        let mut cpu = I8080::new(sample_registers(hlt_pc), ENABLED);
        assert_eq!(
            cpu.run(&mut bus, UNLIMITED),
            Report {
                steps: 1,
                cycles: 7,
                exit: Exit::Halt
            }
        );
        assert_eq!(*cpu.registers(), sample_registers(return_pc));
        assert_eq!(
            *cpu.latches(),
            Latches {
                halted: true,
                ..ENABLED
            }
        );
        let mut trace = fetch(hlt_pc, 0x76, false).to_vec();
        trace.extend([Event::StopPoll(false), Event::InterruptPoll(false)]);
        assert_eq!(bus.take_log(), trace);

        bus.interrupt = true;
        bus.acknowledges.push_back(0xD7);
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 11,
                exit: Exit::Budget
            }
        );
        let mut expected = sample_registers(0x0010);
        expected.sp = 0x8F8E;
        assert_eq!(*cpu.registers(), expected);
        assert_eq!(*cpu.latches(), DISABLED);
        assert_eq!(bus.memory[0x8F8F], high);
        assert_eq!(bus.memory[0x8F8E], low);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::InterruptPoll(true),
                Event::Read {
                    address: return_pc,
                    kind: MemoryAccess::InterruptAcknowledge,
                    value: 0xD7
                },
                Event::Write {
                    address: 0x8F8F,
                    kind: MemoryAccess::Stack,
                    value: high
                },
                Event::Write {
                    address: 0x8F8E,
                    kind: MemoryAccess::Stack,
                    value: low
                },
            ]
        );
    }
}

#[test]
fn every_non_rst_acknowledge_is_unsupported_and_retryable_even_while_halted() {
    // decision-5 items 5/6 deliberately preserve even INTE/halt on an
    // unsupported acknowledge, departing from the INTE reset on p. 5-14.
    // Test all other bytes, including implemented instructions (EI/DI/HLT),
    // and retry the same response: only the acknowledge reaches the bus.
    for opcode in 0..=255 {
        if matches!(
            opcode,
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF
        ) {
            continue;
        }
        for halted in [false, true] {
            let mut bus = RecordingBus::new();
            bus.interrupt = true;
            bus.acknowledges.extend([opcode, opcode]);
            bus.load(0x1234, &[0x3C]);
            let memory = bus.memory.clone();
            let mut cpu = I8080::new(sample_registers(0x1234), Latches { halted, ..ENABLED });
            let boundary = cpu.clone();
            for _ in 0..2 {
                let report = cpu.step(&mut bus);
                assert_eq!((report.steps, report.cycles), (0, 0));
                assert!(matches!(report.exit, Exit::Unsupported(Unsupported {
                    pc: 0x1234, opcode: byte, acknowledge: true, ..
                }) if byte == opcode));
                assert_eq!(cpu, boundary);
                assert_eq!(bus.memory, memory);
                assert_eq!(
                    bus.take_log(),
                    [
                        Event::StopPoll(false),
                        Event::InterruptPoll(true),
                        Event::Read {
                            address: 0x1234,
                            kind: MemoryAccess::InterruptAcknowledge,
                            value: opcode
                        },
                    ]
                );
            }
        }
    }
}

#[test]
fn unsupported_acknowledge_reports_prior_progress_and_can_retry_a_different_byte() {
    // decision-5 item 6: preserve the state at this boundary, retaining the
    // accounting of earlier instructions; a new acknowledge can then succeed.
    let mut bus = RecordingBus::new();
    bus.interrupt_levels.push_back(false);
    bus.interrupt = true;
    bus.acknowledges.extend([0xFB, 0xC7]);
    let mut cpu = I8080::new(sample_registers(0x1234), ENABLED);
    let memory = bus.memory.clone();
    let report = cpu.run(&mut bus, UNLIMITED);
    assert_eq!((report.steps, report.cycles), (1, 4));
    assert!(matches!(
        report.exit,
        Exit::Unsupported(Unsupported {
            pc: 0x1235,
            opcode: 0xFB,
            acknowledge: true,
            ..
        })
    ));
    assert_eq!(*cpu.registers(), sample_registers(0x1235));
    assert_eq!(*cpu.latches(), ENABLED);
    assert_eq!(bus.memory, memory);
    let mut trace = fetch(0x1234, 0x00, false).to_vec();
    trace.extend([
        Event::StopPoll(false),
        Event::InterruptPoll(true),
        Event::Read {
            address: 0x1235,
            kind: MemoryAccess::InterruptAcknowledge,
            value: 0xFB,
        },
    ]);
    assert_eq!(bus.take_log(), trace);
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 11,
            exit: Exit::Budget
        }
    );
    let mut expected = sample_registers(0x0000);
    expected.sp = 0x8F8E;
    assert_eq!(*cpu.registers(), expected);
    assert_eq!(*cpu.latches(), DISABLED);
    assert_eq!(bus.memory[0x8F8F], 0x12);
    assert_eq!(bus.memory[0x8F8E], 0x35);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::InterruptPoll(true),
            Event::Read {
                address: 0x1235,
                kind: MemoryAccess::InterruptAcknowledge,
                value: 0xC7
            },
            Event::Write {
                address: 0x8F8F,
                kind: MemoryAccess::Stack,
                value: 0x12
            },
            Event::Write {
                address: 0x8F8E,
                kind: MemoryAccess::Stack,
                value: 0x35
            },
        ]
    );
}

#[test]
fn budget_and_stop_precede_an_eligible_interrupt_even_while_halted() {
    // All combined conditions of decision-5 item 5: both zero-budget kinds
    // precede polling; Stop precedes acknowledge, Halt and unsupported fetch.
    for halted in [false, true] {
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
            let mut bus = RecordingBus::new();
            bus.stop = true;
            bus.interrupt = true;
            bus.load(0x1234, &[0xDD]);
            let mut cpu = I8080::new(sample_registers(0x1234), Latches { halted, ..ENABLED });
            let boundary = cpu.clone();
            let memory = bus.memory.clone();
            assert_eq!(
                cpu.run(&mut bus, budget),
                Report {
                    steps: 0,
                    cycles: 0,
                    exit: Exit::Budget
                }
            );
            assert_eq!(cpu, boundary);
            assert_eq!(bus.memory, memory);
            assert!(bus.log.is_empty());
            assert_eq!(
                cpu.run(&mut bus, UNLIMITED),
                Report {
                    steps: 0,
                    cycles: 0,
                    exit: Exit::Stop
                }
            );
            assert_eq!(cpu, boundary);
            assert_eq!(bus.memory, memory);
            assert_eq!(
                bus.take_log(),
                [Event::StopPoll(true), Event::InterruptPoll(true)]
            );
        }
    }
}

#[test]
fn acknowledged_rst_completes_before_the_cycle_budget_exit() {
    // decision-2 overshoot, decision-5 items 5/8; Fig. 2-8's entire 11-state
    // instruction is charged, and there is no poll after reaching the limit.
    for cycles in [1, 10, 11] {
        let mut bus = RecordingBus::new();
        bus.interrupt = true;
        bus.acknowledges.push_back(0xE7);
        let mut cpu = I8080::new(sample_registers(0x1234), ENABLED);
        assert_eq!(
            cpu.run(
                &mut bus,
                Budget {
                    steps: u64::MAX,
                    cycles
                }
            ),
            Report {
                steps: 1,
                cycles: 11,
                exit: Exit::Budget
            }
        );
        let mut expected = sample_registers(0x0020);
        expected.sp = 0x8F8E;
        assert_eq!(*cpu.registers(), expected);
        assert_eq!(*cpu.latches(), DISABLED);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::InterruptPoll(true),
                Event::Read {
                    address: 0x1234,
                    kind: MemoryAccess::InterruptAcknowledge,
                    value: 0xE7
                },
                Event::Write {
                    address: 0x8F8F,
                    kind: MemoryAccess::Stack,
                    value: 0x12
                },
                Event::Write {
                    address: 0x8F8E,
                    kind: MemoryAccess::Stack,
                    value: 0x34
                },
            ]
        );
    }
}
