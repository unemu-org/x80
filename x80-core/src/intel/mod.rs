//! Private Intel-family register access (decision-16 rules 1–3/5/6).
//!
//! Sources: Intel 98-153B (September 1975) and Intel 205775-002 (January
//! 1983). Shared facts carry both citations in `access`; model-owned flag
//! implementations retain their own authority and public type identity.

pub(crate) mod access;
pub(crate) mod arithmetic;
pub(crate) mod branch;
pub(crate) mod data_transfer;
pub(crate) mod logical;
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

/// Condition-flag accessors, model-owned PSW conversion and ALU facts.
/// Design: decision-16 rule 3; flags: 98-153B Ch. 4 "Condition Flags"
/// pp. 4-2/4-3; 205775-002 §5.5 p. 5-3. PSW images: 98-153B Ch. 4
/// p. 4-13; 205775-002 §5.6.5 pp. 5-15/5-16. Facts and the 8085 V/K
/// rules (decision-18 §4) stay in each model's implementation.
pub(crate) trait FlagRules: Copy {
    fn sign(self) -> bool;
    fn zero(self) -> bool;
    fn parity(self) -> bool;
    fn carry(self) -> bool;
    fn auxiliary_carry(self) -> bool;
    fn set_carry(&mut self, carry: bool);
    fn to_byte(self) -> u8;
    fn from_byte(byte: u8) -> Self;
    // S/Z/P and operation-supplied AC/CY: 98-153B Ch. 4 pp. 4-2/4-3;
    // 205775-002 §5.5 p. 5-3. Subtraction: 98-153B Ch. 4 p. 4-6;
    // 205775-002 §5.6.2 p. 5-6. The implementations keep their derivations.
    fn result(value: u8, auxiliary_carry: bool, carry: bool) -> Self;
    // Addition and increment: 98-153B Ch. 4 pp. 4-6/4-7;
    // 205775-002 §5.6.2 pp. 5-6 to 5-8. Flags remain model-owned.
    fn add(left: u8, right: u8, carry: bool) -> (u8, Self);
    fn subtract(left: u8, right: u8, borrow: bool) -> (u8, Self);
    // AND: 98-153B Ch. 4 pp. 4-8/4-9; 205775-002 §5.6.3 pp. 5-9/5-10.
    // Model-owned AC, including decision-8's resolution of the 8080 sources.
    fn and(a: u8, operand: u8) -> (u8, Self);

    // decision-19: INX/DCX flag effects; the shared executor supplies the
    // pair transition, while each model owns its update/preservation rules.
    fn pair_step(&mut self, old: u16, result: u16, decrement: bool);
    // decision-19: RLC/RRC/RAL/RAR flag effects after accumulator/CY update.
    fn rotate(&mut self, opcode: u8, old_a: u8, old_carry: bool, result: u8, carry: bool);
    // decision-19: DAD flag effects after the shared CY update.
    fn dad(&mut self, left: u16, right: u16, result: u16);
    // decision-19: DAA completion after the existing five-flag derivation,
    // using the combined low/high correction selected by the executor.
    fn daa(&mut self, old_a: u8, correction: u8, result: u8);
}
