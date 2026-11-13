//! 8-Bit Arithmetic Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 144–171.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-4, p. 47
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::opcodes::{Encoding, States};

// Entry bit-diagram slips on pp. 151, 159: see PROVENANCE.md §1.

pub(super) const MAIN: &[Encoding] = &[
    // UM ADD A,r, p. 145.
    Encoding::new(b"10000RRR", 1, States::single(4)),
    // UM ADD A,n, p. 147.
    Encoding::new(b"11000110", 2, States::single(7)),
    // UM ADD A,(HL), p. 148.
    Encoding::new(b"10000110", 1, States::single(7)),
    // UM ADC A,r, pp. 151–152.
    Encoding::new(b"10001RRR", 1, States::single(4)),
    // UM ADC A,n, pp. 151–152.
    Encoding::new(b"11001110", 2, States::single(7)),
    // UM ADC A,(HL), pp. 151–152.
    Encoding::new(b"10001110", 1, States::single(7)),
    // UM SUB r, pp. 153–154.
    Encoding::new(b"10010RRR", 1, States::single(4)),
    // UM SUB n, pp. 153–154.
    Encoding::new(b"11010110", 2, States::single(7)),
    // UM SUB (HL), pp. 153–154.
    Encoding::new(b"10010110", 1, States::single(7)),
    // UM SBC A,r, pp. 155–156.
    Encoding::new(b"10011RRR", 1, States::single(4)),
    // UM SBC A,n, pp. 155–156.
    Encoding::new(b"11011110", 2, States::single(7)),
    // UM SBC A,(HL), pp. 155–156.
    Encoding::new(b"10011110", 1, States::single(7)),
    // UM AND r, pp. 157–158.
    Encoding::new(b"10100RRR", 1, States::single(4)),
    // UM AND n, pp. 157–158.
    Encoding::new(b"11100110", 2, States::single(7)),
    // UM AND (HL), pp. 157–158.
    Encoding::new(b"10100110", 1, States::single(7)),
    // UM XOR r, pp. 161–162.
    Encoding::new(b"10101RRR", 1, States::single(4)),
    // UM XOR n, pp. 161–162.
    Encoding::new(b"11101110", 2, States::single(7)),
    // UM XOR (HL), pp. 161–162.
    Encoding::new(b"10101110", 1, States::single(7)),
    // UM OR r, pp. 159–160.
    Encoding::new(b"10110RRR", 1, States::single(4)),
    // UM OR n, pp. 159–160.
    Encoding::new(b"11110110", 2, States::single(7)),
    // UM OR (HL), pp. 159–160.
    Encoding::new(b"10110110", 1, States::single(7)),
    // UM CP r, pp. 163–164.
    Encoding::new(b"10111RRR", 1, States::single(4)),
    // UM CP n, pp. 163–164.
    Encoding::new(b"11111110", 2, States::single(7)),
    // UM CP (HL), pp. 163–164.
    Encoding::new(b"10111110", 1, States::single(7)),
    // UM INC r, p. 165.
    Encoding::new(b"00RRR100", 1, States::single(4)),
    // UM INC (HL), p. 167.
    Encoding::new(b"00110100", 1, States::single(11)),
    // UM DEC r, pp. 170–171.
    Encoding::new(b"00RRR101", 1, States::single(4)),
    // UM DEC (HL), pp. 170–171.
    Encoding::new(b"00110101", 1, States::single(11)),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM ADD A,(IX/IY+d), pp. 149–150.
    Encoding::new(b"10000110", 3, States::single(19)),
    // UM ADC A,(IX/IY+d), pp. 151–152.
    Encoding::new(b"10001110", 3, States::single(19)),
    // UM SUB (IX/IY+d), pp. 153–154.
    Encoding::new(b"10010110", 3, States::single(19)),
    // UM SBC A,(IX/IY+d), pp. 155–156.
    Encoding::new(b"10011110", 3, States::single(19)),
    // UM AND (IX/IY+d), pp. 157–158.
    Encoding::new(b"10100110", 3, States::single(19)),
    // UM XOR (IX/IY+d), pp. 161–162.
    Encoding::new(b"10101110", 3, States::single(19)),
    // UM OR (IX/IY+d), pp. 159–160.
    Encoding::new(b"10110110", 3, States::single(19)),
    // UM CP (IX/IY+d), pp. 163–164.
    Encoding::new(b"10111110", 3, States::single(19)),
    // UM INC (IX/IY+d), pp. 168–169.
    Encoding::new(b"00110100", 3, States::single(23)),
    // UM DEC (IX/IY+d), pp. 170–171.
    Encoding::new(b"00110101", 3, States::single(23)),
];
