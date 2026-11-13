//! Call/return witnesses (decision-13 rules 2–5; decision-15 rules 5–8).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), CALL pp. 281–284,
//! RET pp. 285–287, RETI p. 288, RETN p. 290, RST pp. 292–293 (PDF +14).
//! Second reading: 03-0029-01 (1977), Table 7.0-10 p. 53 (PDF 59).
//! IFF distinction: Mostek MK78515 V2.0 (October 1978), Table 2-1 p. 2-15
//! (PDF 29). All encoded facts were read on rendered page images.
//!
//! Polls, access kinds and R accounting follow decision-15 rules 1/5/6.
//! Every entry preserves all six flags: compare the entire register file,
//! including the decision-14 rule 5 zero unused bits. No unknown flags occur.
//! Fail-before: every test below returns Unsupported, zero steps/cycles, on
//! the unlanded group; only opcode fetches occur, with boundary state restored.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ARG: MemoryAccess = MemoryAccess::OperandFetch;
const STACK: MemoryAccess = MemoryAccess::Stack;

fn cpu_at(pc: u16, sp: u16, flags: u8) -> Z80 {
    Z80::new(
        Registers {
            a: 0xA1,
            flags: Flags::from_byte(flags),
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
            sp,
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

fn flag_images() -> impl Iterator<Item = u8> {
    (0..=255).filter(|bits| bits & 0x28 == 0)
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

fn fetch(pc: u16, opcode: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
        read(pc, OP, opcode),
    ]
}

// Boundary expiry is a library rule, separate from instruction effects.
fn expected_after(initial: &Z80, pc: u16, sp: u16, r: u8) -> Z80 {
    let mut expected = initial.clone();
    expected.registers_mut().pc = pc;
    expected.registers_mut().sp = sp;
    expected.registers_mut().r = r;
    expected.latches_mut().ei_delay = false;
    expected.latches_mut().parity_note_pending = false;
    expected.latches_mut().nmi_asserted = false;
    expected
}

fn check(mut cpu: Z80, mut bus: RecordingBus, expected: Z80, cycles: u64, trace: Vec<Event>) {
    let mut memory = bus.memory.clone();
    for event in &trace {
        if let Event::Write { address, value, .. } = *event {
            memory[usize::from(address)] = value;
        }
    }
    let before = cpu.clone();
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles,
            exit: Exit::Budget
        },
        "{before:?}"
    );
    assert_eq!(cpu, expected, "{before:?}");
    assert_eq!(bus.take_log(), trace, "{before:?}");
    assert_eq!(bus.memory, memory, "unexpected memory change");
}

// Literal opcodes, flag masks and required levels: UM pp. 283–284/286;
// TM Table 7.0-10 p. 53. H and N vary independently with every combination.
const CONDITIONS: [(u8, u8, u8, bool); 8] = [
    (0xC4, 0xC0, 0x40, false), // NZ
    (0xCC, 0xC8, 0x40, true),  // Z
    (0xD4, 0xD0, 0x01, false), // NC
    (0xDC, 0xD8, 0x01, true),  // C
    (0xE4, 0xE0, 0x04, false), // PO
    (0xEC, 0xE8, 0x04, true),  // PE
    (0xF4, 0xF0, 0x80, false), // P
    (0xFC, 0xF8, 0x80, true),  // M
];

const RESTARTS: [(u8, u16); 8] = [
    (0xC7, 0x00),
    (0xCF, 0x08),
    (0xD7, 0x10),
    (0xDF, 0x18),
    (0xE7, 0x20),
    (0xEF, 0x28),
    (0xF7, 0x30),
    (0xFF, 0x38),
];

#[test]
fn call_reads_address_then_pushes_next_pc_high_first_in_seventeen_states() {
    // UM pp. 281–282 (including its literal example); TM p. 53.
    for flags in flag_images() {
        let cpu = cpu_at(0x1A47, 0x3002, flags);
        let expected = expected_after(&cpu, 0x2135, 0x3000, 0x80);
        let mut bus = RecordingBus::new();
        bus.load(0x1A47, &[0xCD, 0x35, 0x21]);
        let mut trace = fetch(0x1A47, 0xCD);
        trace.extend([
            read(0x1A48, ARG, 0x35),
            read(0x1A49, ARG, 0x21),
            write(0x3001, 0x1A),
            write(0x3000, 0x4A),
        ]);
        check(cpu, bus, expected, 17, trace);
    }
}

#[test]
fn every_call_condition_both_ways_reads_both_operands_in_ten_or_seventeen_states() {
    // UM pp. 283–284: untaken 3 cycles, 10 (4,3,3); taken 17.
    // TM p. 53 agrees. No stack access occurs on the untaken path.
    for (opcode, _, mask, required) in CONDITIONS {
        for flags in flag_images() {
            let taken = (flags & mask != 0) == required;
            let cpu = cpu_at(0x23FE, 0x8100, flags);
            let expected = expected_after(
                &cpu,
                if taken { 0x4DB6 } else { 0x2401 },
                if taken { 0x80FE } else { 0x8100 },
                0x80,
            );
            let mut bus = RecordingBus::new();
            bus.load(0x23FE, &[opcode, 0xB6, 0x4D]);
            let mut trace = fetch(0x23FE, opcode);
            trace.extend([read(0x23FF, ARG, 0xB6), read(0x2400, ARG, 0x4D)]);
            if taken {
                trace.extend([write(0x80FF, 0x24), write(0x80FE, 0x01)]);
            }
            check(cpu, bus, expected, if taken { 17 } else { 10 }, trace);
        }
    }
}

#[test]
fn call_c_uses_carry_independently_of_zero_despite_primary_table_label() {
    // UM p. 284 labels the Carry (011) flag Z. UM RET cc p. 286 and
    // TM Table 7.0-10 p. 53 instead specify carry. Rechecked at 200 dpi;
    // PROVENANCE §1 records the label slip. Opposite Z/C values pin it.
    for (flags, pc, sp, cycles, writes) in [
        (
            0x01,
            0x4DB6,
            0x80FE,
            17,
            &[(0x80FF, 0x24), (0x80FE, 0x01)][..],
        ),
        (0x40, 0x2401, 0x8100, 10, &[][..]),
    ] {
        let cpu = cpu_at(0x23FE, 0x8100, flags);
        let expected = expected_after(&cpu, pc, sp, 0x80);
        let mut bus = RecordingBus::new();
        bus.load(0x23FE, &[0xDC, 0xB6, 0x4D]);
        let mut trace = fetch(0x23FE, 0xDC);
        trace.extend([read(0x23FF, ARG, 0xB6), read(0x2400, ARG, 0x4D)]);
        trace.extend(writes.iter().map(|&(address, value)| write(address, value)));
        check(cpu, bus, expected, cycles, trace);
    }
}

#[test]
fn ret_pops_low_then_high_in_ten_states() {
    // UM p. 285 literal example; TM p. 53. SP advances twice.
    for flags in flag_images() {
        let cpu = cpu_at(0x3535, 0x2000, flags);
        let expected = expected_after(&cpu, 0x18B5, 0x2002, 0x80);
        let mut bus = RecordingBus::new();
        bus.load(0x3535, &[0xC9]);
        bus.load(0x2000, &[0xB5, 0x18]);
        let mut trace = fetch(0x3535, 0xC9);
        trace.extend([read(0x2000, STACK, 0xB5), read(0x2001, STACK, 0x18)]);
        check(cpu, bus, expected, 10, trace);
    }
}

#[test]
fn every_ret_condition_both_ways_has_no_stack_reads_or_two_in_five_or_eleven_states() {
    // UM pp. 286–287; TM p. 53. Untaken only fetches its opcode.
    for (_, opcode, mask, required) in CONDITIONS {
        for flags in flag_images() {
            let taken = (flags & mask != 0) == required;
            let cpu = cpu_at(0x3535, 0x2000, flags);
            let expected = expected_after(
                &cpu,
                if taken { 0x18B5 } else { 0x3536 },
                if taken { 0x2002 } else { 0x2000 },
                0x80,
            );
            let mut bus = RecordingBus::new();
            bus.load(0x3535, &[opcode]);
            bus.load(0x2000, &[0xB5, 0x18]);
            let mut trace = fetch(0x3535, opcode);
            if taken {
                trace.extend([read(0x2000, STACK, 0xB5), read(0x2001, STACK, 0x18)]);
            }
            check(cpu, bus, expected, if taken { 11 } else { 5 }, trace);
        }
    }
}

fn interrupt_return(opcode: u8, restores_iff1: bool) {
    for flags in flag_images() {
        for iff1 in [false, true] {
            for iff2 in [false, true] {
                let mut cpu = cpu_at(0x1234, 0x0FFE, flags);
                cpu.latches_mut().iff1 = iff1;
                cpu.latches_mut().iff2 = iff2;
                let mut expected = expected_after(&cpu, 0x1A45, 0x1000, 0x81);
                if restores_iff1 {
                    expected.latches_mut().iff1 = iff2;
                }
                let mut bus = RecordingBus::new();
                bus.load(0x1234, &[0xED, opcode]);
                bus.load(0x0FFE, &[0x45, 0x1A]);
                let mut trace = fetch(0x1234, 0xED);
                trace.extend([
                    read(0x1235, OP, opcode),
                    read(0x0FFE, STACK, 0x45),
                    read(0x0FFF, STACK, 0x1A),
                ]);
                check(cpu, bus, expected, 14, trace);
            }
        }
    }
}

#[test]
fn reti_pops_pc_and_preserves_every_iff_combination() {
    // UM p. 288; TM p. 53; MK Table 2-1 p. 2-15. Decision-15 rule 7:
    // ED 4D fetches expose RETI to devices; there is no extra callback.
    interrupt_return(0x4D, false);
}

#[test]
fn retn_pops_pc_and_copies_iff2_to_iff1_for_every_iff_combination() {
    // UM p. 290 (literal restored PC/SP); TM p. 53; MK Table 2-1 p. 2-15.
    interrupt_return(0x45, true);
}

#[test]
fn all_rst_vectors_push_next_pc_high_first_through_shared_helper_in_eleven_states() {
    // UM pp. 292–293; TM p. 53. This witnesses push_pc_and_jump via RST.
    for (opcode, target) in RESTARTS {
        for flags in flag_images() {
            let cpu = cpu_at(0x15B3, 0x8100, flags);
            let expected = expected_after(&cpu, target, 0x80FE, 0x80);
            let mut bus = RecordingBus::new();
            bus.load(0x15B3, &[opcode]);
            let mut trace = fetch(0x15B3, opcode);
            trace.extend([write(0x80FF, 0x15), write(0x80FE, 0xB4)]);
            check(cpu, bus, expected, 11, trace);
        }
    }
}

#[test]
fn call_operand_return_pc_and_both_push_decrements_wrap_derived() {
    // Derived from 16-bit PC/SP (UM p. 2; TM §2.1 p. 3), CALL pp. 281/283.
    // Exercise unconditional and taken/untaken conditional calls.
    for (opcode, flags, taken) in [(0xCD, 0xD7, true), (0xC4, 0, true), (0xC4, 0xD7, false)] {
        for (pc, arg_low, arg_high, return_pc, return_low) in [
            (0xFFFD, 0xFFFE, 0xFFFF, 0x0000, 0x00),
            (0xFFFE, 0xFFFF, 0x0000, 0x0001, 0x01),
            (0xFFFF, 0x0000, 0x0001, 0x0002, 0x02),
        ] {
            for (sp, high_at, low_at) in [(0, 0xFFFF, 0xFFFE), (1, 0, 0xFFFF)] {
                let cpu = cpu_at(pc, sp, flags);
                let expected = expected_after(
                    &cpu,
                    if taken { 0x4DB6 } else { return_pc },
                    if taken { low_at } else { sp },
                    0x80,
                );
                let mut bus = RecordingBus::new();
                bus.load(pc, &[opcode, 0xB6, 0x4D]);
                let mut trace = fetch(pc, opcode);
                trace.extend([read(arg_low, ARG, 0xB6), read(arg_high, ARG, 0x4D)]);
                if taken {
                    trace.extend([write(high_at, 0), write(low_at, return_low)]);
                }
                check(cpu, bus, expected, if taken { 17 } else { 10 }, trace);
            }
        }
    }
}

#[test]
fn every_return_class_wraps_each_pop_increment_derived() {
    // Derived 16-bit SP wrap (UM p. 2; TM §2.1 p. 3), pop pp. 285–290.
    // ED prefix/key also cross an address-byte boundary (rules 5/6).
    for (bytes, flags, cycles, r) in [
        (&[0xC9][..], 0xD7, 10, 0x80),
        (&[0xC0][..], 0, 11, 0x80),
        (&[0xED, 0x4D][..], 0xD7, 14, 0x81),
        (&[0xED, 0x45][..], 0xD7, 14, 0x81),
    ] {
        for (sp, high_at, next_sp) in [(0xFFFE, 0xFFFF, 0), (0xFFFF, 0, 1)] {
            let cpu = cpu_at(0x7FFF, sp, flags);
            let expected = expected_after(&cpu, 0x4DB6, next_sp, r);
            let mut bus = RecordingBus::new();
            bus.load(0x7FFF, bytes);
            bus.load(sp, &[0xB6, 0x4D]);
            let mut trace = fetch(0x7FFF, bytes[0]);
            if let Some(&opcode) = bytes.get(1) {
                trace.push(read(0x8000, OP, opcode));
            }
            trace.extend([read(sp, STACK, 0xB6), read(high_at, STACK, 0x4D)]);
            check(cpu, bus, expected, cycles, trace);
        }
    }
}

#[test]
fn rst_wraps_return_pc_and_both_stack_decrements_derived() {
    // Derived 16-bit PC/SP wrap (UM p. 2; TM §2.1 p. 3);
    // RST pp. 292–293, TM p. 53.
    for (opcode, target) in RESTARTS {
        for (sp, high_at, low_at) in [(0, 0xFFFF, 0xFFFE), (1, 0, 0xFFFF)] {
            let cpu = cpu_at(0xFFFF, sp, 0xD7);
            let expected = expected_after(&cpu, target, low_at, 0x80);
            let mut bus = RecordingBus::new();
            bus.load(0xFFFF, &[opcode]);
            let mut trace = fetch(0xFFFF, opcode);
            trace.extend([write(high_at, 0), write(low_at, 0)]);
            check(cpu, bus, expected, 11, trace);
        }
    }
}
