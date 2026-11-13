//! Decision-20 §4 replaces zero Y/X and former unknown preservation here.
//! H class rules: Young v0.91 §§4.1–4.3 pp. 15–17 / §§8.1–8.11 pp. 26–35;
//! Rak z80test v1.2a corresponding load/ALU/BIT/block-I/O groups, pinned in
//! PROVENANCE §1. SCF/CCF use decision-20 §6 Q; parity-window rules stay L.
//! Machine-control witnesses: decisions 13–15. Zilog UM008011-0816
//! rev. 11 (August 2016), NOP/EI/DI/HALT/IM pp. 180–186 (PDF 194–200);
//! 03-0029-01 (1977), Table 7.0-5 p. 48 (PDF 54). Rendered at 150 dpi.
//! EI/EI re-arming and DI cancellation are decision-15 rule 7's unverified
//! latch model; prompt Halt is decision-2's departure from idle refresh.
//! Before landing: NOP passes; all other executing witnesses fail with
//! Unsupported, 0 new steps/cycles (the sequences stop at their first EI/HALT).

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, MemoryAccess, Report};

const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

fn cpu_at(pc: u16) -> Z80 {
    Z80::new(
        Registers {
            a: 0x73,
            flags: Flags::from_byte(0x85),
            b: 0x12,
            c: 0x34,
            d: 0x56,
            e: 0x78,
            h: 0x9A,
            l: 0xBC,
            a_alternate: 0xDE,
            flags_alternate: Flags::from_byte(0x42),
            b_alternate: 0xF0,
            c_alternate: 0x13,
            d_alternate: 0x25,
            e_alternate: 0x37,
            h_alternate: 0x49,
            l_alternate: 0x5B,
            ix: 0x6789,
            iy: 0xABCD,
            sp: 0xDEF0,
            pc,
            i: 0xF8,
            r: 0xFF,
        },
        Latches {
            q: false,
            wz: 0,
            iff1: false,
            iff2: true,
            interrupt_mode: InterruptMode::Mode2,
            ei_delay: false,
            halted: false,
            nmi_asserted: false,
            parity_note_pending: false,
        },
    )
}

fn report(steps: u64, cycles: u64, exit: Exit) -> Report {
    Report {
        steps,
        cycles,
        exit,
    }
}

fn fetch(pc: u16, opcode: u8, int: bool) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(int),
        Event::Read {
            address: pc,
            kind: MemoryAccess::OpcodeFetch,
            value: opcode,
        },
    ]
}

#[test]
fn nop_preserves_registers_flags_iffs_and_mode_with_derived_pc_r_wrap() {
    // UM p. 180, TM p. 48. PC wrap is DERIVED from 16-bit width (UM
    // p. 3); low-seven-bit R increments preserve bit 7 (decision-15 rule 6).
    for (r, next_r) in [(0x7F, 0x00), (0xFF, 0x80), (0x80, 0x81)] {
        for flags in [0x00, 0xD7] {
            let mut cpu = cpu_at(0xFFFF);
            cpu.registers_mut().r = r;
            cpu.registers_mut().flags = Flags::from_byte(flags);
            cpu.latches_mut().ei_delay = true;
            cpu.latches_mut().parity_note_pending = true;
            let mut expected = cpu.clone();
            expected.registers_mut().pc = 0;
            expected.registers_mut().r = next_r;
            expected.latches_mut().ei_delay = false;
            expected.latches_mut().parity_note_pending = false;
            let mut bus = RecordingBus::new();
            let memory = bus.memory.clone();
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(cpu, expected);
            assert_eq!(bus.memory, memory);
            assert_eq!(bus.take_log(), fetch(0xFFFF, 0x00, false));
        }
    }
}

#[test]
fn ei_one_step_sets_both_iffs_and_leaves_new_delay_armed() {
    // UM p. 183, TM p. 48. This pins that run consumes the OLD delay
    // BEFORE execute: clearing after execute would lose EI's new delay.
    for iff1 in [false, true] {
        for iff2 in [false, true] {
            for delay in [false, true] {
                for flags in [0x00, 0xD7] {
                    let mut cpu = cpu_at(0xFFFF);
                    cpu.registers_mut().flags = Flags::from_byte(flags);
                    cpu.latches_mut().iff1 = iff1;
                    cpu.latches_mut().iff2 = iff2;
                    cpu.latches_mut().ei_delay = delay;
                    let mut expected = cpu.clone();
                    expected.registers_mut().pc = 0;
                    expected.registers_mut().r = 0x80;
                    expected.latches_mut().iff1 = true;
                    expected.latches_mut().iff2 = true;
                    expected.latches_mut().ei_delay = true;
                    let mut bus = RecordingBus::new();
                    bus.load(0xFFFF, &[0xFB]);
                    let memory = bus.memory.clone();
                    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
                    assert!(cpu.latches().ei_delay);
                    assert!(cpu.latches().iff1 && cpu.latches().iff2);
                    assert_eq!(cpu, expected);
                    assert_eq!(bus.memory, memory);
                    assert_eq!(bus.take_log(), fetch(0xFFFF, 0xFB, false));
                }
            }
        }
    }
}

#[test]
fn asserted_int_after_ei_allows_following_instruction_through_run() {
    // UM p. 183: INT inhibited through the following instruction.
    // The INT seam still declines acceptance in this branch, so this pins
    // EI's boundary latch plus continued execution, not actual acceptance
    // after expiry. Acknowledge/stack accesses are absent from the full log.
    let mut cpu = cpu_at(0x2000);
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0xFB, 0x2F]); // EI; CPL (UM p. 175)
    let memory = bus.memory.clone();
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    let mut expected = cpu_at(0x2001);
    expected.registers_mut().r = 0x80;
    expected.latches_mut().iff1 = true;
    expected.latches_mut().ei_delay = true;
    assert_eq!(cpu, expected);
    assert_eq!(bus.take_log(), fetch(0x2000, 0xFB, false));
    bus.int = true;
    assert!(cpu.latches().ei_delay); // boundary immediately after EI
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: 1,
                cycles: u64::MAX
            }
        ),
        report(1, 4, Exit::Budget)
    );
    expected.registers_mut().pc = 0x2002;
    expected.registers_mut().r = 0x81;
    expected.registers_mut().a = 0x8C;
    expected.registers_mut().flags = Flags::from_byte(0x9F);
    expected.latches_mut().q = true;
    expected.latches_mut().ei_delay = false;
    assert_eq!(cpu, expected);
    assert_eq!(bus.memory, memory);
    assert_eq!(bus.take_log(), fetch(0x2001, 0x2F, true));
}

#[test]
fn ei_ei_rearms_delay_unverified_edge_case_then_nop_consumes_it() {
    // Decision-15 rule 7: UNVERIFIED edge case, not stated by UM p. 183.
    let mut cpu = cpu_at(0x2000);
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0xFB, 0xFB, 0x00]);
    bus.int_levels.extend([false, true, true]);
    let memory = bus.memory.clone();
    for (pc, opcode, next_pc, r, delay, int) in [
        (0x2000, 0xFB, 0x2001, 0x80, true, false),
        (0x2001, 0xFB, 0x2002, 0x81, true, true),
        (0x2002, 0x00, 0x2003, 0x82, false, true),
    ] {
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        let mut expected = cpu_at(next_pc);
        expected.registers_mut().r = r;
        expected.latches_mut().iff1 = true;
        expected.latches_mut().ei_delay = delay;
        assert_eq!(cpu, expected);
        assert_eq!(bus.take_log(), fetch(pc, opcode, int));
    }
    assert_eq!(bus.memory, memory);
}

#[test]
fn di_clears_both_iffs_and_cancels_pending_delay() {
    // UM p. 182, TM p. 48. Cancellation is decision-15 rule 7's
    // unverified delay bookkeeping. Exercise every old IFF combination.
    for iff1 in [false, true] {
        for iff2 in [false, true] {
            for delay in [false, true] {
                for flags in [0x00, 0xD7] {
                    let mut cpu = cpu_at(0xFFFF);
                    cpu.registers_mut().flags = Flags::from_byte(flags);
                    cpu.latches_mut().iff1 = iff1;
                    cpu.latches_mut().iff2 = iff2;
                    cpu.latches_mut().ei_delay = delay;
                    let mut expected = cpu.clone();
                    expected.registers_mut().pc = 0;
                    expected.registers_mut().r = 0x80;
                    expected.latches_mut().iff1 = false;
                    expected.latches_mut().iff2 = false;
                    expected.latches_mut().ei_delay = false;
                    let mut bus = RecordingBus::new();
                    bus.load(0xFFFF, &[0xF3]);
                    let memory = bus.memory.clone();
                    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
                    assert_eq!(cpu, expected);
                    assert_eq!(bus.memory, memory);
                    assert_eq!(bus.take_log(), fetch(0xFFFF, 0xF3, false));
                }
            }
        }
    }
}

#[test]
fn di_after_ei_cancels_delay_and_keeps_int_disabled() {
    // UM pp. 182–183; decision-15 rule 7's cancellation model.
    let mut cpu = cpu_at(0x2000);
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0xFB, 0xF3, 0x00]);
    bus.int_levels.extend([false, true, true]);
    let memory = bus.memory.clone();
    for (pc, opcode, next_pc, r, enabled, int) in [
        (0x2000, 0xFB, 0x2001, 0x80, true, false),
        (0x2001, 0xF3, 0x2002, 0x81, false, true),
        (0x2002, 0x00, 0x2003, 0x82, false, true),
    ] {
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        let mut expected = cpu_at(next_pc);
        expected.registers_mut().r = r;
        expected.latches_mut().iff1 = enabled;
        expected.latches_mut().iff2 = enabled;
        expected.latches_mut().ei_delay = enabled;
        assert_eq!(cpu, expected);
        assert_eq!(bus.take_log(), fetch(pc, opcode, int));
    }
    assert_eq!(bus.memory, memory);
}

#[test]
fn halt_retires_four_states_keeps_resume_pc_then_run_returns_without_fetch() {
    // UM p. 181, TM p. 48; post-HALT PC is decision-15 rule 7's
    // derived reading. Prompt Halt without idle fetch/R is decision-2.
    for pc in [0x2000, 0xFFFF] {
        for flags in [0x00, 0xD7] {
            let mut cpu = cpu_at(pc);
            cpu.registers_mut().flags = Flags::from_byte(flags);
            let mut expected = cpu.clone();
            expected.registers_mut().pc = if pc == 0xFFFF { 0 } else { 0x2001 };
            expected.registers_mut().r = 0x80;
            expected.latches_mut().halted = true;
            let mut bus = RecordingBus::new();
            bus.load(pc, &[0x76, 0x2F]);
            let memory = bus.memory.clone();
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(cpu, expected);
            assert_eq!(bus.take_log(), fetch(pc, 0x76, false));
            for _ in 0..2 {
                assert_eq!(cpu.run(&mut bus, UNLIMITED), report(0, 0, Exit::Halt));
                assert_eq!(cpu, expected);
                assert_eq!(
                    bus.take_log(),
                    [
                        Event::StopPoll(false),
                        Event::NmiPoll(false),
                        Event::IntPoll(false),
                    ]
                );
            }
            assert_eq!(bus.memory, memory);
        }
    }
}

#[test]
fn interrupt_modes_change_only_mode_with_two_opcode_fetches() {
    // UM pp. 184–186; TM p. 48. Two M1s: R FF->81 (rule 6).
    for (opcode, mode) in [
        (0x46, InterruptMode::Mode0),
        (0x56, InterruptMode::Mode1),
        (0x5E, InterruptMode::Mode2),
    ] {
        for old_mode in [
            InterruptMode::Mode0,
            InterruptMode::Mode1,
            InterruptMode::Mode2,
        ] {
            for flags in [0x00, 0xD7] {
                let mut cpu = cpu_at(0xFFFF);
                cpu.registers_mut().flags = Flags::from_byte(flags);
                cpu.latches_mut().interrupt_mode = old_mode;
                let mut expected = cpu.clone();
                expected.registers_mut().pc = 1; // DERIVED PC wrap
                expected.registers_mut().r = 0x81;
                expected.latches_mut().interrupt_mode = mode;
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[0xED, opcode]);
                let memory = bus.memory.clone();
                assert_eq!(cpu.step(&mut bus), report(1, 8, Exit::Budget));
                assert_eq!(cpu, expected);
                assert_eq!(bus.memory, memory);
                let mut trace = fetch(0xFFFF, 0xED, false);
                trace.push(Event::Read {
                    address: 0,
                    kind: MemoryAccess::OpcodeFetch,
                    value: opcode,
                });
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}
