//! 8-Bit Load Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 70–97.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-1, p. 44
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::opcodes::{Encoding, States};

pub(super) const MAIN: &[Encoding] = &[
    // UM LD r,r', p. 71.
    Encoding::new(b"01RRRSSS", 1, States::single(4)),
    // UM LD r,n, p. 72.
    Encoding::new(b"00RRR110", 2, States::single(7)),
    // UM LD r,(HL), p. 74.
    Encoding::new(b"01RRR110", 1, States::single(7)),
    // UM LD (HL),r, p. 79.
    Encoding::new(b"01110SSS", 1, States::single(7)),
    // UM LD (HL),n, p. 85.
    Encoding::new(b"00110110", 2, States::single(10)),
    // UM LD A,(BC), p. 88.
    Encoding::new(b"00001010", 1, States::single(7)),
    // UM LD A,(DE), p. 89.
    Encoding::new(b"00011010", 1, States::single(7)),
    // UM LD A,(nn), p. 90.
    Encoding::new(b"00111010", 3, States::single(13)),
    // UM LD (BC),A, p. 91.
    Encoding::new(b"00000010", 1, States::single(7)),
    // UM LD (DE),A, p. 92.
    Encoding::new(b"00010010", 1, States::single(7)),
    // UM LD (nn),A, p. 93.
    Encoding::new(b"00110010", 3, States::single(13)),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM LD r,(IX/IY+d), pp. 75, 77.
    Encoding::new(b"01RRR110", 3, States::single(19)),
    // UM LD (IX/IY+d),r, pp. 81, 83.
    Encoding::new(b"01110SSS", 3, States::single(19)),
    // UM LD (IX/IY+d),n, pp. 86–87.
    Encoding::new(b"00110110", 4, States::single(19)),
];

pub(super) const ED: &[Encoding] = &[
    // UM LD A,I, p. 94.
    Encoding::new(b"01010111", 2, States::single(9)),
    // UM LD A,R, p. 95.
    Encoding::new(b"01011111", 2, States::single(9)),
    // UM LD I,A, p. 96.
    Encoding::new(b"01000111", 2, States::single(9)),
    // UM LD R,A, p. 97.
    Encoding::new(b"01001111", 2, States::single(9)),
];
