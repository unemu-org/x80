//! The 8085 Arithmetic encodings and executor (plan §2.2; decision-10).
//!
//! Intel 205775-002 (January 1983), §5.6.2, pp. 5-6 to 5-9, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Bus order: Table 5-1 p. 5-19 (PDF 104); Table 5-2 p. 5-20 is
//! the opcode list. Ch. 2 pp. 2-8 to 2-12 (PDF 31-35) defines OF/MR/MW
//! and bus idle. Access kinds and completed-boundary state: decision-12,
//! extending decision-5 items 3/8. No wait states or HOLD are modelled.
//! All cited source values checked on rendered page images. The executor
//! delegates to `intel::arithmetic`; DAA carry remains derived per model
//! (decision-16 rule 6), as recorded below and in PROVENANCE.md §1.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::intel::arithmetic::DaaCarry;
use crate::Bus;

/// Complete group list, in per-instruction page order.
/// Register-field exclusions live in `opcodes` (§5.2 p. 5-1).
pub(super) const ENCODINGS: &[Encoding] = &[
    // ADD r: 205775-002 p. 5-6.
    Encoding::new(b"10000SSS", 1, States::single(4)),
    // ADD M: 205775-002 p. 5-6.
    Encoding::new(b"10000110", 1, States::single(7)),
    // ADI data: 205775-002 p. 5-6.
    Encoding::new(b"11000110", 2, States::single(7)),
    // ADC r: 205775-002 p. 5-6.
    Encoding::new(b"10001SSS", 1, States::single(4)),
    // ADC M: 205775-002 p. 5-7.
    Encoding::new(b"10001110", 1, States::single(7)),
    // ACI data: 205775-002 p. 5-7.
    Encoding::new(b"11001110", 2, States::single(7)),
    // SUB r: 205775-002 p. 5-7.
    Encoding::new(b"10010SSS", 1, States::single(4)),
    // SUB M: 205775-002 p. 5-7.
    Encoding::new(b"10010110", 1, States::single(7)),
    // SUI data: 205775-002 p. 5-7.
    Encoding::new(b"11010110", 2, States::single(7)),
    // SBB r: 205775-002 p. 5-7.
    Encoding::new(b"10011SSS", 1, States::single(4)),
    // SBB M: 205775-002 p. 5-8.
    Encoding::new(b"10011110", 1, States::single(7)),
    // SBI data: 205775-002 p. 5-8.
    Encoding::new(b"11011110", 2, States::single(7)),
    // INR r: 205775-002 p. 5-8.
    Encoding::new(b"00DDD100", 1, States::single(4)),
    // INR M: 205775-002 p. 5-8.
    Encoding::new(b"00110100", 1, States::single(10)),
    // DCR r: 205775-002 p. 5-8.
    Encoding::new(b"00DDD101", 1, States::single(4)),
    // DCR M: 205775-002 p. 5-8.
    Encoding::new(b"00110101", 1, States::single(10)),
    // INX rp: 205775-002 p. 5-9.
    Encoding::new(b"00RP0011", 1, States::single(6)),
    // DCX rp: 205775-002 p. 5-9.
    Encoding::new(b"00RP1011", 1, States::single(6)),
    // DAD rp: 205775-002 p. 5-9.
    Encoding::new(b"00RP1001", 1, States::single(10)),
    // DAA: 205775-002 p. 5-9.
    Encoding::new(b"00100111", 1, States::single(4)),
];

/// Derived for this model, NOT explicit in 205775-002 (January 1983),
/// §5.6.2 p. 5-9 (PDF 94): retain incoming CY and apply §5.5 p. 5-3's
/// CY rule per adjustment. Step 1's bit-7 carry triggers step 2, and either
/// step's carry sets final CY. Intel 9800301-04 (May 1981), Ch. 3 p. 3-19
/// (PDF 73), repeats the steps without settling retention. Its decimal
/// addition routine needs that carry for higher bytes (Ch. 6 pp. 6-12 to
/// 6-14, PDF 178-180). As re-derived for the PROVENANCE.md §1 8085 DAA row:
/// 99+99 needs 32H,CY=AC=1 -> 98H,CY=1; 95+65 needs FAH -> 00H with
/// carry -> 60H,CY=1. Maintainer-authorized rule (x80-4gk.12), not a new
/// hardware claim. Witnesses: tests/i8085_daa.rs, exhaustive two-step
/// transcription, literal examples and all decimal sums. The 8085 oracle
/// qualification remains in PROVENANCE.md §2 (decision-16 rule 6).
const DAA_CARRY: DaaCarry = DaaCarry {
    retain_incoming: true,
    propagate_low: true,
};

/// Executes an encoding that the opcode map assigns to this group.
/// No model-only arithmetic opcodes are currently admitted (decision-16 rule 1).
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
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
