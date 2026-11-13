//! The 8080 Stack, I/O and Machine Control Group: its encodings and its
//! executor.
//!
//! Primary source: Intel 98-153B (September 1975), Ch. 4, "Stack, I/O, and
//! Machine Control Group", pp. 4-13 and 4-14, checked against page images.
//! Field letters are those of `Encoding` in `opcodes`.
//!
//! Design: decision-5 items 3, 7 and 8 (bus kinds and execution), plan §2.2
//! (registers and flags); items 4/5 cover EI delay and halt boundaries.
//! Source revisions and hashes are pinned in PROVENANCE.md; secondary citations
//! below refer to Intel 9800301-04 (May 1981), Ch. 3, 8080 state counts.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8080;
use crate::intel::stack_io::{self, XthlOrder};
use crate::Bus;

/// NOP, 00000000 (98-153B p. 4-14).
const NOP: u8 = 0b0000_0000;

/// The group's encodings, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // PUSH rp, 11RP0101: "Register pair rp = SP may not be specified",
    // p. 4-13: 11 states. Listed for B, D and H.
    // Second witness: 9800301-04 Ch. 3 p. 3-44, PUSH.
    Encoding::new(b"11000101", 1, States::single(11)),
    Encoding::new(b"11010101", 1, States::single(11)),
    Encoding::new(b"11100101", 1, States::single(11)),
    // PUSH PSW, p. 4-13: 11 states; 9800301-04 Ch. 3 p. 3-45.
    Encoding::new(b"11110101", 1, States::single(11)),
    // POP rp, 11RP0001, with the same restriction, p. 4-13: 10 states;
    // 9800301-04 Ch. 3 p. 3-43.
    Encoding::new(b"11000001", 1, States::single(10)),
    Encoding::new(b"11010001", 1, States::single(10)),
    Encoding::new(b"11100001", 1, States::single(10)),
    // POP PSW, p. 4-13: 10 states; 9800301-04 Ch. 3 p. 3-43.
    Encoding::new(b"11110001", 1, States::single(10)),
    // XTHL, p. 4-14: 18 states; 9800301-04 Ch. 3 p. 3-69.
    Encoding::new(b"11100011", 1, States::single(18)),
    // SPHL, p. 4-14: 5 states; 9800301-04 Ch. 3 p. 3-61.
    Encoding::new(b"11111001", 1, States::single(5)),
    // IN port: opcode and port byte, p. 4-14: 10 states;
    // 9800301-04 Ch. 3 pp. 3-24, 3-25.
    Encoding::new(b"11011011", 2, States::single(10)),
    // OUT port, p. 4-14: 10 states; 9800301-04 Ch. 3 p. 3-41.
    Encoding::new(b"11010011", 2, States::single(10)),
    // EI, p. 4-14: 4 states; 9800301-04 Ch. 3 p. 3-23.
    Encoding::new(b"11111011", 1, States::single(4)),
    // DI, p. 4-14: 4 states; 9800301-04 Ch. 3 pp. 3-22/3-23.
    Encoding::new(b"11110011", 1, States::single(4)),
    // HLT, p. 4-14: 7 states; 9800301-04 Ch. 3 p. 3-24 (8080).
    Encoding::new(b"01110110", 1, States::single(7)),
    // NOP, p. 4-14: "Cycles: 1, States: 4". Second witness: 9800301-04
    // p. A-4, NOP, 8080 column: 4.
    Encoding::new(b"00000000", 1, States::single(4)),
];

// XTHL address order: 98-153B (September 1975) Ch. 2 pp. 2-18/2-19,
// notes 15/16 p. 2-20: read SP, SP+1, write SP+1, SP, then replace HL.
// Byte mapping: Ch. 4 p. 4-14; second witness: 9800301-04 (May 1981)
// Ch. 3 p. 3-69. Design: decision-16 rule 6; witnesses: i8080_stack_io_control.
const XTHL_ORDER: XthlOrder = XthlOrder {
    reads: [0, 1],
    writes: [1, 0],
};

/// Executes `opcode`, an instruction of this group.
///
/// The opcode map guarantees that `opcode` belongs to this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Path {
    match opcode {
        // decision-5 item 4; 98-153B Ch. 4 EI p. 4-14 and 9800301-04
        // Ch. 3 EI p. 3-23: enabled following the next instruction.
        // INTE and delay together represent this; run expires the delay.
        0xFB => {
            let latches = cpu.latches_mut();
            latches.interrupt_enable = true;
            latches.ei_delay = true;
        }
        // Same primary page, DI; second witness pp. 3-22/3-23:
        // interrupts disabled immediately following DI. Cancelling the
        // internal delay is an unverified model choice for EI;DI.
        0xF3 => {
            let latches = cpu.latches_mut();
            latches.interrupt_enable = false;
            latches.ei_delay = false;
        }
        // decision-5 item 5; 98-153B Ch. 4 HLT p. 4-14, Ch. 2
        // Halt Sequences p. 2-13: stopped, registers/flags unchanged.
        // 9800301-04 Ch. 3 HLT p. 3-24 explicitly leaves PC at the
        // next instruction. run returns Halt at the next boundary.
        0x76 => cpu.latches_mut().halted = true,
        // 98-153B Ch. 4 p. 4-14 NOP: no operation; registers and flags
        // unaffected, apart from sequential PC advancement below.
        NOP => {}
        _ => {
            return stack_io::execute(
                &mut super::view(cpu.registers_mut()),
                system,
                opcode,
                instruction.length,
                XTHL_ORDER,
            );
        }
    }
    // Sequential PC advancement: 98-153B Ch. 4 p. 4-2 and the fetch cycles
    // in Ch. 2 pp. 2-16, 2-18. PC/SP arithmetic wraps by their 16-bit width
    // (Ch. 2 p. 2-2); wrapping itself is derived, not stated explicitly.
    let registers = cpu.registers_mut();
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    // All entries have one total; both Path slots select the same cost.
    Path::Untaken
}
