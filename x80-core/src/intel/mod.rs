//! Private Intel-family register access (decision-16 rules 1–3/5/6).
//!
//! Sources: Intel 98-153B (September 1975) and Intel 205775-002 (January
//! 1983). Shared facts carry both citations in `access`; model-owned flag
//! implementations retain their own authority and public type identity.

pub(crate) mod access;
pub(crate) mod branch;
pub(crate) mod data_transfer;
pub(crate) mod stack_io;

/// A concrete borrowed view, a library choice under decision-16 rule 2.
/// Register widths: 98-153B Ch. 2 "Registers" pp. 2-2/2-3;
/// 205775-002 §2.2.1/§2.2.2 p. 2-2. No flags image is specified here.
pub(crate) struct View<'a, F> {
    pub a: &'a mut u8,
    pub flags: &'a mut F,
    pub b: &'a mut u8,
    pub c: &'a mut u8,
    pub d: &'a mut u8,
    pub e: &'a mut u8,
    pub h: &'a mut u8,
    pub l: &'a mut u8,
    pub sp: &'a mut u16,
    pub pc: &'a mut u16,
}

/// Condition-flag accessors and model-owned PSW conversion.
/// Design: decision-16 rule 3; flags: 98-153B Ch. 4 "Condition Flags"
/// pp. 4-2/4-3; 205775-002 §5.5 p. 5-3. PSW images: 98-153B Ch. 4
/// p. 4-13; 205775-002 §5.6.5 pp. 5-15/5-16. Facts and the 8085 filler-bit
/// library choice (decision-11) stay in each model's implementation.
pub(crate) trait FlagRules: Copy {
    fn sign(self) -> bool;
    fn zero(self) -> bool;
    fn parity(self) -> bool;
    fn carry(self) -> bool;
    fn to_byte(self) -> u8;
    fn from_byte(byte: u8) -> Self;
}
