//! Five stored flags and the PSW image (decision-10 and decision-11).
//!
//! Intel 205775-002 (January 1983) §2.2.2 p. 2-2 and §5.6.5 p. 5-15,
//! FLAG WORD: S Z X AC X P X CY. X means undefined, independently confirmed
//! by 9800301-04 (May 1981) Ch. 3 PUSH PSW p. 3-45. POP PSW loads only the
//! five flags (205775-002 p. 5-16). Checked against rendered page images.

/// The five 8085 condition flags (205775-002 §2.2.2, p. 2-2).
///
/// `Default` clears all flags: a library choice, not a power-up guarantee.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Flags {
    /// Sign, PSW bit 7 (205775-002 p. 5-15, FLAG WORD).
    pub sign: bool,
    /// Zero, PSW bit 6 (same figure).
    pub zero: bool,
    /// Auxiliary carry, PSW bit 4 (same figure).
    pub auxiliary_carry: bool,
    /// Parity, PSW bit 2 (same figure).
    pub parity: bool,
    /// Carry, PSW bit 0 (same figure).
    pub carry: bool,
}

impl Flags {
    /// Five flags at the positions in 205775-002 p. 5-15, FLAG WORD.
    /// Undefined bits 5, 3, 1 are 0, 0, 1 by library choice (decision-11).
    pub(crate) const fn to_byte(self) -> u8 {
        ((self.sign as u8) << 7)
            | ((self.zero as u8) << 6)
            | ((self.auxiliary_carry as u8) << 4)
            | ((self.parity as u8) << 2)
            | 0x02
            | self.carry as u8
    }

    /// POP PSW's five destinations (205775-002 §5.6.5 p. 5-16).
    pub(crate) const fn from_byte(byte: u8) -> Self {
        Self {
            sign: byte & 0x80 != 0,
            zero: byte & 0x40 != 0,
            auxiliary_carry: byte & 0x10 != 0,
            parity: byte & 0x04 != 0,
            carry: byte & 0x01 != 0,
        }
    }
}

// Model-owned flag access (decision-16 rule 3): Intel 205775-002 (January 1983),
// §5.5 p. 5-3.
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

    // PSW image: 205775-002 (January 1983) §5.6.5 pp. 5-15/5-16;
    // second witness: 9800301-04 (May 1981) Ch. 3 pp. 3-43/3-45.
    // Undefined filler bits retain decision-11's library choice.
    // Design: decision-16 rule 3; witnesses: i8085_registers and
    // i8085_stack_io_control. Delegate to the existing model conversion.
    fn to_byte(self) -> u8 {
        Flags::to_byte(self)
    }

    fn from_byte(byte: u8) -> Self {
        Flags::from_byte(byte)
    }
}
