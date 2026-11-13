//! Independent Z80 ALU witnesses (decision-13 rule 7; decision-14 rules 5–6).
//!
//! Primary: Zilog UM008011-0816, revision 11 (August 2016), Tables 21/22
//! pp. 65–66, flag definitions pp. 67–69, and the entries cited per test.
//! Second witness: Zilog 03-0029-01 (1977), §6 pp. 39–40 and Tables
//! 7.0-4/5/6 pp. 47–49. PDF pages = printed + 14 (UM), + 6 (TM).
//! Both hashes match PROVENANCE.md §1. Pages were read as 150 dpi images;
//! disagreements were re-read as enlarged 200 dpi crops from both manuals.
//! The sources share documentation lineage; these are no hardware measurements.
//!
//! Reference arithmetic uses widened signed arithmetic, range checks and
//! low-digit comparisons; logical operations use a per-bit truth table.
//! Expected F is packed independently, without Flags or production helpers.
//! Decision-20 §4 H: Young v0.91 §§8.4–8.6 pp. 30–31 / Rak ALU groups.
//! All eight flag bits are compared, including CP source-operand Y/X.
//! Included from the private ALU, like i8085_arithmetic_reference: no Cargo
//! integration target or public ALU API is needed.
//!
//! Fail-before for every test below: the base has no z80::alu or helper API.
//! With only this module's test wiring present, compilation fails resolving
//! the helper imports (E0432), before any witness can execute. These helpers
//! do not execute encodings, so Unsupported/bus/PC/R/timing witnesses belong
//! to the consumer beads, not this helper-only bead.

use super::{adc16, add, add16, and, compare, decrement, increment, or, sbc16, subtract, xor};
use crate::z80::Flags;

/// Signed interpretation without a narrowing cast or sign-bit XOR formula.
fn signed(value: u16, width: u32) -> i64 {
    let modulus = 1_i64 << width;
    let value = i64::from(value);
    if value >= modulus / 2 {
        value - modulus
    } else {
        value
    }
}

/// UM pp. 67–69 and TM §6 p. 39: S is the high bit, Z tests the
/// whole result, P/V is supplied by the operation, and H/N/C are independent.
fn pack(value: u16, width: u32, half: bool, pv: bool, subtract: bool, carry: bool) -> u8 {
    // Decision-20 §4 H: Young §§8.4–8.6 pp. 30–31 / Rak ALU groups.
    let mut image = if width == 8 {
        value as u8 & 0x28
    } else {
        (value >> 8) as u8 & 0x28
    };
    for (condition, bit) in [
        (value >= (1 << (width - 1)), 7),
        (value == 0, 6),
        (half, 4),
        (pv, 2),
        (subtract, 1),
        (carry, 0),
    ] {
        if condition {
            image += 1 << bit;
        }
    }
    image
}

/// UM ADD/ADC pp. 145–146/152, SUB/SBC/CP pp. 154/156/164,
/// ADC/SBC HL pp. 190–193; TM §6 p. 39, Tables 7.0-4/6 pp. 47/49.
/// H is carry/borrow at the low 4 or 12 bits; P/V tests the mathematical
/// signed result's range. Wrapping follows the stated 8/16-bit widths.
fn arithmetic(left: u16, right: u16, input: bool, subtract: bool, width: u32) -> (u16, u8) {
    let input = i64::from(input);
    let direction = if subtract { -1 } else { 1 };
    let total = i64::from(left) + direction * (i64::from(right) + input);
    let signed_total = signed(left, width) + direction * (signed(right, width) + input);
    let modulus = 1_i64 << width;
    let digit_modulus = 1_u16 << (width - 4);
    let low_left = i64::from(left % digit_modulus);
    let low_right = i64::from(right % digit_modulus) + input;
    let half = if subtract {
        low_left < low_right
    } else {
        low_left + low_right >= i64::from(digit_modulus)
    };
    let value = total.rem_euclid(modulus) as u16;
    let overflow = !(-modulus / 2..modulus / 2).contains(&signed_total);
    (
        value,
        pack(
            value,
            width,
            half,
            overflow,
            subtract,
            !(0..modulus).contains(&total),
        ),
    )
}

/// UM AND/OR/XOR pp. 158/160/162, P/V definition p. 67;
/// TM §6 p. 39 and Table 7.0-4 p. 47 (P, not V, for all three).
fn logical(left: u8, right: u8, operation: u8) -> (u8, u8) {
    let mut value = 0;
    let mut ones = 0;
    for bit in 0..8 {
        let a = (left >> bit) % 2 == 1;
        let b = (right >> bit) % 2 == 1;
        let set = match operation {
            0 => a && b,
            1 => a || b,
            _ => a != b,
        };
        if set {
            value += 1 << bit;
            ones += 1;
        }
    }
    (
        value,
        pack(
            u16::from(value),
            8,
            operation == 0,
            ones % 2 == 0,
            false,
            false,
        ),
    )
}

fn check8(actual: (u8, Flags), expected: (u16, u8), context: (u8, u8, bool)) {
    assert_eq!(
        (u16::from(actual.0), actual.1.to_byte()),
        (expected.0, expected.1 | (expected.0 as u8 & 0x28)),
        "{context:?}"
    );
}

fn check16(actual: (u16, Flags), expected: (u16, u8), context: (u16, u16, bool)) {
    assert_eq!(
        (actual.0, actual.1.to_byte()),
        (expected.0, expected.1 | ((expected.0 >> 8) as u8 & 0x28)),
        "{context:?}"
    );
}

// UM ADD pp. 145–146, ADC p. 152; TM Table 7.0-4 p. 47.
#[test]
fn add_adc_exhaustive() {
    for left in 0..=u8::MAX {
        for right in 0..=u8::MAX {
            for carry in [false, true] {
                check8(
                    add(left, right, carry),
                    arithmetic(left.into(), right.into(), carry, false, 8),
                    (left, right, carry),
                );
            }
        }
    }
}

// UM SUB/SBC pp. 154/156, overflow definition p. 67; TM Table 7.0-4 p. 47.
#[test]
fn sub_sbc_exhaustive() {
    for left in 0..=u8::MAX {
        for right in 0..=u8::MAX {
            for borrow in [false, true] {
                check8(
                    subtract(left, right, borrow),
                    arithmetic(left.into(), right.into(), borrow, true, 8),
                    (left, right, borrow),
                );
            }
        }
    }
}

// UM CP p. 164: same flag rules as subtraction, without storing A.
// TM Table 7.0-4 p. 47 prints A-s with no assignment to A.
#[test]
fn cp_exhaustive_returns_only_flags() {
    for left in 0..=u8::MAX {
        for right in 0..=u8::MAX {
            let expected =
                (arithmetic(left.into(), right.into(), false, true, 8).1 & 0xD7) | (right & 0x28);
            assert_eq!(
                compare(left, right).to_byte(),
                expected,
                "{left:02X} {right:02X}"
            );
        }
    }
}

macro_rules! logical_witness {
    ($name:ident, $helper:ident, $operation:expr) => {
        #[test]
        fn $name() {
            for left in 0..=u8::MAX {
                for right in 0..=u8::MAX {
                    let expected = logical(left, right, $operation);
                    check8(
                        $helper(left, right),
                        (expected.0.into(), expected.1),
                        (left, right, false),
                    );
                }
            }
        }
    };
}

// UM pp. 158/160/162 (P/V text slips reconciled with p. 67);
// TM Table 7.0-4 p. 47. Each test checks every byte pair and all eight flag bits.
logical_witness!(and_exhaustive, and, 0);
logical_witness!(or_exhaustive, or, 1);
logical_witness!(xor_exhaustive, xor, 2);

// UM INC pp. 165–166 / DEC p. 171; TM Table 7.0-4 p. 47.
// Carry is preserved for both starting values; P/V is overflow, not parity.
#[test]
fn inc_dec_exhaustive_preserve_carry() {
    for value in 0..=u8::MAX {
        for carry in [false, true] {
            for (actual, subtract) in [
                (increment(value, carry), false),
                (decrement(value, carry), true),
            ] {
                let (result, flags) = arithmetic(value.into(), 1, false, subtract, 8);
                check8(
                    actual,
                    (result, (flags & !1) | u8::from(carry)),
                    (value, 1, carry),
                );
            }
        }
    }
}

const EDGES: [u16; 20] = [
    0, 1, 2, 0x000F, 0x0010, 0x007F, 0x0080, 0x00FF, 0x0100, 0x07FF, 0x0800, 0x0FFF, 0x1000,
    0x7FFE, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF, 0xF000,
];

fn expected_add16(left: u16, right: u16, old: u8) -> (u16, u8) {
    let (value, flags) = arithmetic(left, right, false, false, 16);
    (value, (flags & 0x39) | (old & 0xC4))
}

// UM ADD HL/IX/IY pp. 188–189/194–197: preserve S/Z/P-V independently
// of result, replace H/N/C. TM Table 7.0-6 p. 49 agrees except H unknown;
// existing PROVENANCE §1 row resolves H from the primary, not preservation.
#[test]
fn add16_edges_preserve_szpv_for_every_initial_flag_state() {
    for left in EDGES {
        for right in EDGES {
            for image in 0..=u8::MAX {
                // Decision-20 §4: all 256 initial full-byte images.
                check16(
                    add16(left, right, Flags::from_byte(image)),
                    expected_add16(left, right, image),
                    (left, right, false),
                );
            }
        }
    }
}

// UM ADC HL pp. 190–191 / SBC HL pp. 192–193; TM Table 7.0-6 p. 49.
// Z tests all 16 bits, S bit 15, P/V the signed 16-bit range, H bit 11/12.
#[test]
fn adc16_sbc16_edges_with_both_carry_inputs() {
    for left in EDGES {
        for right in EDGES {
            for input in [false, true] {
                check16(
                    adc16(left, right, input),
                    arithmetic(left, right, input, false, 16),
                    (left, right, input),
                );
                check16(
                    sbc16(left, right, input),
                    arithmetic(left, right, input, true, 16),
                    (left, right, input),
                );
            }
        }
    }
}

// Same citations as the pair-edge witnesses. Fixed LCG, no external oracle
// or random dependency: 262,144 pairs, both carry inputs for ADC/SBC and
// varied incoming flags for ADD. Each pair is selected before calling helpers.
#[test]
fn pair_arithmetic_large_deterministic_sample() {
    let mut state = 0x8A5C_739D_41E2_B607_u64;
    for _ in 0..262_144 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        let left = (state >> 16) as u16;
        let right = (state >> 48) as u16;
        let old = state as u8;
        check16(
            add16(left, right, Flags::from_byte(old)),
            expected_add16(left, right, old),
            (left, right, false),
        );
        for input in [false, true] {
            check16(
                adc16(left, right, input),
                arithmetic(left, right, input, false, 16),
                (left, right, input),
            );
            check16(
                sbc16(left, right, input),
                arithmetic(left, right, input, true, 16),
                (left, right, input),
            );
        }
    }
}

#[test]
fn manual_byte_arithmetic_examples() {
    // Literal entry examples; F is derived independently from each entry's
    // Condition Bits Affected, not printed as part of the numeric example.
    // UM p. 146: ADD A,C, A=44h, C=11h -> 55h.
    check8(add(0x44, 0x11, false), (0x55, 0x00), (0x44, 0x11, false));
    // UM p. 152: ADC A,(HL), A=16h, operand=10h, C=1 -> 27h.
    check8(add(0x16, 0x10, true), (0x27, 0x00), (0x16, 0x10, true));
    // UM p. 154: SUB D, A=29h, D=11h -> 18h.
    check8(
        subtract(0x29, 0x11, false),
        (0x18, 0x02),
        (0x29, 0x11, false),
    );
    // UM p. 156: SBC A,(HL), A=16h, operand=05h, C=1 -> 10h.
    check8(subtract(0x16, 0x05, true), (0x10, 0x02), (0x16, 0x05, true));
    // UM p. 164: CP (HL), A=63h, operand=60h, P/V reset.
    assert_eq!(compare(0x63, 0x60).to_byte(), 0x22);
}

#[test]
fn manual_logical_examples() {
    // UM p. 158: AND B, C3h AND 7Bh -> 43h (odd parity).
    check8(and(0xC3, 0x7B), (0x43, 0x10), (0xC3, 0x7B, false));
    // UM p. 160: OR H, 12h OR 48h -> 5Ah (even parity).
    check8(or(0x12, 0x48), (0x5A, 0x04), (0x12, 0x48, false));
    // UM p. 162: XOR 5Dh, 96h XOR 5Dh -> CBh (odd parity).
    check8(xor(0x96, 0x5D), (0xCB, 0x80), (0x96, 0x5D, false));
}

#[test]
fn manual_inc_dec_examples() {
    // UM p. 166: INC D, 28h -> 29h; p. 171: DEC D, 2Ah -> 29h.
    for carry in [false, true] {
        check8(
            increment(0x28, carry),
            (0x29, u8::from(carry)),
            (0x28, 1, carry),
        );
        check8(
            decrement(0x2A, carry),
            (0x29, 0x02 | u8::from(carry)),
            (0x2A, 1, carry),
        );
    }
}

#[test]
fn manual_pair_examples() {
    // UM p. 189: ADD HL,DE, 4242h + 1111h -> 5353h.
    // Initial S/Z/P-V all set intentionally: the example has no initial F.
    check16(
        add16(0x4242, 0x1111, Flags::from_byte(0xD7)),
        (0x5353, 0xC4),
        (0x4242, 0x1111, false),
    );
    // UM p. 191: ADC HL,BC, 5437h + 2222h + 1 -> 765Ah.
    check16(
        adc16(0x5437, 0x2222, true),
        (0x765A, 0x00),
        (0x5437, 0x2222, true),
    );
    // UM p. 193: SBC HL,DE, 9999h - 1111h - 1 -> 8887h.
    check16(
        sbc16(0x9999, 0x1111, true),
        (0x8887, 0x82),
        (0x9999, 0x1111, true),
    );
}

// New disagreement: UM p. 156 says P/V is reset both with and without
// overflow. UM p. 67's definition and TM §6 p. 39 / Table 7.0-4 p. 47
// agree on P/V SET for signed overflow. Both images re-read at 200 dpi.
// Resolve the entry's first "reset" as a text slip, per the requested rule.
#[test]
fn sbc_pv_text_slip_uses_signed_overflow() {
    check8(subtract(0x80, 0x01, true), (0x7E, 0x16), (0x80, 0x01, true));
    check8(subtract(0x00, 0x00, true), (0xFF, 0x93), (0x00, 0x00, true));
}

// New disagreements: UM AND p. 158 prints "reset if overflow"; OR p. 160
// prints "set if overflow". UM p. 67 explicitly assigns even parity to
// logical operations, as do TM §6 p. 39 and Table 7.0-4 p. 47 (P, not V).
// Both sources re-read at 200 dpi. Resolve from the agreeing definitions.
#[test]
fn and_or_pv_text_slips_use_even_parity() {
    check8(and(0, 0), (0, 0x54), (0, 0, false));
    check8(and(1, 1), (1, 0x10), (1, 1, false));
    check8(or(0, 0), (0, 0x44), (0, 0, false));
    check8(or(0, 1), (1, 0), (0, 1, false));
}

// New internal disagreement: UM pp. 195/197 print IX/IY=333h, BC=5555h,
// result=8888h. UM pp. 194/196 and TM Table 7.0-6 p. 49 define ordinary
// pair addition. The equations imply 5888h for the printed operands;
// 8888h instead requires 3333h. Images of both sources re-read at 200 dpi.
// Pin both operands explicitly; do not silently change the numeric example.
#[test]
fn index_add_examples_follow_the_printed_operands() {
    for (left, expected) in [(0x0333, 0x5888), (0x3333, 0x8888)] {
        check16(
            add16(left, 0x5555, Flags::default()),
            (expected, 0),
            (left, 0x5555, false),
        );
    }
}

// Existing PROVENANCE §1 disagreement: TM Table 7.0-6 p. 49 marks H
// unknown; UM pp. 188/190/192/194/196 explicitly define it. H stays in
// the full comparison. These derived edges distinguish bit 11 from byte/nibble boundaries,
// set and clear H, and distinguish carry from borrow for subtraction.
#[test]
fn pair_h_definedness_follows_primary_entries() {
    check16(
        add16(0x0FFF, 1, Flags::default()),
        (0x1000, 0x10),
        (0x0FFF, 1, false),
    );
    check16(
        add16(0x00FF, 1, Flags::from_byte(0x12)),
        (0x0100, 0),
        (0x00FF, 1, false),
    );
    check16(adc16(0x0FFF, 0, true), (0x1000, 0x10), (0x0FFF, 0, true));
    check16(adc16(0x00FF, 0, true), (0x0100, 0), (0x00FF, 0, true));
    check16(sbc16(0x1000, 0, true), (0x0FFF, 0x12), (0x1000, 0, true));
    check16(sbc16(0x0100, 0, true), (0x00FF, 0x02), (0x0100, 0, true));
}

// Literal flag-definition examples: UM p. 67 and TM §6 p. 39.
#[test]
fn overflow_definition_examples_are_distinct_from_carry_and_parity() {
    check8(add(120, 105, false), (0xE1, 0x94), (120, 105, false));
    // TM p. 39: -5 + -16 = -21, carry set without overflow.
    check8(add(0xFB, 0xF0, false), (0xEB, 0x81), (0xFB, 0xF0, false));
    // UM p. 67: +127 - (-64) = +191, outside signed byte range.
    check8(
        subtract(0x7F, 0xC0, false),
        (0xBF, 0x87),
        (0x7F, 0xC0, false),
    );
}

// Decision-20 §4 H: Young §§8.4–8.6 pp. 30–31 / Rak ALU groups.
#[test]
fn full_flag_images_include_class_selected_yx() {
    for (flags, expected) in [
        (add(0x7F, 0, true).1, 0x94),
        (subtract(0, 1, true).1, 0xBB),
        (compare(0, 1), 0x93),
        (and(0, 0).1, 0x54),
        (or(0, 0).1, 0x44),
        (xor(0, 0).1, 0x44),
        (increment(0x7F, true).1, 0x95),
        (decrement(0x80, true).1, 0x3F),
        (add16(0xFFFF, 1, Flags::from_byte(0xFF)).1, 0xD5),
        (adc16(0x7FFF, 0, true).1, 0x94),
        (sbc16(0, 0, true).1, 0xBB),
    ] {
        assert_eq!(flags.to_byte(), expected);
    }
}
