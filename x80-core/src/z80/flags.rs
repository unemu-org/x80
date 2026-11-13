//! Six stored flags and the F/F' image (decision-14 rule 5).
//!
//! Zilog UM008011-0816 (revision 11, August 2016), Table 21 p. 65 (PDF 79):
//! S Z X H X P/V N C. Table 22 and text p. 66 (PDF 80) call bits 5/3 unused;
//! they do not specify zero. Values checked on rendered page images.

/// The six Z80 condition flags (UM008011-0816 Tables 21/22, pp. 65–66).
///
/// `Default` clears every flag by library choice, not a power-up guarantee.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Flags {
    /// Sign, F bit 7 (Table 21).
    pub sign: bool,
    /// Zero, F bit 6 (Table 21).
    pub zero: bool,
    /// Half carry, F bit 4 (Table 21).
    pub half_carry: bool,
    /// Parity/overflow, F bit 2 (Table 21).
    pub parity_overflow: bool,
    /// Add/subtract (N), F bit 1 (Table 21).
    pub subtract: bool,
    /// Carry, F bit 0 (Table 21).
    pub carry: bool,
}

impl Flags {
    /// Packs the six flags at their Table 21 positions (UM p. 65).
    /// Bits 5 and 3 read as zero by library choice (decision-14 rule 5).
    pub const fn to_byte(self) -> u8 {
        ((self.sign as u8) << 7)
            | ((self.zero as u8) << 6)
            | ((self.half_carry as u8) << 4)
            | ((self.parity_overflow as u8) << 2)
            | ((self.subtract as u8) << 1)
            | self.carry as u8
    }

    /// Reads the six Table 21 flags, ignoring unused bits 5 and 3 by
    /// library choice (UM pp. 65–66; decision-14 rule 5).
    pub const fn from_byte(byte: u8) -> Self {
        Self {
            sign: byte & 0x80 != 0,
            zero: byte & 0x40 != 0,
            half_carry: byte & 0x10 != 0,
            parity_overflow: byte & 0x04 != 0,
            subtract: byte & 0x02 != 0,
            carry: byte & 0x01 != 0,
        }
    }
}
