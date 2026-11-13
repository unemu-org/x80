//! The 8085 Logical Group encodings and executor.
//!
//! Intel 205775-002 (January 1983), §5.6.3, pp. 5-9 to 5-12, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Design: plan §2.2; decision-10 (model-owned facts); decision-5 items 3/8;
//! decision-12 (boundary API). Source hashes match PROVENANCE.md §1.
//! Bus sequences: Table 5-1 p. 5-19 (F or F R), not Table 5-2's opcode
//! list. Ch. 2 §§2.3.1/2.3.2 pp. 2-7 to 2-10 define opcode/memory reads.
//! Independent execution witnesses: tests/i8085_logical.rs. Standard flags
//! and subtraction are this model's `alu`; AND has its own 8085 rule.
//! These rules reach `intel::logical` through FlagRules (decision-16 rules 3/5).

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::Bus;

/// Complete group list, in per-instruction page order.
/// Register-field exclusions live in `opcodes` (§5.2 p. 5-1).
pub(super) const ENCODINGS: &[Encoding] = &[
    // ANA r: 205775-002 p. 5-9.
    Encoding::new(b"10100SSS", 1, States::single(4)),
    // ANA M: 205775-002 p. 5-10.
    Encoding::new(b"10100110", 1, States::single(7)),
    // ANI data: 205775-002 p. 5-10.
    Encoding::new(b"11100110", 2, States::single(7)),
    // XRA r: 205775-002 p. 5-10.
    Encoding::new(b"10101SSS", 1, States::single(4)),
    // XRA M: 205775-002 p. 5-10.
    Encoding::new(b"10101110", 1, States::single(7)),
    // XRI data: 205775-002 p. 5-10.
    Encoding::new(b"11101110", 2, States::single(7)),
    // ORA r: 205775-002 p. 5-10.
    Encoding::new(b"10110SSS", 1, States::single(4)),
    // ORA M: 205775-002 p. 5-11.
    Encoding::new(b"10110110", 1, States::single(7)),
    // ORI data: 205775-002 p. 5-11.
    Encoding::new(b"11110110", 2, States::single(7)),
    // CMP r: 205775-002 p. 5-11.
    Encoding::new(b"10111SSS", 1, States::single(4)),
    // CMP M: 205775-002 p. 5-11.
    Encoding::new(b"10111110", 1, States::single(7)),
    // CPI data: 205775-002 p. 5-11.
    Encoding::new(b"11111110", 2, States::single(7)),
    // RLC: 205775-002 p. 5-11.
    Encoding::new(b"00000111", 1, States::single(4)),
    // RRC: 205775-002 p. 5-12.
    Encoding::new(b"00001111", 1, States::single(4)),
    // RAL: 205775-002 p. 5-12.
    Encoding::new(b"00010111", 1, States::single(4)),
    // RAR: 205775-002 p. 5-12.
    Encoding::new(b"00011111", 1, States::single(4)),
    // CMA: 205775-002 p. 5-12.
    Encoding::new(b"00101111", 1, States::single(4)),
    // CMC: 205775-002 p. 5-12.
    Encoding::new(b"00111111", 1, States::single(4)),
    // STC: 205775-002 p. 5-12.
    Encoding::new(b"00110111", 1, States::single(4)),
];

/// Executes only encodings dispatched to this group (decision-10).
/// The opcode map guarantees that the byte belongs to this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
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
