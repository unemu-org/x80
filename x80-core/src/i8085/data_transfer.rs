//! The 8085 Data Transfer encodings and executor.
//!
//! Intel 205775-002 (January 1983), §5.6.1, pp. 5-4 to 5-6, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witnesses: `tests/i8085_data_transfer.rs`, cross-read with
//! Intel 9800301-04 (May 1981), Ch. 3 entries cited below. Source hashes
//! match PROVENANCE.md §1; all cited values checked on rendered page images.
//! Bus order: Table 5-1 p. 5-19 (machine cycles); Table 5-2 p. 5-20
//! lists opcodes, not cycles. Ch. 2 §§2.3.1-2.3.3 pp. 2-8 to 2-12
//! distinguishes opcode fetch, memory read and memory write. OperandFetch
//! versus Data is the decision-5 item 3 API distinction.
//! Every instruction preserves flags (§5.6.1 p. 5-4).
//! Design: decision-10, decision-11, decision-12 rules 1/10;
//! decision-5 items 3/7/8; plan §2.2/§2.6.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::Bus;

/// Complete group list, in per-instruction page order.
/// LDAX/STAX permit only B and D pairs (205775-002 pp. 5-5/5-6).
/// Register-field exclusions live in `opcodes` (§5.2 p. 5-1).
pub(super) const ENCODINGS: &[Encoding] = &[
    // MOV r1,r2: 205775-002 p. 5-4; 9800301-04 p. 3-36 (8085 total).
    Encoding::new(b"01DDDSSS", 1, States::single(4)),
    // MOV r,M: 205775-002 p. 5-4; 9800301-04 pp. 3-36/3-37.
    Encoding::new(b"01DDD110", 1, States::single(7)),
    // MOV M,r: 205775-002 p. 5-4; 9800301-04 p. 3-36.
    Encoding::new(b"01110SSS", 1, States::single(7)),
    // MVI r,data: 205775-002 p. 5-4; 9800301-04 p. 3-37.
    Encoding::new(b"00DDD110", 2, States::single(7)),
    // MVI M,data: 205775-002 p. 5-4; 9800301-04 p. 3-38.
    Encoding::new(b"00110110", 2, States::single(10)),
    // LXI rp,data16: 205775-002 p. 5-5; 9800301-04 p. 3-35.
    Encoding::new(b"00RP0001", 3, States::single(10)),
    // LDA addr: 205775-002 p. 5-5; 9800301-04 pp. 3-32/3-33.
    Encoding::new(b"00111010", 3, States::single(13)),
    // STA addr: 205775-002 p. 5-5; 9800301-04 pp. 3-61/3-62.
    Encoding::new(b"00110010", 3, States::single(13)),
    // LHLD addr: 205775-002 p. 5-5; 9800301-04 p. 3-34.
    Encoding::new(b"00101010", 3, States::single(16)),
    // SHLD addr: 205775-002 p. 5-5; 9800301-04 pp. 3-58/3-59.
    Encoding::new(b"00100010", 3, States::single(16)),
    // LDAX B: 205775-002 p. 5-5; 9800301-04 p. 3-33.
    // 231718-001 Table 6 p. 16-28 misprints "LDAX B" as "00101010".
    // Use "00RP1010", RP=00 here, from 205775-002 pp. 5-1/5-5;
    // Tables 5-1/5-2 pp. 5-19/5-20 and 9800301-04 p. 3-33 agree on 0A.
    Encoding::new(b"00001010", 1, States::single(7)),
    // LDAX D: 205775-002 p. 5-5; 9800301-04 p. 3-33.
    Encoding::new(b"00011010", 1, States::single(7)),
    // STAX B: 205775-002 p. 5-6; 9800301-04 p. 3-62.
    Encoding::new(b"00000010", 1, States::single(7)),
    // STAX D: 205775-002 p. 5-6; 9800301-04 p. 3-62.
    Encoding::new(b"00010010", 1, States::single(7)),
    // XCHG: 205775-002 p. 5-6; 9800301-04 p. 3-65.
    Encoding::new(b"11101011", 1, States::single(4)),
];

/// Executes an opcode already fetched and dispatched to this group.
/// The opcode map guarantees that the byte belongs to this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Path {
    crate::intel::data_transfer::execute(
        &mut super::view(cpu.registers_mut()),
        system,
        opcode,
        instruction.length,
    )
}
