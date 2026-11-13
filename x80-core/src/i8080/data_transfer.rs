//! The 8080 Data Transfer Group: its encodings and its executor.
//!
//! Primary source: 98-153B Ch. 4, "Data Transfer Group", pp. 4-4 and 4-5,
//! checked against the page images. Field letters are those of
//! `Encoding` in `opcodes`.
//!
//! Design: decision-5 items 3, 7, 8; decision-6 and plan §2.2/§2.6.
//! Access order: 98-153B Ch. 2 machine-cycle table pp. 2-16/2-17,
//! notes 1, 6, 7 on p. 2-20. The caller has already fetched the opcode.
//! All instructions preserve flags (Ch. 4 p. 4-4, group introduction).
//! Independent witnesses: tests/i8080_data_transfer.rs, also checked against
//! Intel 9800301-04 (May 1981), Ch. 3 entries cited with the encodings below.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8080;
use crate::Bus;

/// The group's encodings, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // MOV r1,r2: 5 states, 98-153B Ch. 4 p. 4-4;
    // 9800301-04 Ch. 3 p. 3-36 (8080 total).
    Encoding::new(b"01DDDSSS", 1, States::single(5)),
    // MOV r,M: 7 states, 98-153B p. 4-4; 9800301-04 p. 3-37.
    Encoding::new(b"01DDD110", 1, States::single(7)),
    // MOV M,r: 7 states, 98-153B p. 4-4; 9800301-04 p. 3-36.
    Encoding::new(b"01110SSS", 1, States::single(7)),
    // MVI r,data: 7 states, opcode and data byte, 98-153B p. 4-4;
    // 9800301-04 p. 3-37.
    Encoding::new(b"00DDD110", 2, States::single(7)),
    // MVI M,data: 10 states, 98-153B p. 4-4; 9800301-04 p. 3-38.
    Encoding::new(b"00110110", 2, States::single(10)),
    // LXI rp,data 16: 10 states; opcode, low data, high data,
    // 98-153B p. 4-4; 9800301-04 p. 3-35.
    Encoding::new(b"00RP0001", 3, States::single(10)),
    // LDA addr: 13 states; opcode, low address, high address,
    // 98-153B p. 4-5; 9800301-04 pp. 3-32/3-33.
    Encoding::new(b"00111010", 3, States::single(13)),
    // STA addr: 13 states, 98-153B p. 4-5; 9800301-04 pp. 3-61/3-62.
    Encoding::new(b"00110010", 3, States::single(13)),
    // LHLD addr: 16 states, 98-153B p. 4-5; 9800301-04 p. 3-34.
    Encoding::new(b"00101010", 3, States::single(16)),
    // SHLD addr: 16 states, 98-153B p. 4-5; 9800301-04 pp. 3-58/3-59.
    Encoding::new(b"00100010", 3, States::single(16)),
    // LDAX rp, 00RP1010: "only register pairs rp=B (registers B and C) or
    // rp=D (registers D and E) may be specified", 7 states, 98-153B p. 4-5;
    // 9800301-04 p. 3-33.
    Encoding::new(b"00001010", 1, States::single(7)),
    Encoding::new(b"00011010", 1, States::single(7)),
    // STAX rp, 00RP0010, same restriction, 7 states, 98-153B p. 4-5;
    // 9800301-04 p. 3-62.
    Encoding::new(b"00000010", 1, States::single(7)),
    Encoding::new(b"00010010", 1, States::single(7)),
    // XCHG: 4 states, 98-153B p. 4-5; 9800301-04 p. 3-65.
    Encoding::new(b"11101011", 1, States::single(4)),
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
    crate::intel::data_transfer::execute(
        &mut super::view(cpu.registers_mut()),
        system,
        opcode,
        instruction.length,
    )
}
