//! Z80 bit-group witnesses (decisions 13–15).
//!
//! Primary: Zilog UM008011-0816 rev. 11 (August 2016), Bit Set, Reset,
//! and Test Group pp. 243–260 (PDF = printed + 14). Second transcription:
//! 03-0029-01 (1977), Table 7.0-8 p. 51 (PDF 57). Source hashes match
//! PROVENANCE.md §1; values read on 150 dpi rendered page images.
//! BIT uses hardware mask 53h; S/P/V preservation is a separate library
//! assertion (decision-14 rule 6), not a claim about unknown silicon flags.
//! Existing PROVENANCE BIT H/N label-slip resolution applies to pp. 246/250.
//! Fail-before: all twelve witnesses return Unsupported with zero retirement
//! on base 5f5f6e8, because this group's LANDED switch is false.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ARG: MemoryAccess = MemoryAccess::OperandFetch;
const DATA: MemoryAccess = MemoryAccess::Data;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operation {
    Bit,
    Set,
    Res,
}

// Literal expansion of UM pp. 243/245/247/249/251/253/255/257/259–260
// and TM Table 7.0-8 p. 51. Columns: BIT/SET/RES register-B key,
// BIT/SET/RES memory key, then results for operands 00, FF, A5, 5A.
// Each result is (BIT zero, SET byte, RES byte); no executor helper is used.
struct Row {
    register: [u8; 3],
    memory: [u8; 3],
    results: [(bool, u8, u8); 4],
}

const ROWS: [Row; 8] = [
    Row {
        register: [0x40, 0xC0, 0x80],
        memory: [0x46, 0xC6, 0x86],
        results: [
            (true, 0x01, 0x00),
            (false, 0xFF, 0xFE),
            (false, 0xA5, 0xA4),
            (true, 0x5B, 0x5A),
        ],
    },
    Row {
        register: [0x48, 0xC8, 0x88],
        memory: [0x4E, 0xCE, 0x8E],
        results: [
            (true, 0x02, 0x00),
            (false, 0xFF, 0xFD),
            (true, 0xA7, 0xA5),
            (false, 0x5A, 0x58),
        ],
    },
    Row {
        register: [0x50, 0xD0, 0x90],
        memory: [0x56, 0xD6, 0x96],
        results: [
            (true, 0x04, 0x00),
            (false, 0xFF, 0xFB),
            (false, 0xA5, 0xA1),
            (true, 0x5E, 0x5A),
        ],
    },
    Row {
        register: [0x58, 0xD8, 0x98],
        memory: [0x5E, 0xDE, 0x9E],
        results: [
            (true, 0x08, 0x00),
            (false, 0xFF, 0xF7),
            (true, 0xAD, 0xA5),
            (false, 0x5A, 0x52),
        ],
    },
    Row {
        register: [0x60, 0xE0, 0xA0],
        memory: [0x66, 0xE6, 0xA6],
        results: [
            (true, 0x10, 0x00),
            (false, 0xFF, 0xEF),
            (true, 0xB5, 0xA5),
            (false, 0x5A, 0x4A),
        ],
    },
    Row {
        register: [0x68, 0xE8, 0xA8],
        memory: [0x6E, 0xEE, 0xAE],
        results: [
            (true, 0x20, 0x00),
            (false, 0xFF, 0xDF),
            (false, 0xA5, 0x85),
            (true, 0x7A, 0x5A),
        ],
    },
    Row {
        register: [0x70, 0xF0, 0xB0],
        memory: [0x76, 0xF6, 0xB6],
        results: [
            (true, 0x40, 0x00),
            (false, 0xFF, 0xBF),
            (true, 0xE5, 0xA5),
            (false, 0x5A, 0x1A),
        ],
    },
    Row {
        register: [0x78, 0xF8, 0xB8],
        memory: [0x7E, 0xFE, 0xBE],
        results: [
            (true, 0x80, 0x00),
            (false, 0xFF, 0x7F),
            (false, 0xA5, 0x25),
            (true, 0xDA, 0x5A),
        ],
    },
];

// Independent construction of flags from literal images (UM Table 21 p. 65;
// decision-14 rule 5). Include independent S/P/V values and both C/H/N/Z.
fn flags(byte: u8) -> Flags {
    Flags {
        sign: byte >= 0x80,
        zero: !(byte / 0x40).is_multiple_of(2),
        half_carry: !(byte / 0x10).is_multiple_of(2),
        parity_overflow: !(byte / 4).is_multiple_of(2),
        subtract: !(byte / 2).is_multiple_of(2),
        carry: !byte.is_multiple_of(2),
    }
}

fn cpu(flags_byte: u8) -> Z80 {
    Z80::new(
        Registers {
            a: 0xA1,
            flags: flags(flags_byte),
            b: 0xB2,
            c: 0xC3,
            d: 0xD4,
            e: 0xE5,
            h: 0x36,
            l: 0x47,
            a_alternate: 0x18,
            flags_alternate: flags(0x42),
            b_alternate: 0x29,
            c_alternate: 0x3A,
            d_alternate: 0x4B,
            e_alternate: 0x5C,
            h_alternate: 0x6D,
            l_alternate: 0x7E,
            ix: 0x8091,
            iy: 0x92A3,
            sp: 0xDEF0,
            pc: 0xFFFF,
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

// Register field transcription: UM pp. 243/251/260; TM p. 51.
fn put(registers: &mut Registers, code: u8, value: u8) {
    match code {
        0 => registers.b = value,
        1 => registers.c = value,
        2 => registers.d = value,
        3 => registers.e = value,
        4 => registers.h = value,
        5 => registers.l = value,
        7 => registers.a = value,
        _ => panic!("not a documented register operand"),
    }
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

fn polls() -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ]
}

fn column(operation: Operation) -> usize {
    match operation {
        Operation::Bit => 0,
        Operation::Set => 1,
        Operation::Res => 2,
    }
}

fn result(operation: Operation, input: u8, row: (bool, u8, u8)) -> u8 {
    match operation {
        Operation::Bit => input,
        Operation::Set => row.1,
        Operation::Res => row.2,
    }
}

fn expected(mut before: Z80, operation: Operation, zero: bool, pc: u16) -> Z80 {
    before.registers_mut().pc = pc;
    // Two M1 increments FF -> 80 -> 81, retaining bit 7 (decision-15
    // rules 5–6; UM Registers p. 3; TM §§2/3 pp. 4/7). Indexed fourth
    // byte as OperandFetch is the explicitly unverified rule-5 reading.
    before.registers_mut().r = 0x81;
    if operation == Operation::Bit {
        before.registers_mut().flags.zero = zero;
        before.registers_mut().flags.half_carry = true;
        before.registers_mut().flags.subtract = false;
    }
    before.latches_mut().ei_delay = false;
    before.latches_mut().nmi_asserted = false;
    before.latches_mut().parity_note_pending = false;
    before
}

fn check(cpu: &Z80, expected: Z80, report: Report, cycles: u64, operation: Operation, zero: bool) {
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles,
            exit: Exit::Budget
        }
    );
    if operation == Operation::Bit {
        // Hardware claim only: Z=~bit, H=1, N=0, C unchanged.
        let known = if zero { 0x50 } else { 0x10 } + u8::from(expected.registers().flags.carry);
        assert_eq!(cpu.registers().flags.to_byte() & 0x53, known);
        // Separate deterministic library claim from both independent S/P/V
        // polarities; do not treat either unknown flag as a hardware result.
        assert_eq!(cpu.registers().flags.sign, expected.registers().flags.sign);
        assert_eq!(
            cpu.registers().flags.parity_overflow,
            expected.registers().flags.parity_overflow
        );
    }
    assert_eq!(cpu.registers().flags.to_byte() & 0x28, 0);
    assert_eq!(*cpu, expected);
}

fn registers(operation: Operation) {
    let mut bus = RecordingBus::new();
    for row in &ROWS {
        for (input, outcome) in [0x00, 0xFF, 0xA5, 0x5A].into_iter().zip(row.results) {
            for old in [0x00, 0xD7, 0x80, 0x04, 0x43, 0x92] {
                for code in [0, 1, 2, 3, 4, 5, 7] {
                    let mut cpu = cpu(old);
                    put(cpu.registers_mut(), code, input);
                    let opcode = row.register[column(operation)] + code;
                    bus.load(0xFFFF, &[0xCB, opcode]);
                    let memory = bus.memory.clone();
                    let mut want = expected(cpu.clone(), operation, outcome.0, 1);
                    put(
                        want.registers_mut(),
                        code,
                        result(operation, input, outcome),
                    );
                    let report = cpu.step(&mut bus);
                    check(&cpu, want, report, 8, operation, outcome.0);
                    assert_eq!(bus.memory, memory);
                    let mut trace = polls();
                    trace.extend([read(0xFFFF, OP, 0xCB), read(0, OP, opcode)]);
                    assert_eq!(bus.take_log(), trace);
                }
            }
        }
    }
}

fn memory(operation: Operation, prefix: Option<u8>) {
    // Literal signed offsets: 0, +127, -128, -1; plus both carry/borrow
    // wraps. DERIVED 16-bit wrap from UM Registers pp. 2–3; signed d is
    // stated in BIT pp. 247/249 and SET pp. 255/257; RES p. 259 inherits.
    // base, d, effective address, stream PC, completed PC. Keep stream
    // separate from Data at 0000/FFFF in the effective-address wrap cases.
    let addresses: &[(u16, u8, u16, u16, u16)] = if prefix.is_some() {
        &[
            (0x4000, 0x00, 0x4000, 0xFFFD, 0x0001),
            (0x4000, 0x7F, 0x407F, 0xFFFD, 0x0001),
            (0x4000, 0x80, 0x3F80, 0xFFFD, 0x0001),
            (0x4000, 0xFF, 0x3FFF, 0xFFFD, 0x0001),
            (0xFFFF, 0x01, 0x0000, 0x1000, 0x1004),
            (0x0000, 0xFF, 0xFFFF, 0x1000, 0x1004),
        ]
    } else {
        &[
            (0x3647, 0, 0x3647, 0xFFFF, 0x0001),
            (0xFFFF, 0, 0xFFFF, 0x1000, 0x1002),
        ]
    };
    let mut bus = RecordingBus::new();
    for row in &ROWS {
        for (input, outcome) in [0x00, 0xFF, 0xA5, 0x5A].into_iter().zip(row.results) {
            for old in [0x00, 0xD7, 0x80, 0x04, 0x43, 0x92] {
                for &(base, displacement, address, pc, next_pc) in addresses {
                    let mut cpu = cpu(old);
                    cpu.registers_mut().pc = pc;
                    let opcode = row.memory[column(operation)];
                    let (cycles, mut trace) = if let Some(prefix) = prefix {
                        if prefix == 0xDD {
                            cpu.registers_mut().ix = base;
                        } else {
                            cpu.registers_mut().iy = base;
                        }
                        bus.load(pc, &[prefix, 0xCB, displacement, opcode]);
                        let mut trace = polls();
                        trace.extend([
                            read(pc, OP, prefix),
                            read(pc.wrapping_add(1), OP, 0xCB),
                            read(pc.wrapping_add(2), ARG, displacement),
                            read(pc.wrapping_add(3), ARG, opcode),
                        ]);
                        (if operation == Operation::Bit { 20 } else { 23 }, trace)
                    } else {
                        cpu.registers_mut().h = (base / 256) as u8;
                        cpu.registers_mut().l = (base % 256) as u8;
                        bus.load(pc, &[0xCB, opcode]);
                        let mut trace = polls();
                        trace.extend([read(pc, OP, 0xCB), read(pc.wrapping_add(1), OP, opcode)]);
                        (if operation == Operation::Bit { 12 } else { 15 }, trace)
                    };
                    bus.memory[usize::from(address)] = input;
                    let mut want_memory = bus.memory.clone();
                    let want = expected(cpu.clone(), operation, outcome.0, next_pc);
                    let value = result(operation, input, outcome);
                    trace.push(read(address, DATA, input));
                    if operation != Operation::Bit {
                        want_memory[usize::from(address)] = value;
                        trace.push(Event::Write {
                            address,
                            kind: DATA,
                            value,
                        });
                    }
                    let report = cpu.step(&mut bus);
                    check(&cpu, want, report, cycles, operation, outcome.0);
                    assert_eq!(bus.memory, want_memory);
                    assert_eq!(bus.take_log(), trace);
                }
            }
        }
    }
}

// Each class checks complete before/after CPU and memory, all polls/accesses,
// both BIT outcomes, every bit number, R/PC, and whole-instruction Report cost.
// Register PC FFFF -> 0001 and memory stream wraps are DERIVED (UM p. 2).
#[test]
fn bit_registers_um_243_244() {
    registers(Operation::Bit);
}
#[test]
fn set_registers_um_251_252() {
    registers(Operation::Set);
}
#[test]
fn res_registers_um_259_260() {
    registers(Operation::Res);
}
#[test]
fn bit_hl_um_245_246_label_slip() {
    memory(Operation::Bit, None);
}
#[test]
fn set_hl_um_253_254() {
    memory(Operation::Set, None);
}
#[test]
fn res_hl_um_259_260() {
    memory(Operation::Res, None);
}
#[test]
fn bit_ix_um_247_248() {
    memory(Operation::Bit, Some(0xDD));
}
#[test]
fn bit_iy_um_249_250_label_slip() {
    memory(Operation::Bit, Some(0xFD));
}
#[test]
fn set_ix_um_255_256() {
    memory(Operation::Set, Some(0xDD));
}
#[test]
fn set_iy_um_257_258() {
    memory(Operation::Set, Some(0xFD));
}
#[test]
fn res_ix_um_259_260() {
    memory(Operation::Res, Some(0xDD));
}
#[test]
fn res_iy_um_259_260() {
    memory(Operation::Res, Some(0xFD));
}
