//! Bit Set, Reset, and Test Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 242–260.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-8, p. 51
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::opcodes::{Encoding, States};

// Compact Table 14 is corrupt: entry patterns prevail (PROVENANCE.md §1).

pub(super) const CB: &[Encoding] = &[
    // UM BIT b,r, pp. 243–244.
    Encoding::new(b"01BBBRRR", 2, States::single(8)),
    // UM BIT b,(HL), pp. 245–246.
    Encoding::new(b"01BBB110", 2, States::single(12)),
    // UM SET b,r, pp. 251–252.
    Encoding::new(b"11BBBRRR", 2, States::single(8)),
    // UM SET b,(HL), pp. 253–254.
    Encoding::new(b"11BBB110", 2, States::single(15)),
    // UM RES b,r, pp. 259–260.
    Encoding::new(b"10BBBRRR", 2, States::single(8)),
    // UM RES b,(HL), pp. 259–260.
    Encoding::new(b"10BBB110", 2, States::single(15)),
];

pub(super) const INDEX_CB: &[Encoding] = &[
    // UM BIT b,(IX/IY+d), pp. 247–250.
    Encoding::new(b"01BBB110", 4, States::single(20)),
    // UM SET b,(IX/IY+d), pp. 255–258.
    Encoding::new(b"11BBB110", 4, States::single(23)),
    // UM RES b,(IX/IY+d), pp. 259–260.
    Encoding::new(b"10BBB110", 4, States::single(23)),
];
