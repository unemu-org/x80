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

use super::Flags;

/// S/Z/P from the byte result; AC/CY supplied by the operation.
/// V=0, K=sign is L for ANA/ORA/XRA (decision-18 §4, KV note [4],
/// ALU constant-carry/enables; exact hardware effect unverified).
/// 205775-002 §5.5 p. 5-3; 9800301-04 Ch. 1 pp. 1-10/1-11.
pub(super) const fn result_flags(value: u8, auxiliary_carry: bool, carry: bool) -> Flags {
    Flags {
        overflow: false,
        k: value & 0x80 != 0,
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
    let mut flags = result_flags(value, nibble_total >= 16, total >= 256);
    // D/S: signed ADD/ADC overflow including carry input; INR is +1.
    // decision-18 §4, W p. 145/T Fig. 2-9 p. 2-15, KV V circuit and
    // ALU store_v_flag/arith_to_flags. K's silicon conflict resolution
    // selects V XOR computed result sign (KV note [4]).
    flags.overflow = (!(left ^ right) & (left ^ value) & 0x80) != 0;
    flags.k = flags.overflow ^ flags.sign;
    (value, flags)
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
    let mut flags = result_flags(value, nibble_difference >= 0, difference < 0);
    // D/S: signed SUB/SBB/CMP overflow including borrow; DCR is -1.
    // decision-18 §4, W complemented-O2 rule/T Fig. 2-9 p. 2-15;
    // KV circuit/note [5], ALU enables. Internal overflow is independent
    // of exposed CY borrow; CMP's temporary result supplies K's sign.
    flags.overflow = ((left ^ right) & (left ^ value) & 0x80) != 0;
    flags.k = flags.overflow ^ flags.sign;
    (value, flags)
}

/// 8085 ANA/ANI: AND into A, CY cleared and AC set, independently of
/// operands: Intel 205775-002 (January 1983), §5.6.3 pp. 5-9/5-10;
/// Intel 9800301-04 (May 1981), Ch. 1 p. 1-12, Ch. 3 pp. 3-6 to 3-8.
/// S/Z/P: 205775-002 §5.5 p. 5-3. Witness: tests/i8085_logical.rs,
/// every A/operand/source. Model-owned behind FlagRules::and
/// (decision-16 rule 3); the 8080 AC rule differs.
pub(super) const fn and(a: u8, operand: u8) -> (u8, Flags) {
    let value = a & operand;
    (value, result_flags(value, true, false))
}

/// Model-owned completion for the separate 16-bit incrementer.
pub(super) fn pair_step(flags: &mut Flags, old: u16, decrement: bool) {
    // K D/S: decision-18 §4, W p. 144/T Fig. 2-9 p. 2-15;
    // KV incrementer circuit, ALU carry_to_k_flag. V preserved L: the
    // sources do not state its exact effect; no ALU result-sign equation.
    flags.k = old == if decrement { 0 } else { 0xFFFF };
}

/// Model-owned DAD completion; documented flags already came from execution.
pub(super) fn dad(flags: &mut Flags, left: u16, right: u16, result: u16) {
    // V S / die-reading reconciliation: decision-18 §4, W p. 145's
    // arithmetic definition/T Fig. 2-9 p. 2-15's 16-bit V and KV note [4]
    // explicit DAD statement take precedence over an omitted enable-list
    // item. Whole-word overflow includes the low-byte carry. K preserved L.
    flags.overflow = (!(left ^ right) & (left ^ result) & 0x8000) != 0;
}

/// Model-owned rotation completion; S/Z/AC/P and K remain independent.
pub(super) fn rotate(flags: &mut Flags, old_a: u8, result: u8, right: bool) {
    // L: decision-18 §4, following KV note [4]/ALU V controls.
    // RRC/RAR clear V; RLC/RAL use signed A+A (+old CY for RAL).
    // The completed result contains the carry input where applicable;
    // either left addition overflows exactly when its sign changes.
    // K preservation is L: no explicit rotate K effect, never use S latch.
    flags.overflow = !right && (old_a ^ result) & 0x80 != 0;
}

/// Model-owned DAA completion, retaining the existing five-flag derivation.
pub(super) fn daa(flags: &mut Flags, old_a: u8, correction: u8, result: u8) {
    // L: decision-18 §4 exact combined ACT correction, ALU "The accumulator
    // and ACT registers" and KV V circuit. The shared executor supplies its
    // combined correction; incoming CY is not an extra addend. This also
    // retains 7A+06 overflow when there is no high correction, and sets V
    // for 20+60 (not only a top-digit 7->8 transition). K=V XOR result sign.
    flags.overflow = (!(old_a ^ correction) & (old_a ^ result) & 0x80) != 0;
    flags.k = flags.overflow ^ (result & 0x80 != 0);
}

#[cfg(test)]
#[path = "../../tests/i8085_arithmetic_reference/mod.rs"]
mod reference;

#[cfg(test)]
mod tests {
    use super::Flags;
    use super::{add, reference, result_flags, subtract};

    // Mechanical decision-18 migration: retain every original five-flag
    // expectation; independent V/K literals live in i8085_vk_flags.rs.
    fn documented(mut flags: Flags) -> Flags {
        flags.overflow = false;
        flags.k = false;
        flags
    }

    #[test]
    fn result_flags_exhaustive() {
        for value in u8::MIN..=u8::MAX {
            for auxiliary in [false, true] {
                for carry in [false, true] {
                    assert_eq!(
                        documented(result_flags(value, auxiliary, carry)),
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
                    let (value, flags) = add(left, right, carry);
                    assert_eq!(
                        (value, documented(flags)),
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
                    let (value, flags) = subtract(left, right, borrow);
                    assert_eq!(
                        (value, documented(flags)),
                        reference::arithmetic(left, right, borrow, true),
                        "left={left:02X}, right={right:02X}, borrow={borrow}"
                    );
                }
            }
        }
    }
}
