//! General-Purpose Arithmetic and CPU Control Groups: encodings and execution (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 172–186.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-5, p. 48
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::{Decoded, Table};
use super::opcodes::{Encoding, Instruction, States};
use super::{alu, InterruptMode, Z80};
use crate::decode::Path;
use crate::Bus;

pub(super) const MAIN: &[Encoding] = &[
    // UM DAA, pp. 173–174.
    Encoding::new(b"00100111", 1, States::single(4)),
    // UM CPL, p. 175.
    Encoding::new(b"00101111", 1, States::single(4)),
    // UM CCF, p. 178.
    Encoding::new(b"00111111", 1, States::single(4)),
    // UM SCF, p. 179.
    Encoding::new(b"00110111", 1, States::single(4)),
    // UM NOP, p. 180.
    Encoding::new(b"00000000", 1, States::single(4)),
    // UM HALT, p. 181.
    Encoding::new(b"01110110", 1, States::single(4)),
    // UM DI, p. 182.
    Encoding::new(b"11110011", 1, States::single(4)),
    // UM EI, p. 183.
    Encoding::new(b"11111011", 1, States::single(4)),
];

pub(super) const ED: &[Encoding] = &[
    // UM NEG, p. 176.
    Encoding::new(b"01000100", 2, States::single(8)),
    // UM IM 0, p. 184.
    Encoding::new(b"01000110", 2, States::single(8)),
    // UM IM 1, p. 185.
    Encoding::new(b"01010110", 2, States::single(8)),
    // UM IM 2, p. 186.
    Encoding::new(b"01011110", 2, States::single(8)),
];

/// PC/R and complete costs belong to the dispatcher (decision-13 rule 4;
/// decision-15 rules 5–7). This group has no operand or data accesses.
/// Primary: UM008011-0816 rev. 11, pp. 173–186; second cross-read:
/// 03-0029-01 (1977), Table 7.0-5 p. 48. Independent instruction witnesses:
/// tests/z80_general_control.rs, z80_daa.rs and z80_machine_control.rs.
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    _system: &mut S,
    _instruction: &Instruction,
    decoded: &Decoded,
) -> Path {
    match (decoded.table, decoded.opcode) {
        // UM DAA pp. 173–174; detailed readings below and in PROVENANCE §1.
        (Table::Main, 0x27) => decimal_adjust(cpu),
        // UM CPL p. 175: invert A, set H/N, preserve S/Z/P-V/C.
        (Table::Main, 0x2F) => {
            let registers = cpu.registers_mut();
            registers.a = !registers.a;
            registers.flags.half_carry = true;
            registers.flags.subtract = true;
        }
        // UM NEG pp. 176–177: 0-A, H is borrow from bit 4, P/V iff
        // old A=80h and C iff old A!=00h. Only documented ED 44 is mapped
        // (decision-14 rules 1–2). subtract supplies these flag equations.
        (Table::Ed, 0x44) => {
            let registers = cpu.registers_mut();
            (registers.a, registers.flags) = alu::subtract(0, registers.a, false);
        }
        // UM CCF p. 178 explicitly copies old C to H; TM p. 48 prints
        // H unknown. Existing PROVENANCE §1 H-definedness row and
        // decision-14 rule 6 follow the primary, including H in witnesses.
        (Table::Main, 0x3F) => {
            let flags = &mut cpu.registers_mut().flags;
            flags.half_carry = flags.carry;
            flags.carry = !flags.carry;
            flags.subtract = false;
        }
        // UM SCF p. 179: C=1, H=N=0, S/Z/P-V unchanged.
        (Table::Main, 0x37) => {
            let flags = &mut cpu.registers_mut().flags;
            flags.carry = true;
            flags.half_carry = false;
            flags.subtract = false;
        }
        // UM NOP p. 180: only normal dispatcher/loop bookkeeping changes.
        (Table::Main, 0x00) => {}
        // UM HALT p. 181: entry costs four states, flags unaffected.
        // decision-15 rule 7 keeps post-HALT PC (derived) and decision-2's
        // prompt Halt departure: no idle instruction, fetch or refresh.
        (Table::Main, 0x76) => cpu.latches_mut().halted = true,
        // UM DI p. 182 / EI p. 183: both IFFs clear/set immediately.
        // The loop consumes OLD delay before execute (rule 7). DI cancels
        // and EI/EI re-arms: UNVERIFIED edge-case model, not source claims.
        (Table::Main, 0xF3) => {
            let latches = cpu.latches_mut();
            latches.iff1 = false;
            latches.iff2 = false;
            latches.ei_delay = false;
        }
        (Table::Main, 0xFB) => {
            let latches = cpu.latches_mut();
            latches.iff1 = true;
            latches.iff2 = true;
            latches.ei_delay = true;
        }
        // UM IM 0/1/2 pp. 184/185/186: mode only, flags unaffected.
        (Table::Ed, 0x46) => cpu.latches_mut().interrupt_mode = InterruptMode::Mode0,
        (Table::Ed, 0x56) => cpu.latches_mut().interrupt_mode = InterruptMode::Mode1,
        (Table::Ed, 0x5E) => cpu.latches_mut().interrupt_mode = InterruptMode::Mode2,
        // The fixed opcode map assigns only the twelve entries above.
        _ => {}
    }
    Path::Untaken
}

/// DAA: UM008011-0816 pp. 173–174; TM Table 7.0-5 p. 48.
/// The digit/H/C selection below agrees with every literal p. 173 row;
/// tests/z80_daa.rs transcribes all thirteen rows without widening ranges.
/// LIBRARY EXTENSION, UNVERIFIED outside that table (which covers 684/2048 inputs):
/// choose 06h if input low digit >9 or H, and 60h if input A>99h or C;
/// add/subtract according to N. C records selection of 60h. The primary
/// documents only table cases; this extension claims no silicon behaviour.
/// DERIVED H-after reading, including table cases: use the adjustment's
/// bit-4 carry/borrow (TM section 6 p. 39). UM p. 174 points to p. 173,
/// whose table has H-before but no H-after column; TM p. 48 marks H
/// result-dependent. This library rule is not explicitly stated for DAA.
/// See PROVENANCE section 1 and the separate literal extension witness.
fn decimal_adjust(cpu: &mut Z80) {
    let registers = cpu.registers_mut();
    let previous = registers.flags;
    let a = registers.a;
    let low = previous.half_carry || a & 0x0F > 9;
    let carry = previous.carry || a > 0x99;
    let correction = if low { 0x06 } else { 0 } | if carry { 0x60 } else { 0 };
    let (value, mut flags) = if previous.subtract {
        alu::subtract(a, correction, false)
    } else {
        alu::add(a, correction, false)
    };
    // UM p. 174: result S/Z/even parity, N unchanged, C from the table
    // rather than the correction's binary carry (e.g. input C is retained).
    flags.parity_overflow = value.count_ones() & 1 == 0;
    flags.subtract = previous.subtract;
    flags.carry = carry;
    registers.a = value;
    registers.flags = flags;
}
