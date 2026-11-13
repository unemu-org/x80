//! The 8080 Arithmetic Group: its encodings and its executor.
//!
//! Primary source: 98-153B Ch. 4, "Arithmetic Group", pp. 4-6 to 4-8 (DAA is
//! the last entry before "Logical Group" on p. 4-8), checked against the page
//! images. Field letters are those of `Encoding` in `opcodes`.
//!
//! Design: plan §2.2; decision-5 items 3 and 8; decision-6. Arithmetic
//! delegates to `intel::arithmetic`, including DAA. Flag definitions live in
//! `alu`; the derived DAA carry policy remains below (decision-16 rule 6).

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8080;
use crate::intel::arithmetic::DaaCarry;
use crate::Bus;

/// The group's encodings, in page order. Each total is the entry's
/// "States:" value on the cited printed page (PDF page = 44 + printed page).
pub(super) const ENCODINGS: &[Encoding] = &[
    // ADD r, p. 4-6.
    Encoding::new(b"10000SSS", 1, States::single(4)),
    // ADD M, p. 4-6.
    Encoding::new(b"10000110", 1, States::single(7)),
    // ADI data: opcode and data byte, p. 4-6.
    Encoding::new(b"11000110", 2, States::single(7)),
    // ADC r, p. 4-6.
    Encoding::new(b"10001SSS", 1, States::single(4)),
    // ADC M, p. 4-6.
    Encoding::new(b"10001110", 1, States::single(7)),
    // ACI data, p. 4-6.
    Encoding::new(b"11001110", 2, States::single(7)),
    // SUB r, p. 4-6.
    Encoding::new(b"10010SSS", 1, States::single(4)),
    // SUB M, p. 4-7.
    Encoding::new(b"10010110", 1, States::single(7)),
    // SUI data, p. 4-7.
    Encoding::new(b"11010110", 2, States::single(7)),
    // SBB r, p. 4-7.
    Encoding::new(b"10011SSS", 1, States::single(4)),
    // SBB M, p. 4-7.
    Encoding::new(b"10011110", 1, States::single(7)),
    // SBI data, p. 4-7.
    Encoding::new(b"11011110", 2, States::single(7)),
    // INR r, p. 4-7.
    Encoding::new(b"00DDD100", 1, States::single(5)),
    // INR M, p. 4-7.
    Encoding::new(b"00110100", 1, States::single(10)),
    // DCR r, p. 4-7.
    Encoding::new(b"00DDD101", 1, States::single(5)),
    // DCR M, p. 4-8.
    Encoding::new(b"00110101", 1, States::single(10)),
    // INX rp, p. 4-8.
    Encoding::new(b"00RP0011", 1, States::single(5)),
    // DCX rp, p. 4-8.
    Encoding::new(b"00RP1011", 1, States::single(5)),
    // DAD rp, p. 4-8.
    Encoding::new(b"00RP1001", 1, States::single(10)),
    // DAA, p. 4-8.
    Encoding::new(b"00100111", 1, States::single(4)),
];

/// Derived CY rule, maintainer decision (2026-11-12), plan §2.2:
/// CY after = CY before OR step-1 carry out of bit 7 OR step-2 carry out
/// of bit 7. Both retention and step-1 carry propagation are derived, not
/// stated; falsifiable by the oracle run (x80-ecy.17).
/// Intel 98-153B Ch. 4 "Carry", p. 4-2 (PDF 46), is read per step:
/// step 1's +6 carry sets CY before step 2 tests CY (DAA p. 4-8, PDF 52).
/// Thus FAH + 6 = 00H with CY=1, then +60H = 60H with CY=1.
/// Intel 9800301-04 Ch. 6 "Decimal Addition", p. 6-13 (PDF 179),
/// requires the correct two-digit decimal result and carry for higher bytes;
/// pp. 6-12 to 6-14 (PDF 178-180) require retention for 0099 + 0099.
/// Contrary readings: apply p. 4-2's "otherwise it is reset" once to the
/// whole instruction, or use literal CY in OR high-adjustment carry,
/// excluding step 1's carry. Neither is the adopted per-step reading.
/// Witnesses: tests/i8080_daa.rs (exhaustive two-step transcription,
/// literal edge cases and decimal examples). Matching code does not upgrade
/// this derivation to an explicit primary statement (decision-16 rule 6).
const DAA_CARRY: DaaCarry = DaaCarry {
    retain_incoming: true,
    propagate_low: true,
};

/// Executes an encoding that the opcode map assigns to this group.
/// No model-only arithmetic opcodes are currently admitted (decision-16 rule 1).
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Path {
    crate::intel::arithmetic::execute(
        &mut super::view(cpu.registers_mut()),
        system,
        opcode,
        instruction.length,
        DAA_CARRY,
    )
}
