//! Full F/F' images: decision-20 §4 (library representation).
//! A documented positions: Zilog UM008011-0816 rev. 11 (August 2016),
//! Tables 21/22 pp. 65–66; 03-0029-01 (1977), §6 p. 39.
//!
//! H transfer rule: Young, The Undocumented Z80 Documented v0.91 (2005),
//! §2.3 p. 8 / §§8.2–8.3 pp. 28–29; Rak z80test v1.2a .selftest
//! lines 297–302, .poppush_af lines 1218–1223 and .ex_af_af.

/// Eight independently stored Z80 flag bits (decision-20 §4).
/// `Default` clears every bit by library choice, not a power-up guarantee.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Flags {
    /// Sign, F bit 7 (Table 21).
    pub sign: bool,
    /// Zero, F bit 6 (Table 21).
    pub zero: bool,
    /// Y (YF), F bit 5 (Young v0.91 §2.3 p. 8).
    pub y: bool,
    /// X (XF), F bit 3 (Young v0.91 §2.3 p. 8).
    pub x: bool,
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
    /// Selects Y/X from a byte; each caller cites its decision-20 §4 class.
    pub(super) fn set_yx(&mut self, value: u8) {
        self.y = value & 0x20 != 0;
        self.x = value & 0x08 != 0;
    }

    /// Packs all eight stored bits unchanged (decision-20 §4, H transfers).
    pub const fn to_byte(self) -> u8 {
        ((self.sign as u8) << 7)
            | ((self.zero as u8) << 6)
            | ((self.y as u8) << 5)
            | ((self.x as u8) << 3)
            | ((self.half_carry as u8) << 4)
            | ((self.parity_overflow as u8) << 2)
            | ((self.subtract as u8) << 1)
            | self.carry as u8
    }

    /// Loads all eight bits independently (decision-20 §4, library storage).
    pub const fn from_byte(byte: u8) -> Self {
        Self {
            sign: byte & 0x80 != 0,
            zero: byte & 0x40 != 0,
            y: byte & 0x20 != 0,
            x: byte & 0x08 != 0,
            half_carry: byte & 0x10 != 0,
            parity_overflow: byte & 0x04 != 0,
            subtract: byte & 0x02 != 0,
            carry: byte & 0x01 != 0,
        }
    }
}
