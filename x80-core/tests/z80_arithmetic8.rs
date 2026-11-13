//! 8-bit arithmetic execution witnesses (decisions 13–15).
//!
//! Zilog UM008011-0816 rev. 11 (August 2016), entries pp. 145–171,
//! P/V definition p. 67; independent 03-0029-01 (1977), Table 7.0-4
//! p. 47. PDF = printed + 14 / + 6 respectively. Source hashes match
//! PROVENANCE.md §1; values read on rendered 150-dpi images.
//! Expectations below are literal transcriptions/derivations of the entries,
//! never calls to the production ALU. All six flags are defined here.
//! Full F comparisons additionally pin decision-14's zero unused-bit image.
//!
//! Every test fails before landing: the base's disabled arithmetic8 group
//! returns Unsupported with zero steps/cycles at the first instruction.
//! Every case checks the entire CPU, memory, poll/access trace and Report.
//! R and access kinds follow decision-15 rules 5–6; boundary latch expiry
//! follows rules 2/7/10. All PC/address wraps below are DERIVED u16 wraps.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ARG: MemoryAccess = MemoryAccess::OperandFetch;
const DATA: MemoryAccess = MemoryAccess::Data;

fn cpu_at(pc: u16, a: u8, flags: u8) -> Z80 {
    Z80::new(
        Registers {
            a,
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
            sp: 0xDEF0,
            pc,
            i: 0x38,
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

fn after(cpu: &Z80, pc: u16, r: u8, flags: u8) -> Z80 {
    let mut expected = cpu.clone();
    expected.registers_mut().pc = pc;
    expected.registers_mut().r = r;
    expected.registers_mut().flags = Flags::from_byte(flags);
    expected.latches_mut().ei_delay = false;
    expected.latches_mut().nmi_asserted = false;
    expected.latches_mut().parity_note_pending = false;
    expected
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
        kind: DATA,
        value,
    }
}

fn check(
    cpu: &mut Z80,
    bus: &mut RecordingBus,
    expected: &Z80,
    memory: &[u8],
    cycles: u64,
    accesses: &[Event],
) {
    assert_eq!(
        cpu.step(bus),
        Report {
            steps: 1,
            cycles,
            exit: Exit::Budget,
        },
        "accesses: {accesses:?}"
    );
    assert_eq!(cpu, expected, "accesses: {accesses:?}");
    assert_eq!(bus.memory, memory);
    let mut trace = vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ];
    trace.extend_from_slice(accesses);
    assert_eq!(bus.take_log(), trace);
}

// Test-owned field mapping: UM pp. 145,152,154,156,158,160,162,164–165,170;
// TM p. 47. Register A is covered separately because it aliases the left input.
fn set_register(r: &mut Registers, code: u8, value: u8) {
    match code {
        0 => r.b = value,
        1 => r.c = value,
        2 => r.d = value,
        3 => r.e = value,
        4 => r.h = value,
        5 => r.l = value,
        7 => r.a = value,
        _ => panic!("not a register"),
    }
}

#[derive(Clone, Copy)]
enum Operand {
    Register(u8),
    Immediate,
    Hl(u16),
    // Literal effective address, independently supplied alongside base and d.
    Index(u8, u16, u8, u16),
}

// Signed extrema, -1 and zero, for BOTH indexes. Underflow and overflow
// across 0000h/FFFFh are DERIVED from the two's-complement rule, UM pp. 149–150.
const INDEXES: [(u16, u8, u16); 4] = [
    (0x0040, 0x80, 0xFFC0),
    (0x2345, 0xFF, 0x2344),
    (0x3456, 0x00, 0x3456),
    (0xFFC0, 0x7F, 0x003F),
];

// A before, source, incoming C, A after, complete F after.
type Binary = (u8, u8, u8, u8, u8);

fn binary(base: u8, immediate: u8, form: Operand, row: Binary) {
    let (a, source, carry, result, flags) = row;
    // Flip every incoming non-C flag: each instruction must replace all of them.
    for initial in [carry, carry | 0xD6] {
        let mut cpu = cpu_at(0xFFFF, a, initial);
        let mut bus = RecordingBus::new();
        let (pc, r, cycles, accesses) = match form {
            Operand::Register(code) => {
                assert!(code != 7 || a == source);
                set_register(cpu.registers_mut(), code, source);
                let opcode = base | code;
                bus.load(0xFFFF, &[opcode]);
                (0, 0x80, 4, vec![read(0xFFFF, OP, opcode)])
            }
            Operand::Immediate => {
                bus.load(0xFFFF, &[immediate, source]);
                (
                    1,
                    0x80,
                    7,
                    vec![read(0xFFFF, OP, immediate), read(0, ARG, source)],
                )
            }
            Operand::Hl(address) => {
                [cpu.registers_mut().h, cpu.registers_mut().l] = address.to_be_bytes();
                let opcode = base | 6;
                bus.load(0xFFFF, &[opcode]);
                bus.load(address, &[source]);
                (
                    0,
                    0x80,
                    7,
                    vec![read(0xFFFF, OP, opcode), read(address, DATA, source)],
                )
            }
            Operand::Index(prefix, index, d, address) => {
                cpu.registers_mut().pc = 0xFFFE;
                if prefix == 0xDD {
                    cpu.registers_mut().ix = index;
                } else {
                    cpu.registers_mut().iy = index;
                }
                let opcode = base | 6;
                bus.load(0xFFFE, &[prefix, opcode, d]);
                bus.load(address, &[source]);
                (
                    1,
                    0x81,
                    19,
                    vec![
                        read(0xFFFE, OP, prefix),
                        read(0xFFFF, OP, opcode),
                        read(0, ARG, d),
                        read(address, DATA, source),
                    ],
                )
            }
        };
        let mut expected = after(&cpu, pc, r, flags);
        expected.registers_mut().a = result;
        let memory = bus.memory.clone();
        check(&mut cpu, &mut bus, &expected, &memory, cycles, &accesses);
        assert_eq!(cpu.registers().flags.to_byte(), flags);
    }
}

fn all_binary_forms(base: u8, immediate: u8, rows: &[Binary]) {
    for &row in rows {
        for code in 0..6 {
            binary(base, immediate, Operand::Register(code), row);
        }
        binary(base, immediate, Operand::Immediate, row);
        binary(base, immediate, Operand::Hl(0x3434), row);
        for prefix in [0xDD, 0xFD] {
            for (index, d, address) in INDEXES {
                binary(
                    base,
                    immediate,
                    Operand::Index(prefix, index, d, address),
                    row,
                );
            }
        }
    }
}

#[test]
fn add_all_operand_classes_literal_flags() {
    // UM pp. 145–150; TM p. 47. C input is ignored. Overflow differs
    // from parity (e.g. 01+02=03 has even parity but no overflow).
    all_binary_forms(
        0x80,
        0xC6,
        &[
            (0x44, 0x11, 1, 0x55, 0x00),
            (0x23, 0x33, 0, 0x56, 0x00),
            (0xA0, 0x08, 0, 0xA8, 0x80),
            (0x11, 0x22, 0, 0x33, 0x00),
            (0x0F, 0x01, 1, 0x10, 0x10),
            (0x7F, 0x01, 0, 0x80, 0x94),
            (0x80, 0x80, 0, 0x00, 0x45),
            (0xFF, 0x01, 1, 0x00, 0x51),
            (0x01, 0x02, 0, 0x03, 0x00),
            (0x78, 0x69, 0, 0xE1, 0x94), // UM p. 67 overflow example.
        ],
    );
}

#[test]
fn adc_all_operand_classes_literal_flags() {
    // UM pp. 151–152; TM p. 47. Same operands with both C inputs,
    // carry-in half carry, full carry and both overflow directions.
    all_binary_forms(
        0x88,
        0xCE,
        &[
            (0x16, 0x10, 1, 0x27, 0x00),
            (0x16, 0x10, 0, 0x26, 0x00),
            (0x0F, 0x00, 1, 0x10, 0x10),
            (0x7F, 0x00, 1, 0x80, 0x94),
            (0x7F, 0x00, 0, 0x7F, 0x00),
            (0xFF, 0x00, 1, 0x00, 0x51),
            (0x80, 0xFF, 1, 0x80, 0x91),
            (0x80, 0x80, 0, 0x00, 0x45),
            (0x80, 0x80, 1, 0x01, 0x05),
            (0x00, 0xFF, 1, 0x00, 0x51),
        ],
    );
}

#[test]
fn sub_all_operand_classes_literal_flags() {
    // UM pp. 153–154; TM p. 47. Borrow sets C; C input is ignored.
    all_binary_forms(
        0x90,
        0xD6,
        &[
            (0x29, 0x11, 1, 0x18, 0x02),
            (0x10, 0x01, 0, 0x0F, 0x12),
            (0x00, 0x01, 1, 0xFF, 0x93),
            (0x80, 0x01, 0, 0x7F, 0x16),
            (0x7F, 0xFF, 0, 0x80, 0x87),
            (0x42, 0x42, 1, 0x00, 0x42),
            (0x03, 0x00, 0, 0x03, 0x02),
            (0x7F, 0xC0, 0, 0xBF, 0x87), // UM p. 67 overflow example.
        ],
    );
}

#[test]
fn sbc_all_operand_classes_overflow_not_the_entry_misprint() {
    // UM pp. 155–156, corrected by p. 67 and TM p. 47 (existing
    // PROVENANCE row): P/V SETS on overflow, not permanently reset.
    all_binary_forms(
        0x98,
        0xDE,
        &[
            (0x16, 0x05, 1, 0x10, 0x02),
            (0x16, 0x05, 0, 0x11, 0x02),
            (0x10, 0x00, 1, 0x0F, 0x12),
            (0x00, 0x00, 1, 0xFF, 0x93),
            (0x80, 0x00, 1, 0x7F, 0x16),
            (0x80, 0x00, 0, 0x80, 0x82),
            (0x7F, 0xFF, 1, 0x7F, 0x13),
            (0x80, 0x7F, 1, 0x00, 0x56),
            (0x00, 0xFF, 1, 0x00, 0x53),
            (0x01, 0x00, 1, 0x00, 0x42),
        ],
    );
}

#[test]
fn and_all_operand_classes_parity_not_the_entry_misprint() {
    // UM pp. 157–158, p. 67 / TM p. 47 resolve the existing P/V slip.
    // H always sets, N/C clear; both parities and S/Z directions.
    all_binary_forms(
        0xA0,
        0xE6,
        &[
            (0xC3, 0x7B, 1, 0x43, 0x10),
            (0xFF, 0x03, 1, 0x03, 0x14),
            (0xFF, 0x80, 0, 0x80, 0x90),
            (0xFF, 0x81, 0, 0x81, 0x94),
            (0xF0, 0x0F, 1, 0x00, 0x54),
        ],
    );
}

#[test]
fn or_all_operand_classes_parity_not_the_entry_misprint() {
    // UM pp. 159–160, p. 67 / TM p. 47 resolve the existing P/V slip.
    // FD B6 d is OR, not the entry's erroneous AE bit diagram.
    all_binary_forms(
        0xB0,
        0xF6,
        &[
            (0x12, 0x48, 1, 0x5A, 0x04),
            (0x00, 0x00, 1, 0x00, 0x44),
            (0x80, 0x00, 0, 0x80, 0x80),
            (0x80, 0x01, 1, 0x81, 0x84),
            (0x01, 0x02, 0, 0x03, 0x04),
        ],
    );
}

#[test]
fn xor_all_operand_classes_literal_flags() {
    // UM pp. 161–162; TM p. 47: parity, H/N/C reset.
    all_binary_forms(
        0xA8,
        0xEE,
        &[
            (0x96, 0x5D, 1, 0xCB, 0x80),
            (0xFF, 0xFF, 1, 0x00, 0x44),
            (0x80, 0x00, 0, 0x80, 0x80),
            (0x80, 0x01, 1, 0x81, 0x84),
            (0x01, 0x02, 0, 0x03, 0x04),
        ],
    );
}

#[test]
fn cp_all_operand_classes_leave_a_and_source_unchanged() {
    // UM pp. 163–164; TM p. 47: flags of A-s without assigning A.
    all_binary_forms(
        0xB8,
        0xFE,
        &[
            (0x63, 0x60, 1, 0x63, 0x02),
            (0x42, 0x42, 1, 0x42, 0x42),
            (0x10, 0x01, 0, 0x10, 0x12),
            (0x00, 0x01, 1, 0x00, 0x93),
            (0x80, 0x01, 0, 0x80, 0x16),
            (0x7F, 0xFF, 0, 0x7F, 0x87),
        ],
    );
}

#[test]
fn accumulator_source_aliases_read_a_before_replacement() {
    // Same entries as above. These are literal self-operation results,
    // including C input to ADC/SBC and A preservation by CP.
    for (base, immediate, row) in [
        (0x80, 0xC6, (0x7F, 0x7F, 1, 0xFE, 0x94)),
        (0x80, 0xC6, (0x80, 0x80, 0, 0x00, 0x45)),
        (0x88, 0xCE, (0x7F, 0x7F, 1, 0xFF, 0x94)),
        (0x88, 0xCE, (0x80, 0x80, 0, 0x00, 0x45)),
        (0x90, 0xD6, (0x93, 0x93, 1, 0x00, 0x42)),
        (0x98, 0xDE, (0x93, 0x93, 0, 0x00, 0x42)),
        (0x98, 0xDE, (0x93, 0x93, 1, 0xFF, 0x93)),
        (0xA0, 0xE6, (0x81, 0x81, 1, 0x81, 0x94)),
        (0xB0, 0xF6, (0x81, 0x81, 1, 0x81, 0x84)),
        (0xA8, 0xEE, (0x81, 0x81, 1, 0x00, 0x44)),
        (0xB8, 0xFE, (0x81, 0x81, 1, 0x81, 0x42)),
    ] {
        binary(base, immediate, Operand::Register(7), row);
    }
}

// Input byte, output byte, F excluding the preserved C.
type Unary = (u8, u8, u8);

fn unary(decrement: bool, form: Operand, row: Unary) {
    let (value, result, flags) = row;
    for initial in [0, 1, 0xD6, 0xD7] {
        let mut cpu = cpu_at(0xFFFF, 0xA1, initial);
        let mut bus = RecordingBus::new();
        let low = if decrement { 5 } else { 4 };
        let (pc, r, cycles, mut accesses, destination) = match form {
            Operand::Register(code) => {
                set_register(cpu.registers_mut(), code, value);
                let opcode = (code << 3) | low;
                bus.load(0xFFFF, &[opcode]);
                (0, 0x80, 4, vec![read(0xFFFF, OP, opcode)], None)
            }
            Operand::Hl(address) => {
                [cpu.registers_mut().h, cpu.registers_mut().l] = address.to_be_bytes();
                let opcode = 0x30 | low;
                bus.load(0xFFFF, &[opcode]);
                bus.load(address, &[value]);
                (
                    0,
                    0x80,
                    11,
                    vec![read(0xFFFF, OP, opcode), read(address, DATA, value)],
                    Some(address),
                )
            }
            Operand::Index(prefix, index, d, address) => {
                cpu.registers_mut().pc = 0xFFFE;
                if prefix == 0xDD {
                    cpu.registers_mut().ix = index;
                } else {
                    cpu.registers_mut().iy = index;
                }
                let opcode = 0x30 | low;
                bus.load(0xFFFE, &[prefix, opcode, d]);
                bus.load(address, &[value]);
                (
                    1,
                    0x81,
                    23,
                    vec![
                        read(0xFFFE, OP, prefix),
                        read(0xFFFF, OP, opcode),
                        read(0, ARG, d),
                        read(address, DATA, value),
                    ],
                    Some(address),
                )
            }
            Operand::Immediate => panic!("INC/DEC has no immediate form"),
        };
        let final_flags = flags | (initial & 1);
        let mut expected = after(&cpu, pc, r, final_flags);
        let mut memory = bus.memory.clone();
        if let Some(address) = destination {
            memory[usize::from(address)] = result;
            accesses.push(write(address, result));
        } else if let Operand::Register(code) = form {
            set_register(expected.registers_mut(), code, result);
        }
        check(&mut cpu, &mut bus, &expected, &memory, cycles, &accesses);
        assert_eq!(cpu.registers().flags.to_byte(), final_flags);
    }
}

fn all_unary_forms(decrement: bool, rows: &[Unary]) {
    for &row in rows {
        for code in [0, 1, 2, 3, 4, 5, 7] {
            unary(decrement, Operand::Register(code), row);
        }
        unary(decrement, Operand::Hl(0x3434), row);
        for prefix in [0xDD, 0xFD] {
            for (index, d, address) in INDEXES {
                unary(decrement, Operand::Index(prefix, index, d, address), row);
            }
        }
    }
}

#[test]
fn inc_all_forms_preserve_both_carry_values_and_read_modify_write_data() {
    // UM pp. 165–169; TM p. 47. P/V only on 7F->80; no C input.
    all_unary_forms(
        false,
        &[
            (0x28, 0x29, 0x00),
            (0x82, 0x83, 0x80),
            (0x34, 0x35, 0x00),
            (0x0F, 0x10, 0x10),
            (0x7F, 0x80, 0x94),
            (0xFF, 0x00, 0x50),
            (0x00, 0x01, 0x00),
            (0x80, 0x81, 0x80),
        ],
    );
}

#[test]
fn dec_all_forms_preserve_both_carry_values_and_read_modify_write_data() {
    // UM pp. 170–171; TM p. 47. P/V only on 80->7F; N sets.
    all_unary_forms(
        true,
        &[
            (0x2A, 0x29, 0x02),
            (0x10, 0x0F, 0x12),
            (0x80, 0x7F, 0x16),
            (0x00, 0xFF, 0x92),
            (0x01, 0x00, 0x42),
            (0x81, 0x80, 0x82),
            (0x7F, 0x7E, 0x02),
        ],
    );
}

#[test]
fn manual_binary_examples_keep_the_named_registers_and_addresses() {
    // ADD examples, UM pp. 146–150 (flags derived from the entry rules).
    binary(0x80, 0xC6, Operand::Register(1), (0x44, 0x11, 0, 0x55, 0));
    binary(0x80, 0xC6, Operand::Immediate, (0x23, 0x33, 0, 0x56, 0));
    binary(0x80, 0xC6, Operand::Hl(0x2323), (0xA0, 8, 0, 0xA8, 0x80));
    for prefix in [0xDD, 0xFD] {
        binary(
            0x80,
            0xC6,
            Operand::Index(prefix, 0x1000, 5, 0x1005),
            (0x11, 0x22, 0, 0x33, 0),
        );
    }
    // ADC p. 152, SUB p. 154, SBC p. 156.
    binary(0x88, 0xCE, Operand::Hl(0x6666), (0x16, 0x10, 1, 0x27, 0));
    binary(0x90, 0xD6, Operand::Register(2), (0x29, 0x11, 0, 0x18, 2));
    binary(0x98, 0xDE, Operand::Hl(0x3433), (0x16, 5, 1, 0x10, 2));
    // AND p. 158, OR p. 160, XOR p. 162, CP p. 164.
    binary(
        0xA0,
        0xE6,
        Operand::Register(0),
        (0xC3, 0x7B, 0, 0x43, 0x10),
    );
    binary(0xB0, 0xF6, Operand::Register(4), (0x12, 0x48, 0, 0x5A, 4));
    binary(0xA8, 0xEE, Operand::Immediate, (0x96, 0x5D, 0, 0xCB, 0x80));
    binary(0xB8, 0xFE, Operand::Hl(0x6000), (0x63, 0x60, 0, 0x63, 2));
}

#[test]
fn manual_unary_examples_keep_the_named_registers_and_addresses() {
    // INC D p. 166, INC (HL) p. 167, INC (IX/IY+d) pp. 168–169.
    unary(false, Operand::Register(2), (0x28, 0x29, 0));
    unary(false, Operand::Hl(0x3434), (0x82, 0x83, 0x80));
    for prefix in [0xDD, 0xFD] {
        unary(
            false,
            Operand::Index(prefix, 0x2020, 0x10, 0x2030),
            (0x34, 0x35, 0),
        );
    }
    // DEC D, UM p. 171.
    unary(true, Operand::Register(2), (0x2A, 0x29, 2));
}
