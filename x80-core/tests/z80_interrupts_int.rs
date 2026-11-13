//! Decision-20 §4 replaces zero Y/X and former unknown preservation here.
//! H class rules: Young v0.91 §§4.1–4.3 pp. 15–17 / §§8.1–8.11 pp. 26–35;
//! Rak z80test v1.2a corresponding load/ALU/BIT/block-I/O groups, pinned in
//! PROVENANCE §1. SCF/CCF stay interim L until §6; parity-window rules stay L.
//! INT acceptance witnesses: decision-15 rules 1–3, 6–7, 9–10, 13.
//!
//! Primary: UM008011-0816 rev. 11 (August 2016), printed pp. 3, 6,
//! 12–14, 18–20 (Fig. 17), 94, 180–183, 288, 290, 292–293;
//! PDF = printed + 14. Second witnesses: 03-0029-01 (1977), §2 p. 4,
//! §3 pp. 7–8, §4 pp. 16/18, §8 pp. 55–57, Table 7.0-10 p. 53
//! (PDF = printed + 6); 03-0002-01 (January 1978), pp. 48/248
//! (PDF 52/257); MK78515 V2.0 (October 1978), Table 2-1 p. 2-15
//! (PDF 29). Values checked on rendered 150 dpi pages, independently of
//! implementation helpers. Polls, rollback and budgets are API choices.
//! IM 0/1 totals are DERIVED: UM p. 19's +2 and RST p. 293's 11,
//! also TM pp. 56/53. IM 2's 19 is STATED: UM p. 20/TM p. 57.
//! Rule 10's next-boundary parity window remains UNVERIFIED.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, MemoryAccess, Report, Unsupported};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ACK: MemoryAccess = MemoryAccess::InterruptAcknowledge;
const DATA: MemoryAccess = MemoryAccess::Data;
const STACK: MemoryAccess = MemoryAccess::Stack;
const MODES: [InterruptMode; 3] = [
    InterruptMode::Mode0,
    InterruptMode::Mode1,
    InterruptMode::Mode2,
];

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

fn latches(mode: InterruptMode) -> Latches {
    Latches {
        iff1: true,
        iff2: true,
        interrupt_mode: mode,
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
fn im0_accepts_every_rst_with_unchanged_boundary_pc_and_derived_thirteen_states() {
    // UM CPU Response p. 19/RST pp. 292–293; TM pp. 56/53.
    for (opcode, target) in [
        (0xC7, 0x0000),
        (0xCF, 0x0008),
        (0xD7, 0x0010),
        (0xDF, 0x0018),
        (0xE7, 0x0020),
        (0xEF, 0x0028),
        (0xF7, 0x0030),
        (0xFF, 0x0038),
    ] {
        let mut cpu = Z80::new(registers(), latches(InterruptMode::Mode0));
        let mut bus = RecordingBus::new();
        bus.int = true;
        bus.acknowledges.extend([opcode, 0xA5]);
        bus.load(0x2345, &[0xED, 0]); // Must not fetch/decode this memory byte.
        let mut memory = bus.memory.clone();
        memory[0x80FF] = 0x23;
        memory[0x80FE] = 0x45;
        assert_eq!(cpu.step(&mut bus), report(1, 13, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: target,
                sp: 0x80FE,
                r: 0x80,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: target,
                iff1: false,
                iff2: false,
                ..latches(InterruptMode::Mode0)
            }
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(bus.acknowledges, [0xA5]);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(true),
                read(0x2345, ACK, opcode),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ]
        );
    }
}

#[test]
fn im1_ignores_every_acknowledge_byte_and_uses_0038_in_derived_thirteen_states() {
    // UM p. 19; TM p. 56: restart at 0038h regardless of device data.
    for opcode in 0..=255 {
        let mut cpu = Z80::new(registers(), latches(InterruptMode::Mode1));
        let mut bus = RecordingBus::new();
        bus.int = true;
        bus.acknowledges.extend([opcode, 0xA5]);
        bus.load(0x2345, &[0xED, 0]);
        let mut memory = bus.memory.clone();
        memory[0x80FF] = 0x23;
        memory[0x80FE] = 0x45;
        assert_eq!(cpu.step(&mut bus), report(1, 13, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 0x0038,
                sp: 0x80FE,
                r: 0x80,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: 0x0038,
                iff1: false,
                iff2: false,
                ..latches(InterruptMode::Mode1)
            }
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(bus.acknowledges, [0xA5]);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(true),
                read(0x2345, ACK, opcode),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ]
        );
    }
}

#[test]
fn im2_uses_i_and_even_byte_then_reads_low_high_after_push_in_stated_nineteen_states() {
    // UM p. 20/Fig. 17; TM p. 57. Highest legal pointer is FFFEh;
    // +1 reaches FFFFh. A vector-read wrap cannot occur with an even byte.
    for (i, opcode, low_at, high_at) in [
        (0xF8, 0x24, 0xF824, 0xF825),
        (0, 0, 0, 1),
        (0xFF, 0xFE, 0xFFFE, 0xFFFF),
        (0xF8, 0xFE, 0xF8FE, 0xF8FF),
    ] {
        let initial = Registers { i, ..registers() };
        let mut cpu = Z80::new(initial, latches(InterruptMode::Mode2));
        let mut bus = RecordingBus::new();
        bus.int = true;
        bus.acknowledges.extend([opcode, 0xA5]);
        bus.load(low_at, &[0x78, 0x56]);
        let mut memory = bus.memory.clone();
        memory[0x80FF] = 0x23;
        memory[0x80FE] = 0x45;
        assert_eq!(cpu.step(&mut bus), report(1, 19, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 0x5678,
                sp: 0x80FE,
                r: 0x80,
                ..initial
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: 0x5678,
                iff1: false,
                iff2: false,
                ..latches(InterruptMode::Mode2)
            }
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(bus.acknowledges, [0xA5]);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(true),
                read(0x2345, ACK, opcode),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
                read(low_at, DATA, 0x78),
                read(high_at, DATA, 0x56),
            ]
        );
    }
}

#[test]
fn im2_stack_alias_changes_the_vector_before_lookup() {
    // UM p. 20/TM p. 57 explicitly order PC save before table lookup.
    let initial = Registers {
        i: 0x80,
        ..registers()
    };
    let mut cpu = Z80::new(initial, latches(InterruptMode::Mode2));
    let mut bus = RecordingBus::new();
    bus.int = true;
    bus.acknowledges.push_back(0xFE);
    bus.load(0x80FE, &[0xBB, 0xAA]);
    let mut memory = bus.memory.clone();
    memory[0x80FF] = 0x23;
    memory[0x80FE] = 0x45;
    assert_eq!(cpu.step(&mut bus), report(1, 19, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x2345,
            sp: 0x80FE,
            r: 0x80,
            ..initial
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            wz: 0x2345,
            iff1: false,
            iff2: false,
            ..latches(InterruptMode::Mode2)
        }
    );
    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2345, ACK, 0xFE),
            write(0x80FF, 0x23),
            write(0x80FE, 0x45),
            read(0x80FE, DATA, 0x45),
            read(0x80FF, DATA, 0x23),
        ]
    );
}

#[test]
fn refusals_restore_every_register_and_latch_including_r_and_nmi_history_on_retry() {
    // Rules 2/9/13: library RST-only IM 0 and even-only IM 2 restrictions.
    // Hardware permits more IM 0 instructions (UM p. 19/TM p. 56).
    for (mode, bytes) in [
        (
            InterruptMode::Mode0,
            &[0x00, 0x76, 0xFB, 0xF3, 0xCD, 0xED, 0xDD, 0xCB, 0xFE][..],
        ),
        (InterruptMode::Mode2, &[0x01, 0x7F, 0xFF][..]),
    ] {
        for &opcode in bytes {
            let initial = Latches {
                halted: true,
                nmi_asserted: true,
                parity_note_pending: true,
                ..latches(mode)
            };
            let mut cpu = Z80::new(registers(), initial);
            let before = cpu.clone();
            let mut bus = RecordingBus::new();
            bus.int = true;
            bus.load(0x2345, &[0xED, 0]);
            let memory = bus.memory.clone();
            let mut previous = None;
            // Low samples would re-arm NMI if committed; the third, high
            // sample must still see the restored, already-asserted history.
            for nmi in [false, false, true] {
                bus.nmi = nmi;
                bus.acknowledges.push_back(opcode);
                let refusal = cpu.step(&mut bus);
                assert_eq!((refusal.steps, refusal.cycles), (0, 0));
                assert!(matches!(refusal.exit, Exit::Unsupported(Unsupported {
                    pc: 0x2345, opcode: byte, acknowledge: true,
                    preceding: [0, 0, 0], preceding_len: 0, ..
                }) if byte == opcode));
                if let Some(previous) = previous {
                    assert_eq!(refusal, previous);
                }
                previous = Some(refusal);
                assert_eq!(cpu, before);
                assert_eq!(bus.memory, memory);
                assert!(bus.acknowledges.is_empty()); // Read effects are not rolled back.
                assert_eq!(
                    bus.take_log(),
                    [
                        Event::StopPoll(false),
                        Event::NmiPoll(nmi),
                        Event::IntPoll(true),
                        read(0x2345, ACK, opcode),
                    ]
                );
            }
        }
    }
}

#[test]
fn iff1_clear_blocks_int_even_with_iff2_set_including_while_halted() {
    // UM p. 18/HALT Exit p. 14; TM pp. 55/18. An ineligible request
    // cannot clear P/V. Ordinary NOP expires the note; Halt preserves it.
    for mode in MODES {
        for halted in [false, true] {
            let initial = Latches {
                iff1: false,
                halted,
                parity_note_pending: true,
                ..latches(mode)
            };
            let mut cpu = Z80::new(registers(), initial);
            let mut bus = RecordingBus::new();
            bus.int = true;
            let memory = bus.memory.clone();
            if halted {
                let before = cpu.clone();
                assert_eq!(cpu.step(&mut bus), report(0, 0, Exit::Halt));
                assert_eq!(cpu, before);
                assert_eq!(bus.take_log(), polls(false, false, true));
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
                    *cpu.latches(),
                    Latches {
                        parity_note_pending: false,
                        ..initial
                    }
                );
                assert_eq!(
                    bus.take_log(),
                    [
                        Event::StopPoll(false),
                        Event::NmiPoll(false),
                        Event::IntPoll(true),
                        read(0x2345, OP, 0),
                    ]
                );
            }
            assert_eq!(bus.memory, memory);
        }
    }
}

#[test]
fn deasserted_int_does_not_accept_even_with_iff1_set() {
    // Rule 3 / UM p. 6, TM p. 8: INT is an asserted level request.
    for mode in MODES {
        let mut cpu = Z80::new(registers(), latches(mode));
        let mut bus = RecordingBus::new();
        let memory = bus.memory.clone();
        assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 0x2346,
                r: 0x80,
                ..registers()
            }
        );
        assert_eq!(*cpu.latches(), latches(mode));
        assert_eq!(bus.memory, memory);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0x2345, OP, 0),
            ]
        );
    }
}

#[test]
fn ei_defers_asserted_int_through_one_instruction_then_accepts() {
    // UM EI p. 183/Table 1 p. 18; TM §8 p. 55. NOP: UM p. 180.
    let initial = Latches {
        iff1: false,
        iff2: false,
        ..latches(InterruptMode::Mode1)
    };
    let mut cpu = Z80::new(registers(), initial);
    let mut bus = RecordingBus::new();
    bus.int = true;
    bus.acknowledges.push_back(0xDD);
    bus.load(0x2345, &[0xFB, 0, 0xED, 0]);
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
        *cpu.latches(),
        Latches {
            ei_delay: true,
            ..latches(InterruptMode::Mode1)
        }
    );
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2345, OP, 0xFB),
        ]
    );
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x2347,
            r: 0x81,
            ..registers()
        }
    );
    assert_eq!(*cpu.latches(), latches(InterruptMode::Mode1));
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2346, OP, 0),
        ]
    );
    assert_eq!(cpu.step(&mut bus), report(1, 13, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0038,
            sp: 0x80FE,
            r: 0x82,
            ..registers()
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            wz: 0x0038,
            iff1: false,
            iff2: false,
            ..latches(InterruptMode::Mode1)
        }
    );
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2347, ACK, 0xDD),
            write(0x80FF, 0x23),
            write(0x80FE, 0x47),
        ]
    );
}

#[test]
fn di_during_ei_delay_blocks_the_still_asserted_int() {
    // UM DI/EI pp. 182–183/Table 1 p. 18; TM pp. 55–56.
    // DI cancelling the bookkeeping delay is rule 7's unverified reading.
    let initial = Latches {
        iff1: false,
        iff2: false,
        ..latches(InterruptMode::Mode1)
    };
    let mut cpu = Z80::new(registers(), initial);
    let mut bus = RecordingBus::new();
    bus.int = true;
    bus.load(0x2345, &[0xFB, 0xF3, 0]);
    let memory = bus.memory.clone();
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(
        *cpu.latches(),
        Latches {
            ei_delay: true,
            ..latches(InterruptMode::Mode1)
        }
    );
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(*cpu.latches(), initial);
    assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x2348,
            r: 0x82,
            ..registers()
        }
    );
    assert_eq!(*cpu.latches(), initial);
    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2345, OP, 0xFB),
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2346, OP, 0xF3),
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2347, OP, 0),
        ]
    );
}

#[test]
fn halted_cpu_wakes_on_eligible_int_and_saves_the_post_halt_pc() {
    // UM HALT p. 181/HALT Exit p. 14; TM §4 p. 18. Post-HALT PC is
    // DERIVED (rule 7). Prompt Halt without idle fetches is an API departure.
    let mut cpu = Z80::new(registers(), latches(InterruptMode::Mode1));
    let mut bus = RecordingBus::new();
    bus.load(0x2345, &[0x76, 0xED, 0]);
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
        *cpu.latches(),
        Latches {
            halted: true,
            ..latches(InterruptMode::Mode1)
        }
    );
    let halted = cpu.clone();
    assert_eq!(cpu.step(&mut bus), report(0, 0, Exit::Halt));
    assert_eq!(cpu, halted);
    bus.int = true;
    bus.acknowledges.push_back(0);
    let mut memory = bus.memory.clone();
    memory[0x80FF] = 0x23;
    memory[0x80FE] = 0x46;
    assert_eq!(cpu.step(&mut bus), report(1, 13, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0038,
            sp: 0x80FE,
            r: 0x81,
            ..registers()
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            wz: 0x0038,
            iff1: false,
            iff2: false,
            ..latches(InterruptMode::Mode1)
        }
    );
    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            read(0x2345, OP, 0x76),
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2346, ACK, 0),
            write(0x80FF, 0x23),
            write(0x80FE, 0x46),
        ]
    );
}

#[test]
fn nmi_edge_beats_simultaneous_eligible_int_then_retn_allows_the_held_int() {
    // UM pp. 6, 13–14, 19/RETN p. 290; TM pp. 8, 18, 56/53;
    // ALPM RETN p. 248. NMI's 11 is DERIVED (restart equivalence + RST).
    for (mode, opcode, target, states) in [
        (InterruptMode::Mode0, 0xC7, 0, 13),
        (InterruptMode::Mode1, 0xDD, 0x0038, 13),
        (InterruptMode::Mode2, 0x24, 0x5678, 19),
    ] {
        for halted in [false, true] {
            let initial = Latches {
                halted,
                ..latches(mode)
            };
            let mut cpu = Z80::new(registers(), initial);
            let mut bus = RecordingBus::new();
            bus.nmi = true;
            bus.int = true;
            bus.load(0x2345, &[0xED, 0]);
            bus.load(0x0066, &[0xED, 0x45]);
            bus.load(0xF824, &[0x78, 0x56]);
            bus.acknowledges.push_back(opcode);
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
                    wz: 0x0066,
                    iff1: false,
                    nmi_asserted: true,
                    ..latches(mode)
                }
            );
            assert_eq!(bus.acknowledges, [opcode]);
            assert_eq!(
                bus.take_log(),
                [
                    Event::StopPoll(false),
                    Event::NmiPoll(true),
                    Event::IntPoll(true),
                    read(0x2345, OP, 0xED),
                    write(0x80FF, 0x23),
                    write(0x80FE, 0x45),
                ]
            );
            // Held NMI is no new edge; IFF1=0 blocks INT during RETN.
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
                    wz: 0x2345,
                    nmi_asserted: true,
                    ..latches(mode)
                }
            );
            assert_eq!(
                bus.take_log(),
                [
                    Event::StopPoll(false),
                    Event::NmiPoll(true),
                    Event::IntPoll(true),
                    read(0x0066, OP, 0xED),
                    read(0x0067, OP, 0x45),
                    read(0x80FE, STACK, 0x45),
                    read(0x80FF, STACK, 0x23),
                ]
            );
            assert_eq!(cpu.step(&mut bus), report(1, states, Exit::Budget));
            assert_eq!(
                *cpu.registers(),
                Registers {
                    pc: target,
                    sp: 0x80FE,
                    r: 0x83,
                    ..registers()
                }
            );
            assert_eq!(
                *cpu.latches(),
                Latches {
                    wz: target,
                    iff1: false,
                    iff2: false,
                    nmi_asserted: true,
                    ..latches(mode)
                }
            );
            assert_eq!(bus.memory, memory);
            let mut trace = vec![
                Event::StopPoll(false),
                Event::NmiPoll(true),
                Event::IntPoll(true),
                read(0x2345, ACK, opcode),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ];
            if mode == InterruptMode::Mode2 {
                trace.extend([read(0xF824, DATA, 0x78), read(0xF825, DATA, 0x56)]);
            }
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn ld_a_i_then_int_clears_pv_but_an_intervening_nop_expires_the_note() {
    // UM LD A,I p. 94/Table 1 p. 18; ALPM p. 48. Rule 10's window
    // is UNVERIFIED. NOP preserves P/V (UM p. 180); acceptance alone
    // without a pending note also preserves it, even while clearing IFF2.
    for intervening in [false, true] {
        let mut cpu = Z80::new(registers(), latches(InterruptMode::Mode1));
        let mut bus = RecordingBus::new();
        bus.load(0x2345, &[0xED, 0x57, 0, 0]);
        assert_eq!(cpu.step(&mut bus), report(1, 9, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                a: 0xF8,
                flags: Flags::from_byte(0xAD),
                pc: 0x2347,
                r: 0x81,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                parity_note_pending: true,
                ..latches(InterruptMode::Mode1)
            }
        );
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0x2345, OP, 0xED),
                read(0x2346, OP, 0x57),
            ]
        );
        let (boundary, saved_low, r, flags) = if intervening {
            assert_eq!(cpu.step(&mut bus), report(1, 4, Exit::Budget));
            assert_eq!(
                *cpu.registers(),
                Registers {
                    a: 0xF8,
                    flags: Flags::from_byte(0xAD),
                    pc: 0x2348,
                    r: 0x82,
                    ..registers()
                }
            );
            assert_eq!(*cpu.latches(), latches(InterruptMode::Mode1));
            assert_eq!(
                bus.take_log(),
                [
                    Event::StopPoll(false),
                    Event::NmiPoll(false),
                    Event::IntPoll(false),
                    read(0x2347, OP, 0),
                ]
            );
            (0x2348, 0x48, 0x83, 0xAD)
        } else {
            (0x2347, 0x47, 0x82, 0xA9)
        };
        bus.int = true;
        bus.stop = true;
        let before = cpu.clone();
        assert_eq!(cpu.step(&mut bus), report(0, 0, Exit::Stop));
        assert_eq!(cpu, before);
        assert_eq!(bus.take_log(), polls(true, false, true));
        bus.stop = false;
        bus.acknowledges.push_back(0xFB); // Ignored, never executes EI.
        let mut memory = bus.memory.clone();
        memory[0x80FF] = 0x23;
        memory[0x80FE] = saved_low;
        assert_eq!(cpu.step(&mut bus), report(1, 13, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                a: 0xF8,
                flags: Flags::from_byte(flags),
                pc: 0x0038,
                sp: 0x80FE,
                r,
                ..registers()
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: 0x0038,
                iff1: false,
                iff2: false,
                ..latches(InterruptMode::Mode1)
            }
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(true),
                read(boundary, ACK, 0xFB),
                write(0x80FF, 0x23),
                write(0x80FE, saved_low),
            ]
        );
    }
}

#[test]
fn each_mode_consumes_pending_parity_only_and_clears_both_iff_polarities() {
    // UM p. 18/p. 94; TM p. 55; ALPM p. 48; decision-15 rules 9–10.
    for (mode, opcode, target, states) in [
        (InterruptMode::Mode0, 0xEF, 0x0028, 13),
        (InterruptMode::Mode1, 0x00, 0x0038, 13),
        (InterruptMode::Mode2, 0x24, 0x5678, 19),
    ] {
        for iff2 in [false, true] {
            for (pending, flags, after) in [(false, 0xD7, 0xD7), (true, 0xD7, 0xD3), (true, 0, 0)] {
                let initial = Registers {
                    flags: Flags::from_byte(flags),
                    ..registers()
                };
                let mut cpu = Z80::new(
                    initial,
                    Latches {
                        iff2,
                        parity_note_pending: pending,
                        ..latches(mode)
                    },
                );
                let mut bus = RecordingBus::new();
                bus.int = true;
                bus.acknowledges.push_back(opcode);
                bus.load(0xF824, &[0x78, 0x56]);
                assert_eq!(cpu.step(&mut bus), report(1, states, Exit::Budget));
                assert_eq!(
                    *cpu.registers(),
                    Registers {
                        flags: Flags::from_byte(after),
                        pc: target,
                        sp: 0x80FE,
                        r: 0x80,
                        ..initial
                    }
                );
                assert_eq!(
                    *cpu.latches(),
                    Latches {
                        wz: target,
                        iff1: false,
                        iff2: false,
                        ..latches(mode)
                    }
                );
                let mut trace = vec![
                    Event::StopPoll(false),
                    Event::NmiPoll(false),
                    Event::IntPoll(true),
                    read(0x2345, ACK, opcode),
                    write(0x80FF, 0x23),
                    write(0x80FE, 0x45),
                ];
                if mode == InterruptMode::Mode2 {
                    trace.extend([read(0xF824, DATA, 0x78), read(0xF825, DATA, 0x56)]);
                }
                assert_eq!(bus.take_log(), trace);
            }
        }
    }
}

#[test]
fn reti_after_im2_service_restores_pc_without_reenabling_interrupts() {
    // Decision-20 §1: L RETI copies IFF2 (Young v0.91 §5.3 pp. 20–21);
    // IM2 service cleared both, so this expectation remains disabled.
    // UM p. 20/RETI p. 288; TM pp. 57/53; MK Table 2-1 p. 2-15
    // explicitly leaves both IFFs unchanged (decision-15 rule 7).
    let mut cpu = Z80::new(registers(), latches(InterruptMode::Mode2));
    let mut bus = RecordingBus::new();
    bus.int = true;
    bus.acknowledges.push_back(0x24);
    bus.load(0xF824, &[0x78, 0x56]);
    bus.load(0x5678, &[0xED, 0x4D]);
    let mut memory = bus.memory.clone();
    memory[0x80FF] = 0x23;
    memory[0x80FE] = 0x45;
    assert_eq!(cpu.step(&mut bus), report(1, 19, Exit::Budget));
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x5678,
            sp: 0x80FE,
            r: 0x80,
            ..registers()
        }
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
            wz: 0x2345,
            iff1: false,
            iff2: false,
            ..latches(InterruptMode::Mode2)
        }
    );
    assert_eq!(bus.memory, memory);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2345, ACK, 0x24),
            write(0x80FF, 0x23),
            write(0x80FE, 0x45),
            read(0xF824, DATA, 0x78),
            read(0xF825, DATA, 0x56),
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x5678, OP, 0xED),
            read(0x5679, OP, 0x4D),
            read(0x80FE, STACK, 0x45),
            read(0x80FF, STACK, 0x23),
        ]
    );
}

#[test]
fn acknowledge_counts_one_refresh_increment_preserving_bit_seven() {
    // UM Registers p. 3/TM §2 p. 4: seven counter bits. One special
    // M1 is DERIVED from UM Fig. 9 p. 13/TM §4 p. 16, rule 6.
    for mode in MODES {
        for (r, next) in [
            (0, 1),
            (0x7E, 0x7F),
            (0x7F, 0),
            (0x80, 0x81),
            (0xFE, 0xFF),
            (0xFF, 0x80),
        ] {
            let initial = Registers { r, ..registers() };
            let mut cpu = Z80::new(initial, latches(mode));
            let mut bus = RecordingBus::new();
            bus.int = true;
            // C7 is an RST for IM 0, ignored for IM 1, odd for IM 2:
            // choose FE there, with the vector near the page's end.
            let (opcode, target, states) = match mode {
                InterruptMode::Mode0 => (0xC7, 0, 13),
                InterruptMode::Mode1 => (0xC7, 0x0038, 13),
                InterruptMode::Mode2 => (0xFE, 0x5678, 19),
            };
            bus.acknowledges.push_back(opcode);
            bus.load(0xF8FE, &[0x78, 0x56]);
            assert_eq!(cpu.step(&mut bus), report(1, states, Exit::Budget));
            assert_eq!(
                *cpu.registers(),
                Registers {
                    pc: target,
                    sp: 0x80FE,
                    r: next,
                    ..initial
                }
            );
            assert_eq!(
                *cpu.latches(),
                Latches {
                    wz: target,
                    iff1: false,
                    iff2: false,
                    ..latches(mode)
                }
            );
            let mut trace = vec![
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(true),
                read(0x2345, ACK, opcode),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ];
            if mode == InterruptMode::Mode2 {
                trace.extend([read(0xF8FE, DATA, 0x78), read(0xF8FF, DATA, 0x56)]);
            }
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn stack_wrap_is_derived_and_boundary_pc_ffff_does_not_advance() {
    // DERIVED wrap from 16-bit SP (UM p. 2; TM §2.1 p. 3), not an
    // explicit hardware wrap statement. Push: UM p. 292/TM p. 53.
    for (sp, high_at, low_at) in [(0, 0xFFFF, 0xFFFE), (1, 0, 0xFFFF)] {
        let initial = Registers {
            pc: 0xFFFF,
            sp,
            ..registers()
        };
        let mut cpu = Z80::new(initial, latches(InterruptMode::Mode0));
        let mut bus = RecordingBus::new();
        bus.int = true;
        bus.acknowledges.push_back(0xCF);
        let mut memory = bus.memory.clone();
        memory[usize::from(high_at)] = 0xFF;
        memory[usize::from(low_at)] = 0xFF;
        assert_eq!(cpu.step(&mut bus), report(1, 13, Exit::Budget));
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 8,
                sp: low_at,
                r: 0x80,
                ..initial
            }
        );
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: 8,
                iff1: false,
                iff2: false,
                ..latches(InterruptMode::Mode0)
            }
        );
        assert_eq!(bus.memory, memory);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(true),
                read(0xFFFF, ACK, 0xCF),
                write(high_at, 0xFF),
                write(low_at, 0xFF),
            ]
        );
    }
}

#[test]
fn cycle_budget_overshoots_by_one_whole_acceptance_without_handler_fetch() {
    // Rule 3: complete one step even if its cost exceeds the cycle limit.
    // 13 DERIVED / 19 STATED on the source pages cited at file scope.
    for (mode, opcode, target, states) in [
        (InterruptMode::Mode0, 0xFF, 0x0038, 13),
        (InterruptMode::Mode1, 0xED, 0x0038, 13),
        (InterruptMode::Mode2, 0x24, 0x5678, 19),
    ] {
        for cycles in [1, states - 1, states] {
            let mut cpu = Z80::new(registers(), latches(mode));
            let mut bus = RecordingBus::new();
            bus.int = true;
            bus.acknowledges.push_back(opcode);
            bus.load(0xF824, &[0x78, 0x56]);
            assert_eq!(
                cpu.run(&mut bus, Budget { steps: 10, cycles }),
                report(1, states, Exit::Budget)
            );
            assert_eq!(
                *cpu.registers(),
                Registers {
                    pc: target,
                    sp: 0x80FE,
                    r: 0x80,
                    ..registers()
                }
            );
            assert_eq!(
                *cpu.latches(),
                Latches {
                    wz: target,
                    iff1: false,
                    iff2: false,
                    ..latches(mode)
                }
            );
            let mut trace = vec![
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(true),
                read(0x2345, ACK, opcode),
                write(0x80FF, 0x23),
                write(0x80FE, 0x45),
            ];
            if mode == InterruptMode::Mode2 {
                trace.extend([read(0xF824, DATA, 0x78), read(0xF825, DATA, 0x56)]);
            }
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn budget_splits_before_and_after_acceptance_preserve_state_and_literal_trace() {
    // Rules 1–3: no polling at a reached budget; retirement separates the
    // preceding NOP, acceptance, and first handler NOP. UM NOP p. 180.
    for (mode, opcode, target, states, total) in [
        (InterruptMode::Mode0, 0xEF, 0x0028, 13, 21),
        (InterruptMode::Mode1, 0xDD, 0x0038, 13, 21),
        (InterruptMode::Mode2, 0x24, 0x5678, 19, 27),
    ] {
        let mut whole = Z80::new(registers(), latches(mode));
        let mut whole_bus = RecordingBus::new();
        whole_bus.int_levels.extend([false, true, true]);
        whole_bus.acknowledges.push_back(opcode);
        whole_bus.load(0xF824, &[0x78, 0x56]);
        assert_eq!(
            whole.run(
                &mut whole_bus,
                Budget {
                    steps: 3,
                    cycles: 100
                }
            ),
            report(3, total, Exit::Budget)
        );
        let mut literal = vec![
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            read(0x2345, OP, 0),
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(0x2346, ACK, opcode),
            write(0x80FF, 0x23),
            write(0x80FE, 0x46),
        ];
        if mode == InterruptMode::Mode2 {
            literal.extend([read(0xF824, DATA, 0x78), read(0xF825, DATA, 0x56)]);
        }
        literal.extend([
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            read(target, OP, 0),
        ]);
        assert_eq!(whole_bus.log, literal);
        let mut split = Z80::new(registers(), latches(mode));
        let mut split_bus = RecordingBus::new();
        split_bus.int_levels.extend([false, true, true]);
        split_bus.acknowledges.push_back(opcode);
        split_bus.load(0xF824, &[0x78, 0x56]);
        let mut progress = (0, 0);
        for cycles in [4, states, 4] {
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
                let levels = split_bus.int_levels.clone();
                let acknowledges = split_bus.acknowledges.clone();
                assert_eq!(
                    split.run(&mut split_bus, budget),
                    report(0, 0, Exit::Budget)
                );
                assert_eq!(split, before);
                assert_eq!(split_bus.log, log);
                assert_eq!(split_bus.int_levels, levels);
                assert_eq!(split_bus.acknowledges, acknowledges);
            }
            let part = split.run(&mut split_bus, Budget { steps: 100, cycles });
            assert_eq!(part, report(1, cycles, Exit::Budget));
            progress.0 += part.steps;
            progress.1 += part.cycles;
        }
        assert_eq!(progress, (3, total));
        assert_eq!(
            *split.registers(),
            Registers {
                pc: target + 1,
                sp: 0x80FE,
                r: 0x82,
                ..registers()
            }
        );
        assert_eq!(
            *split.latches(),
            Latches {
                wz: target,
                iff1: false,
                iff2: false,
                ..latches(mode)
            }
        );
        assert_eq!(split, whole);
        assert_eq!(split_bus.memory, whole_bus.memory);
        assert_eq!(split_bus.log, literal);
    }
}
