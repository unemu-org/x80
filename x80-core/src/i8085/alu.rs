//! Private 8085 arithmetic and result flags (plan §2.2; decision-10, Choice).
//!
//! Intel 205775-002 (January 1983), §5.5 "Condition Flags", p. 5-3
//! (PDF 88), defines S/Z/P, CY as carry or borrow, and AC as carry from
//! bit 3 into bit 4, explicitly including subtraction. §5.6.2, p. 5-6
//! (PDF 91), specifies two's-complement subtraction and CY=1 for borrow;
//! ADD/ADC and SUB/SBB, pp. 5-6 to 5-8 (PDF 91-93), supply the operations.
//! Thus subtraction retains the two's-complement addition's bit-3 carry
//! for AC, while CY reports borrow. Equivalently, AC means no low-nibble
//! borrow. This is derived from the 8085 rules, not carried over from 8080.
//! The remaining arithmetic entries on p. 5-9 are outside these primitives.
//!
//! Second witness: Intel 9800301-04 (May 1981), Ch. 1 "Condition Flags",
//! pp. 1-9 to 1-12 (PDF 19-22): CY as carry/borrow, S as bit 7, Z for
//! zero, P for even parity, AC as bit-3 carry, and subtraction among the
//! operations affecting AC. Both sources checked against rendered images;
//! their hashes match PROVENANCE.md §1.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Consumers arrive in the arithmetic and logical group beads."
    )
)]

use super::Flags;

/// S/Z/P from the byte result; AC/CY supplied by the operation.
/// 205775-002 §5.5 p. 5-3; 9800301-04 Ch. 1 pp. 1-10/1-11.
pub(super) const fn result_flags(value: u8, auxiliary_carry: bool, carry: bool) -> Flags {
    Flags {
        sign: value & 0x80 != 0,
        zero: value == 0,
        auxiliary_carry,
        parity: value.count_ones() & 1 == 0,
        carry,
    }
}

/// ADD/ADC: carry out of bits 3 and 7, including the carry input.
/// 205775-002 §5.5 p. 5-3, §5.6.2 pp. 5-6/5-7;
/// 9800301-04 Ch. 1 pp. 1-10 to 1-12.
pub(super) fn add(left: u8, right: u8, carry: bool) -> (u8, Flags) {
    let input = u16::from(carry);
    let total = u16::from(left) + u16::from(right) + input;
    let nibble_total = u16::from(left & 15) + u16::from(right & 15) + input;
    let value = total as u8;
    (value, result_flags(value, nibble_total >= 16, total >= 256))
}

/// SUB/SBB: CY reports borrow; AC retains the bit-3 carry (see module docs).
/// 205775-002 §5.5 p. 5-3, §5.6.2 pp. 5-6 to 5-8;
/// 9800301-04 Ch. 1 pp. 1-10 to 1-12.
pub(super) fn subtract(left: u8, right: u8, borrow: bool) -> (u8, Flags) {
    let input = i16::from(borrow);
    let difference = i16::from(left) - i16::from(right) - input;
    // A + !B + !borrow carries out of bit 3 exactly when this is nonnegative.
    let nibble_difference = i16::from(left & 15) - i16::from(right & 15) - input;
    let value = difference as u8;
    (
        value,
        result_flags(value, nibble_difference >= 0, difference < 0),
    )
}

#[cfg(test)]
#[path = "../../tests/i8085_arithmetic_reference/mod.rs"]
mod reference;

#[cfg(test)]
mod tests {
    use super::{add, reference, result_flags, subtract};

    #[test]
    fn result_flags_exhaustive() {
        for value in u8::MIN..=u8::MAX {
            for auxiliary in [false, true] {
                for carry in [false, true] {
                    assert_eq!(
                        result_flags(value, auxiliary, carry),
                        reference::result_flags(value, auxiliary, carry),
                        "value={value:02X}, AC={auxiliary}, CY={carry}"
                    );
                }
            }
        }
    }

    #[test]
    fn add_with_carry_exhaustive() {
        for left in u8::MIN..=u8::MAX {
            for right in u8::MIN..=u8::MAX {
                for carry in [false, true] {
                    assert_eq!(
                        add(left, right, carry),
                        reference::arithmetic(left, right, carry, false),
                        "left={left:02X}, right={right:02X}, carry={carry}"
                    );
                }
            }
        }
    }

    #[test]
    fn subtract_with_borrow_exhaustive() {
        for left in u8::MIN..=u8::MAX {
            for right in u8::MIN..=u8::MAX {
                for borrow in [false, true] {
                    assert_eq!(
                        subtract(left, right, borrow),
                        reference::arithmetic(left, right, borrow, true),
                        "left={left:02X}, right={right:02X}, borrow={borrow}"
                    );
                }
            }
        }
    }
}
