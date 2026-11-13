//! RIM/SIM witnesses for decision-10 and decision-12 rules 2, 8 and 9.
//!
//! Intel 205775-002 (January 1983), §5.6.5 pp. 5-17/5-18 (PDF 102/103),
//! Table 5-1 p. 5-19 (PDF 104), Table 5-2 p. 5-20 (PDF 105), §2.3.8
//! p. 2-19 (PDF 42); Intel 9800301-04 (May 1981), RIM p. 3-49 (PDF 103),
//! SIM p. 3-60 (PDF 114). All cited values checked on rendered page images;
//! source hashes checked against PROVENANCE.md §1. Expectations below are
//! literal or test-side references derived from those 8085 pages.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Exit, MemoryAccess, Report};

const STEP: Report = Report {
    steps: 1,
    cycles: 4,
    exit: Exit::Budget,
};

fn registers(pc: u16, a: u8) -> Registers {
    Registers {
        a,
        flags: Flags {
            overflow: false,
            k: false,
            sign: true,
            zero: false,
            auxiliary_carry: true,
            parity: false,
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
    }
}

fn fetch(pc: u16, opcode: u8) -> Vec<Event> {
    // Decision-12 rule 1's boundary polls precede the single OpcodeFetch.
    // 205775-002 Table 5-1 p. 5-19 gives F only for both instructions;
    // Table 5-2 p. 5-20 pins 20/30. Ch. 2 p. 2-9 describes OF at PC.
    vec![
        Event::StopPoll(false),
        Event::TrapPoll(false),
        Event::Rst7_5Poll(false),
        Event::Rst6_5Poll(false),
        Event::Rst5_5Poll(false),
        Event::IntrPoll(false),
        Event::Read {
            address: pc,
            kind: MemoryAccess::OpcodeFetch,
            value: opcode,
        },
    ]
}

fn rim_trace(pc: u16, sid: bool, rst6_5: bool, rst5_5: bool) -> Vec<Event> {
    // §2.3.8 p. 2-19 transfers SID and then interrupt pins during RIM.
    // The exact API read order is decision-12 rule 8, not pin-cycle timing.
    let mut trace = fetch(pc, 0x20);
    trace.extend([
        Event::SidPoll(sid),
        Event::Rst6_5Poll(rst6_5),
        Event::Rst5_5Poll(rst5_5),
    ]);
    trace
}

fn digit(value: u8, place: u8) -> bool {
    (value / place) % 2 == 1
}

#[test]
fn rim_assembles_all_input_combinations() {
    // 205775-002 RIM p. 5-17; 9800301-04 RIM p. 3-49:
    // MSB first: SID, I7.5, I6.5, I5.5, IE, M7.5, M6.5, M5.5.
    // Form the reference as a binary numeral, independently of the crate's
    // shifts/OR. Use TRAP's saved IE to exercise both IE values while live
    // interrupt enable is false, so pending requests cannot preempt RIM.
    // First-RIM saved status: 205775-002 p. 5-17; 231718-001 p. 16-15.
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0x20]);
    let memory = bus.memory.clone();
    for combination in 0..=255 {
        let bits = [128, 64, 32, 16, 8, 4, 2, 1].map(|place| digit(combination, place));
        let [sid, pending7_5, rst6_5, rst5_5, ie, mask7_5, mask6_5, mask5_5] = bits;
        let expected_a = bits
            .into_iter()
            .fold(0, |number, bit| number * 2 + u8::from(bit));
        let initial = Latches {
            rst7_5_pending: pending7_5,
            mask7_5,
            mask6_5,
            mask5_5,
            trap_saved_interrupt_enable: ie,
            trap_rim_pending: true,
            sod: true,
            ..Latches::default()
        };
        let mut cpu = I8085::new(registers(0x1234, 0xA5), initial);
        bus.sid = sid;
        bus.rst6_5_levels.extend([false, rst6_5]);
        bus.rst5_5_levels.extend([false, rst5_5]);
        assert_eq!(cpu.step(&mut bus), STEP, "inputs={combination:02X}");
        assert_eq!(*cpu.registers(), registers(0x1235, expected_a));
        assert_eq!(
            *cpu.latches(),
            Latches {
                trap_rim_pending: false,
                ..initial
            }
        );
        assert_eq!(bus.take_log(), rim_trace(0x1234, sid, rst6_5, rst5_5));
    }
    assert_eq!(bus.memory, memory);
}

#[test]
fn rim_reads_live_lines_once_after_fetch_in_fixed_order() {
    // 205775-002 §2.3.8 p. 2-19 and RIM p. 5-17; 9800301-04 p. 3-49.
    // Boundary 6.5/5.5 levels are opposite to the instruction's samples.
    // A held-high 7.5 pin with a cleared flip-flop must still contribute 0.
    let initial = Latches {
        mask6_5: false,
        last_rst7_5: true,
        ..Latches::default()
    };
    let mut cpu = I8085::new(registers(0x2345, 0xFF), initial);
    let mut bus = RecordingBus::new();
    bus.load(0x2345, &[0x20]);
    bus.rst7_5 = true;
    bus.sid_levels.extend([true, false]);
    bus.rst6_5_levels.extend([false, true, false]);
    bus.rst5_5_levels.extend([true, false, true]);
    assert_eq!(cpu.step(&mut bus), STEP);
    assert_eq!(*cpu.registers(), registers(0x2346, 0xA5));
    assert_eq!(*cpu.latches(), initial);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(true),
            Event::IntrPoll(false),
            Event::Read {
                address: 0x2345,
                kind: MemoryAccess::OpcodeFetch,
                value: 0x20,
            },
            Event::SidPoll(true),
            Event::Rst6_5Poll(true),
            Event::Rst5_5Poll(false),
        ]
    );
    assert_eq!(bus.sid_levels.into_iter().collect::<Vec<_>>(), [false]);
    assert_eq!(bus.rst6_5_levels.into_iter().collect::<Vec<_>>(), [false]);
    assert_eq!(bus.rst5_5_levels.into_iter().collect::<Vec<_>>(), [true]);
}

#[test]
fn rim_trap_saved_enable_is_consumed_once_then_current_enable_is_used() {
    // 205775-002 RIM p. 5-17's special TRAP case; 231718-001, handbook
    // p. 16-15 explicitly says first RIM uses saved IE, all later RIMs use
    // current IE. 9800301-04 p. 3-49 independently pins IE at bit 3.
    // Set latches directly: TRAP acceptance belongs to a later bead.
    for current in [false, true] {
        for saved in [false, true] {
            for pending in [false, true] {
                let initial = Latches {
                    interrupt_enable: current,
                    trap_saved_interrupt_enable: saved,
                    trap_rim_pending: pending,
                    ..Latches::default()
                };
                let mut cpu = I8085::new(registers(0x3456, 0xFF), initial);
                let mut bus = RecordingBus::new();
                bus.load(0x3456, &[0x20, 0x20]);
                let first_ie = if pending { saved } else { current };
                for (pc, next_pc, ie) in [(0x3456, 0x3457, first_ie), (0x3457, 0x3458, current)] {
                    assert_eq!(cpu.step(&mut bus), STEP);
                    assert_eq!(
                        *cpu.registers(),
                        registers(next_pc, if ie { 0x0F } else { 0x07 })
                    );
                    assert_eq!(
                        *cpu.latches(),
                        Latches {
                            trap_rim_pending: false,
                            ..initial
                        }
                    );
                    assert_eq!(bus.take_log(), rim_trace(pc, false, false, false));
                }
            }
        }
    }
}

#[test]
fn rim_ie_bit_during_pending_ei_delay_is_unverified() {
    // UNVERIFIED: 205775-002 p. 5-17 (EI and RIM) and 9800301-04
    // pp. 3-23/3-49 do not settle RIM's IE bit during EI's delay. Pin the
    // model's stored enable, without treating this as a hardware witness.
    // Retirement expires the delay in the existing boundary loop (rule 6).
    for (enable, expected_a) in [(false, 0x07), (true, 0x0F)] {
        let initial = Latches {
            interrupt_enable: enable,
            ei_delay: true,
            ..Latches::default()
        };
        let mut cpu = I8085::new(registers(0x4567, 0xFF), initial);
        let mut bus = RecordingBus::new();
        bus.load(0x4567, &[0x20]);
        assert_eq!(cpu.step(&mut bus), STEP);
        assert_eq!(*cpu.registers(), registers(0x4568, expected_a));
        assert_eq!(
            *cpu.latches(),
            Latches {
                ei_delay: false,
                ..initial
            }
        );
        assert_eq!(bus.take_log(), rim_trace(0x4567, false, false, false));
    }
}

#[test]
fn sim_all_accumulator_bit_classes_and_initial_mask_pending_sod_combinations() {
    // 205775-002 SIM p. 5-18; 9800301-04 SIM p. 3-60. Exhaustively
    // include MSE=0 (keep masks), SOE=0 (keep SOD, no notification), bit 4
    // alone, bit 5 ignored, both values of bit 7, and combined operations.
    // This test-side reference uses quotient/remainder digits, not the
    // crate's masks. Callback timing/count is decision-12 rule 8's API.
    let mut bus = RecordingBus::new();
    bus.load(0x5678, &[0x30]);
    let memory = bus.memory.clone();
    for a in 0..=255 {
        for masks in 0..8 {
            for pending in [false, true] {
                for sod in [false, true] {
                    let initial = Latches {
                        mask5_5: digit(masks, 1),
                        mask6_5: digit(masks, 2),
                        mask7_5: digit(masks, 4),
                        rst7_5_pending: pending,
                        trap_saved_interrupt_enable: true,
                        trap_rim_pending: true,
                        sod,
                        ..Latches::default()
                    };
                    let mut expected = initial;
                    if digit(a, 8) {
                        expected.mask5_5 = digit(a, 1);
                        expected.mask6_5 = digit(a, 2);
                        expected.mask7_5 = digit(a, 4);
                    }
                    expected.rst7_5_pending = pending && !digit(a, 16);
                    let mut trace = fetch(0x5678, 0x30);
                    if digit(a, 64) {
                        expected.sod = digit(a, 128);
                        trace.push(Event::SodChanged(expected.sod));
                    }
                    let mut cpu = I8085::new(registers(0x5678, a), initial);
                    assert_eq!(cpu.step(&mut bus), STEP, "A={a:02X}, initial={initial:?}");
                    assert_eq!(*cpu.registers(), registers(0x5679, a));
                    assert_eq!(*cpu.latches(), expected, "A={a:02X}, initial={initial:?}");
                    assert_eq!(bus.take_log(), trace, "A={a:02X}");
                }
            }
        }
    }
    assert_eq!(bus.memory, memory);
}

#[test]
fn sim_bit4_alone_clears_flip_flop_until_a_new_rising_sample() {
    // 205775-002 pp. 5-17/5-18 and 9800301-04 p. 3-60: bit 4 clears
    // the latch independently of MSE/SOE; only a rising edge sets it.
    // Decision-12 rule 2 models edges from boundary levels, not pin cycles.
    let mut cpu = I8085::new(
        registers(0x6789, 0x10),
        Latches {
            rst7_5_pending: true,
            last_rst7_5: true,
            sod: true,
            ..Latches::default()
        },
    );
    let mut bus = RecordingBus::new();
    bus.load(0x6789, &[0x30, 0x20, 0x20, 0x20]);
    bus.rst7_5 = true;
    assert_eq!(cpu.step(&mut bus), STEP);
    assert_eq!(*cpu.registers(), registers(0x678A, 0x10));
    assert_eq!(
        *cpu.latches(),
        Latches {
            last_rst7_5: true,
            sod: true,
            ..Latches::default()
        }
    );
    let mut trace = fetch(0x6789, 0x30);
    trace[2] = Event::Rst7_5Poll(true);
    assert_eq!(bus.take_log(), trace);
    for (pc, next_pc, line, expected_a, pending) in [
        (0x678A, 0x678B, true, 0x07, false),
        (0x678B, 0x678C, false, 0x07, false),
        (0x678C, 0x678D, true, 0x47, true),
    ] {
        bus.rst7_5 = line;
        assert_eq!(cpu.step(&mut bus), STEP);
        assert_eq!(*cpu.registers(), registers(next_pc, expected_a));
        assert_eq!(
            *cpu.latches(),
            Latches {
                rst7_5_pending: pending,
                last_rst7_5: line,
                sod: true,
                ..Latches::default()
            }
        );
        let mut trace = rim_trace(pc, false, false, false);
        trace[2] = Event::Rst7_5Poll(line);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn sim_notifies_in_program_order_even_when_sod_level_is_repeated() {
    // 205775-002 p. 5-18 and 9800301-04 p. 3-60: SOE loads SOD.
    // Decision-12 rule 8 notifies every enabled write, including repeated
    // levels, during SIM. This deliberately simplifies §2.3.8 p. 2-19's
    // physical transfer during the instruction following SIM.
    let mut cpu = I8085::new(registers(0x789A, 0xC0), Latches::default());
    let mut bus = RecordingBus::new();
    bus.load(0x789A, &[0x30, 0x30, 0x30, 0x30, 0x30]);
    let mut trace = Vec::new();
    for (pc, next_pc, a, level, notify) in [
        (0x789A, 0x789B, 0xC0, true, true),
        (0x789B, 0x789C, 0xC0, true, true),
        (0x789C, 0x789D, 0x40, false, true),
        (0x789D, 0x789E, 0x40, false, true),
        (0x789E, 0x789F, 0x80, false, false),
    ] {
        cpu.registers_mut().a = a;
        assert_eq!(cpu.step(&mut bus), STEP);
        assert_eq!(*cpu.registers(), registers(next_pc, a));
        assert_eq!(
            *cpu.latches(),
            Latches {
                sod: level,
                ..Latches::default()
            }
        );
        trace.extend(fetch(pc, 0x30));
        if notify {
            trace.push(Event::SodChanged(level));
        }
    }
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn rim_and_sim_preserve_every_flag_combination() {
    // Both entries say Flags: none, 205775-002 pp. 5-17/5-18;
    // 9800301-04 pp. 3-49/3-60. Compare fields, never a crate PSW helper.
    for combination in 0..32 {
        let flags = Flags {
            overflow: false,
            k: false,
            sign: digit(combination, 16),
            zero: digit(combination, 8),
            auxiliary_carry: digit(combination, 4),
            parity: digit(combination, 2),
            carry: digit(combination, 1),
        };
        for (opcode, expected_a) in [(0x20, 0x07), (0x30, 0xFF)] {
            let mut initial = registers(0x89AB, 0xFF);
            initial.flags = flags;
            let mut cpu = I8085::new(initial, Latches::default());
            let mut bus = RecordingBus::new();
            bus.load(0x89AB, &[opcode]);
            assert_eq!(cpu.step(&mut bus), STEP);
            assert_eq!(
                *cpu.registers(),
                Registers {
                    pc: 0x89AC,
                    a: expected_a,
                    ..initial
                }
            );
            let trace = if opcode == 0x20 {
                rim_trace(0x89AB, false, false, false)
            } else {
                let mut trace = fetch(0x89AB, 0x30);
                trace.push(Event::SodChanged(true));
                trace
            };
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn rim_and_sim_pc_wrap_is_derived_from_the_16_bit_width() {
    // DERIVED: wrapping is not stated on the instruction pages. Sequential
    // execution is 205775-002 §5.4 p. 5-3; the register width is §2.2.1
    // p. 2-2. Both instructions are one byte and four states (pp. 5-17/
    // 5-18, Table 5-1 p. 5-19; 9800301-04 pp. 3-49/3-60).
    for (opcode, expected_a) in [(0x20, 0x07), (0x30, 0x80)] {
        let mut cpu = I8085::new(registers(0xFFFF, 0x80), Latches::default());
        let mut bus = RecordingBus::new();
        bus.load(0xFFFF, &[opcode]);
        assert_eq!(cpu.step(&mut bus), STEP);
        assert_eq!(*cpu.registers(), registers(0x0000, expected_a));
        assert_eq!(*cpu.latches(), Latches::default());
        let trace = if opcode == 0x20 {
            rim_trace(0xFFFF, false, false, false)
        } else {
            fetch(0xFFFF, 0x30)
        };
        assert_eq!(bus.take_log(), trace);
    }
}
