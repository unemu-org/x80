//! The 8080 opcode map (decision-3, decision-5 item 7, decision-6, plan §2.6).
//!
//! For each of the 256 opcode bytes the map says whether 98-153B documents it
//! as an instruction and, if so, its length in bytes, its instruction group
//! and its state totals.
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
use crate::decode::{build, FieldRestriction};

pub(super) use crate::decode::{Encoding, Path, States};

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

/// The map record specialized to the 8080's instruction groups.
/// Length includes the opcode and its data or address bytes (98-153B p. 4-2,
/// "Instruction and Data Formats").
pub(super) type Instruction = crate::decode::Instruction<Group>;

/// Field rules for the group's encoding lists (decision-6, decision-10,
/// plan §2.6). Intel 98-153B (September 1975), Ch. 4 p. 4-3:
/// DDD (bits 5-3) and SSS (bits 2-0) take 111 A, 000 B, 001 C, 010 D,
/// 011 E, 100 H and 101 L. 110 names no register; memory forms have their
/// own entries (pp. 4-4/4-5). In particular, MOV M,M must not match MOV.
/// The independent summary/map witness in `opcodes/tests.rs` checks these
/// exclusions against 98-153B p. 4-15 and 9800301-04 (May 1981) p. A-5.
///
/// Other fields need no exclusion: RP (bits 5-4) takes 00 B, 01 D, 10 H,
/// 11 SP (p. 4-3); CCC (bits 5-3, p. 4-11) and NNN (bits 5-3, p. 4-12)
/// take every value. Restricted pairs are listed once per allowed pair.
const FIELD_RESTRICTIONS: &[FieldRestriction] = &[
    FieldRestriction {
        field: b'D',
        excluded: 0b110,
    },
    FieldRestriction {
        field: b'S',
        excluded: 0b110,
    },
];

// State totals belong to each entry: 98-153B p. 4-4 item 6 lists both times
// for an instruction with two execution times; p. 4-15 note 2 ties the two
// totals to condition flags. A single time fills both slots (decision-10).
/// The opcode map, indexed by opcode byte; `None` is not documented.
pub(super) static OPCODES: [Option<Instruction>; 256] = build(
    &[
        (Group::DataTransfer, data_transfer::ENCODINGS),
        (Group::Arithmetic, arithmetic::ENCODINGS),
        (Group::Logical, logical::ENCODINGS),
        (Group::Branch, branch::ENCODINGS),
        (Group::StackIoControl, stack_io_control::ENCODINGS),
    ],
    FIELD_RESTRICTIONS,
);

#[cfg(test)]
mod tests;
