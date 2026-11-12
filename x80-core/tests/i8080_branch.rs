//! Branch witnesses for plan §2.2/2.3 and decision-5 items 3 and 8.
//!
//! Intel 98-153B (September 1975), Ch. 4 Branch Group pp. 4-11/4-12:
//! no branch instruction changes flags. Ch. 2 machine-cycle table pp. 2-18/
//! 2-19 and notes 6, 11-17 p. 2-20 fix access order and kinds. Intel
//! 9800301-04 (May 1981), Ch. 3 entries provide the second witness below.
//! All cited values were checked against page images. Expectations use
//! literals or a test-side transcription, never the core's opcode/flag helpers.

mod support;

use support::{sample_registers, Event, RecordingBus};
use x80_core::i8080::{Flags, Latches, Registers, I8080};
use x80_core::{Exit, MemoryAccess, Report};

const LATCHES: Latches = Latches {
    interrupt_enable: true,
    ei_delay: false,
    halted: false,
};

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
        kind: MemoryAccess::Stack,
        value,
    }
}

fn fetch(pc: u16, opcode: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::InterruptPoll(false),
        read(pc, MemoryAccess::OpcodeFetch, opcode),
    ]
}

// Enumerate the five independent flag inputs, without packing/unpacking a PSW
// through crate code. The bit positions here are test-loop indices only.
fn flags(bits: u8) -> Flags {
    Flags {
        sign: bits & 16 != 0,
        zero: bits & 8 != 0,
        auxiliary_carry: bits & 4 != 0,
        parity: bits & 2 != 0,
        carry: bits & 1 != 0,
    }
}

// (Jcc, Ccc, Rcc, selected test-loop bit, required value). Independent
// transcription of 98-153B Ch. 4 p. 4-11 condition table and p. 4-12 encodings.
// Second witness: 9800301-04 Ch. 3 Jcc pp. 3-26..3-32, Ccc pp. 3-10/3-11,
// 3-14..3-18, Rcc pp. 3-47, 3-50..3-53, 3-55/3-56.
const CONDITIONS: [(u8, u8, u8, u8, bool); 8] = [
    (0xC2, 0xC4, 0xC0, 8, false),  // NZ
    (0xCA, 0xCC, 0xC8, 8, true),   // Z
    (0xD2, 0xD4, 0xD0, 1, false),  // NC
    (0xDA, 0xDC, 0xD8, 1, true),   // C
    (0xE2, 0xE4, 0xE0, 2, false),  // PO
    (0xEA, 0xEC, 0xE8, 2, true),   // PE
    (0xF2, 0xF4, 0xF0, 16, false), // P
    (0xFA, 0xFC, 0xF8, 16, true),  // M
];

// Check the whole register file (including all flags), latches, report, memory
// and complete bus trace. Only the explicitly expected writes may alter memory.
fn check(
    mut bus: RecordingBus,
    initial: Registers,
    expected: Registers,
    states: u64,
    trace: Vec<Event>,
) {
    let mut memory = bus.memory.clone();
    for event in &trace {
        if let Event::Write { address, value, .. } = *event {
            memory[usize::from(address)] = value;
        }
    }
    let mut cpu = I8080::new(initial, LATCHES);
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: states,
            exit: Exit::Budget
        },
        "initial {initial:?}"
    );
    assert_eq!(*cpu.registers(), expected, "initial {initial:?}");
    assert_eq!(*cpu.latches(), LATCHES);
    assert_eq!(bus.log, trace, "initial {initial:?}");
    assert!(bus.memory == memory, "unexpected memory change");
}

fn address_reads(trace: &mut Vec<Event>) {
    trace.extend([
        read(0x12FF, MemoryAccess::OperandFetch, 0x67),
        read(0x1300, MemoryAccess::OperandFetch, 0x45),
    ]);
}

#[test]
fn jmp_reads_low_then_high_and_preserves_all_flags() {
    // 98-153B Ch. 4 JMP p. 4-11: PC = byte3:byte2, 10 states, flags none;
    // Ch. 2 pp. 2-18/2-19: M2 B2 -> Z, M3 B3 -> W (memory reads, note 6
    // p. 2-20). 9800301-04 Ch. 3 JMP p. 3-28 agrees.
    for bits in 0..32 {
        let initial = Registers {
            flags: flags(bits),
            ..sample_registers(0x12FE)
        };
        let mut bus = RecordingBus::new();
        bus.load(0x12FE, &[0xC3, 0x67, 0x45]);
        let mut trace = fetch(0x12FE, 0xC3);
        address_reads(&mut trace);
        check(
            bus,
            initial,
            Registers {
                pc: 0x4567,
                ..initial
            },
            10,
            trace,
        );
    }
}

#[test]
fn every_jcc_reads_both_operands_taken_and_untaken_in_ten_states() {
    // 98-153B Ch. 4 Jcondition p. 4-11: 10 states on both paths. Ch. 2
    // pp. 2-18/2-19: M2 and M3 both read the operand stream; note 12 p. 2-20
    // only changes the following fetch address. No operand cycle is skipped.
    // 9800301-04 Ch. 3 Jcc pp. 3-26..3-32 agrees (8080 column).
    for (opcode, _, _, mask, required) in CONDITIONS {
        for bits in 0..32 {
            let taken = (bits & mask != 0) == required;
            let initial = Registers {
                flags: flags(bits),
                ..sample_registers(0x12FE)
            };
            let mut bus = RecordingBus::new();
            bus.load(0x12FE, &[opcode, 0x67, 0x45]);
            let mut trace = fetch(0x12FE, opcode);
            address_reads(&mut trace);
            check(
                bus,
                initial,
                Registers {
                    pc: if taken { 0x4567 } else { 0x1301 },
                    ..initial
                },
                10,
                trace,
            );
        }
    }
}

#[test]
fn call_reads_operands_then_pushes_next_pc_high_then_low() {
    // 98-153B Ch. 4 CALL p. 4-11: 17 states, next PC high -> SP-1, low ->
    // SP-2, SP -= 2. Ch. 2 pp. 2-18/2-19 M2/M3 operands, M4/M5 stack
    // writes (note 16 p. 2-20). 9800301-04 Ch. 3 CALL pp. 3-8/3-9 agrees.
    for bits in 0..32 {
        let initial = Registers {
            flags: flags(bits),
            ..sample_registers(0x12FE)
        };
        let mut bus = RecordingBus::new();
        bus.load(0x12FE, &[0xCD, 0x67, 0x45]);
        let mut trace = fetch(0x12FE, 0xCD);
        address_reads(&mut trace);
        trace.extend([write(0x8F8F, 0x13), write(0x8F8E, 0x01)]);
        check(
            bus,
            initial,
            Registers {
                pc: 0x4567,
                sp: 0x8F8E,
                ..initial
            },
            17,
            trace,
        );
    }
}

#[test]
fn every_ccc_reads_operands_on_both_paths_and_only_pushes_when_taken() {
    // 98-153B Ch. 4 Ccondition p. 4-12: 11/17 states, CALL if true, otherwise
    // sequential. Ch. 2 pp. 2-18/2-19 M2/M3 still read operands; note 13
    // p. 2-20 skips ONLY M4/M5 when false. 9800301-04 Ch. 3 Ccc pp. 3-10/
    // 3-11, 3-14..3-18 agrees: three or five cycles, eleven or seventeen states.
    for (_, opcode, _, mask, required) in CONDITIONS {
        for bits in 0..32 {
            let taken = (bits & mask != 0) == required;
            let initial = Registers {
                flags: flags(bits),
                ..sample_registers(0x12FE)
            };
            let mut bus = RecordingBus::new();
            bus.load(0x12FE, &[opcode, 0x67, 0x45]);
            let mut trace = fetch(0x12FE, opcode);
            address_reads(&mut trace);
            let (pc, sp, states) = if taken {
                trace.extend([write(0x8F8F, 0x13), write(0x8F8E, 0x01)]);
                (0x4567, 0x8F8E, 17)
            } else {
                (0x1301, 0x8F90, 11)
            };
            check(bus, initial, Registers { pc, sp, ..initial }, states, trace);
        }
    }
}

#[test]
fn ret_reads_stack_low_then_high_and_preserves_all_flags() {
    // 98-153B Ch. 4 RET p. 4-12: 10 states, PCL <- (SP), PCH <- (SP+1),
    // SP += 2; Ch. 2 pp. 2-18/2-19 M2/M3 stack reads (note 15 p. 2-20).
    // 9800301-04 Ch. 3 RET p. 3-48 agrees.
    for bits in 0..32 {
        let initial = Registers {
            flags: flags(bits),
            ..sample_registers(0x12FE)
        };
        let mut bus = RecordingBus::new();
        bus.load(0x12FE, &[0xC9]);
        bus.load(0x8F90, &[0x67, 0x45]);
        let mut trace = fetch(0x12FE, 0xC9);
        trace.extend([
            read(0x8F90, MemoryAccess::Stack, 0x67),
            read(0x8F91, MemoryAccess::Stack, 0x45),
        ]);
        check(
            bus,
            initial,
            Registers {
                pc: 0x4567,
                sp: 0x8F92,
                ..initial
            },
            10,
            trace,
        );
    }
}

#[test]
fn every_rcc_only_reads_stack_when_taken_in_five_or_eleven_states() {
    // 98-153B Ch. 4 Rcondition p. 4-12: 5/11 states. Ch. 2 pp. 2-18/
    // 2-19 and note 14 p. 2-20: false skips both M2/M3 stack reads.
    // 9800301-04 Ch. 3 Rcc pp. 3-47, 3-50..3-53, 3-55/3-56 agrees.
    for (_, _, opcode, mask, required) in CONDITIONS {
        for bits in 0..32 {
            let taken = (bits & mask != 0) == required;
            let initial = Registers {
                flags: flags(bits),
                ..sample_registers(0x12FE)
            };
            let mut bus = RecordingBus::new();
            bus.load(0x12FE, &[opcode, 0xA5, 0x3C]);
            bus.load(0x8F90, &[0x67, 0x45]);
            let mut trace = fetch(0x12FE, opcode);
            let (pc, sp, states) = if taken {
                trace.extend([
                    read(0x8F90, MemoryAccess::Stack, 0x67),
                    read(0x8F91, MemoryAccess::Stack, 0x45),
                ]);
                (0x4567, 0x8F92, 11)
            } else {
                (0x12FF, 0x8F90, 5)
            };
            check(bus, initial, Registers { pc, sp, ..initial }, states, trace);
        }
    }
}

#[test]
fn every_rst_pushes_next_pc_and_selects_its_literal_vector() {
    // 98-153B Ch. 4 RST p. 4-12: 11 states, PC <- 8*n, push next PC;
    // Ch. 2 pp. 2-18/2-19 M2 high then M3 low, stack writes (note 16
    // p. 2-20). 9800301-04 Ch. 3 RST pp. 3-54/3-55 agrees.
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
        for bits in 0..32 {
            let initial = Registers {
                flags: flags(bits),
                ..sample_registers(0x12FF)
            };
            let mut bus = RecordingBus::new();
            bus.load(0x12FF, &[opcode, 0xA5]);
            let mut trace = fetch(0x12FF, opcode);
            trace.extend([write(0x8F8F, 0x13), write(0x8F8E, 0x00)]);
            check(
                bus,
                initial,
                Registers {
                    pc: vector,
                    sp: 0x8F8E,
                    ..initial
                },
                11,
                trace,
            );
        }
    }
}

#[test]
fn pchl_copies_h_and_l_without_reading_memory_at_hl() {
    // 98-153B Ch. 4 PCHL p. 4-12: 5 states, PCH <- H, PCL <- L, flags none;
    // Ch. 2 p. 2-18: only M1, internal HL -> PC. 9800301-04 Ch. 3 p. 3-42.
    for bits in 0..32 {
        for (h, l, pc) in [(0x45, 0x67, 0x4567), (0, 0, 0), (0xFF, 0xFF, 0xFFFF)] {
            let initial = Registers {
                flags: flags(bits),
                h,
                l,
                ..sample_registers(0x12FE)
            };
            let mut bus = RecordingBus::new();
            bus.load(0x12FE, &[0xE9, 0xA5]);
            check(
                bus,
                initial,
                Registers { pc, ..initial },
                5,
                fetch(0x12FE, 0xE9),
            );
        }
    }
}

#[test]
fn call_and_rst_wrap_stack_decrements_at_zero() {
    // 98-153B Ch. 4 pp. 4-11/4-12 push order and totals, Ch. 2 pp. 2-18/
    // 2-19 stack cycles. Arithmetic modulo 65536 follows the 16-bit SP
    // (98-153B Ch. 2 p. 2-2); no page explicitly describes this push wrap.
    for (sp, high_at, low_at, final_sp) in [
        (0x0000, 0xFFFF, 0xFFFE, 0xFFFE),
        (0x0001, 0x0000, 0xFFFF, 0xFFFF),
        (0xFFFF, 0xFFFE, 0xFFFD, 0xFFFD),
    ] {
        for (opcode, pc, low, states) in [(0xCD, 0x4567, 0x01, 17), (0xFF, 0x0038, 0xFF, 11)] {
            let initial = Registers {
                sp,
                ..sample_registers(0x12FE)
            };
            let mut bus = RecordingBus::new();
            bus.load(0x12FE, &[opcode, 0x67, 0x45]);
            let mut trace = fetch(0x12FE, opcode);
            if opcode == 0xCD {
                address_reads(&mut trace);
            }
            trace.extend([
                write(high_at, if opcode == 0xCD { 0x13 } else { 0x12 }),
                write(low_at, low),
            ]);
            check(
                bus,
                initial,
                Registers {
                    pc,
                    sp: final_sp,
                    ..initial
                },
                states,
                trace,
            );
        }
    }
}

#[test]
fn ret_wraps_stack_reads_and_increment_at_ffff() {
    // 98-153B Ch. 4 RET p. 4-12, Ch. 2 pp. 2-18/2-19 stack reads.
    // 16-bit wrap derived from SP width, Ch. 2 p. 2-2; independently,
    // 9800301-04 Ch. 3 INX p. 3-26 explicitly witnesses FFFF -> 0000 for SP.
    for (sp, high_at, final_sp) in [
        (0xFFFE, 0xFFFF, 0x0000),
        (0xFFFF, 0x0000, 0x0001),
        (0x0000, 0x0001, 0x0002),
    ] {
        let initial = Registers {
            sp,
            ..sample_registers(0x12FE)
        };
        let mut bus = RecordingBus::new();
        bus.load(0x12FE, &[0xC9]);
        bus.load(sp, &[0x67, 0x45]);
        let mut trace = fetch(0x12FE, 0xC9);
        trace.extend([
            read(sp, MemoryAccess::Stack, 0x67),
            read(high_at, MemoryAccess::Stack, 0x45),
        ]);
        check(
            bus,
            initial,
            Registers {
                pc: 0x4567,
                sp: final_sp,
                ..initial
            },
            10,
            trace,
        );
    }
}

#[test]
fn operand_fetches_and_return_addresses_wrap_pc_at_ffff() {
    // 98-153B Ch. 2 p. 2-2: 16-bit PC, incremented on fetch; Ch. 2
    // pp. 2-18/2-19 stream reads and stack writes; Ch. 4 pp. 4-11/4-12
    // next-instruction return address. Wrap is derived from the register width.
    for (pc, low_at, high_at, next, return_low) in [
        (0xFFFD, 0xFFFE, 0xFFFF, 0x0000, 0x00),
        (0xFFFE, 0xFFFF, 0x0000, 0x0001, 0x01),
        (0xFFFF, 0x0000, 0x0001, 0x0002, 0x02),
    ] {
        for (opcode, target, states, pushes) in [
            (0xC3, 0x4567, 10, false), // JMP
            (0xCA, next, 10, false),   // JZ false
            (0xCD, 0x4567, 17, true),  // CALL
            (0xCC, next, 11, false),   // CZ false
        ] {
            let initial = sample_registers(pc); // Z is false.
            let mut bus = RecordingBus::new();
            bus.load(pc, &[opcode, 0x67, 0x45]);
            let mut trace = fetch(pc, opcode);
            trace.extend([
                read(low_at, MemoryAccess::OperandFetch, 0x67),
                read(high_at, MemoryAccess::OperandFetch, 0x45),
            ]);
            if pushes {
                trace.extend([write(0x8F8F, 0x00), write(0x8F8E, return_low)]);
            }
            check(
                bus,
                initial,
                Registers {
                    pc: target,
                    sp: if pushes { 0x8F8E } else { 0x8F90 },
                    ..initial
                },
                states,
                trace,
            );
        }
    }
    let mut bus = RecordingBus::new();
    bus.load(0xFFFF, &[0xC7]);
    let initial = sample_registers(0xFFFF);
    let mut trace = fetch(0xFFFF, 0xC7);
    trace.extend([write(0x8F8F, 0x00), write(0x8F8E, 0x00)]);
    check(
        bus,
        initial,
        Registers {
            pc: 0x0000,
            sp: 0x8F8E,
            ..initial
        },
        11,
        trace,
    );

    let mut bus = RecordingBus::new();
    bus.load(0xFFFF, &[0xC8]); // RZ false: no stack read, PC wraps to zero.
    check(
        bus,
        initial,
        Registers {
            pc: 0x0000,
            ..initial
        },
        5,
        fetch(0xFFFF, 0xC8),
    );
}
