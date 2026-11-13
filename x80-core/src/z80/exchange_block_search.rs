//! Exchange, Block Transfer, and Search Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 123–143.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-3, p. 46
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::opcodes::{Encoding, States};

// Repeats use terminating/repeating slots (decision-15 rule 11).

pub(super) const MAIN: &[Encoding] = &[
    // UM EX DE,HL, p. 124.
    Encoding::new(b"11101011", 1, States::single(4)),
    // UM EX AF,AF', p. 125.
    Encoding::new(b"00001000", 1, States::single(4)),
    // UM EXX, p. 126.
    Encoding::new(b"11011001", 1, States::single(4)),
    // UM EX (SP),HL, p. 127.
    Encoding::new(b"11100011", 1, States::single(19)),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM EX (SP),IX/IY, pp. 128–129.
    Encoding::new(b"11100011", 2, States::single(23)),
];

pub(super) const ED: &[Encoding] = &[
    // UM LDI, p. 130.
    Encoding::new(b"10100000", 2, States::single(16)),
    // UM LDIR, p. 132.
    Encoding::new(
        b"10110000",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM LDD, p. 134.
    Encoding::new(b"10101000", 2, States::single(16)),
    // UM LDDR, p. 136.
    Encoding::new(
        b"10111000",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM CPI, p. 138.
    Encoding::new(b"10100001", 2, States::single(16)),
    // UM CPIR, p. 139; termination wording: PROVENANCE §1.
    Encoding::new(
        b"10110001",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM CPD, p. 141.
    Encoding::new(b"10101001", 2, States::single(16)),
    // UM CPDR, p. 142; termination wording: PROVENANCE §1.
    Encoding::new(
        b"10111001",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
];
