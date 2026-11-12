//! Central 8080 arithmetic and result flags (plan §2.2; decision-6).
//!
//! Sources: Intel 98-153B (September 1975), "Condition Flags", pp. 4-2/4-3,
//! and "Arithmetic Group", pp. 4-6 to 4-8; Intel 9800301-04 (May 1981),
//! Ch. 1 pp. 1-9 to 1-12 and Ch. 3 SUB/SBB entries. Page images checked.
//!
//! Subtraction AC: 98-153B p. 4-3 defines AC as carry out of bit 3 into bit
//! 4, explicitly including subtraction and decrement. p. 4-6 specifies
//! two's-complement subtraction and CY=1 for borrow. It gives AC no borrow
//! exception. 9800301-04 p. 1-11 also defines AC as carry out of bit 3;
//! p. 1-12 lists subtraction/decrement among the operations affecting it.
//! SUB p. 3-64 explicitly complements the addition's carry to form CY, but
//! leaves AC set for the bit-3 carry in 3E - 3E. SBB pp. 3-56/3-57 uses
//! two's-complement addition and shows AC=1 for 4 - 2 - 1. Thus both manuals
//! describe the addition's bit-3 carry, while CY reports borrow.

use super::Flags;

/// Standard S/Z/P, with the instruction's independently computed AC/CY.
/// 98-153B pp. 4-2/4-3: S is bit 7, Z means zero, P means even parity.
pub(super) const fn result_flags(value: u8, auxiliary_carry: bool, carry: bool) -> Flags {
    Flags {
        sign: value & 0x80 != 0,
        zero: value == 0,
        auxiliary_carry,
        parity: value.count_ones() & 1 == 0,
        carry,
    }
}

/// ADD/ADC (98-153B p. 4-6): carry out of bits 3 and 7.
pub(super) fn add(left: u8, right: u8, carry: bool) -> (u8, Flags) {
    let input = u16::from(carry);
    let sum = u16::from(left) + u16::from(right) + input;
    let low = u16::from(left & 0x0F) + u16::from(right & 0x0F) + input;
    let value = sum as u8;
    (value, result_flags(value, low > 0x0F, sum > 0xFF))
}

/// SUB/SBB and DCR (98-153B pp. 4-6 to 4-8): A + !operand + !borrow.
/// AC is the addition's bit-3 carry; CY is the complement of bit-7 carry.
/// See the two manuals' explicit AC statements above.
pub(super) fn subtract(left: u8, right: u8, borrow: bool) -> (u8, Flags) {
    let input = u16::from(!borrow);
    let complement = !right;
    let sum = u16::from(left) + u16::from(complement) + input;
    let low = u16::from(left & 0x0F) + u16::from(complement & 0x0F) + input;
    let value = sum as u8;
    (value, result_flags(value, low > 0x0F, sum <= 0xFF))
}
