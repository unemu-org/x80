//! The 8080 condition flags and their byte in the processor status word.
//!
//! Design: plan §2.2 (flags centralized within the model), decision-5 item 1.
//! Primary source: Intel 8080 Microcomputer Systems User's Manual, 98-153B
//! (September 1975), all checked against the page images:
//!
//! - p. 2-3, "Arithmetic and Logic Unit (ALU)": "A 5-bit flag register: zero,
//!   carry, sign, parity and auxiliary carry"; Figure 2-2 (p. 2-2) shows
//!   "FLAG (5) FLIP-FLOPS". No storage for flag-byte bits 1, 3 and 5 is shown.
//! - p. 4-2, "Condition Flags": five flags, "each represented by a 1-bit
//!   register in the CPU".
//! - p. 4-13, "FLAG WORD" figure with PUSH PSW and POP PSW: D7 S, D6 Z, D5 0,
//!   D4 AC, D3 0, D2 P, D1 1, D0 CY. PUSH PSW stores CY, 1, P, 0, AC, 0, Z
//!   and S in bits 0 to 7 of the byte at SP - 2. POP PSW loads CY, P, AC, Z
//!   and S from bits 0, 2, 4, 6 and 7 and names no destination for bits 1, 3
//!   and 5.
//!
//! Second witness: Intel 8080/8085 Assembly Language Programming Manual,
//! 9800301-04, p. 3-45, PUSH PSW: the same layout, and "On the 8080, bits 3 and
//! 5 are always zero; bit one is always set to one." Ch. 1 pp. 1-9 to 1-12
//! describe the five flags but give no bit positions.

// Flag byte masks, 98-153B p. 4-13 "FLAG WORD"; 9800301-04 p. 3-45.
/// D0: carry (CY).
const CARRY: u8 = 0b0000_0001;
/// D1: always 1 in the flag byte that PUSH PSW stores.
const BIT1_ONE: u8 = 0b0000_0010;
/// D2: parity (P).
const PARITY: u8 = 0b0000_0100;
// D3: always 0 in the flag byte that PUSH PSW stores.
/// D4: auxiliary carry (AC).
const AUXILIARY_CARRY: u8 = 0b0001_0000;
// D5: always 0 in the flag byte that PUSH PSW stores.
/// D6: zero (Z).
const ZERO: u8 = 0b0100_0000;
/// D7: sign (S).
const SIGN: u8 = 0b1000_0000;

/// The five 8080 condition flags, each a one-bit register (98-153B p. 4-2).
///
/// Fields are in flag-byte order, from D7 down to D0. How each instruction
/// updates them is fixed with that instruction, from the cited pages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Flags {
    /// Sign (S), flag-byte bit 7.
    pub sign: bool,
    /// Zero (Z), flag-byte bit 6.
    pub zero: bool,
    /// Auxiliary carry (AC), flag-byte bit 4.
    pub auxiliary_carry: bool,
    /// Parity (P), flag-byte bit 2.
    pub parity: bool,
    /// Carry (CY), flag-byte bit 0.
    pub carry: bool,
}

impl Flags {
    /// The flag byte as PUSH PSW stores it: the five flags at their positions,
    /// bit 1 set, bits 3 and 5 clear (98-153B p. 4-13).
    pub(crate) const fn to_byte(self) -> u8 {
        let mut byte = BIT1_ONE;
        if self.sign {
            byte |= SIGN;
        }
        if self.zero {
            byte |= ZERO;
        }
        if self.auxiliary_carry {
            byte |= AUXILIARY_CARRY;
        }
        if self.parity {
            byte |= PARITY;
        }
        if self.carry {
            byte |= CARRY;
        }
        byte
    }

    /// The five flags read from their positions in `byte`, as the POP PSW
    /// transfer lists them (98-153B p. 4-13).
    ///
    /// Bits 1, 3 and 5 are dropped: the CPU has no storage for them (p. 2-3).
    /// The POP PSW instruction itself is witnessed with its implementation.
    pub(crate) const fn from_byte(byte: u8) -> Self {
        Self {
            sign: byte & SIGN != 0,
            zero: byte & ZERO != 0,
            auxiliary_carry: byte & AUXILIARY_CARRY != 0,
            parity: byte & PARITY != 0,
            carry: byte & CARRY != 0,
        }
    }
}

// Model-owned flag access (decision-16 rule 3): Intel 98-153B (September 1975),
// Ch. 4 "Condition Flags" pp. 4-2/4-3.
impl crate::intel::FlagRules for Flags {
    fn sign(self) -> bool {
        self.sign
    }

    fn zero(self) -> bool {
        self.zero
    }

    fn parity(self) -> bool {
        self.parity
    }

    fn carry(self) -> bool {
        self.carry
    }
}
