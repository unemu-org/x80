//! The 8085 Branch encodings and executor.
//!
//! Intel 205775-002 (January 1983), §5.6.4, pp. 5-13 to 5-15, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witnesses: `tests/i8085_branch.rs` and `opcodes/tests.rs`.
//! Intel 9800301-04 (May 1981), Ch. 3 instruction entries below, supplies
//! the second readings. Source hashes match PROVENANCE.md §1.
//! Table 5-1 p. 5-19 (PDF 104) gives cycle sequences; Table 5-2 p. 5-20
//! (PDF 105) gives the opcode list. Ch. 2 §2.3.1 p. 2-7 (PDF 30) describes
//! stream reads; §2.2.3 p. 2-2 (PDF 25) the stack; Fig. 2-18 p. 2-14
//! (PDF 37) fixes high-then-low stack writes.
//! Design: plan §2.2/2.3; decisions 10/11; decision-5 item 3 and
//! decision-12 rules 4/5 (explicit return PC/vector for later interrupts).

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::intel::branch::{self, BranchFacts};
use crate::Bus;

/// Complete group list, in per-instruction page order.
/// CCC takes all eight conditions (205775-002 p. 5-13); NNN takes
/// all eight restarts (p. 5-14). Costs are untaken/taken respectively;
/// 9800301-04 p. A-1 explains that order. Its CPE 9/17 disagrees with
/// this primary source's 9/18, pinned in `opcodes/tests.rs`.
pub(super) const ENCODINGS: &[Encoding] = &[
    // JMP addr: 205775-002 p. 5-13.
    Encoding::new(b"11000011", 3, States::single(10)),
    // Jcondition addr: 205775-002 p. 5-13.
    Encoding::new(
        b"11CCC010",
        3,
        States {
            untaken: 7,
            taken: 10,
        },
    ),
    // CALL addr: 205775-002 p. 5-13.
    Encoding::new(b"11001101", 3, States::single(18)),
    // Ccondition addr: 205775-002 p. 5-14.
    Encoding::new(
        b"11CCC100",
        3,
        States {
            untaken: 9,
            taken: 18,
        },
    ),
    // RET: 205775-002 p. 5-14.
    Encoding::new(b"11001001", 1, States::single(10)),
    // Rcondition: 205775-002 p. 5-14.
    Encoding::new(
        b"11CCC000",
        1,
        States {
            untaken: 6,
            taken: 12,
        },
    ),
    // RST n: 205775-002 p. 5-14.
    Encoding::new(b"11NNN111", 1, States::single(12)),
    // PCHL: 205775-002 p. 5-15.
    Encoding::new(b"11101001", 1, States::single(6)),
];

/// Untaken Jcond/Ccond read only byte 2: 205775-002 (January 1983),
/// Table 5-1 p. 5-19 (F R / S R), Ch. 2 §2.3.1 p. 2-7 (stream order).
/// The extra untaken read footnotes apply only to the 8080A. Second witness:
/// 9800301-04 (May 1981), Ch. 3 pp. 3-10, 3-27 (two cycles).
/// Kept model-owned under decision-16 rule 4.
const BRANCH_FACTS: BranchFacts = BranchFacts {
    untaken_operand_reads: 1,
};

/// Executes after opcode fetch (decision-5 item 3; decision-16 rules 1/2).
/// This group currently has no model-only opcodes; all delegate to the shared
/// executor. Model-specific arms belong here before that fallback.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
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
