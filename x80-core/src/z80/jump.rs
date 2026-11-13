//! Jump Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 261–279.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-9, p. 52
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::opcodes::{Encoding, States};

pub(super) const MAIN: &[Encoding] = &[
    // UM JP nn, p. 262.
    Encoding::new(b"11000011", 3, States::single(10)),
    // UM JP cc,nn, pp. 263–264; both paths are 10.
    Encoding::new(b"11CCC010", 3, States::single(10)),
    // UM JR e, p. 265.
    Encoding::new(b"00011000", 2, States::single(12)),
    // UM JR C,e, p. 267.
    Encoding::new(
        b"00111000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JR NC,e, p. 269.
    Encoding::new(
        b"00110000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JR Z,e, p. 271.
    Encoding::new(
        b"00101000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JR NZ,e, p. 273.
    Encoding::new(
        b"00100000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JP (HL), p. 275.
    Encoding::new(b"11101001", 1, States::single(4)),
    // UM DJNZ e, p. 278.
    Encoding::new(
        b"00010000",
        2,
        States {
            untaken: 8,
            taken: 13,
        },
    ),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM JP (IX/IY), pp. 276–277.
    Encoding::new(b"11101001", 2, States::single(8)),
];
