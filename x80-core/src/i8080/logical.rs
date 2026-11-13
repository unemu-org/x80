//! The 8080 Logical Group: its encodings and its executor.
//!
//! Primary source: Intel 98-153B (September 1975), Ch. 4, "Logical Group",
//! pp. 4-8 to 4-11 (STC is
//! the last entry before "Branch Group" on p. 4-11), checked against the page
//! images. Field letters are those of `Encoding` in `opcodes`.
//!
//! Design: plan §2.2; decision-5 items 3/8; decision-8 (settled AND AC).
//! Source revisions and hashes are pinned in PROVENANCE.md §1. Standard
//! result flags, subtraction and AND live in `alu`, behind FlagRules. AND
//! AC follows 9800301-04 p. 1-12 despite the contrary ANI wording in
//! 98-153B p. 4-9; `alu::and` retains the decision-8 evidence.
//! The shared body is `intel::logical` (decision-16 rules 3/5).

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8080;
use crate::Bus;

/// The group's encodings, in page order. Each total is the entry's "States:"
/// value on the cited printed page (PDF page = 44 + printed page).
pub(super) const ENCODINGS: &[Encoding] = &[
    // ANA r, p. 4-8.
    Encoding::new(b"10100SSS", 1, States::single(4)),
    // ANA M, p. 4-8.
    Encoding::new(b"10100110", 1, States::single(7)),
    // ANI data: opcode and data byte, p. 4-9.
    Encoding::new(b"11100110", 2, States::single(7)),
    // XRA r, p. 4-9.
    Encoding::new(b"10101SSS", 1, States::single(4)),
    // XRA M, p. 4-9.
    Encoding::new(b"10101110", 1, States::single(7)),
    // XRI data, p. 4-9.
    Encoding::new(b"11101110", 2, States::single(7)),
    // ORA r, p. 4-9.
    Encoding::new(b"10110SSS", 1, States::single(4)),
    // ORA M, p. 4-9.
    Encoding::new(b"10110110", 1, States::single(7)),
    // ORI data, p. 4-9.
    Encoding::new(b"11110110", 2, States::single(7)),
    // CMP r, p. 4-9.
    Encoding::new(b"10111SSS", 1, States::single(4)),
    // CMP M, p. 4-10.
    Encoding::new(b"10111110", 1, States::single(7)),
    // CPI data, p. 4-10.
    Encoding::new(b"11111110", 2, States::single(7)),
    // RLC, p. 4-10.
    Encoding::new(b"00000111", 1, States::single(4)),
    // RRC, p. 4-10.
    Encoding::new(b"00001111", 1, States::single(4)),
    // RAL, p. 4-10.
    Encoding::new(b"00010111", 1, States::single(4)),
    // RAR, p. 4-10.
    Encoding::new(b"00011111", 1, States::single(4)),
    // CMA, p. 4-10.
    Encoding::new(b"00101111", 1, States::single(4)),
    // CMC, p. 4-11.
    Encoding::new(b"00111111", 1, States::single(4)),
    // STC, p. 4-11.
    Encoding::new(b"00110111", 1, States::single(4)),
];

/// Executes `opcode`, an instruction of this group.
///
/// The opcode map guarantees that `opcode` belongs to this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Path {
    crate::intel::logical::execute(
        &mut super::view(cpu.registers_mut()),
        system,
        opcode,
        instruction.length,
    )
}
