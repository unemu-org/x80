//! Rotate and Shift Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 204–241.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-7, p. 50
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::opcodes::{Encoding, States};

// Entry bit-diagram slips on pp. 225, 228: see PROVENANCE.md §1.

pub(super) const MAIN: &[Encoding] = &[
    // UM RLCA, p. 205.
    Encoding::new(b"00000111", 1, States::single(4)),
    // UM RLA, p. 207.
    Encoding::new(b"00010111", 1, States::single(4)),
    // UM RRCA, p. 209.
    Encoding::new(b"00001111", 1, States::single(4)),
    // UM RRA, p. 211; agrees with TM Table 7.0-7, p. 50.
    Encoding::new(b"00011111", 1, States::single(4)),
];

pub(super) const CB: &[Encoding] = &[
    // UM RLC r, p. 213.
    Encoding::new(b"00000RRR", 2, States::single(8)),
    // UM RLC (HL), p. 215.
    Encoding::new(b"00000110", 2, States::single(15)),
    // UM RRC r, pp. 224–226.
    Encoding::new(b"00001RRR", 2, States::single(8)),
    // UM RRC (HL), pp. 224–226.
    Encoding::new(b"00001110", 2, States::single(15)),
    // UM RL r, pp. 221–222.
    Encoding::new(b"00010RRR", 2, States::single(8)),
    // UM RL (HL), pp. 221–222.
    Encoding::new(b"00010110", 2, States::single(15)),
    // UM RR r, pp. 227–229.
    Encoding::new(b"00011RRR", 2, States::single(8)),
    // UM RR (HL), pp. 227–229.
    Encoding::new(b"00011110", 2, States::single(15)),
    // UM SLA r, pp. 230–231.
    Encoding::new(b"00100RRR", 2, States::single(8)),
    // UM SLA (HL), pp. 230–231.
    Encoding::new(b"00100110", 2, States::single(15)),
    // UM SRA r, pp. 233–234.
    Encoding::new(b"00101RRR", 2, States::single(8)),
    // UM SRA (HL), pp. 233–234.
    Encoding::new(b"00101110", 2, States::single(15)),
    // UM SRL r, pp. 236–237.
    Encoding::new(b"00111RRR", 2, States::single(8)),
    // UM SRL (HL), pp. 236–237.
    Encoding::new(b"00111110", 2, States::single(15)),
];

pub(super) const INDEX_CB: &[Encoding] = &[
    // UM RLC (IX/IY+d), pp. 217, 219.
    Encoding::new(b"00000110", 4, States::single(23)),
    // UM RRC (IX/IY+d), pp. 224–226.
    Encoding::new(b"00001110", 4, States::single(23)),
    // UM RL (IX/IY+d), pp. 221–222.
    Encoding::new(b"00010110", 4, States::single(23)),
    // UM RR (IX/IY+d), pp. 227–229.
    Encoding::new(b"00011110", 4, States::single(23)),
    // UM SLA (IX/IY+d), pp. 230–231.
    Encoding::new(b"00100110", 4, States::single(23)),
    // UM SRA (IX/IY+d), pp. 233–234.
    Encoding::new(b"00101110", 4, States::single(23)),
    // UM SRL (IX/IY+d), pp. 236–237.
    Encoding::new(b"00111110", 4, States::single(23)),
];

pub(super) const ED: &[Encoding] = &[
    // UM RLD, p. 238.
    Encoding::new(b"01101111", 2, States::single(18)),
    // UM RRD, p. 240.
    Encoding::new(b"01100111", 2, States::single(18)),
];
