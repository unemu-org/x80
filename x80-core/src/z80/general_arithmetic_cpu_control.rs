//! General-Purpose Arithmetic and CPU Control Groups: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 172–186.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-5, p. 48
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::opcodes::{Encoding, States};

pub(super) const MAIN: &[Encoding] = &[
    // UM DAA, pp. 173–174.
    Encoding::new(b"00100111", 1, States::single(4)),
    // UM CPL, p. 175.
    Encoding::new(b"00101111", 1, States::single(4)),
    // UM CCF, p. 178.
    Encoding::new(b"00111111", 1, States::single(4)),
    // UM SCF, p. 179.
    Encoding::new(b"00110111", 1, States::single(4)),
    // UM NOP, p. 180.
    Encoding::new(b"00000000", 1, States::single(4)),
    // UM HALT, p. 181.
    Encoding::new(b"01110110", 1, States::single(4)),
    // UM DI, p. 182.
    Encoding::new(b"11110011", 1, States::single(4)),
    // UM EI, p. 183.
    Encoding::new(b"11111011", 1, States::single(4)),
];

pub(super) const ED: &[Encoding] = &[
    // UM NEG, p. 176.
    Encoding::new(b"01000100", 2, States::single(8)),
    // UM IM 0, p. 184.
    Encoding::new(b"01000110", 2, States::single(8)),
    // UM IM 1, p. 185.
    Encoding::new(b"01010110", 2, States::single(8)),
    // UM IM 2, p. 186.
    Encoding::new(b"01011110", 2, States::single(8)),
];
