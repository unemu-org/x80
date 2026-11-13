//! Boundary and refusal witnesses: decisions 2, 5 (items 5–8), 13–15.
//!
//! Hardware references: Zilog UM008011-0816 rev. 11 (August 2016), Registers
//! p. 3 (R), Pin Functions p. 6 (NMI), NOP p. 180, EI p. 183; second
//! witness 03-0029-01 (1977), §2 p. 4, §3 pp. 7–8, §8 p. 55 and Table
//! 7.0-5 p. 48. Poll ordering, rollback and prompt Halt are API choices.
//! Prefix/refusal sets: UM Tables 6/11 pp. 42/50, Fig. 39 p. 54,
//! BIT p. 247, IN p. 296; TM Tables 7.0-1/7/8/11 pp. 44/50/51/54.
//! Indexed-CB's fourth-byte OperandFetch is the unverified reading of
//! decision-15 rule 5. No semantics are inferred from another emulator.

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

fn cpu_at(pc: u16) -> Z80 {
    Z80::new(
        Registers {
            a: 0xA1,
            flags: Flags::from_byte(0x95),
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
            sp: 0xDEF0,
            pc,
            i: 0xF8,
            r: 0xFF,
        },
        Latches {
            iff1: true,
            iff2: true,
            interrupt_mode: InterruptMode::Mode2,
            ei_delay: true,
            halted: false,
            nmi_asserted: true,
            parity_note_pending: true,
        },
    )
}

fn polls(stop: bool, nmi: bool, int: bool) -> [Event; 3] {
    [
        Event::StopPoll(stop),
        Event::NmiPoll(nmi),
        Event::IntPoll(int),
    ]
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

fn retired(steps: u64, cycles: u64) -> Report {
    Report {
        steps,
        cycles,
        exit: Exit::Budget,
    }
}

#[test]
fn either_zero_budget_wins_without_any_poll_or_read() {
    for budget in [
        Budget {
            steps: 0,
            cycles: 99,
        },
        Budget {
            steps: 99,
            cycles: 0,
        },
    ] {
        let mut cpu = cpu_at(0x1000);
        cpu.latches_mut().halted = true;
        let before = cpu.clone();
        let mut bus = RecordingBus::new();
        bus.stop = true;
        bus.nmi = true;
        bus.int = true;
        bus.load(0x1000, &[0xDD, 0x44]);
        assert_eq!(cpu.run(&mut bus, budget), retired(0, 0));
        assert_eq!(cpu, before);
        assert!(bus.take_log().is_empty());
    }
}

#[test]
fn stop_wins_over_interrupts_halt_and_fetch_but_polls_every_line() {
    for halted in [false, true] {
        let mut cpu = cpu_at(0x1000);
        cpu.latches_mut().halted = halted;
        cpu.latches_mut().nmi_asserted = false;
        cpu.latches_mut().ei_delay = false;
        let before = cpu.clone();
        let mut bus = RecordingBus::new();
        bus.stop = true;
        bus.nmi = true;
        bus.int = true;
        assert_eq!(
            cpu.run(&mut bus, UNLIMITED),
            Report {
                steps: 0,
                cycles: 0,
                exit: Exit::Stop
            }
        );
        assert_eq!(cpu, before);
        assert_eq!(bus.take_log(), polls(true, true, true));
    }
}

#[test]
fn halted_cpu_returns_promptly_without_refresh_and_keeps_samples() {
    let mut cpu = cpu_at(0x1000);
    cpu.latches_mut().halted = true;
    let mut bus = RecordingBus::new();
    // No edge and INT inhibited: independent of the interrupt beads landing.
    bus.nmi_levels.extend([true, true, false]);
    bus.int = true;
    let before = cpu.clone();
    for _ in 0..2 {
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 0,
                cycles: 0,
                exit: Exit::Halt
            }
        );
        assert_eq!(cpu, before);
        assert_eq!(bus.take_log(), polls(false, true, true));
    }
    assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    let mut expected = before;
    expected.latches_mut().nmi_asserted = false;
    assert_eq!(cpu, expected);
    assert_eq!(bus.take_log(), polls(false, false, true));
}

#[test]
fn nop_wraps_pc_retires_four_states_and_preserves_refresh_bit_seven() {
    // UM NOP p. 180; TM Table 7.0-5 p. 48. PC wrap is derived from its
    // 16-bit width (UM p. 3), not an explicit wrap statement.
    for (r, next_r) in [(0x7F, 0x00), (0xFF, 0x80), (0x80, 0x81)] {
        let mut cpu = cpu_at(0xFFFF);
        cpu.registers_mut().r = r;
        let mut expected = cpu.clone();
        expected.registers_mut().pc = 0;
        expected.registers_mut().r = next_r;
        expected.latches_mut().nmi_asserted = false;
        expected.latches_mut().ei_delay = false;
        expected.latches_mut().parity_note_pending = false;
        let mut bus = RecordingBus::new();
        let memory = bus.memory.clone();
        assert_eq!(cpu.step(&mut bus), retired(1, 4));
        assert_eq!(cpu, expected);
        assert_eq!(bus.memory, memory);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0xFFFF, OP, 0)
            ]
        );
    }
}

#[test]
fn cycle_overshoot_and_step_limit_stop_before_another_poll() {
    for budget in [
        Budget {
            steps: 10,
            cycles: 5,
        },
        Budget {
            steps: 2,
            cycles: 99,
        },
    ] {
        let mut cpu = cpu_at(0xFFFE);
        let mut bus = RecordingBus::new();
        bus.nmi_levels.extend([false, false, true]);
        assert_eq!(cpu.run(&mut bus, budget), retired(2, 8));
        assert_eq!(cpu.registers().pc, 0);
        assert_eq!(cpu.registers().r, 0x81);
        assert_eq!(bus.nmi_levels.iter().copied().collect::<Vec<_>>(), [true]);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0xFFFE, OP, 0),
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0xFFFF, OP, 0),
            ]
        );
    }
}

// Literal refusal reads, not a whole-set witness. A staging case may complete
// once its group lands: this file never needs editing by a group bead. While
// refused it must have precisely the hole contract, including repeated retry.
struct Landed<'a> {
    pc: u16,
    r: u8,
    cycles: u64,
    trace: &'a [Event],
}

fn refusal(
    bytes: &[u8],
    preceding: [u8; 3],
    len: u8,
    opcode: u8,
    reads: &[Event],
    landed: Option<Landed<'_>>,
) {
    let mut cpu = cpu_at(0xFFFE);
    let before = cpu.clone();
    let mut bus = RecordingBus::new();
    bus.load(0xFFFE, bytes);
    let memory = bus.memory.clone();
    for _ in 0..2 {
        let report = cpu.step(&mut bus);
        if let Some(landed) = &landed {
            if report.exit == Exit::Budget {
                assert_eq!(report, retired(1, landed.cycles));
                assert_eq!(cpu.registers().pc, landed.pc);
                assert_eq!(cpu.registers().r, landed.r);
                // These entries write no memory. Register results other than
                // PC and R belong to the instruction groups' own tests.
                assert_eq!(bus.memory, memory);
                assert_eq!(bus.take_log(), landed.trace);
                return;
            }
        }
        assert_eq!((report.steps, report.cycles), (0, 0));
        let Exit::Unsupported(unsupported) = report.exit else {
            panic!("expected refusal, got {report:?}");
        };
        assert_eq!(unsupported.pc, 0xFFFE);
        assert_eq!(unsupported.opcode, opcode);
        assert!(!unsupported.acknowledge);
        assert_eq!(unsupported.preceding, preceding);
        assert_eq!(unsupported.preceding_len, len);
        assert_eq!(cpu, before);
        assert_eq!(bus.memory, memory);
        let mut trace = Vec::from(polls(false, false, false));
        trace.extend_from_slice(reads);
        assert_eq!(bus.take_log(), trace);
    }
}

macro_rules! hole {
    ($name:ident, $bytes:expr, $preceding:expr, $len:expr, $op:expr, $reads:expr) => {
        #[test]
        fn $name() {
            refusal($bytes, $preceding, $len, $op, $reads, None);
        }
    };
}

hole!(
    dd_index_half,
    &[0xDD, 0x44],
    [0xDD, 0, 0],
    1,
    0x44,
    &[read(0xFFFE, OP, 0xDD), read(0xFFFF, OP, 0x44)]
);
hole!(
    fd_index_half,
    &[0xFD, 0x44],
    [0xFD, 0, 0],
    1,
    0x44,
    &[read(0xFFFE, OP, 0xFD), read(0xFFFF, OP, 0x44)]
);
hole!(
    repeated_dd_stops_before_21,
    &[0xDD, 0xDD, 0x21],
    [0xDD, 0, 0],
    1,
    0xDD,
    &[read(0xFFFE, OP, 0xDD), read(0xFFFF, OP, 0xDD)]
);
hole!(
    dd_ed,
    &[0xDD, 0xED],
    [0xDD, 0, 0],
    1,
    0xED,
    &[read(0xFFFE, OP, 0xDD), read(0xFFFF, OP, 0xED)]
);
// Decision-20 §1: ED70 and ED00 now retire; literal witnesses live in
// z80_undocumented_ed.rs. CB/DD/FD refusals remain until §§2–3.
hole!(
    cb_missing_shift,
    &[0xCB, 0x30],
    [0xCB, 0, 0],
    1,
    0x30,
    &[read(0xFFFE, OP, 0xCB), read(0xFFFF, OP, 0x30)]
);
hole!(
    dd_cb_register_result,
    &[0xDD, 0xCB, 0x80, 0x47],
    [0xDD, 0xCB, 0x80],
    3,
    0x47,
    &[
        read(0xFFFE, OP, 0xDD),
        read(0xFFFF, OP, 0xCB),
        read(0, ARG, 0x80),
        read(1, ARG, 0x47)
    ]
);
hole!(
    fd_cb_register_result,
    &[0xFD, 0xCB, 0x7F, 0x47],
    [0xFD, 0xCB, 0x7F],
    3,
    0x47,
    &[
        read(0xFFFE, OP, 0xFD),
        read(0xFFFF, OP, 0xCB),
        read(0, ARG, 0x7F),
        read(1, ARG, 0x47)
    ]
);
hole!(
    ignored_index_prefix,
    &[0xFD, 0x00],
    [0xFD, 0, 0],
    1,
    0x00,
    &[read(0xFFFE, OP, 0xFD), read(0xFFFF, OP, 0x00)]
);

#[test]
fn staging_main_load_does_not_fetch_immediate_before_refusal() {
    // UM LD r,n p. 72; TM Table 7.0-1 p. 44.
    // Landed traces follow decision-15 rules 5–6: OP increments R's low
    // seven bits, ARG and Data do not. Initial R is FFh in cpu_at.
    refusal(
        &[0x3E, 0xA5],
        [0; 3],
        0,
        0x3E,
        &[read(0xFFFE, OP, 0x3E)],
        Some(Landed {
            pc: 0,
            r: 0x80,
            cycles: 7,
            trace: &[
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0xFFFE, OP, 0x3E),
                read(0xFFFF, ARG, 0xA5),
            ],
        }),
    );
}

#[test]
fn staging_cb_rotate() {
    // UM RLC r p. 213; TM Table 7.0-7 p. 50.
    refusal(
        &[0xCB, 0x00],
        [0xCB, 0, 0],
        1,
        0,
        &[read(0xFFFE, OP, 0xCB), read(0xFFFF, OP, 0)],
        Some(Landed {
            pc: 0,
            r: 0x81,
            cycles: 8,
            trace: &[
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0xFFFE, OP, 0xCB),
                read(0xFFFF, OP, 0),
            ],
        }),
    );
}

#[test]
fn staging_ed_neg() {
    // UM NEG p. 176; TM Table 7.0-5 p. 48.
    refusal(
        &[0xED, 0x44],
        [0xED, 0, 0],
        1,
        0x44,
        &[read(0xFFFE, OP, 0xED), read(0xFFFF, OP, 0x44)],
        Some(Landed {
            pc: 0,
            r: 0x81,
            cycles: 8,
            trace: &[
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0xFFFE, OP, 0xED),
                read(0xFFFF, OP, 0x44),
            ],
        }),
    );
}

#[test]
fn staging_index_load_does_not_fetch_immediate_before_refusal() {
    // UM LD IX,nn p. 101; TM Table 7.0-2 p. 45.
    refusal(
        &[0xDD, 0x21, 0xA5, 0x5A],
        [0xDD, 0, 0],
        1,
        0x21,
        &[read(0xFFFE, OP, 0xDD), read(0xFFFF, OP, 0x21)],
        Some(Landed {
            pc: 2,
            r: 0x81,
            cycles: 14,
            trace: &[
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0xFFFE, OP, 0xDD),
                read(0xFFFF, OP, 0x21),
                read(0, ARG, 0xA5),
                read(1, ARG, 0x5A),
            ],
        }),
    );
}

#[test]
fn staging_index_memory_load_does_not_fetch_displacement_before_refusal() {
    // UM LD r,(IY+d) p. 77; TM Table 7.0-1 p. 44.
    // cpu_at sets IY = 92A3h; d = -128 gives 9223h, initially zero.
    refusal(
        &[0xFD, 0x46, 0x80],
        [0xFD, 0, 0],
        1,
        0x46,
        &[read(0xFFFE, OP, 0xFD), read(0xFFFF, OP, 0x46)],
        Some(Landed {
            pc: 1,
            r: 0x81,
            cycles: 19,
            trace: &[
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0xFFFE, OP, 0xFD),
                read(0xFFFF, OP, 0x46),
                read(0, ARG, 0x80),
                read(0x9223, MemoryAccess::Data, 0),
            ],
        }),
    );
}

#[test]
fn staging_index_cb_bit_reads_displacement_and_key_only() {
    // UM BIT b,(IY+d) p. 249; TM Table 7.0-8 p. 51.
    // cpu_at sets IY = 92A3h; d = -2 gives 92A1h, initially zero.
    refusal(
        &[0xFD, 0xCB, 0xFE, 0x46],
        [0xFD, 0xCB, 0xFE],
        3,
        0x46,
        &[
            read(0xFFFE, OP, 0xFD),
            read(0xFFFF, OP, 0xCB),
            read(0, ARG, 0xFE),
            read(1, ARG, 0x46),
        ],
        Some(Landed {
            pc: 2,
            r: 0x81,
            cycles: 20,
            trace: &[
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0xFFFE, OP, 0xFD),
                read(0xFFFF, OP, 0xCB),
                read(0, ARG, 0xFE),
                read(1, ARG, 0x46),
                read(0x92A1, MemoryAccess::Data, 0),
            ],
        }),
    );
}

#[test]
fn unsupported_keeps_earlier_retirement_and_restores_that_boundary() {
    // Decision-20 §1: use DD00 for rollback; ED00 now executes as L NOP.
    let mut cpu = cpu_at(0x2000);
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0, 0xDD, 0]);
    bus.nmi_levels.extend([false, false]);
    let report = cpu.run(&mut bus, UNLIMITED);
    assert_eq!((report.steps, report.cycles), (1, 4));
    let Exit::Unsupported(unsupported) = report.exit else {
        panic!("{report:?}");
    };
    assert_eq!(unsupported.pc, 0x2001);
    assert_eq!(unsupported.opcode, 0);
    assert_eq!(unsupported.preceding, [0xDD, 0, 0]);
    assert_eq!(unsupported.preceding_len, 1);
    assert!(!unsupported.acknowledge);
    let mut expected = cpu_at(0x2001);
    expected.registers_mut().r = 0x80;
    expected.latches_mut().nmi_asserted = false;
    expected.latches_mut().ei_delay = false;
    expected.latches_mut().parity_note_pending = false;
    assert_eq!(cpu, expected);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            read(0x2000, OP, 0),
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            read(0x2001, OP, 0xDD),
            read(0x2002, OP, 0),
        ]
    );
}

#[test]
fn stop_pulse_is_resampled_only_if_still_asserted() {
    for held in [false, true] {
        let mut cpu = cpu_at(0x3000);
        cpu.latches_mut().nmi_asserted = false;
        let before = cpu.clone();
        let mut bus = RecordingBus::new();
        bus.stop = true;
        bus.nmi = true;
        assert_eq!(cpu.step(&mut bus).exit, Exit::Stop);
        assert_eq!(cpu, before);
        assert_eq!(bus.take_log(), polls(true, true, false));
        bus.stop = false;
        bus.nmi = held;
        let report = cpu.step(&mut bus);
        // Until NMI lands an eligible edge falls through to NOP; afterwards
        // the same seam accepts it. Both paths must retain the new sample.
        if held && report.cycles == 11 {
            assert_eq!(report, retired(1, 11));
            assert_eq!(cpu.registers().pc, 0x66);
            assert_eq!(
                bus.take_log(),
                [
                    Event::StopPoll(false),
                    Event::NmiPoll(true),
                    Event::IntPoll(false),
                    read(0x3000, OP, 0),
                    Event::Write {
                        address: 0xDEEF,
                        kind: MemoryAccess::Stack,
                        value: 0x30
                    },
                    Event::Write {
                        address: 0xDEEE,
                        kind: MemoryAccess::Stack,
                        value: 0
                    },
                ]
            );
        } else {
            assert_eq!(report, retired(1, 4));
            assert_eq!(
                bus.take_log(),
                [
                    Event::StopPoll(false),
                    Event::NmiPoll(held),
                    Event::IntPoll(false),
                    read(0x3000, OP, 0)
                ]
            );
        }
        assert_eq!(cpu.latches().nmi_asserted, held);
        assert_eq!(cpu.registers().r, 0x80);
    }
}

#[test]
fn ei_delay_and_parity_note_survive_nonretiring_exits_and_expire_before_nop() {
    let mut cpu = cpu_at(0x4000);
    let mut bus = RecordingBus::new();
    bus.nmi = true; // already sampled: no edge
    bus.int = true;
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: 0,
                cycles: 0
            }
        ),
        retired(0, 0)
    );
    assert!(bus.take_log().is_empty());
    bus.stop = true;
    assert_eq!(cpu.step(&mut bus).exit, Exit::Stop);
    assert_eq!(bus.take_log(), polls(true, true, true));
    assert!(cpu.latches().ei_delay);
    assert!(cpu.latches().parity_note_pending);
    bus.stop = false;
    assert_eq!(cpu.step(&mut bus), retired(1, 4));
    assert!(!cpu.latches().ei_delay);
    assert!(!cpu.latches().parity_note_pending);
    assert!(cpu.registers().flags.parity_overflow);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(true),
            read(0x4000, OP, 0)
        ]
    );
    // An embedder can re-arm the delay across a budget split. Ineligible INT
    // must leave P/V alone; NOP consumes only the bookkeeping, not the flag.
    cpu.latches_mut().ei_delay = true;
    cpu.latches_mut().parity_note_pending = true;
    assert_eq!(cpu.step(&mut bus), retired(1, 4));
    assert!(!cpu.latches().ei_delay);
    assert!(!cpu.latches().parity_note_pending);
    assert!(cpu.registers().flags.parity_overflow);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(true),
            Event::IntPoll(true),
            read(0x4001, OP, 0)
        ]
    );
}
