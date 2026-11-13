//! Z80 ALU primitives (decision-13 rule 7; decision-14 rules 5–6).
//!
//! Primary: Zilog UM008011-0816, revision 11 (August 2016), Tables 21/22
//! pp. 65–66 and flag definitions pp. 67–69. Second witness: Zilog
//! 03-0029-01 (1977), §6 pp. 39–40, Tables 7.0-4/5/6 pp. 47–49.
//! Printed + 14 = PDF for UM; printed + 6 = PDF for TM. Hashes match
//! PROVENANCE.md §1. Values checked on rendered 150 dpi page images.
//!
//! P/V means signed overflow for arithmetic and even parity for logical
//! operations. UM p. 67 and TM §6 p. 39 / Table 7.0-4 p. 47 resolve
//! text slips in UM SBC p. 156 ("reset if overflow"), AND p. 158 (also
//! "reset if overflow"), and OR p. 160 ("set if overflow"). Both sources
//! were re-read at 200 dpi with enlarged crops; dedicated witnesses pin
//! the requested overflow/parity readings. These new disagreements are
//! recorded in the owned reference tests for supervisor provenance upkeep.
//!
//! TM Table 7.0-6 p. 49 marks pair H unknown. The primary's explicit
//! bit-11 carry / bit-12 borrow rules prevail (existing PROVENANCE §1 row;
//! decision-14 rule 6). No flags here are unknown under the primary.
//! Flags stores only the six documented flags; zero F bits 5/3 are a
//! library choice, not a hardware claim (decision-14 rule 5).

use super::Flags;

/// Result S/Z, with the operation supplying H/P-V/N/C. UM pp. 65–69;
/// TM §6 p. 39. Arithmetic supplies overflow; logical ops supply parity.
fn flags8(
    value: u8,
    half_carry: bool,
    parity_overflow: bool,
    subtract: bool,
    carry: bool,
) -> Flags {
    Flags {
        sign: value & 0x80 != 0,
        zero: value == 0,
        half_carry,
        parity_overflow,
        subtract,
        carry,
    }
}

/// ADD/ADC: H carries from bit 3, C from bit 7, N resets, P/V is
/// overflow. UM ADD A,r pp. 145–146, ADC p. 152, P/V p. 67;
/// TM §6 p. 39, Table 7.0-4 p. 47. Use carry=false for ADD.
pub(super) fn add(left: u8, right: u8, carry: bool) -> (u8, Flags) {
    let (partial, first_carry) = left.overflowing_add(right);
    let (value, second_carry) = partial.overflowing_add(u8::from(carry));
    // Derived: XOR of operands and result at bit 4 is the carry into it,
    // including carry-in. Overflow is same-sign operands changing sign.
    let half = (left ^ right ^ value) & 0x10 != 0;
    let overflow = (!(left ^ right) & (left ^ value)) & 0x80 != 0;
    (
        value,
        flags8(value, half, overflow, false, first_carry || second_carry),
    )
}

/// SUB/SBC: H is BORROW FROM bit 4, C is borrow, N sets; P/V is
/// signed overflow, not parity. UM SUB p. 154, SBC p. 156 (P/V slip
/// resolved in module docs), P/V p. 67, Table 23 p. 68;
/// TM §6 p. 39, Table 7.0-4 p. 47. Use borrow=false for SUB.
pub(super) fn subtract(left: u8, right: u8, borrow: bool) -> (u8, Flags) {
    let (partial, first_borrow) = left.overflowing_sub(right);
    let (value, second_borrow) = partial.overflowing_sub(u8::from(borrow));
    // Derived: at bit 4, left XOR right XOR result is borrow into that
    // bit. Unlike-sign operands overflow when the result changes left's sign.
    let half = (left ^ right ^ value) & 0x10 != 0;
    let overflow = ((left ^ right) & (left ^ value)) & 0x80 != 0;
    (
        value,
        flags8(value, half, overflow, true, first_borrow || second_borrow),
    )
}

/// CP supplies subtraction's flags without returning a replacement A.
/// UM CP p. 164; TM Table 7.0-4 p. 47 (A-s, no assignment to A).
pub(super) fn compare(left: u8, right: u8) -> Flags {
    subtract(left, right, false).1
}

/// Logical results have even parity, N=C=0; H differs for AND.
/// UM P/V p. 67, AND/OR/XOR pp. 158/160/162 (slips: module docs);
/// TM §6 p. 39, Table 7.0-4 p. 47.
fn logical_flags(value: u8, half: bool) -> Flags {
    flags8(value, half, value.count_ones() & 1 == 0, false, false)
}

/// AND: H=1. UM p. 158, P/V p. 67; TM Table 7.0-4 p. 47.
pub(super) fn and(left: u8, right: u8) -> (u8, Flags) {
    let value = left & right;
    (value, logical_flags(value, true))
}

/// OR: H=0. UM p. 160, P/V p. 67; TM Table 7.0-4 p. 47.
pub(super) fn or(left: u8, right: u8) -> (u8, Flags) {
    let value = left | right;
    (value, logical_flags(value, false))
}

/// XOR: H=0. UM p. 162; TM Table 7.0-4 p. 47.
pub(super) fn xor(left: u8, right: u8) -> (u8, Flags) {
    let value = left ^ right;
    (value, logical_flags(value, false))
}

/// INC: byte addition of one, preserving C (not used as input).
/// P/V sets exactly for 7Fh -> 80h. UM pp. 165–166;
/// TM §6 p. 39, Table 7.0-4 p. 47.
pub(super) fn increment(value: u8, carry: bool) -> (u8, Flags) {
    let (result, mut flags) = add(value, 1, false);
    flags.carry = carry;
    (result, flags)
}

/// DEC: byte subtraction of one, preserving C (not used as input).
/// P/V sets exactly for 80h -> 7Fh; H is borrow from bit 4, N=1.
/// UM p. 171; TM §6 p. 39, Table 7.0-4 p. 47.
pub(super) fn decrement(value: u8, carry: bool) -> (u8, Flags) {
    let (result, mut flags) = subtract(value, 1, false);
    flags.carry = carry;
    (result, flags)
}

/// ADD HL/IX/IY: H from bit 11, C from bit 15, N=0; S/Z/P-V
/// unchanged. UM pp. 188–189, 194–197; TM Table 7.0-6 p. 49
/// (H disagreement: module docs). Incoming C does not enter the sum.
pub(super) fn add16(left: u16, right: u16, previous: Flags) -> (u16, Flags) {
    let (value, carry) = left.overflowing_add(right);
    let flags = Flags {
        half_carry: (left ^ right ^ value) & 0x1000 != 0,
        subtract: false,
        carry,
        ..previous
    };
    (value, flags)
}

/// ADC HL: all six flags from the 16-bit operation, P/V is signed
/// overflow, H carries from bit 11, C from bit 15, N=0.
/// UM pp. 190–191; TM §6 p. 39, Table 7.0-6 p. 49 (H: module docs).
pub(super) fn adc16(left: u16, right: u16, carry: bool) -> (u16, Flags) {
    let (partial, first_carry) = left.overflowing_add(right);
    let (value, second_carry) = partial.overflowing_add(u16::from(carry));
    let flags = Flags {
        sign: value & 0x8000 != 0,
        zero: value == 0,
        half_carry: (left ^ right ^ value) & 0x1000 != 0,
        parity_overflow: (!(left ^ right) & (left ^ value)) & 0x8000 != 0,
        subtract: false,
        carry: first_carry || second_carry,
    };
    (value, flags)
}

/// SBC HL: all six flags from the 16-bit operation, P/V is signed
/// overflow, H is BORROW FROM bit 12, C is borrow, N=1.
/// UM pp. 192–193; TM §6 p. 39, Table 7.0-6 p. 49 (H: module docs).
pub(super) fn sbc16(left: u16, right: u16, borrow: bool) -> (u16, Flags) {
    let (partial, first_borrow) = left.overflowing_sub(right);
    let (value, second_borrow) = partial.overflowing_sub(u16::from(borrow));
    let flags = Flags {
        sign: value & 0x8000 != 0,
        zero: value == 0,
        half_carry: (left ^ right ^ value) & 0x1000 != 0,
        parity_overflow: ((left ^ right) & (left ^ value)) & 0x8000 != 0,
        subtract: true,
        carry: first_borrow || second_borrow,
    };
    (value, flags)
}

#[cfg(test)]
#[path = "../../tests/z80_arithmetic_reference/mod.rs"]
mod reference;
