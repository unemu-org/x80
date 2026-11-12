//! The 8080 Stack, I/O and Machine Control Group: its encodings and its
//! executor.
//!
//! Primary source: Intel 98-153B (September 1975), Ch. 4, "Stack, I/O, and
//! Machine Control Group", pp. 4-13 and 4-14, checked against page images.
//! Field letters are those of `Encoding` in `opcodes`.
//!
//! Design: decision-5 items 3, 7 and 8 (bus kinds and execution), plan §2.2
//! (registers and flags). EI, DI and HLT remain Unsupported for x80-ecy.13.
//! Source revisions and hashes are pinned in PROVENANCE.md; secondary citations
//! below refer to Intel 9800301-04 (May 1981), Ch. 3, 8080 state counts.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8080;
use crate::{Bus, MemoryAccess};

/// NOP, 00000000 (98-153B p. 4-14).
const NOP: u8 = 0b0000_0000;

/// The group's encodings, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // PUSH rp, 11RP0101: "Register pair rp = SP may not be specified",
    // p. 4-13: 11 states. Listed for B, D and H.
    // Second witness: 9800301-04 Ch. 3 p. 3-44, PUSH.
    Encoding::new(b"11000101", 1).with_states(States::single(11)),
    Encoding::new(b"11010101", 1).with_states(States::single(11)),
    Encoding::new(b"11100101", 1).with_states(States::single(11)),
    // PUSH PSW, p. 4-13: 11 states; 9800301-04 Ch. 3 p. 3-45.
    Encoding::new(b"11110101", 1).with_states(States::single(11)),
    // POP rp, 11RP0001, with the same restriction, p. 4-13: 10 states;
    // 9800301-04 Ch. 3 p. 3-43.
    Encoding::new(b"11000001", 1).with_states(States::single(10)),
    Encoding::new(b"11010001", 1).with_states(States::single(10)),
    Encoding::new(b"11100001", 1).with_states(States::single(10)),
    // POP PSW, p. 4-13: 10 states; 9800301-04 Ch. 3 p. 3-43.
    Encoding::new(b"11110001", 1).with_states(States::single(10)),
    // XTHL, p. 4-14: 18 states; 9800301-04 Ch. 3 p. 3-69.
    Encoding::new(b"11100011", 1).with_states(States::single(18)),
    // SPHL, p. 4-14: 5 states; 9800301-04 Ch. 3 p. 3-61.
    Encoding::new(b"11111001", 1).with_states(States::single(5)),
    // IN port: opcode and port byte, p. 4-14: 10 states;
    // 9800301-04 Ch. 3 pp. 3-24, 3-25.
    Encoding::new(b"11011011", 2).with_states(States::single(10)),
    // OUT port, p. 4-14: 10 states; 9800301-04 Ch. 3 p. 3-41.
    Encoding::new(b"11010011", 2).with_states(States::single(10)),
    // EI, p. 4-14.
    Encoding::new(b"11111011", 1),
    // DI, p. 4-14.
    Encoding::new(b"11110011", 1),
    // HLT, p. 4-14.
    Encoding::new(b"01110110", 1),
    // NOP, p. 4-14: "Cycles: 1, States: 4". Second witness: 9800301-04
    // p. A-4, NOP, 8080 column: 4.
    Encoding::new(b"00000000", 1).with_states(States::single(4)),
];

/// Executes `opcode`, an instruction of this group.
///
/// Returns `None`, having changed nothing and accessed nothing, for an opcode
/// the group does not execute yet (decision-5 item 7).
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Option<Path> {
    let registers = cpu.registers_mut();
    match opcode {
        // 98-153B Ch. 4 p. 4-13 PUSH rp/PSW: high byte at SP-1, low at
        // SP-2, SP -= 2, no flags changed. Pair codes: p. 4-3. PSW uses
        // the centralized FLAG WORD image (p. 4-13), including fixed bits.
        // Ch. 2 pp. 2-18, 2-19: M2 writes high, M3 writes low; p. 2-20
        // note 16 identifies both as stack writes (decision-5 item 3).
        0xC5 | 0xD5 | 0xE5 | 0xF5 => {
            let word = match opcode {
                0xC5 => registers.bc(),
                0xD5 => registers.de(),
                0xE5 => registers.hl(),
                _ => registers.psw(),
            };
            let [high, low] = word.to_be_bytes();
            system.write(registers.sp.wrapping_sub(1), high, MemoryAccess::Stack);
            system.write(registers.sp.wrapping_sub(2), low, MemoryAccess::Stack);
            registers.sp = registers.sp.wrapping_sub(2);
        }
        // 98-153B Ch. 4 p. 4-13 POP rp/PSW: low from SP, high from SP+1,
        // SP += 2; only POP PSW changes flags. Its transfer list assigns
        // bits 0,2,4,6,7 only; set_psw drops filler bits 1,3,5 (plan §2.2).
        // Ch. 2 pp. 2-18, 2-19: M2 reads low, M3 reads high; p. 2-20
        // note 15 identifies stack reads (decision-5 item 3).
        0xC1 | 0xD1 | 0xE1 | 0xF1 => {
            let low = system.read(registers.sp, MemoryAccess::Stack);
            let high = system.read(registers.sp.wrapping_add(1), MemoryAccess::Stack);
            let word = u16::from_be_bytes([high, low]);
            match opcode {
                0xC1 => registers.set_bc(word),
                0xD1 => registers.set_de(word),
                0xE1 => registers.set_hl(word),
                _ => registers.set_psw(word),
            }
            registers.sp = registers.sp.wrapping_add(2);
        }
        // 98-153B Ch. 4 p. 4-14 XTHL: L <-> [SP], H <-> [SP+1]; flags
        // unchanged. Ch. 2 pp. 2-18, 2-19: M2 reads SP, M3 reads SP+1,
        // M4 writes H at SP+1, M5 writes L at SP, then WZ -> HL. Notes
        // 15-16 (p. 2-20): all four are Stack accesses (decision-5 item 3).
        // SP is unchanged; also explicit in 9800301-04 Ch. 3 p. 3-69.
        0xE3 => {
            let low = system.read(registers.sp, MemoryAccess::Stack);
            let high = system.read(registers.sp.wrapping_add(1), MemoryAccess::Stack);
            system.write(
                registers.sp.wrapping_add(1),
                registers.h,
                MemoryAccess::Stack,
            );
            system.write(registers.sp, registers.l, MemoryAccess::Stack);
            registers.h = high;
            registers.l = low;
        }
        // 98-153B Ch. 4 p. 4-14 SPHL: SP <- HL, no flags changed.
        // Ch. 2 p. 2-16: only the opcode-fetch cycle, no memory data access.
        0xF9 => registers.sp = registers.hl(),
        // 98-153B Ch. 4 p. 4-14 IN/OUT: byte 2 is the port, IN loads A,
        // OUT sends A, neither changes flags. Ch. 2 p. 2-9 Figs. 2-6/2-7
        // and pp. 2-18, 2-19: M2 fetches byte 2, M3 performs the I/O.
        // OperandFetch and the u8 port interface follow decision-5 item 3.
        0xDB | 0xD3 => {
            let port = system.read(registers.pc.wrapping_add(1), MemoryAccess::OperandFetch);
            if opcode == 0xDB {
                registers.a = system.input(port);
            } else {
                system.output(port, registers.a);
            }
        }
        // 98-153B Ch. 4 p. 4-14 NOP: no operation; registers and flags
        // unaffected, apart from sequential PC advancement below.
        NOP => {}
        _ => return None,
    }
    // Sequential PC advancement: 98-153B Ch. 4 p. 4-2 and the fetch cycles
    // in Ch. 2 pp. 2-16, 2-18. PC/SP arithmetic wraps by their 16-bit width
    // (Ch. 2 p. 2-2); wrapping itself is derived, not stated explicitly.
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    Some(Path::Taken)
}
