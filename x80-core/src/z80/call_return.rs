//! Call and Return Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 280–293.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-10, p. 53
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::opcodes::{Encoding, States};

pub(super) const MAIN: &[Encoding] = &[
    // UM CALL nn, p. 281.
    Encoding::new(b"11001101", 3, States::single(17)),
    // UM CALL cc,nn, pp. 283–284.
    Encoding::new(
        b"11CCC100",
        3,
        States {
            untaken: 10,
            taken: 17,
        },
    ),
    // UM RET, p. 285.
    Encoding::new(b"11001001", 1, States::single(10)),
    // UM RET cc, pp. 286–287.
    Encoding::new(
        b"11CCC000",
        1,
        States {
            untaken: 5,
            taken: 11,
        },
    ),
    // UM RST p, pp. 292–293.
    Encoding::new(b"11TTT111", 1, States::single(11)),
];

pub(super) const ED: &[Encoding] = &[
    // UM RETI, p. 288.
    Encoding::new(b"01001101", 2, States::single(14)),
    // UM RETN, p. 290.
    Encoding::new(b"01000101", 2, States::single(14)),
];
