//! 8085 branch witnesses: plan §2.2/2.3; decisions 10, 11 and 12.
//!
//! Intel 205775-002 (January 1983), §5.6.4 pp. 5-13..5-15 (PDF 98..100):
//! conditions, operations, no flag changes and 8085 state totals. Table 5-1
//! p. 5-19 (PDF 104) gives the cycle sequences (Table 5-2 is the opcode list).
//! Ch. 2 §2.3.1 p. 2-7 (PDF 30): stream reads low then high; §2.2.3 p. 2-2
//! (PDF 25): descending stack; Fig. 2-18 p. 2-14 (PDF 37): high then low writes.
//! Access kinds and boundary polls are API choices (decision-5/12).
//! Intel 9800301-04 (May 1981), Ch. 3 supplies independent instruction entries
//! below; its parenthesized values are the 8085 totals. All cited values were
//! checked on page images and source hashes match PROVENANCE.md §1.
//! Expectations are literals or a separate test-side condition table.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Budget, Exit, MemoryAccess, Report};

fn flags(bits: u8) -> Flags {
    // Loop indices, not PSW positions; all five inputs vary independently.
    Flags {
        zero: bits & 1 != 0,
        carry: bits & 2 != 0,
        parity: bits & 4 != 0,
        sign: bits & 8 != 0,
        auxiliary_carry: bits & 16 != 0,
    }
}

fn registers(pc: u16, bits: u8) -> Registers {
    Registers {
        a: 0x95,
        b: 0x21,
        c: 0x43,
        d: 0x65,
        e: 0x87,
        h: 0xA9,
        l: 0xCB,
        flags: flags(bits),
        sp: 0x8100,
        pc,
    }
}

fn latches() -> Latches {
    Latches {
        interrupt_enable: true,
        mask5_5: false,
        rst7_5_pending: true,
        sod: true,
        trap_saved_interrupt_enable: true,
        trap_rim_pending: true,
        ..Latches::default()
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
        kind: MemoryAccess::Stack,
        value,
    }
}

fn fetch(pc: u16, opcode: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::TrapPoll(false),
        Event::Rst7_5Poll(false),
        Event::Rst6_5Poll(false),
        Event::Rst5_5Poll(false),
        Event::IntrPoll(false),
        read(pc, MemoryAccess::OpcodeFetch, opcode),
    ]
}

fn report(steps: u64, cycles: u64) -> Report {
    Report {
        steps,
        cycles,
        exit: Exit::Budget,
    }
}

fn operand_count(trace: &[Event]) -> usize {
    trace
        .iter()
        .filter(|event| {
            matches!(
                event,
                Event::Read {
                    kind: MemoryAccess::OperandFetch,
                    ..
                }
            )
        })
        .count()
}

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
    let mut cpu = I8085::new(initial, latches());
    assert_eq!(cpu.step(&mut bus), report(1, states), "initial {initial:?}");
    assert_eq!(*cpu.registers(), expected, "initial {initial:?}");
    assert_eq!(*cpu.latches(), latches());
    assert_eq!(operand_count(&bus.log), operand_count(&trace));
    assert_eq!(bus.log, trace, "initial {initial:?}");
    assert!(bus.memory == memory, "unexpected memory change");
}

// (Jcond, Ccond, Rcond, loop mask, required value): 205775-002 p. 5-13
// condition table, pp. 5-13/5-14 encodings. Second witness 9800301-04 Ch. 3:
// Jcond pp. 3-26..3-32; Ccond pp. 3-10/3-11, 3-14..3-18;
// Rcond pp. 3-47, 3-50..3-53, 3-55/3-56. These are 8085 readings.
const CONDITIONS: [(u8, u8, u8, u8, bool); 8] = [
    (0xC2, 0xC4, 0xC0, 1, false), // NZ
    (0xCA, 0xCC, 0xC8, 1, true),  // Z
    (0xD2, 0xD4, 0xD0, 2, false), // NC
    (0xDA, 0xDC, 0xD8, 2, true),  // C
    (0xE2, 0xE4, 0xE0, 4, false), // PO
    (0xEA, 0xEC, 0xE8, 4, true),  // PE
    (0xF2, 0xF4, 0xF0, 8, false), // P
    (0xFA, 0xFC, 0xF8, 8, true),  // M
];

#[test]
fn jmp_reads_low_then_high_in_ten_states() {
    // 205775-002 p. 5-13; Table 5-1: F R R. Ch. 2 p. 2-7 fixes byte order.
    // 9800301-04 JMP p. 3-28: 3 cycles, 10 states, flags none.
    for bits in 0..32 {
        let initial = registers(0x23FE, bits);
        let mut bus = RecordingBus::new();
        bus.load(0x23FE, &[0xC3, 0xB6, 0x4D]);
        let mut trace = fetch(0x23FE, 0xC3);
        trace.extend([
            read(0x23FF, MemoryAccess::OperandFetch, 0xB6),
            read(0x2400, MemoryAccess::OperandFetch, 0x4D),
        ]);
        check(
            bus,
            initial,
            Registers {
                pc: 0x4DB6,
                ..initial
            },
            10,
            trace,
        );
    }
}

#[test]
fn every_jcond_has_one_operand_fetch_untaken_and_two_taken() {
    // 205775-002 p. 5-13: 7/10 states. Table 5-1 p. 5-19: F R / F R R;
    // its dagger says the longer sequence always applies on the 8080A.
    // Ch. 2 p. 2-7 identifies the first stream read as byte 2 (low).
    // 9800301-04 pp. 3-26..3-32: 2/3 cycles, 7/10 states on the 8085.
    for (opcode, _, _, mask, required) in CONDITIONS {
        for bits in 0..32 {
            let taken = (bits & mask != 0) == required;
            let initial = registers(0x23FE, bits);
            let mut bus = RecordingBus::new();
            bus.load(0x23FE, &[opcode, 0xB6, 0x4D]);
            let mut trace = fetch(0x23FE, opcode);
            trace.push(read(0x23FF, MemoryAccess::OperandFetch, 0xB6));
            if taken {
                trace.push(read(0x2400, MemoryAccess::OperandFetch, 0x4D));
            }
            assert_eq!(operand_count(&trace), if taken { 2 } else { 1 });
            check(
                bus,
                initial,
                Registers {
                    pc: if taken { 0x4DB6 } else { 0x2401 },
                    ..initial
                },
                if taken { 10 } else { 7 },
                trace,
            );
        }
    }
}

#[test]
fn call_reads_operands_then_pushes_next_pc_high_then_low() {
    // 205775-002 p. 5-13; Table 5-1: S R R W W; Fig. 2-18 p. 2-14:
    // M4 writes PCH at SP-1, M5 PCL at SP-2. 9800301-04 pp. 3-8/3-9:
    // next sequential PC saved, 18 states on the 8085, flags none.
    for bits in 0..32 {
        let initial = registers(0x23FE, bits);
        let mut bus = RecordingBus::new();
        bus.load(0x23FE, &[0xCD, 0xB6, 0x4D]);
        let mut trace = fetch(0x23FE, 0xCD);
        trace.extend([
            read(0x23FF, MemoryAccess::OperandFetch, 0xB6),
            read(0x2400, MemoryAccess::OperandFetch, 0x4D),
            write(0x80FF, 0x24),
            write(0x80FE, 0x01),
        ]);
        check(
            bus,
            initial,
            Registers {
                pc: 0x4DB6,
                sp: 0x80FE,
                ..initial
            },
            18,
            trace,
        );
    }
}

#[test]
fn every_ccond_has_one_operand_fetch_untaken_and_pushes_only_taken() {
    // 205775-002 p. 5-14: 9/18 states; Table 5-1: S R / S R R W W.
    // Bullet footnote adds an 8080A read only; Ch. 2 p. 2-7: byte 2 first.
    // 9800301-04 pp. 3-10/3-11, 3-14..3-18 agrees, including CPE p. 3-16.
    // Its App. A p. A-1 CPE 9/17 is the already-recorded misprint; use 9/18.
    for (_, opcode, _, mask, required) in CONDITIONS {
        for bits in 0..32 {
            let taken = (bits & mask != 0) == required;
            let initial = registers(0x23FE, bits);
            let mut bus = RecordingBus::new();
            bus.load(0x23FE, &[opcode, 0xB6, 0x4D]);
            let mut trace = fetch(0x23FE, opcode);
            trace.push(read(0x23FF, MemoryAccess::OperandFetch, 0xB6));
            let (pc, sp, states) = if taken {
                trace.extend([
                    read(0x2400, MemoryAccess::OperandFetch, 0x4D),
                    write(0x80FF, 0x24),
                    write(0x80FE, 0x01),
                ]);
                (0x4DB6, 0x80FE, 18)
            } else {
                (0x2401, 0x8100, 9)
            };
            assert_eq!(operand_count(&trace), if taken { 2 } else { 1 });
            check(bus, initial, Registers { pc, sp, ..initial }, states, trace);
        }
    }
}

#[test]
fn ret_reads_stack_low_then_high_in_ten_states() {
    // 205775-002 p. 5-14: low (SP), high (SP+1), SP+2; Table 5-1: F R R.
    // Ch. 2 §2.2.3 p. 2-2: pop increments. 9800301-04 RET p. 3-48: 10 states.
    for bits in 0..32 {
        let initial = registers(0x23FE, bits);
        let mut bus = RecordingBus::new();
        bus.load(0x23FE, &[0xC9, 0xA5]);
        bus.load(0x8100, &[0xB6, 0x4D]);
        let mut trace = fetch(0x23FE, 0xC9);
        trace.extend([
            read(0x8100, MemoryAccess::Stack, 0xB6),
            read(0x8101, MemoryAccess::Stack, 0x4D),
        ]);
        check(
            bus,
            initial,
            Registers {
                pc: 0x4DB6,
                sp: 0x8102,
                ..initial
            },
            10,
            trace,
        );
    }
}

#[test]
fn every_rcond_reads_stack_only_taken_in_six_or_twelve_states() {
    // 205775-002 p. 5-14: 6/12 states; Table 5-1: S / S R R.
    // 9800301-04 pp. 3-47, 3-50..3-53, 3-55/3-56: 6/12 on the 8085.
    for (_, _, opcode, mask, required) in CONDITIONS {
        for bits in 0..32 {
            let taken = (bits & mask != 0) == required;
            let initial = registers(0x23FE, bits);
            let mut bus = RecordingBus::new();
            bus.load(0x23FE, &[opcode, 0xA5]);
            bus.load(0x8100, &[0xB6, 0x4D]);
            let mut trace = fetch(0x23FE, opcode);
            let (pc, sp, states) = if taken {
                trace.extend([
                    read(0x8100, MemoryAccess::Stack, 0xB6),
                    read(0x8101, MemoryAccess::Stack, 0x4D),
                ]);
                (0x4DB6, 0x8102, 12)
            } else {
                (0x23FF, 0x8100, 6)
            };
            check(bus, initial, Registers { pc, sp, ..initial }, states, trace);
        }
    }
}

#[test]
fn every_rst_pushes_next_pc_and_loads_its_literal_vector() {
    // 205775-002 p. 5-14: next PC saved, PC=8*n, 12 states; Table 5-1: S W W.
    // Stack write order: Fig. 2-18 p. 2-14; 9800301-04 RST pp. 3-54/3-55.
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
            let initial = registers(0x23FF, bits);
            let mut bus = RecordingBus::new();
            bus.load(0x23FF, &[opcode, 0xA5]);
            let mut trace = fetch(0x23FF, opcode);
            trace.extend([write(0x80FF, 0x24), write(0x80FE, 0x00)]);
            check(
                bus,
                initial,
                Registers {
                    pc: vector,
                    sp: 0x80FE,
                    ..initial
                },
                12,
                trace,
            );
        }
    }
}

#[test]
fn pchl_copies_h_and_l_in_six_states_without_a_data_read() {
    // 205775-002 p. 5-15: PCH=H, PCL=L; Table 5-1: S only.
    // 9800301-04 PCHL p. 3-42: one cycle, 6 states on the 8085, flags none.
    for bits in 0..32 {
        for (h, l, pc) in [(0x9C, 0x37, 0x9C37), (0, 0, 0), (0xFF, 0xFF, 0xFFFF)] {
            let initial = Registers {
                h,
                l,
                ..registers(0x23FE, bits)
            };
            let mut bus = RecordingBus::new();
            bus.load(0x23FE, &[0xE9, 0xA5]);
            check(
                bus,
                initial,
                Registers { pc, ..initial },
                6,
                fetch(0x23FE, 0xE9),
            );
        }
    }
}

#[test]
fn call_ret_round_trip_restores_sp_and_returns_after_three_bytes() {
    // 205775-002 pp. 5-13/5-14, Table 5-1; 9800301-04 pp. 3-8/3-9, 3-48.
    let initial = registers(0x23FE, 31);
    let mut cpu = I8085::new(initial, latches());
    let mut bus = RecordingBus::new();
    bus.load(0x23FE, &[0xCD, 0xB6, 0x4D]);
    bus.load(0x4DB6, &[0xC9]);
    let mut memory = bus.memory.clone();
    memory[0x80FF] = 0x24;
    memory[0x80FE] = 0x01;
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: 2,
                cycles: u64::MAX
            }
        ),
        report(2, 28)
    );
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x2401,
            ..initial
        }
    );
    assert_eq!(*cpu.latches(), latches());
    let mut trace = fetch(0x23FE, 0xCD);
    trace.extend([
        read(0x23FF, MemoryAccess::OperandFetch, 0xB6),
        read(0x2400, MemoryAccess::OperandFetch, 0x4D),
        write(0x80FF, 0x24),
        write(0x80FE, 0x01),
    ]);
    trace.extend(fetch(0x4DB6, 0xC9));
    trace.extend([
        read(0x80FE, MemoryAccess::Stack, 0x01),
        read(0x80FF, MemoryAccess::Stack, 0x24),
    ]);
    assert_eq!(bus.log, trace);
    assert!(bus.memory == memory);
}

#[test]
fn derived_call_and_rst_stack_decrements_wrap_at_zero() {
    // Derived modulo 65536 from 16-bit SP (205775-002 p. 2-2), not an
    // explicit hardware wrap claim. Push rules pp. 5-13/5-14, Fig. 2-18;
    // second witness 9800301-04 pp. 3-8/3-9, 3-54/3-55.
    for (sp, high_at, low_at, final_sp) in [
        (0x0000, 0xFFFF, 0xFFFE, 0xFFFE),
        (0x0001, 0x0000, 0xFFFF, 0xFFFF),
        (0xFFFF, 0xFFFE, 0xFFFD, 0xFFFD),
    ] {
        for (opcode, target, low, states) in [(0xCD, 0x4DB6, 0x01, 18), (0xFF, 0x0038, 0xFF, 12)] {
            let initial = Registers {
                sp,
                ..registers(0x23FE, 31)
            };
            let mut bus = RecordingBus::new();
            bus.load(0x23FE, &[opcode, 0xB6, 0x4D]);
            let mut trace = fetch(0x23FE, opcode);
            if opcode == 0xCD {
                trace.extend([
                    read(0x23FF, MemoryAccess::OperandFetch, 0xB6),
                    read(0x2400, MemoryAccess::OperandFetch, 0x4D),
                ]);
            }
            trace.extend([
                write(high_at, if opcode == 0xCD { 0x24 } else { 0x23 }),
                write(low_at, low),
            ]);
            check(
                bus,
                initial,
                Registers {
                    pc: target,
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
fn derived_ret_stack_reads_and_increments_wrap_at_ffff() {
    // Derived from 16-bit SP (205775-002 p. 2-2), RET p. 5-14, Table 5-1.
    // 9800301-04 p. 3-48 agrees on pop; p. 3-26 shows INX SP FFFF->0000.
    for (sp, high_at, final_sp) in [
        (0xFFFE, 0xFFFF, 0x0000),
        (0xFFFF, 0x0000, 0x0001),
        (0x0000, 0x0001, 0x0002),
    ] {
        let initial = Registers {
            sp,
            ..registers(0x23FE, 31)
        };
        let mut bus = RecordingBus::new();
        bus.load(0x23FE, &[0xC9]);
        bus.load(sp, &[0xB6, 0x4D]);
        let mut trace = fetch(0x23FE, 0xC9);
        trace.extend([
            read(sp, MemoryAccess::Stack, 0xB6),
            read(high_at, MemoryAccess::Stack, 0x4D),
        ]);
        check(
            bus,
            initial,
            Registers {
                pc: 0x4DB6,
                sp: final_sp,
                ..initial
            },
            10,
            trace,
        );
    }
}

#[test]
fn derived_operand_reads_skips_and_return_addresses_wrap_pc() {
    // Derived from 16-bit PC (205775-002 p. 2-2), pp. 5-13/5-14 and
    // Table 5-1's shorter untaken sequences; stream reads Ch. 2 p. 2-7.
    // 9800301-04 pp. 3-8/3-9, 3-18, 3-28, 3-32, 3-55/3-56.
    for (pc, low_at, high_at, next, return_low) in [
        (0xFFFD, 0xFFFE, 0xFFFF, 0x0000, 0x00),
        (0xFFFE, 0xFFFF, 0x0000, 0x0001, 0x01),
        (0xFFFF, 0x0000, 0x0001, 0x0002, 0x02),
    ] {
        for (opcode, target, states, full, pushes) in [
            (0xC3, 0x4DB6, 10, true, false),
            (0xCA, next, 7, false, false),
            (0xCD, 0x4DB6, 18, true, true),
            (0xCC, next, 9, false, false),
        ] {
            let initial = registers(pc, 0); // Z=false.
            let mut bus = RecordingBus::new();
            bus.load(pc, &[opcode, 0xB6, 0x4D]);
            let mut trace = fetch(pc, opcode);
            trace.push(read(low_at, MemoryAccess::OperandFetch, 0xB6));
            if full {
                trace.push(read(high_at, MemoryAccess::OperandFetch, 0x4D));
            }
            if pushes {
                trace.extend([write(0x80FF, 0), write(0x80FE, return_low)]);
            }
            check(
                bus,
                initial,
                Registers {
                    pc: target,
                    sp: if pushes { 0x80FE } else { 0x8100 },
                    ..initial
                },
                states,
                trace,
            );
        }
    }
    for (opcode, states, pushes) in [(0xC7, 12, true), (0xC8, 6, false)] {
        let initial = registers(0xFFFF, 0);
        let mut bus = RecordingBus::new();
        bus.load(0xFFFF, &[opcode]);
        let mut trace = fetch(0xFFFF, opcode);
        if pushes {
            trace.extend([write(0x80FF, 0), write(0x80FE, 0)]);
        }
        check(
            bus,
            initial,
            Registers {
                pc: 0,
                sp: if pushes { 0x80FE } else { 0x8100 },
                ..initial
            },
            states,
            trace,
        );
    }
}
