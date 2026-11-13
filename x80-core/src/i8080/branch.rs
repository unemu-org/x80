//! The 8080 Branch Group: its encodings and its executor.
//!
//! Design: plan §2.2/2.3, decision-5 items 3 and 8, decision-6 (8080 only).
//! Primary source: Intel 98-153B (September 1975), Ch. 4 "Branch Group",
//! pp. 4-11/4-12; Ch. 2 machine-cycle table pp. 2-18/2-19 and notes p. 2-20.
//! Second witness: Intel 9800301-04 (May 1981), Ch. 3 instruction entries.
//! All cited values were checked against the page images.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8080;
use crate::intel::branch::{self, BranchFacts};
use crate::Bus;

/// The group's encodings and per-entry state totals, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // 98-153B Ch. 4 JMP p. 4-11: 10 states; 9800301-04 Ch. 3 p. 3-28.
    Encoding::new(b"11000011", 3, States::single(10)),
    // Jcondition p. 4-11: 10 states on either path; second witness Ch. 3
    // Jcc pp. 3-26..3-32 (8080 totals, not the different 8085 totals).
    Encoding::new(
        b"11CCC010",
        3,
        States {
            untaken: 10,
            taken: 10,
        },
    ),
    // CALL p. 4-11: 17 states; second witness Ch. 3 pp. 3-8/3-9.
    Encoding::new(b"11001101", 3, States::single(17)),
    // Ccondition p. 4-12: 11/17 states; second witness Ch. 3 pp. 3-10/
    // 3-11, 3-14..3-18. 98-153B p. 4-15 note 2: flag-dependent totals.
    Encoding::new(
        b"11CCC100",
        3,
        States {
            untaken: 11,
            taken: 17,
        },
    ),
    // RET p. 4-12: 10 states; second witness Ch. 3 p. 3-48.
    Encoding::new(b"11001001", 1, States::single(10)),
    // Rcondition p. 4-12: 5/11 states; second witness Ch. 3 pp. 3-47,
    // 3-50..3-53, 3-55/3-56.
    Encoding::new(
        b"11CCC000",
        1,
        States {
            untaken: 5,
            taken: 11,
        },
    ),
    // RST n p. 4-12: 11 states; second witness Ch. 3 pp. 3-54/3-55.
    Encoding::new(b"11NNN111", 1, States::single(11)),
    // PCHL p. 4-12: 5 states; second witness Ch. 3 p. 3-42.
    Encoding::new(b"11101001", 1, States::single(5)),
];

/// Untaken Jcond/Ccond read both operands: 98-153B (September 1975),
/// Ch. 2 pp. 2-18/2-19 M2/M3; notes 12/13 p. 2-20 skip only the
/// transfer/stack writes. Second witness: 9800301-04 (May 1981),
/// Ch. 3 Jcc pp. 3-26 to 3-32, Ccc pp. 3-10/3-11, 3-14 to 3-18.
/// Kept model-owned under decision-16 rule 4.
const BRANCH_FACTS: BranchFacts = BranchFacts {
    untaken_operand_reads: 2,
};

/// Executes after opcode fetch (decision-5 item 3; decision-16 rules 1/2).
/// This group currently has no model-only opcodes; all delegate to the shared
/// executor. Model-specific arms belong here before that fallback.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Path {
    branch::execute(
        &mut super::view(cpu.registers_mut()),
        system,
        opcode,
        instruction.length,
        &BRANCH_FACTS,
    )
}
