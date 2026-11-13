//! Five Z80 maps, assembled with the unchanged fact-free builder.
//!
//! Table contexts and complete lengths: decision-13 rules 3–5. Admission and
//! reconciled source errors: decision-14 rule 1 and PROVENANCE.md §1.
//! Prefixes have no terminal entry; cpu.rs owns their dispatch.

use crate::decode::{build, FieldRestriction};
pub(super) use crate::decode::{Encoding, States};

/// UM008011-0816's eleven instruction-description sections (decision-13 rule 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Group {
    Load8,
    Load16,
    ExchangeBlockSearch,
    Arithmetic8,
    GeneralArithmeticCpuControl,
    Arithmetic16,
    RotateShift,
    BitSetResetTest,
    Jump,
    CallReturn,
    InputOutput,
}

pub(super) type Instruction = crate::decode::Instruction<Group>;

// R/S are registers only: UM pp. 71–72, 74, 79, 145, 165, 170
// enumerate B,C,D,E,H,L,A, omitting 110. Memory costs have separate entries.
// P/Q (pair), C (condition) and T (restart) are unrestricted: pp. 99, 115,
// 119, 188, 198, 201, 263–264, 283–284, 286–287, 292–293.
pub(super) static MAIN: [Option<Instruction>; 256] = build(
    &[
        (Group::Load8, super::load8::MAIN),
        (Group::Load16, super::load16::MAIN),
        (
            Group::ExchangeBlockSearch,
            super::exchange_block_search::MAIN,
        ),
        (Group::Arithmetic8, super::arithmetic8::MAIN),
        (
            Group::GeneralArithmeticCpuControl,
            super::general_arithmetic_cpu_control::MAIN,
        ),
        (Group::Arithmetic16, super::arithmetic16::MAIN),
        (Group::RotateShift, super::rotate_shift::MAIN),
        (Group::Jump, super::jump::MAIN),
        (Group::CallReturn, super::call_return::MAIN),
        (Group::InputOutput, super::input_output::MAIN),
    ],
    &[
        FieldRestriction {
            field: b'R',
            excluded: 0b110,
        },
        FieldRestriction {
            field: b'S',
            excluded: 0b110,
        },
    ],
);

// R excludes memory 110 (UM pp. 213, 225, 243–244, 251–252, 259–260).
// B is a bit number, including 110 (pp. 244, 252, 260); it must stay distinct.
pub(super) static CB: [Option<Instruction>; 256] = build(
    &[
        (Group::RotateShift, super::rotate_shift::CB),
        (Group::BitSetResetTest, super::bit_set_reset_test::CB),
    ],
    &[FieldRestriction {
        field: b'R',
        excluded: 0b110,
    }],
);

// Decision-20 §1 admits all ED terminals, including r=110 IN/OUT;
// A: TM Table 7.0-11 p. 54; L: Young v0.91 §3.4 pp. 12–13.
// P is a pair field with all four values, including HL=10 (pp. 103–104,
// 108–109, 190, 192); no alias exclusion is inferred from compact Table 7.
pub(super) static ED: [Option<Instruction>; 256] = build(
    &[
        (Group::Load8, super::load8::ED),
        (Group::Load16, super::load16::ED),
        (Group::ExchangeBlockSearch, super::exchange_block_search::ED),
        (
            Group::GeneralArithmeticCpuControl,
            super::general_arithmetic_cpu_control::ED,
        ),
        (Group::Arithmetic16, super::arithmetic16::ED),
        (Group::RotateShift, super::rotate_shift::ED),
        (Group::CallReturn, super::call_return::ED),
        (Group::InputOutput, super::input_output::ED),
    ],
    &[],
);

// R/S exclude only 110, preserving real H/L operands beside indexed memory
// (UM pp. 75–78, 81–84). P is unrestricted BC,DE,IX/IY,SP (pp. 194, 196).
// No index halves or ignored-prefix forms are admitted (decision-14 rule 1).
pub(super) static INDEX: [Option<Instruction>; 256] = build(
    &[
        (Group::Load8, super::load8::INDEX),
        (Group::Load16, super::load16::INDEX),
        (
            Group::ExchangeBlockSearch,
            super::exchange_block_search::INDEX,
        ),
        (Group::Arithmetic8, super::arithmetic8::INDEX),
        (Group::Arithmetic16, super::arithmetic16::INDEX),
        (Group::Jump, super::jump::INDEX),
    ],
    &[
        FieldRestriction {
            field: b'R',
            excluded: 0b110,
        },
        FieldRestriction {
            field: b'S',
            excluded: 0b110,
        },
    ],
);

// Only the documented memory terminals: UM pp. 217, 219, 221–237,
// 247–250, 255–260. B is any bit number; low bits are fixed 110.
pub(super) static INDEX_CB: [Option<Instruction>; 256] = build(
    &[
        (Group::RotateShift, super::rotate_shift::INDEX_CB),
        (Group::BitSetResetTest, super::bit_set_reset_test::INDEX_CB),
    ],
    &[],
);

#[cfg(test)]
mod tests;
