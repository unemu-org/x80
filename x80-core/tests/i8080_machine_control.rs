//! Machine-control witnesses: decision-5 items 4, 5, 8, 9; decision-2.
//! Intel 98-153B (September 1975) and 9800301-04 (May 1981), pinned in
//! PROVENANCE.md §1. Cited pages checked as images. Expectations are literal.

mod support;

use support::{is_fetch_unsupported, sample_registers, Event, RecordingBus};
use x80_core::i8080::{Flags, Latches, Registers, I8080};
use x80_core::{Budget, Exit, MemoryAccess, Report};

const DISABLED: Latches = Latches {
    interrupt_enable: false,
    ei_delay: false,
    halted: false,
};
const PENDING: Latches = Latches {
    interrupt_enable: true,
    ei_delay: true,
    halted: false,
};
const ENABLED: Latches = Latches {
    interrupt_enable: true,
    ei_delay: false,
    halted: false,
};
const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

fn fetch(pc: u16, opcode: u8) -> [Event; 3] {
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

#[test]
fn ei_enables_only_after_the_following_instruction_across_steps() {
    // 98-153B Ch. 4 EI/NOP p. 4-14: enable following the next instruction,
    // 4 states each, no flags. 9800301-04 Ch. 3 EI p. 3-23 agrees.
    // decision-5 item 4: eligibility requires enable AND no pending delay.
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0xFB, 0x00, 0x00]);
    let memory = bus.memory.clone();
    let mut cpu = I8080::new(sample_registers(0x1234), DISABLED);
    for (pc, opcode, next_pc, latches) in [
        (0x1234, 0xFB, 0x1235, PENDING),
        (0x1235, 0x00, 0x1236, ENABLED),
        (0x1236, 0x00, 0x1237, ENABLED),
    ] {
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 4,
                exit: Exit::Budget
            }
        );
        assert_eq!(*cpu.registers(), sample_registers(next_pc));
        assert_eq!(*cpu.latches(), latches);
        assert_eq!(bus.take_log(), fetch(pc, opcode));
    }
    assert_eq!(bus.memory, memory);
}

#[test]
fn di_disables_immediately_including_after_ei() {
    // 98-153B Ch. 4 DI p. 4-14: disabled immediately following DI;
    // 9800301-04 Ch. 3 DI pp. 3-22/3-23 agrees, 4 states, no flags.
    // EI;DI's final disabled state follows DI's rule. Cancellation of the
    // pending delay itself is an unverified internal model choice.
    for initial in [DISABLED, ENABLED, PENDING] {
        let mut bus = RecordingBus::new();
        bus.load(0x1234, &[0xFB, 0xF3, 0x00]);
        let memory = bus.memory.clone();
        let mut cpu = I8080::new(sample_registers(0x1235), initial);
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 4,
                exit: Exit::Budget
            }
        );
        assert_eq!(*cpu.registers(), sample_registers(0x1236));
        assert_eq!(*cpu.latches(), DISABLED);
        assert_eq!(bus.take_log(), fetch(0x1235, 0xF3));
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 4,
                exit: Exit::Budget
            }
        );
        assert_eq!(*cpu.registers(), sample_registers(0x1237));
        assert_eq!(*cpu.latches(), DISABLED);
        assert_eq!(bus.take_log(), fetch(0x1236, 0x00));
        assert_eq!(bus.memory, memory);

        let mut cpu = I8080::new(sample_registers(0x1234), initial);
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 4,
                exit: Exit::Budget
            }
        );
        assert_eq!(*cpu.latches(), PENDING);
        assert_eq!(bus.take_log(), fetch(0x1234, 0xFB));
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 4,
                exit: Exit::Budget
            }
        );
        assert_eq!(*cpu.registers(), sample_registers(0x1236));
        assert_eq!(*cpu.latches(), DISABLED);
        assert_eq!(bus.take_log(), fetch(0x1235, 0xF3));
    }
}

#[test]
fn consecutive_ei_rearms_delay_provisionally() {
    // Unverified edge: 98-153B Ch. 4 EI p. 4-14 and 9800301-04 Ch. 3
    // EI p. 3-23 give the one-instruction rule but do not discuss EI;EI.
    // Pin the simplest model (each EI arms a delay), not a hardware claim.
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0xFB, 0xFB, 0x00]);
    let mut cpu = I8080::new(sample_registers(0x1234), DISABLED);
    for (pc, opcode, next_pc, latches) in [
        (0x1234, 0xFB, 0x1235, PENDING),
        (0x1235, 0xFB, 0x1236, PENDING),
        (0x1236, 0x00, 0x1237, ENABLED),
    ] {
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 4,
                exit: Exit::Budget
            }
        );
        assert_eq!(*cpu.registers(), sample_registers(next_pc));
        assert_eq!(*cpu.latches(), latches);
        assert_eq!(bus.take_log(), fetch(pc, opcode));
    }
}

#[test]
fn pending_delay_survives_exits_without_a_completed_instruction() {
    // decision-5 items 5/6: Budget, Stop, Halt and Unsupported do not retire
    // instructions or change state. EI's delay counts instructions, not calls
    // (98-153B Ch. 4 p. 4-14; 9800301-04 Ch. 3 p. 3-23).
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0xFB, 0xDD]);
    let mut cpu = I8080::new(sample_registers(0x1234), DISABLED);
    assert_eq!(cpu.step(&mut bus).steps, 1);
    assert_eq!(bus.take_log(), fetch(0x1234, 0xFB));
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: 0,
                cycles: 100
            }
        ),
        Report {
            steps: 0,
            cycles: 0,
            exit: Exit::Budget
        }
    );
    assert_eq!(*cpu.latches(), PENDING);
    assert!(bus.take_log().is_empty());
    bus.stop = true;
    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 0,
            cycles: 0,
            exit: Exit::Stop
        }
    );
    assert_eq!(*cpu.latches(), PENDING);
    assert_eq!(
        bus.take_log(),
        [Event::StopPoll(true), Event::InterruptPoll(false)]
    );
    bus.stop = false;
    for _ in 0..2 {
        let report = cpu.step(&mut bus);
        assert_eq!((report.steps, report.cycles), (0, 0));
        assert!(is_fetch_unsupported(report.exit, 0x1235, 0xDD));
        assert_eq!(*cpu.latches(), PENDING);
        assert_eq!(*cpu.registers(), sample_registers(0x1235));
        assert_eq!(bus.take_log(), fetch(0x1235, 0xDD));
    }
    cpu.latches_mut().halted = true;
    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 0,
            cycles: 0,
            exit: Exit::Halt
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            halted: true,
            ..PENDING
        }
    );
    assert_eq!(
        bus.take_log(),
        [Event::StopPoll(false), Event::InterruptPoll(false)]
    );
    cpu.latches_mut().halted = false;
    bus.memory[0x1235] = 0x00;
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert_eq!(*cpu.latches(), ENABLED);
    assert_eq!(*cpu.registers(), sample_registers(0x1236));
    assert_eq!(bus.take_log(), fetch(0x1235, 0x00));
}

#[test]
fn hlt_takes_seven_states_and_later_runs_make_zero_progress() {
    // 98-153B Ch. 4 HLT p. 4-14: 7 states, registers/flags unaffected;
    // Ch. 2 Halt Sequences p. 2-13, Figs. 2-11/2-12 p. 2-14: halt persists.
    // 9800301-04 Ch. 3 HLT p. 3-24: PC holds next sequential address.
    // decision-2 and decision-5 items 5/8: budget first, halt without fetch.
    for (pc, next_pc) in [(0x1234, 0x1235), (0xFFFF, 0x0000)] {
        for initial in [DISABLED, ENABLED, PENDING] {
            let mut bus = RecordingBus::new();
            bus.load(pc, &[0x76, 0xDD]);
            let memory = bus.memory.clone();
            let mut cpu = I8080::new(sample_registers(pc), initial);
            assert_eq!(
                cpu.step(&mut bus),
                Report {
                    steps: 1,
                    cycles: 7,
                    exit: Exit::Budget
                }
            );
            let expected = Latches {
                halted: true,
                ei_delay: false,
                interrupt_enable: initial.interrupt_enable,
            };
            assert_eq!(*cpu.latches(), expected);
            assert_eq!(*cpu.registers(), sample_registers(next_pc));
            assert_eq!(bus.take_log(), fetch(pc, 0x76));
            for _ in 0..2 {
                assert_eq!(
                    cpu.run(&mut bus, UNLIMITED),
                    Report {
                        steps: 0,
                        cycles: 0,
                        exit: Exit::Halt
                    }
                );
                assert_eq!(*cpu.latches(), expected);
                assert_eq!(*cpu.registers(), sample_registers(next_pc));
                assert_eq!(
                    bus.take_log(),
                    [Event::StopPoll(false), Event::InterruptPoll(false)]
                );
            }
            assert_eq!(bus.memory, memory);
        }
    }
}

#[test]
fn ei_then_hlt_expires_delay_and_returns_halt_in_one_run() {
    // 98-153B Ch. 2 Start-up p. 2-13 and 9800301-04 Ch. 3 pp. 3-23/3-24
    // explicitly give EI;HLT as the interrupt-enabled start-up sequence.
    // Ch. 4 p. 4-14: 4 + 7 states; decision-5 item 5: next boundary is Halt.
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0xFB, 0x76, 0xDD]);
    let mut cpu = I8080::new(sample_registers(0x1234), DISABLED);
    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 2,
            cycles: 11,
            exit: Exit::Halt
        }
    );
    assert_eq!(*cpu.registers(), sample_registers(0x1236));
    assert_eq!(
        *cpu.latches(),
        Latches {
            interrupt_enable: true,
            ei_delay: false,
            halted: true
        }
    );
    let mut trace = [fetch(0x1234, 0xFB), fetch(0x1235, 0x76)].concat();
    trace.extend([Event::StopPoll(false), Event::InterruptPoll(false)]);
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn reset_clears_pc_enable_delay_and_halt_preserving_literal_registers() {
    // decision-5 item 9; 98-153B Ch. 2 Halt Sequences/Start-up p. 2-13
    // (PDF 27), Figs. 2-12/2-13 pp. 2-14/2-15 (PDF 28/29), Ch. 5
    // RESET/INTE p. 5-14 (PDF 76): PC=0, INTE=0, halt exits; flags,
    // accumulator, working registers and SP unchanged. At power-up their
    // contents remain indeterminate, not zeroed by reset (p. 2-13).
    // 9800301-04 Ch. 3 EI/HLT pp. 3-23/3-24 agrees on PC/INTE/halt.
    // EI-delay clearing is explicitly unverified by these reset descriptions.
    for interrupt_enable in [false, true] {
        for ei_delay in [false, true] {
            for halted in [false, true] {
                for flags in [
                    Flags {
                        sign: true,
                        zero: false,
                        auxiliary_carry: true,
                        parity: false,
                        carry: true,
                    },
                    Flags {
                        sign: false,
                        zero: true,
                        auxiliary_carry: false,
                        parity: true,
                        carry: false,
                    },
                ] {
                    let mut cpu = I8080::new(
                        Registers {
                            a: 0xA1,
                            flags,
                            b: 0xB2,
                            c: 0xC3,
                            d: 0xD4,
                            e: 0xE5,
                            h: 0xF6,
                            l: 0x07,
                            sp: 0x8F90,
                            pc: 0xABCD,
                        },
                        Latches {
                            interrupt_enable,
                            ei_delay,
                            halted,
                        },
                    );
                    let expected = Registers {
                        a: 0xA1,
                        flags,
                        b: 0xB2,
                        c: 0xC3,
                        d: 0xD4,
                        e: 0xE5,
                        h: 0xF6,
                        l: 0x07,
                        sp: 0x8F90,
                        pc: 0x0000,
                    };
                    for _ in 0..2 {
                        cpu.reset();
                        assert_eq!(*cpu.registers(), expected);
                        assert_eq!(*cpu.latches(), DISABLED);
                    }
                }
            }
        }
    }
}

#[test]
fn reset_restarts_a_cpu_halted_by_ei_hlt_at_address_zero() {
    // decision-5 item 9; 98-153B Ch. 2 p. 2-13, Ch. 5 RESET p. 5-14;
    // 9800301-04 Ch. 3 HLT p. 3-24: reset restarts from address zero.
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0xFB, 0x76]);
    let memory = bus.memory.clone();
    let mut cpu = I8080::new(sample_registers(0x1234), DISABLED);
    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 2,
            cycles: 11,
            exit: Exit::Halt
        }
    );
    bus.log.clear();
    cpu.reset();
    assert_eq!(*cpu.registers(), sample_registers(0x0000));
    assert_eq!(*cpu.latches(), DISABLED);
    assert!(bus.log.is_empty());
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget
        }
    );
    assert_eq!(*cpu.registers(), sample_registers(0x0001));
    assert_eq!(*cpu.latches(), DISABLED);
    assert_eq!(bus.take_log(), fetch(0x0000, 0x00));
    assert_eq!(bus.memory, memory);
}
