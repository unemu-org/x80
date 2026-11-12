//! The 8080 opcode map (decision-3, decision-5 item 7, decision-6, plan §2.6).
//!
//! For each of the 256 opcode bytes the map says whether 98-153B documents it
//! as an instruction and, if so, its length in bytes, its instruction group
//! and, once cited, its state totals.
//!
//! Primary source: Intel 8080 Microcomputer Systems User's Manual, 98-153B
//! (September 1975), Chapter 4, checked against the page images. p. 4-1 names
//! the five instruction groups; each group module lists the encodings printed
//! on its pages (data transfer pp. 4-4 and 4-5, arithmetic pp. 4-6 to 4-8,
//! logical pp. 4-8 to 4-11, branch pp. 4-11 and 4-12, stack, I/O and machine
//! control pp. 4-13 and 4-14). The map is built from those lists at compile
//! time. A byte that no listed encoding matches is not documented as an
//! instruction and gives `Unsupported` (decision-3): the undocumented set is
//! the complement of the documented one, never a list of its own.
//!
//! Witness: `opcodes/tests.rs` compares the map with an independent
//! transcription of the instruction summary on 98-153B p. 4-15 and of the
//! opcode-sequence table of 9800301-04 p. A-5.

use super::{arithmetic, branch, data_transfer, logical, stack_io_control};

/// An instruction group of 98-153B Ch. 4 (p. 4-1). Each has its own module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Group {
    /// Data Transfer Group, pp. 4-4 and 4-5.
    DataTransfer,
    /// Arithmetic Group, pp. 4-6 to 4-8.
    Arithmetic,
    /// Logical Group, pp. 4-8 to 4-11.
    Logical,
    /// Branch Group, pp. 4-11 and 4-12.
    Branch,
    /// Stack, I/O and Machine Control Group, pp. 4-13 and 4-14.
    StackIoControl,
}

/// The state totals of an instruction (decision-6, plan §2.6).
///
/// 98-153B p. 4-4, item 6: "If the instruction has two possible execution
/// times, as in a Conditional Jump, both times will be listed, separated by a
/// slash"; p. 4-15, note 2: the two times are "dependent on condition flags".
/// An instruction with one execution time has equal totals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct States {
    /// States when the condition of a conditional instruction is not met.
    pub(super) untaken: u8,
    /// States when the condition is met, or the only total.
    pub(super) taken: u8,
}

impl States {
    /// The totals of an instruction with one execution time.
    pub(super) const fn single(states: u8) -> Self {
        Self {
            untaken: states,
            taken: states,
        }
    }

    /// The total for the path an execution took.
    pub(super) const fn of(self, path: Path) -> u8 {
        match path {
            Path::Untaken => self.untaken,
            Path::Taken => self.taken,
        }
    }
}

/// Which state total an execution spent, reported by a group's executor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Path {
    /// The condition of a conditional instruction was not met.
    #[allow(
        dead_code,
        reason = "returned by the conditional instructions of the branch group"
    )]
    Untaken,
    /// The condition was met, or the instruction has one execution time; its
    /// two totals are then equal.
    Taken,
}

/// One encoding as printed in a 98-153B Ch. 4 entry.
#[derive(Clone, Copy, Debug)]
pub(super) struct Encoding {
    /// Bits D7 down to D0 as printed: `0`, `1` or a field letter.
    ///
    /// Fields (p. 4-3): `DDD` (bits 5-3) and `SSS` (bits 2-0) take the seven
    /// register codes 111 A, 000 B, 001 C, 010 D, 011 E, 100 H and 101 L; 110
    /// names no register, and the memory forms have entries of their own.
    /// `RP` (bits 5-4) takes 00 B, 01 D, 10 H and 11 SP. `CCC` (bits 5-3,
    /// p. 4-11) and `NNN` (bits 5-3) take every value. An encoding that allows
    /// only some pairs is listed once per allowed pair.
    bits: [u8; 8],
    /// The opcode byte and its data or address bytes (p. 4-2, "Instruction
    /// and Data Formats").
    length: u8,
    /// `None` until the group's bead cites the totals from the entry.
    states: Option<States>,
}

impl Encoding {
    /// An encoding without cited state totals.
    pub(super) const fn new(bits: &[u8; 8], length: u8) -> Self {
        Self {
            bits: *bits,
            length,
            states: None,
        }
    }

    /// The same encoding with its cited state totals.
    pub(super) const fn with_states(self, states: States) -> Self {
        Self {
            states: Some(states),
            ..self
        }
    }

    /// Whether `byte` has this encoding.
    const fn matches(&self, byte: u8) -> bool {
        let mut index = 0;
        while index < 8 {
            let bit = (byte >> (7 - index)) & 1;
            let fits = match self.bits[index] {
                b'0' => bit == 0,
                b'1' => bit == 1,
                b'D' | b'S' | b'R' | b'P' | b'C' | b'N' => true,
                _ => panic!("unknown letter in an encoding"),
            };
            if !fits {
                return false;
            }
            index += 1;
        }
        // 110 is not a register code in DDD or SSS (p. 4-3).
        let memory_destination = self.bits[2] == b'D' && (byte >> 3) & 0b111 == 0b110;
        let memory_source = self.bits[5] == b'S' && byte & 0b111 == 0b110;
        !memory_destination && !memory_source
    }
}

/// A documented opcode byte: its group, length and state totals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Instruction {
    /// The Ch. 4 group whose module executes it.
    pub(super) group: Group,
    /// Length in bytes, 1 to 3.
    pub(super) length: u8,
    /// `None` until cited; the instruction does not execute before then.
    pub(super) states: Option<States>,
}

/// The opcode map, indexed by opcode byte; `None` is not documented.
pub(super) static OPCODES: [Option<Instruction>; 256] = build();

/// Places every byte of every group's encodings. Compilation fails if two
/// encodings claim one byte.
const fn build() -> [Option<Instruction>; 256] {
    let groups: [(Group, &[Encoding]); 5] = [
        (Group::DataTransfer, data_transfer::ENCODINGS),
        (Group::Arithmetic, arithmetic::ENCODINGS),
        (Group::Logical, logical::ENCODINGS),
        (Group::Branch, branch::ENCODINGS),
        (Group::StackIoControl, stack_io_control::ENCODINGS),
    ];
    let mut table = [None; 256];
    let mut group_index = 0;
    while group_index < groups.len() {
        let (group, encodings) = groups[group_index];
        let mut encoding_index = 0;
        while encoding_index < encodings.len() {
            let encoding = encodings[encoding_index];
            let mut byte = 0;
            while byte < table.len() {
                if encoding.matches(byte as u8) {
                    assert!(table[byte].is_none(), "two encodings claim one byte");
                    table[byte] = Some(Instruction {
                        group,
                        length: encoding.length,
                        states: encoding.states,
                    });
                }
                byte += 1;
            }
            encoding_index += 1;
        }
        group_index += 1;
    }
    table
}

#[cfg(test)]
mod tests;
