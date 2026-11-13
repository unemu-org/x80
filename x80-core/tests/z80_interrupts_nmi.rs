//! NMI acceptance witnesses (decision-15 rules 2, 3, 6–8, 10).
//!
//! Primary: UM008011-0816 rev. 11 (August 2016), printed pp. 3, 6,
//! 13–14, 19, 94–95, 180, 183, 290, 293; PDF = printed + 14.
//! Second witnesses: 03-0029-01 (1977), §2 p. 4, §3 p. 8, §4 p. 18,
//! §8 pp. 55–56, Table 7.0-10 p. 53 (PDF = printed + 6);
//! 03-0002-01 (January 1978), pp. 48–49, 248 (PDF 52–53, 257).
//! Values read on rendered 150 dpi pages; expectations are literal and do
//! not use CPU execution helpers. Polls, rollback and budget boundaries are
//! API choices. NMI's 11 states are DERIVED from UM CPU Response p. 19's
//! restart equivalence and RST p. 293's 11 (5,3,3), also TM pp. 56/53.
//! EI-delay preservation and the parity-note window are UNVERIFIED model
//! readings, explicitly adopted in decision-15 rules 7/10.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, MemoryAccess, Report, Unsupported};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const STACK: MemoryAccess = MemoryAccess::Stack;
const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

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
        sp: 0x8100,
        pc: 0x2345,
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

fn report(steps: u64, cycles: u64, exit: Exit) -> Report {
    Report {
        steps,
        cycles,
        exit,
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
        kind: STACK,
        value,
    }
}

fn polls(stop: bool, nmi: bool, int: bool) -> [Event; 3] {
    [
        Event::StopPoll(stop),
        Event::NmiPoll(nmi),
        Event::IntPoll(int),
    ]
}

#[test]
fn discarded_fetch_neither_decodes_nor_advances_the_saved_pc() {
    // UM pp. 13–14/19: ignore fetched data. UM RETN p. 290 and ALPM
    // p. 248: unchanged PC, PCH first, 0066h. TM pp. 18/53/56 agrees.
    // Prefixes, EI and RST bytes must all be discarded without execution.
    for byte in [0xDD, 0xFD, 0xCB, 0xED, 0xFB, 0xFF] {
        let mut cpu = Z80::new(registers(), latches());
        let mut bus = RecordingBus::new();
        bus.nmi = true;
        bus.load(0x2345, &[byte, 0xA5, 0x5A]);
        bus.load(0x0066, &[0xED, 0]); // No handler fetch in this step.
        let mut memory = bus.memory.clone();
        memory[0x80FF] = 0x23;
        memory[0x80FE] = 0x45;

        assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 0x0066,
                sp: 0x80FE,
                r: 0x80,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                iff1: false,
                nmi_asserted: true,
                ..latches()
            }
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(true),
                Event::IntPoll(false),
                read(0x2345, OP, byte),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ]
        );
    }
}

#[test]
fn accepts_with_iff1_clear_and_preserves_iff2_mode_and_pending_ei_delay() {
    // UM p. 6/p. 19; TM pp. 8/56: independent of IFF1, keeps IFF2.
    // UM EI p. 183/TM p. 55: delay inhibits INT only; preserving the
    // pending latch on NMI is rule 7's UNVERIFIED interaction model.
    for iff1 in [false, true] {
        for iff2 in [false, true] {
            for delay in [false, true] {
                for mode in [
                    InterruptMode::Mode0,
                    InterruptMode::Mode1,
                    InterruptMode::Mode2,
                ] {
                    let initial = Latches {
                        iff1,
                        iff2,
                        interrupt_mode: mode,
                        ..latches()
                    };
                    let mut cpu = Z80::new(registers(), initial);
                    cpu.latches_mut().ei_delay = delay;
                    let mut bus = RecordingBus::new();
                    bus.nmi = true;
                    bus.int = true;
                    assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
                    assert_eq!(
                        *cpu.registers(),
                        Registers {
                            pc: 0x0066,
                            sp: 0x80FE,
                            r: 0x80,
                            ..registers()
                        }
                    );
                    assert_eq!(
                        *cpu.latches(),
                        Latches {
                            iff1: false,
                            ei_delay: delay,
                            nmi_asserted: true,
                            ..initial
                        }
                    );
                    assert_eq!(
                        bus.take_log(),
                        [
                            Event::StopPoll(false),
                            Event::NmiPoll(true),
                            Event::IntPoll(true),
                            read(0x2345, OP, 0),
                            write(0x80FF, 0x23),
                            write(0x80FE, 0x45),
                        ]
                    );
                }
            }
        }
    }
}

#[test]
fn nmi_has_priority_over_an_eligible_asserted_int() {
    // UM pp. 6/13–14; TM pp. 8/18. Both requests are eligible here.
    let mut cpu = Z80::new(registers(), latches());
    let mut bus = RecordingBus::new();
    bus.nmi = true;
    bus.int = true;
    bus.load(0x2345, &[0xED, 0]);
    bus.acknowledges.push_back(0xCF);
    assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
    assert_eq!(
        (cpu.registers().pc, cpu.registers().sp, cpu.registers().r),
        (0x0066, 0x80FE, 0x80)
    );
    assert!(!cpu.latches().iff1);
    assert!(cpu.latches().iff2);
    assert_eq!(bus.acknowledges.into_iter().collect::<Vec<_>>(), [0xCF]);
    assert_eq!(
        bus.log,
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(true),
            read(0x2345, OP, 0xED),
            write(0x80FF, 0x23),
            write(0x80FE, 0x45),
        ]
    );
}

#[test]
fn halted_cpu_with_iff1_clear_wakes_and_saves_its_resume_pc() {
    // UM HALT Exit p. 14/TM §4 p. 18. Set halted via the public latch
    // seam; HALT entry itself belongs to the control instruction bead.
    let initial = Latches {
        iff1: false,
        ..latches()
    };
    let mut cpu = Z80::new(registers(), initial);
    cpu.latches_mut().halted = true;
    cpu.latches_mut().ei_delay = true;
    let mut bus = RecordingBus::new();
    bus.nmi = true;
    assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0066,
            sp: 0x80FE,
            r: 0x80,
            ..registers()
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            halted: false,
            ei_delay: true,
            nmi_asserted: true,
            ..initial
        }
    );
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(false),
            read(0x2345, OP, 0),
            write(0x80FF, 0x23),
            write(0x80FE, 0x45),
        ]
    );
}

#[test]
fn held_assertion_accepts_once_and_a_not_asserted_sample_rearms() {
    // Negative edge: UM p. 6/TM p. 8; boundary level sampler is rule 2's
    // API choice. NOP's 4 states: UM p. 180 (TM Table 7.0-5 p. 48).
    let mut cpu = Z80::new(registers(), latches());
    let mut bus = RecordingBus::new();
    bus.nmi = true;
    assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(false),
            read(0x2345, OP, 0),
            write(0x80FF, 0x23),
            write(0x80FE, 0x45),
        ]
    );
    for (level, at, next, r) in [
        (true, 0x0066, 0x0067, 0x81),
        (true, 0x0067, 0x0068, 0x82),
        (false, 0x0068, 0x0069, 0x83),
    ] {
        bus.nmi = level;
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: next,
                sp: 0x80FE,
                r,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                iff1: false,
                nmi_asserted: level,
                ..latches()
            }
        );
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(level),
                Event::IntPoll(false),
                read(at, OP, 0),
            ]
        );
    }
    bus.nmi = true;
    assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0066,
            sp: 0x80FC,
            r: 0x84,
            ..registers()
        }
    );
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(false),
            read(0x0069, OP, 0),
            write(0x80FD, 0),
            write(0x80FC, 0x69),
        ]
    );
}

#[test]
fn stop_at_first_edge_requires_the_embedder_to_keep_presenting_it() {
    // Decisions 15 rule 2 and 12 rule 2: polls cannot be undone, and Stop
    // keeps no pending pulse. A still-asserted level is seen again on retry.
    for held in [false, true] {
        let mut cpu = Z80::new(registers(), latches());
        let before = cpu.clone();
        let mut bus = RecordingBus::new();
        let memory = bus.memory.clone();
        bus.stop = true;
        bus.nmi_levels.push_back(true);
        assert_eq!(cpu.step(&mut bus), report(0, 0, Exit::Stop));
        assert_eq!(cpu, before);
        assert_eq!(bus.memory, memory);
        assert!(bus.nmi_levels.is_empty());
        assert_eq!(bus.take_log(), polls(true, true, false));

        bus.stop = false;
        bus.nmi = held;
        if held {
            assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
            assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x0066, 0x80FE));
            assert_eq!(
                bus.take_log(),
                [
                    Event::StopPoll(false),
                    Event::NmiPoll(true),
                    Event::IntPoll(false),
                    read(0x2345, OP, 0),
                    write(0x80FF, 0x23),
                    write(0x80FE, 0x45),
                ]
            );
        } else {
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(
                *cpu.registers(),
                Registers {
                    pc: 0x2346,
                    r: 0x80,
                    ..registers()
                }
            );
            assert_eq!(
                bus.take_log(),
                [
                    Event::StopPoll(false),
                    Event::NmiPoll(false),
                    Event::IntPoll(false),
                    read(0x2345, OP, 0),
                ]
            );
            bus.nmi = true;
            assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
            assert_eq!(
                bus.take_log(),
                [
                    Event::StopPoll(false),
                    Event::NmiPoll(true),
                    Event::IntPoll(false),
                    read(0x2346, OP, 0),
                    write(0x80FF, 0x23),
                    write(0x80FE, 0x46),
                ]
            );
        }
    }
}

#[test]
fn stop_does_not_commit_a_not_asserted_sample() {
    // Rule 2 also restores a would-be re-arm. No new edge until a low
    // sample commits at an ordinary boundary (UM p. 6/TM p. 8 edge).
    let initial = Latches {
        nmi_asserted: true,
        ..latches()
    };
    let mut cpu = Z80::new(registers(), initial);
    let before = cpu.clone();
    let mut bus = RecordingBus::new();
    bus.stop = true;
    assert_eq!(cpu.step(&mut bus), report(0, 0, Exit::Stop));
    assert_eq!(cpu, before);
    assert_eq!(bus.take_log(), polls(true, false, false));
    bus.stop = false;
    bus.nmi = true;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(false),
            read(0x2345, OP, 0),
        ]
    );
    bus.nmi = false;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            read(0x2346, OP, 0),
        ]
    );
    bus.nmi = true;
    assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(false),
            read(0x2347, OP, 0),
            write(0x80FF, 0x23),
            write(0x80FE, 0x47),
        ]
    );
}

#[test]
fn unsupported_restores_sampler_and_reports_the_earlier_nmi_progress() {
    // Rule 2 / decision-12 rule 2 restoration is an API guarantee.
    // NMI itself cannot refuse; its discarded ED is not decoded. A later
    // unsupported CB 30 boundary must undo its tentative low sample.
    // Decision-20 §2: DD00 executes; CB30 still refuses until §3.
    let initial = Latches {
        ei_delay: true,
        ..latches()
    };
    let mut cpu = Z80::new(registers(), initial);
    let mut bus = RecordingBus::new();
    bus.load(0x2345, &[0xED, 0]);
    bus.load(0x0066, &[0xCB, 0x30]);
    bus.nmi_levels.extend([true, false]);
    let assert_refusal = |actual: Report, steps, cycles| {
        assert_eq!((actual.steps, actual.cycles), (steps, cycles));
        assert!(matches!(
            actual.exit,
            Exit::Unsupported(Unsupported {
                pc: 0x0066,
                opcode: 0x30,
                acknowledge: false,
                preceding: [0xCB, 0, 0],
                preceding_len: 1,
                ..
            })
        ));
    };
    assert_refusal(cpu.run(&mut bus, UNLIMITED), 1, 11);
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0066,
            sp: 0x80FE,
            r: 0x80,
            ..registers()
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            iff1: false,
            nmi_asserted: true,
            ..initial
        }
    );
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(false),
            read(0x2345, OP, 0xED),
            write(0x80FF, 0x23),
            write(0x80FE, 0x45),
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            read(0x0066, OP, 0xCB),
            read(0x0067, OP, 0x30),
        ]
    );
    let before = cpu.clone();
    let memory = bus.memory.clone();
    bus.nmi = true; // Restored asserted history prevents a spurious edge.
    assert_refusal(cpu.step(&mut bus), 0, 0);
    assert_eq!(cpu, before);
    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(false),
            read(0x0066, OP, 0xCB),
            read(0x0067, OP, 0x30),
        ]
    );
    bus.nmi = false;
    bus.load(0x0066, &[0]);
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert!(!cpu.latches().nmi_asserted);
    assert!(!cpu.latches().ei_delay);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            read(0x0066, OP, 0),
        ]
    );
    bus.nmi = true;
    assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0066,
            sp: 0x80FC,
            r: 0x82,
            ..registers()
        }
    );
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(false),
            read(0x0067, OP, 0x30),
            write(0x80FD, 0),
            write(0x80FC, 0x67),
        ]
    );
}

#[test]
fn discarded_fetch_increments_only_the_low_seven_refresh_bits_once() {
    // UM Registers p. 3/TM §2 p. 4. Per-NMI M1 increment is DERIVED
    // from UM Fig. 10 p. 14/TM Fig. 4.0-6 p. 18 (rule 6).
    for (r, next) in [
        (0, 1),
        (0x7E, 0x7F),
        (0x7F, 0),
        (0x80, 0x81),
        (0xFE, 0xFF),
        (0xFF, 0x80),
    ] {
        let mut cpu = Z80::new(Registers { r, ..registers() }, latches());
        let mut bus = RecordingBus::new();
        bus.nmi = true;
        bus.load(0x2345, &[0xDD, 0xCB, 0, 0x06]);
        assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 0x0066,
                sp: 0x80FE,
                r: next,
                ..registers()
            }
        );
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(true),
                Event::IntPoll(false),
                read(0x2345, OP, 0xDD),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ]
        );
    }
}

#[test]
fn derived_stack_wrap_preserves_boundary_pc_ffff_without_increment() {
    // SP wrap is DERIVED from its 16-bit width (UM p. 2/TM §2.1 p. 3),
    // not an explicit hardware wrap statement. Push: UM p. 290/ALPM p. 248.
    for (sp, high_at, low_at) in [(0, 0xFFFF, 0xFFFE), (1, 0, 0xFFFF)] {
        let mut cpu = Z80::new(
            Registers {
                pc: 0xFFFF,
                sp,
                ..registers()
            },
            latches(),
        );
        let mut bus = RecordingBus::new();
        bus.nmi = true;
        bus.load(0xFFFF, &[0xED, 0xA5]);
        let mut memory = bus.memory.clone();
        memory[usize::from(high_at)] = 0xFF;
        memory[usize::from(low_at)] = 0xFF;
        assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 0x0066,
                sp: low_at,
                r: 0x80,
                ..registers()
            }
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(true),
                Event::IntPoll(false),
                read(0xFFFF, OP, 0xED),
                write(high_at, 0xFF),
                write(low_at, 0xFF),
            ]
        );
    }
}

#[test]
fn cycle_budget_retires_one_whole_eleven_state_acceptance() {
    // Decision-15 rule 3 / decision-5 overshoot. 11 is DERIVED as above.
    for cycles in [1, 10, 11] {
        let mut cpu = Z80::new(registers(), latches());
        let mut bus = RecordingBus::new();
        bus.nmi = true;
        assert_eq!(
            cpu.run(&mut bus, Budget { steps: 10, cycles }),
            report(1, 11, Exit::Budget)
        );
        assert_eq!(
            (cpu.registers().pc, cpu.registers().sp, cpu.registers().r),
            (0x0066, 0x80FE, 0x80)
        );
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(true),
                Event::IntPoll(false),
                read(0x2345, OP, 0),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ]
        );
    }
}

#[test]
fn budget_splits_before_and_after_nmi_preserve_state_and_literal_trace() {
    // Rules 2–3: Budget neither samples nor consumes an edge; acceptance
    // is one step and the next handler instruction is another (UM p. 19).
    let initial = Latches {
        ei_delay: true,
        ..latches()
    };
    let mut whole = Z80::new(registers(), initial);
    let mut whole_bus = RecordingBus::new();
    whole_bus.nmi_levels.extend([false, true, true]);
    assert_eq!(
        whole.run(
            &mut whole_bus,
            Budget {
                steps: 3,
                cycles: 100
            }
        ),
        report(3, 19, Exit::Budget)
    );
    let literal = [
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
        read(0x2345, OP, 0),
        Event::StopPoll(false),
        Event::NmiPoll(true),
        Event::IntPoll(false),
        read(0x2346, OP, 0),
        write(0x80FF, 0x23),
        write(0x80FE, 0x46),
        Event::StopPoll(false),
        Event::NmiPoll(true),
        Event::IntPoll(false),
        read(0x0066, OP, 0),
    ];
    assert_eq!(whole_bus.log, literal);
    let mut split = Z80::new(registers(), initial);
    let mut split_bus = RecordingBus::new();
    split_bus.nmi_levels.extend([false, true, true]);
    let mut total = (0, 0);
    for cycles in [4, 11, 4] {
        // Zero step and zero cycle limits both leave scripts and CPU alone.
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
            let before = split.clone();
            let log = split_bus.log.clone();
            let levels = split_bus.nmi_levels.clone();
            assert_eq!(
                split.run(&mut split_bus, budget),
                report(0, 0, Exit::Budget)
            );
            assert_eq!(split, before);
            assert_eq!(split_bus.log, log);
            assert_eq!(split_bus.nmi_levels, levels);
        }
        let part = split.step(&mut split_bus);
        assert_eq!(part, report(1, cycles, Exit::Budget));
        total.0 += part.steps;
        total.1 += part.cycles;
    }
    assert_eq!(total, (3, 19));
    assert_eq!(
        *split.registers(),
        Registers {
            pc: 0x0067,
            sp: 0x80FE,
            r: 0x82,
            ..registers()
        }
    );
    assert_eq!(
        *split.latches(),
        Latches {
            iff1: false,
            nmi_asserted: true,
            ..latches()
        }
    );
    assert_eq!(split, whole);
    assert_eq!(split_bus.memory, whole_bus.memory);
    assert_eq!(split_bus.log, literal);
}

#[test]
fn retn_round_trip_restores_iff1_from_the_preserved_iff2() {
    // UM RETN p. 290/Table 1 p. 19; ALPM p. 248; TM Table 7.0-10
    // p. 53: 14 states, low/high Stack pop, IFF2 copied to IFF1.
    for iff2 in [false, true] {
        let initial = Latches {
            iff1: !iff2,
            iff2,
            ..latches()
        };
        let mut cpu = Z80::new(registers(), initial);
        let mut bus = RecordingBus::new();
        bus.nmi = true;
        bus.load(0x0066, &[0xED, 0x45]);
        assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
        assert!(!cpu.latches().iff1);
        assert_eq!(cpu.latches().iff2, iff2);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(true),
                Event::IntPoll(false),
                read(0x2345, OP, 0),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ]
        );
        assert_eq!(cpu.step(&mut bus), report(1, 14, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                r: 0x82,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                iff1: iff2,
                nmi_asserted: true,
                ..initial
            }
        );
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(true),
                Event::IntPoll(false),
                read(0x0066, OP, 0xED),
                read(0x0067, OP, 0x45),
                read(0x80FE, STACK, 0x45),
                read(0x80FF, STACK, 0x23),
            ]
        );
    }
}

#[test]
fn ld_a_i_or_r_then_nmi_clears_only_pending_parity_across_budget_and_stop() {
    // UM LD A,I/R pp. 94–95; ALPM pp. 48–49 (R note omitted, no contrary
    // result). Rule 10's next-boundary interpretation is UNVERIFIED.
    for (opcode, a) in [(0x57, 0xF8), (0x5F, 0x81)] {
        let mut cpu = Z80::new(registers(), latches());
        let mut bus = RecordingBus::new();
        bus.load(0x2345, &[0xED, opcode, 0]);
        assert_eq!(cpu.step(&mut bus), report(1, 9, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                a,
                flags: Flags::from_byte(0x85),
                pc: 0x2347,
                r: 0x81,
                ..registers()
            }
        );
        assert!(cpu.latches().parity_note_pending);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0x2345, OP, 0xED),
                read(0x2346, OP, opcode),
            ]
        );
        let before = cpu.clone();
        bus.nmi = true;
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
        assert_eq!(cpu.step(&mut bus), report(0, 0, Exit::Stop));
        assert_eq!(cpu, before);
        assert_eq!(bus.take_log(), polls(true, true, false));
        bus.stop = false;
        assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                a,
                flags: Flags::from_byte(0x81),
                pc: 0x0066,
                sp: 0x80FE,
                r: 0x82,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                iff1: false,
                nmi_asserted: true,
                parity_note_pending: false,
                ..latches()
            }
        );
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(true),
                Event::IntPoll(false),
                read(0x2347, OP, 0),
                write(0x80FF, 0x23),
                write(0x80FE, 0x47),
            ]
        );
    }
}

#[test]
fn parity_correction_keeps_other_flags_and_requires_a_pending_note() {
    // Rule 10; UM pp. 94–95/ALPM pp. 48–49. No-note acceptance must
    // preserve P/V even though IFF1 clears. Both flag polarities are pinned.
    for (pending, flags, after) in [
        (false, 0xD7, 0xD7),
        (true, 0xD7, 0xD3),
        (false, 0, 0),
        (true, 0, 0),
    ] {
        let initial = Registers {
            flags: Flags::from_byte(flags),
            ..registers()
        };
        let mut cpu = Z80::new(
            initial,
            Latches {
                parity_note_pending: pending,
                ..latches()
            },
        );
        let mut bus = RecordingBus::new();
        bus.nmi = true;
        assert_eq!(cpu.step(&mut bus), report(1, 11, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                flags: Flags::from_byte(after),
                pc: 0x0066,
                sp: 0x80FE,
                r: 0x80,
                ..initial
            }
        );
        assert!(!cpu.latches().parity_note_pending);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(true),
                Event::IntPoll(false),
                read(0x2345, OP, 0),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ]
        );
    }
}
