//! Independent 8085 witnesses (plan §2.2; decision-10, Choice).
//!
//! Intel 205775-002 (January 1983), §5.5 p. 5-3 (PDF 88) and §5.6.2
//! pp. 5-6 to 5-8 (PDF 91-93): S/Z/P, AC as bit-3 carry, CY as
//! carry/borrow, and two's-complement subtraction including borrow input.
//! Second witness: 9800301-04 (May 1981), Ch. 1 pp. 1-9 to 1-12
//! (PDF 19-22), especially AC's definition on p. 1-11 and subtraction
//! affecting AC on p. 1-12. Rendered pages checked. No crate helper is used;
//! only the model's flag data type is shared. Included by the private ALU's
//! unit tests, so this directory does not create an integration-test target.

use crate::i8085::Flags;

// Mechanical decision-18 migration: this reference still specifies only
// S/Z/AC/P/CY. V/K placeholders are excluded by its callers' comparisons;
// independent hidden-flag expectations live in i8085_vk_flags.rs.

/// Count parity by toggling once per set bit, independently of count_ones.
pub fn result_flags(value: u8, auxiliary_carry: bool, carry: bool) -> Flags {
    let mut remaining = value;
    let mut even = true;
    while remaining != 0 {
        even = !even;
        remaining &= remaining - 1;
    }
    Flags {
        overflow: false,
        k: false,
        sign: value >= 128,
        zero: value == 0,
        auxiliary_carry,
        parity: even,
        carry,
    }
}

/// Follow each bit's carry through a full adder, rather than using whole-byte
/// or nibble sums/differences. For subtraction, A + !B + !borrow implements
/// two's complement; retain the bit-3 carry and invert only the bit-7 carry.
pub fn arithmetic(left: u8, right: u8, input: bool, subtract: bool) -> (u8, Flags) {
    let operand = if subtract { !right } else { right };
    let mut carry = u8::from(input != subtract);
    let mut auxiliary = false;
    let mut value = 0;
    for bit in 0..8 {
        let column = ((left >> bit) & 1) + ((operand >> bit) & 1) + carry;
        value |= (column % 2) << bit;
        carry = column / 2;
        if bit == 3 {
            auxiliary = carry == 1;
        }
    }
    (
        value,
        result_flags(value, auxiliary, (carry == 1) != subtract),
    )
}

#[test]
fn literal_boundaries_guard_reference() {
    // Derived directly from 205775-002 §5.5 p. 5-3 and §5.6.2 pp. 5-6
    // to 5-8, witnessed by 9800301-04 pp. 1-10 to 1-12. AC for subtraction
    // is carry in the complemented addition, not low-nibble borrow.
    // Tuples: subtract, left, right, input, (value, S, Z, AC, P, CY).
    let vectors = [
        (
            false,
            0x00,
            0x00,
            false,
            (0x00, false, true, false, true, false),
        ),
        (
            false,
            0x00,
            0x00,
            true,
            (0x01, false, false, false, false, false),
        ),
        (
            false,
            0x0E,
            0x01,
            false,
            (0x0F, false, false, false, true, false),
        ),
        (
            false,
            0x0E,
            0x01,
            true,
            (0x10, false, false, true, false, false),
        ),
        (
            false,
            0x0F,
            0x01,
            false,
            (0x10, false, false, true, false, false),
        ),
        (
            false,
            0x7F,
            0x00,
            true,
            (0x80, true, false, true, false, false),
        ),
        (
            false,
            0x80,
            0x80,
            false,
            (0x00, false, true, false, true, true),
        ),
        (
            false,
            0xFF,
            0x00,
            true,
            (0x00, false, true, true, true, true),
        ),
        (
            false,
            0xFF,
            0xFF,
            true,
            (0xFF, true, false, true, true, true),
        ),
        // 9800301-04 p. 1-10's literal carry example, AE + 74 = 22 + carry.
        (
            false,
            0xAE,
            0x74,
            false,
            (0x22, false, false, true, true, true),
        ),
        // p. 1-11's literal zero-with-carry example, A7 + 59 = 00 + carry.
        (
            false,
            0xA7,
            0x59,
            false,
            (0x00, false, true, true, true, true),
        ),
        (
            true,
            0x00,
            0x00,
            false,
            (0x00, false, true, true, true, false),
        ),
        (
            true,
            0x00,
            0x00,
            true,
            (0xFF, true, false, false, true, true),
        ),
        (
            true,
            0x00,
            0x01,
            false,
            (0xFF, true, false, false, true, true),
        ),
        (
            true,
            0x10,
            0x01,
            false,
            (0x0F, false, false, false, true, false),
        ),
        (
            true,
            0x0F,
            0x00,
            true,
            (0x0E, false, false, true, false, false),
        ),
        (
            true,
            0x10,
            0x00,
            true,
            (0x0F, false, false, false, true, false),
        ),
        (
            true,
            0x11,
            0x01,
            false,
            (0x10, false, false, true, false, false),
        ),
        (
            true,
            0x11,
            0x01,
            true,
            (0x0F, false, false, false, true, false),
        ),
        (
            true,
            0x7F,
            0x80,
            false,
            (0xFF, true, false, true, true, true),
        ),
        (
            true,
            0x80,
            0x01,
            false,
            (0x7F, false, false, false, false, false),
        ),
        (
            true,
            0xFF,
            0xFF,
            false,
            (0x00, false, true, true, true, false),
        ),
        (
            true,
            0x00,
            0xFF,
            true,
            (0x00, false, true, false, true, true),
        ),
        (
            true,
            0xFF,
            0xFF,
            true,
            (0xFF, true, false, false, true, true),
        ),
    ];
    for (subtract, left, right, input, expected) in vectors {
        let (value, flags) = arithmetic(left, right, input, subtract);
        assert_eq!(
            (
                value,
                flags.sign,
                flags.zero,
                flags.auxiliary_carry,
                flags.parity,
                flags.carry,
            ),
            expected,
            "subtract={subtract}, left={left:02X}, right={right:02X}, input={input}"
        );
    }
}
