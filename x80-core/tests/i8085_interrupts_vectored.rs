//! Internally vectored interrupts: decision-12 rules 2-4, 6-8;
//! decision-5 items 3-5/8. Intel 205775-002 (January 1983), §2.2.7
//! pp. 2-4/2-5 (PDF 27/28), §2.3.5 pp. 2-15/2-16 (PDF 38/39),
//! RST p. 5-14 (PDF 99), RIM/SIM pp. 5-17/5-18 (PDF 102/103),
//! Table 5-1 p. 5-19 (PDF 104; Table 5-2 is the opcode list).
//! Second witnesses: Intel 9800301-04 (May 1981), priority p. 1-24,
//! EI/HLT pp. 3-23/3-24, RIM p. 3-49, RST pp. 3-54/3-55,
//! SIM p. 3-60; Intel 231718-001 (September 1987), Table 2 p. 16-13,
//! TRAP/first RIM p. 16-15, Table 3 p. 16-20 (PDF 960/962/967).
//! Source hashes match PROVENANCE.md §1; cited values checked on images.
//!
//! Acceptance's 12 states are DERIVED: Fig. 2-19 has six BI states,
//! replacing RST's six-state fetch; two three-state stack writes follow
//! (Fig. 2-18 p. 2-14, RST p. 5-14, Table 5-1 SWW). BI has no bus
//! call, RD or INTA. Each expected trace/byte/vector is test-side.
//! Boundary edge sampling is the rule 2 API choice, not pin-cycle timing.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Budget, Exit, MemoryAccess, Report};

fn registers() -> Registers {
    Registers {
        a: 0x91,
        b: 0x12,
        c: 0x34,
        d: 0x56,
        e: 0x78,
        h: 0x9A,
        l: 0xBC,
        flags: Flags {
            sign: true,
            zero: false,
            auxiliary_carry: true,
            parity: false,
            carry: true,
        },
        sp: 0x8100,
        pc: 0x23FF,
    }
}

fn enabled() -> Latches {
    Latches {
        interrupt_enable: true,
        mask7_5: false,
        mask6_5: false,
        mask5_5: false,
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

fn polls(lines: [bool; 5]) -> Vec<Event> {
    let [trap, rst7_5, rst6_5, rst5_5, intr] = lines;
    vec![
        Event::StopPoll(false),
        Event::TrapPoll(trap),
        Event::Rst7_5Poll(rst7_5),
        Event::Rst6_5Poll(rst6_5),
        Event::Rst5_5Poll(rst5_5),
        Event::IntrPoll(intr),
    ]
}

fn set_lines(bus: &mut RecordingBus, lines: [bool; 5]) {
    [bus.trap, bus.rst7_5, bus.rst6_5, bus.rst5_5, bus.intr] = lines;
}

fn write(address: u16, value: u8) -> Event {
    Event::Write {
        address,
        kind: MemoryAccess::Stack,
        value,
    }
}

fn fetch(lines: [bool; 5], address: u16, value: u8) -> Vec<Event> {
    let mut trace = polls(lines);
    trace.push(Event::Read {
        address,
        kind: MemoryAccess::OpcodeFetch,
        value,
    });
    trace
}

fn rim(lines: [bool; 5], address: u16) -> Vec<Event> {
    let mut trace = fetch(lines, address, 0x20);
    trace.extend([
        Event::SidPoll(false),
        Event::Rst6_5Poll(lines[2]),
        Event::Rst5_5Poll(lines[3]),
    ]);
    trace
}

fn acceptance_witness(lines: [bool; 5], vector: u16, expected_latches: Latches) {
    let mut cpu = I8085::new(
        registers(),
        Latches {
            halted: true,
            ..enabled()
        },
    );
    let mut bus = RecordingBus::new();
    set_lines(&mut bus, lines);
    // Neither the boundary opcode nor a handler opcode belongs to this step.
    bus.load(0x23FF, &[0x08]);
    bus.load(vector, &[0x08]);
    let mut memory = bus.memory.clone();
    memory[0x80FF] = 0x23;
    memory[0x80FE] = 0xFF;
    assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: vector,
            sp: 0x80FE,
            ..registers()
        }
    );
    assert_eq!(*cpu.latches(), expected_latches);
    assert_eq!(bus.memory, memory);
    let mut trace = polls(lines);
    trace.extend([write(0x80FF, 0x23), write(0x80FE, 0xFF)]);
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn trap_literal_trace_vector_24h_and_derived_twelve_states() {
    // 205775-002 p. 2-5; 9800301-04 p. 3-55; DS Table 2 p. 16-13.
    // TRAP disable/save/disarm: DS p. 16-15, RIM p. 5-17.
    acceptance_witness(
        [true, false, false, false, false],
        0x0024,
        Latches {
            interrupt_enable: false,
            trap_armed: false,
            trap_saved_interrupt_enable: true,
            trap_rim_pending: true,
            last_trap: true,
            ..enabled()
        },
    );
}

#[test]
fn rst7_5_literal_trace_vector_3ch_and_derived_twelve_states() {
    // Same vector sources; latch cleared on acceptance: 205775-002
    // p. 5-18, 9800301-04 p. 3-60. Holding high must not restore it.
    acceptance_witness(
        [false, true, false, false, false],
        0x003C,
        Latches {
            interrupt_enable: false,
            last_rst7_5: true,
            ..enabled()
        },
    );
}

#[test]
fn rst6_5_literal_trace_vector_34h_and_derived_twelve_states() {
    acceptance_witness(
        [false, false, true, false, false],
        0x0034,
        Latches {
            interrupt_enable: false,
            ..enabled()
        },
    );
}

#[test]
fn rst5_5_literal_trace_vector_2ch_and_derived_twelve_states() {
    acceptance_witness(
        [false, false, false, true, false],
        0x002C,
        Latches {
            interrupt_enable: false,
            ..enabled()
        },
    );
}

#[test]
fn every_higher_lower_priority_pair_including_intr() {
    // All ten pairs: 205775-002 p. 2-5; 9800301-04 p. 1-24;
    // 231718-001 Table 2 p. 16-13. INTR must never acknowledge here.
    for (higher, vector) in [(0, 0x0024), (1, 0x003C), (2, 0x0034), (3, 0x002C)] {
        for lower in higher + 1..5 {
            let mut cpu = I8085::new(registers(), enabled());
            let mut bus = RecordingBus::new();
            let mut lines = [false; 5];
            lines[higher] = true;
            lines[lower] = true;
            set_lines(&mut bus, lines);
            assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
            assert_eq!(
                *cpu.registers(),
                Registers {
                    pc: vector,
                    sp: 0x80FE,
                    ..registers()
                }
            );
            assert!(!cpu.latches().interrupt_enable);
            assert_eq!(cpu.latches().rst7_5_pending, lower == 1);
            let mut trace = polls(lines);
            trace.extend([write(0x80FF, 0x23), write(0x80FE, 0xFF)]);
            assert_eq!(bus.take_log(), trace, "higher={higher}, lower={lower}");
        }
    }
}

#[test]
fn rst_sources_require_enable_clear_mask_and_expired_delay() {
    // 205775-002 pp. 2-4, 5-17/5-18; 9800301-04 pp. 3-23/3-60.
    for (source, vector) in [(1, 0x003C), (2, 0x0034), (3, 0x002C)] {
        for blocked_by in 0..3 {
            let mut initial = enabled();
            match blocked_by {
                0 => initial.interrupt_enable = false,
                1 => match source {
                    1 => initial.mask7_5 = true,
                    2 => initial.mask6_5 = true,
                    _ => initial.mask5_5 = true,
                },
                _ => initial.ei_delay = true,
            }
            let mut cpu = I8085::new(registers(), initial);
            let mut bus = RecordingBus::new();
            let mut lines = [false; 5];
            lines[source] = true;
            set_lines(&mut bus, lines);
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(
                *cpu.registers(),
                Registers {
                    pc: 0x2400,
                    ..registers()
                }
            );
            assert_eq!(bus.take_log(), fetch(lines, 0x23FF, 0));
            assert_eq!(cpu.latches().rst7_5_pending, source == 1);
            // Remove only the obstruction; the next boundary accepts.
            let latches = cpu.latches_mut();
            latches.interrupt_enable = true;
            latches.mask7_5 = false;
            latches.mask6_5 = false;
            latches.mask5_5 = false;
            assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
            assert_eq!(cpu.registers().pc, vector);
            let mut trace = polls(lines);
            trace.extend([write(0x80FF, 0x24), write(0x80FE, 0x00)]);
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn masked_rst7_5_edge_survives_low_is_visible_to_rim_and_unmask_accepts() {
    // 205775-002 pp. 5-17/5-18; 9800301-04 pp. 3-49/3-60.
    let mut cpu = I8085::new(
        registers(),
        Latches {
            interrupt_enable: true,
            ..Latches::default()
        },
    );
    let mut bus = RecordingBus::new();
    bus.load(0x23FF, &[0, 0x20, 0x30]);
    bus.load(0x003C, &[0x20]);
    bus.rst7_5 = true;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert!(cpu.latches().rst7_5_pending);
    assert_eq!(
        bus.take_log(),
        fetch([false, true, false, false, false], 0x23FF, 0)
    );
    bus.rst7_5 = false;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(cpu.registers().a, 0x4F); // I7.5 + IE + all masks.
    assert!(cpu.latches().rst7_5_pending);
    assert_eq!(bus.take_log(), rim([false; 5], 0x2400));
    cpu.registers_mut().a = 0x0B; // MSE=1, only 7.5 unmasked; no latch reset.
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert!(cpu.latches().rst7_5_pending);
    assert!(!cpu.latches().mask7_5);
    assert_eq!(bus.take_log(), fetch([false; 5], 0x2401, 0x30));
    assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x003C, 0x80FE));
    assert!(!cpu.latches().rst7_5_pending);
    let mut trace = polls([false; 5]);
    trace.extend([write(0x80FF, 0x24), write(0x80FE, 0x02)]);
    assert_eq!(bus.take_log(), trace);
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(cpu.registers().a, 0x03); // Pending and IE both cleared.
    assert_eq!(bus.take_log(), rim([false; 5], 0x003C));
}

#[test]
fn sim_bit4_cancels_masked_rst7_5_edge_before_unmasking() {
    // 205775-002 p. 5-18; 9800301-04 p. 3-60: SIM bit 4 resets
    // the latch independently of MSE; a held high pin is not a new edge.
    let mut cpu = I8085::new(
        registers(),
        Latches {
            interrupt_enable: true,
            ..Latches::default()
        },
    );
    let mut bus = RecordingBus::new();
    bus.load(0x23FF, &[0, 0x30, 0x30, 0x20]);
    bus.rst7_5 = true;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert!(cpu.latches().rst7_5_pending);
    assert_eq!(
        bus.take_log(),
        fetch([false, true, false, false, false], 0x23FF, 0)
    );
    cpu.registers_mut().a = 0x10;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert!(!cpu.latches().rst7_5_pending);
    assert!(cpu.latches().mask7_5);
    assert_eq!(
        bus.take_log(),
        fetch([false, true, false, false, false], 0x2400, 0x30)
    );
    cpu.registers_mut().a = 0x0B;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(
        bus.take_log(),
        fetch([false, true, false, false, false], 0x2401, 0x30)
    );
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(cpu.registers().a, 0x0B);
    assert!(!cpu.latches().rst7_5_pending);
    assert_eq!(
        bus.take_log(),
        rim([false, true, false, false, false], 0x2402)
    );
}

#[test]
fn accepted_rst7_5_requires_another_rising_edge() {
    // Same latch sources as above, now clearing by acceptance itself.
    let mut cpu = I8085::new(registers(), enabled());
    let mut bus = RecordingBus::new();
    bus.rst7_5 = true;
    assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
    bus.take_log();
    cpu.latches_mut().interrupt_enable = true;
    for (level, pc) in [(true, 0x003C), (false, 0x003D)] {
        bus.rst7_5 = level;
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert!(!cpu.latches().rst7_5_pending);
        assert_eq!(
            bus.take_log(),
            fetch([false, level, false, false, false], pc, 0)
        );
    }
    bus.rst7_5 = true;
    assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x003C, 0x80FC));
    let mut trace = polls([false, true, false, false, false]);
    trace.extend([write(0x80FD, 0), write(0x80FC, 0x3E)]);
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn trap_held_high_does_not_retrigger_and_low_then_high_rearms() {
    // 205775-002 pp. 2-4/2-5 Fig. 2-6B; 231718-001 p. 16-15.
    let mut cpu = I8085::new(registers(), Latches::default());
    let mut bus = RecordingBus::new();
    bus.trap = true;
    assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
    assert!(!cpu.latches().trap_pending);
    assert!(!cpu.latches().trap_armed);
    bus.take_log();
    for pc in [0x0024, 0x0025] {
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert!(!cpu.latches().trap_pending);
        assert!(!cpu.latches().trap_armed);
        assert_eq!(cpu.registers().sp, 0x80FE);
        assert_eq!(
            bus.take_log(),
            fetch([true, false, false, false, false], pc, 0)
        );
    }
    bus.trap = false;
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert!(cpu.latches().trap_armed);
    assert_eq!(bus.take_log(), fetch([false; 5], 0x0026, 0));
    bus.trap = true;
    assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x0024, 0x80FC));
    assert!(!cpu.latches().trap_pending);
    assert!(!cpu.latches().trap_armed);
    let mut trace = polls([true, false, false, false, false]);
    trace.extend([write(0x80FD, 0), write(0x80FC, 0x27)]);
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn trap_wakes_halt_with_interrupts_disabled_and_pushes_post_hlt_pc() {
    // 205775-002 p. 2-16; 9800301-04 p. 3-24 explicitly permits TRAP
    // while disabled. HLT five states, PC is next instruction; no ack.
    let mut cpu = I8085::new(registers(), Latches::default());
    let mut bus = RecordingBus::new();
    bus.load(0x23FF, &[0x76, 0x08]);
    assert_eq!(cpu.step(&mut bus), report(1, 5, Exit::Budget));
    assert!(cpu.latches().halted);
    assert_eq!(bus.take_log(), fetch([false; 5], 0x23FF, 0x76));
    bus.trap = true;
    assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0024,
            sp: 0x80FE,
            ..registers()
        }
    );
    assert!(!cpu.latches().halted);
    assert!(!cpu.latches().interrupt_enable);
    assert!(!cpu.latches().trap_saved_interrupt_enable);
    let mut trace = polls([true, false, false, false, false]);
    trace.extend([write(0x80FF, 0x24), write(0x80FE, 0)]);
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn disabled_rst_lines_do_not_wake_halt() {
    // Same halt sources; unmasked alone is insufficient.
    for source in 1..4 {
        let mut cpu = I8085::new(
            registers(),
            Latches {
                interrupt_enable: false,
                ..enabled()
            },
        );
        let mut bus = RecordingBus::new();
        bus.load(0x23FF, &[0x76]);
        assert_eq!(cpu.step(&mut bus), report(1, 5, Exit::Budget));
        bus.take_log();
        let mut lines = [false; 5];
        lines[source] = true;
        set_lines(&mut bus, lines);
        let memory = bus.memory.clone();
        assert_eq!(cpu.step(&mut bus), report(0, 0, Exit::Halt));
        assert_eq!(cpu.registers().pc, 0x2400);
        assert_eq!(cpu.registers().sp, 0x8100);
        assert!(cpu.latches().halted);
        assert_eq!(bus.memory, memory);
        assert_eq!(bus.take_log(), polls(lines));
    }
}

#[test]
fn first_rim_after_trap_uses_saved_enable_second_uses_current_enable() {
    // 205775-002 RIM p. 5-17; 231718-001 p. 16-15 explicitly first
    // vs subsequent RIM; 9800301-04 p. 3-49 pins IE at bit 3.
    // Opposite current enable also exposes incorrectly saving 'false'.
    for (prior, current, first, second) in [(true, false, 0x0F, 0x07), (false, true, 0x07, 0x0F)] {
        let mut cpu = I8085::new(
            registers(),
            Latches {
                interrupt_enable: prior,
                ..Latches::default()
            },
        );
        let mut bus = RecordingBus::new();
        bus.load(0x0024, &[0, 0x20, 0x20]);
        bus.trap = true;
        assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
        assert_eq!(cpu.latches().trap_saved_interrupt_enable, prior);
        assert!(cpu.latches().trap_rim_pending);
        assert!(!cpu.latches().interrupt_enable);
        bus.take_log();
        cpu.latches_mut().interrupt_enable = current;
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert!(cpu.latches().trap_rim_pending); // NOP does not consume it.
        assert_eq!(
            bus.take_log(),
            fetch([true, false, false, false, false], 0x0024, 0)
        );
        for (pc, next, a) in [(0x0025, 0x0026, first), (0x0026, 0x0027, second)] {
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(
                *cpu.registers(),
                Registers {
                    pc: next,
                    sp: 0x80FE,
                    a,
                    ..registers()
                }
            );
            assert!(!cpu.latches().trap_rim_pending);
            assert_eq!(bus.take_log(), rim([true, false, false, false, false], pc));
        }
    }
}

#[test]
fn rst6_5_and_rst5_5_falling_before_eligible_boundary_are_not_latched() {
    // 205775-002 pp. 2-4/2-5; 231718-001 Table 2 p. 16-13:
    // high level until sampled. Rule 2 samples only at boundaries.
    for (source, vector) in [(2, 0x0034), (3, 0x002C)] {
        let mut cpu = I8085::new(
            registers(),
            Latches {
                ei_delay: true,
                ..enabled()
            },
        );
        let mut bus = RecordingBus::new();
        let mut lines = [false; 5];
        lines[source] = true;
        set_lines(&mut bus, lines);
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert_eq!(bus.take_log(), fetch(lines, 0x23FF, 0));
        set_lines(&mut bus, [false; 5]);
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 0x2401,
                ..registers()
            }
        );
        assert_eq!(bus.take_log(), fetch([false; 5], 0x2400, 0));
        set_lines(&mut bus, lines);
        assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
        assert_eq!(cpu.registers().pc, vector);
        let mut trace = polls(lines);
        trace.extend([write(0x80FF, 0x24), write(0x80FE, 0x01)]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn trap_waits_through_ei_or_di_next_boundary_inhibition_unverified() {
    // Decision-12 rule 6's UNVERIFIED reading of 205775-002 p. 5-17,
    // 9800301-04 p. 3-23. Held pending TRAP accepts once delay expires.
    for (opcode, prior) in [(0xFB, true), (0xF3, false)] {
        let mut cpu = I8085::new(registers(), Latches::default());
        let mut bus = RecordingBus::new();
        bus.load(0x23FF, &[opcode, 0]);
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert_eq!(bus.take_log(), fetch([false; 5], 0x23FF, opcode));
        bus.trap = true;
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert!(cpu.latches().trap_pending);
        assert!(!cpu.latches().ei_delay);
        assert_eq!(
            bus.take_log(),
            fetch([true, false, false, false, false], 0x2400, 0)
        );
        assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
        assert_eq!(cpu.registers().pc, 0x0024);
        assert_eq!(cpu.latches().trap_saved_interrupt_enable, prior);
        let mut trace = polls([true, false, false, false, false]);
        trace.extend([write(0x80FF, 0x24), write(0x80FE, 0x01)]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn acceptance_preserves_masks_sod_and_other_pending_rst7_5() {
    // 205775-002 p. 5-18: only 7.5's own acceptance clears its latch;
    // DS p. 16-15: acceptance clears IE. No SIM/mask/SOD operation.
    for (lines, vector) in [
        ([true, false, false, false, false], 0x0024),
        ([false, false, true, false, false], 0x0034),
        ([false, false, false, true, false], 0x002C),
    ] {
        let initial = Latches {
            mask7_5: true,
            rst7_5_pending: true,
            trap_saved_interrupt_enable: true,
            trap_rim_pending: true,
            sod: true,
            ..enabled()
        };
        let mut cpu = I8085::new(registers(), initial);
        let mut bus = RecordingBus::new();
        set_lines(&mut bus, lines);
        assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
        assert_eq!(cpu.registers().pc, vector);
        assert_eq!(
            *cpu.latches(),
            Latches {
                interrupt_enable: false,
                trap_armed: !lines[0],
                last_trap: lines[0],
                ..initial
            }
        );
        let mut trace = polls(lines);
        trace.extend([write(0x80FF, 0x23), write(0x80FE, 0xFF)]);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn cycle_budget_is_checked_after_complete_acceptance_and_run_continues_at_vector() {
    // Decision-5 items 5/8 and decision-12 rule 4: one step, no handler
    // fetch when budget reached; overshoot completes both stack writes.
    for (budget, steps, cycles, pc) in [
        (
            Budget {
                steps: 10,
                cycles: 1,
            },
            1,
            12,
            0x0034,
        ),
        (
            Budget {
                steps: 10,
                cycles: 12,
            },
            1,
            12,
            0x0034,
        ),
        (
            Budget {
                steps: 2,
                cycles: 100,
            },
            2,
            16,
            0x0035,
        ),
    ] {
        let mut cpu = I8085::new(registers(), enabled());
        let mut bus = RecordingBus::new();
        bus.rst6_5 = true;
        assert_eq!(
            cpu.run(&mut bus, budget),
            report(steps, cycles, Exit::Budget)
        );
        assert_eq!((cpu.registers().pc, cpu.registers().sp), (pc, 0x80FE));
        let mut trace = polls([false, false, true, false, false]);
        trace.extend([write(0x80FF, 0x23), write(0x80FE, 0xFF)]);
        if steps == 2 {
            trace.extend(fetch([false, false, true, false, false], 0x0034, 0));
        }
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn derived_sp_wrap_and_boundary_pc_ffff_is_saved_without_increment() {
    // DERIVED SP wrap, not explicitly stated: 16-bit SP (205775-002
    // p. 2-2), RST push p. 5-14; 9800301-04 pp. 3-54/3-55.
    // The boundary PC is pushed unchanged even at FFFF (p. 2-5 note 1).
    for (sp, high_at, low_at) in [(0, 0xFFFF, 0xFFFE), (1, 0, 0xFFFF)] {
        let mut cpu = I8085::new(
            Registers {
                pc: 0xFFFF,
                sp,
                ..registers()
            },
            enabled(),
        );
        let mut bus = RecordingBus::new();
        bus.rst5_5 = true;
        assert_eq!(cpu.step(&mut bus), report(1, 12, Exit::Budget));
        assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x002C, low_at));
        let mut trace = polls([false, false, false, true, false]);
        trace.extend([write(high_at, 0xFF), write(low_at, 0xFF)]);
        assert_eq!(bus.take_log(), trace);
    }
}
