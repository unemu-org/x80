//! Independent arithmetic witness, using only public model types.
//!
//! Design: plan §2.2; decision-5 items 3, 8; decision-6. Sources checked against
//! images: 98-153B (September 1975), "Condition Flags", pp. 4-2/4-3 and
//! "Arithmetic Group", pp. 4-6/4-7; 9800301-04 (May 1981), Ch. 1 pp. 1-10
//! to 1-12, SUB pp. 3-63/3-64 and SBB pp. 3-56/3-57. AC is the bit-3 carry
//! of two's-complement addition; CY alone is complemented into borrow.

use x80_core::i8080::Flags;

/// A full adder, one bit at a time. Unlike the implementation's wide sums,
/// this follows carries through the individual bit positions. For subtraction
/// feed the complement of the operand and the complement of the borrow input.
pub fn arithmetic(a: u8, operand: u8, input: bool, subtract: bool) -> (u8, Flags) {
    let rhs = if subtract { !operand } else { operand };
    let mut carry = input ^ subtract;
    let mut auxiliary = false;
    let mut value = 0;
    let mut odd = false;
    for bit in 0..8 {
        let left = (a >> bit) & 1 != 0;
        let right = (rhs >> bit) & 1 != 0;
        let digit = left ^ right ^ carry;
        carry = (left && right) || ((left ^ right) && carry);
        if bit == 3 {
            auxiliary = carry;
        }
        if digit {
            value |= 1 << bit;
            odd = !odd;
        }
    }
    (
        value,
        Flags {
            sign: value >= 128,
            zero: value == 0,
            auxiliary_carry: auxiliary,
            parity: !odd,
            carry: carry ^ subtract,
        },
    )
}
