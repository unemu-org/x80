//! The documented 8085 map (decision-10, decision-11).
//!
//! Group lists transcribe Intel 205775-002 (January 1983), §5.6,
//! pp. 5-4 to 5-18. Unmatched bytes are undocumented; documented entries
//! whose executors have not landed also return Unsupported (decision-5
//! item 7). The independent witness uses Tables 5-1/5-2, pp. 5-19/5-20,
//! cross-read with Intel 9800301-04 (May 1981), App. A.

use super::{arithmetic, branch, data_transfer, logical, rim_sim, stack_io_control};
use crate::decode::{build, FieldRestriction};

pub(super) use crate::decode::{Encoding, Path, States};

/// The five §5.6 groups (p. 5-4), with RIM/SIM separated by decision-10.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Group {
    DataTransfer,
    Arithmetic,
    Logical,
    Branch,
    StackIoControl,
    RimSim,
}

pub(super) type Instruction = crate::decode::Instruction<Group>;

/// Model-owned field rules (decision-10): 205775-002 §5.2 p. 5-1
/// lists DDD/SSS = 111 A, 000 B, 001 C, 010 D, 011 E, 100 H, 101 L.
/// 110 is absent from that register list; memory has separate entries
/// (§5.6 pp. 5-4 to 5-12). Table 5-1 p. 5-19 and 9800301-04 p. A-1
/// identify 110 as memory. In particular MOV M,M is excluded: 76 is HLT.
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

/// Every documented byte has length and both state slots, even before its
/// executor lands. Single totals fill both slots (§5.2 p. 5-2 item 6).
pub(super) static OPCODES: [Option<Instruction>; 256] = build(
    &[
        (Group::DataTransfer, data_transfer::ENCODINGS),
        (Group::Arithmetic, arithmetic::ENCODINGS),
        (Group::Logical, logical::ENCODINGS),
        (Group::Branch, branch::ENCODINGS),
        (Group::StackIoControl, stack_io_control::ENCODINGS),
        (Group::RimSim, rim_sim::ENCODINGS),
    ],
    FIELD_RESTRICTIONS,
);

#[cfg(test)]
mod tests;
