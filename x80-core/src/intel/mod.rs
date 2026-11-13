//! Private Intel-family register access (decision-16 rules 1–3/5/6).
//!
//! Sources: Intel 98-153B (September 1975) and Intel 205775-002 (January
//! 1983). Shared facts carry both citations in `access`; model-owned flag
//! implementations retain their own authority and public type identity.

pub(crate) mod access;
pub(crate) mod data_transfer;

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

/// Only the condition-flag accessors needed by `access::condition`.
/// Design: decision-16 rule 3; flags: 98-153B Ch. 4 "Condition Flags"
/// pp. 4-2/4-3; 205775-002 §5.5 p. 5-3. Facts stay in each model's impl.
pub(crate) trait FlagRules: Copy {
    fn sign(self) -> bool;
    fn zero(self) -> bool;
    fn parity(self) -> bool;
    fn carry(self) -> bool;
}
