//! Shared Logical executor (decision-16 rules 1–3/5/6; plan §2.2).
//!
//! Primaries: Intel 98-153B (September 1975), Ch. 4 pp. 4-8 to 4-11;
//! Intel 205775-002 (January 1983), §5.6.3 pp. 5-9 to 5-12.
//! Second witness: Intel 9800301-04 (May 1981), Ch. 3 entries below.
//! Independent witnesses remain in tests/i8080_logical.rs and
//! tests/i8085_logical.rs. AND AC and subtraction flags stay model-owned.

use super::access::{operand, read_operand};
use super::{FlagRules, View};
use crate::decode::Path;
use crate::Bus;

/// Executes a model-admitted opcode after fetch; lengths/costs stay per model.
pub(crate) fn execute<S: Bus, F: FlagRules>(
    r: &mut View<'_, F>,
    system: &mut S,
    opcode: u8,
    length: u8,
) -> Path {
    match opcode {
        // Register/M/immediate encodings, SSS bits 2..0 and operation bits
        // 4..3: 98-153B Ch. 4 pp. 4-8 to 4-10; 205775-002 §5.2 p. 5-1,
        // §5.6.3 pp. 5-9 to 5-11. Sample A before writeback, including
        // source A. Second witness: 9800301-04 Ch. 3 pp. 3-6 to 3-8,
        // 3-12/3-13, 3-16/3-17, 3-38 to 3-40, 3-66 to 3-68.
        0xA0..=0xBF | 0xE6 | 0xEE | 0xF6 | 0xFE => {
            let operand = if opcode & 0xC0 == 0xC0 {
                // Immediate byte 2 at PC+1: 98-153B Ch. 2 p. 2-18,
                // note 6 p. 2-20; 205775-002 §2.3.1 p. 2-7,
                // Table 5-1 p. 5-19. Access kind: decision-5 item 3.
                operand(r, system, 1)
            } else {
                read_operand(r, system, opcode & 7)
            };
            let a = *r.a;
            match opcode & 0x18 {
                // ANA/ANI: AND into A, CY cleared: 98-153B Ch. 4
                // pp. 4-8/4-9; 205775-002 §5.6.3 pp. 5-9/5-10.
                // AC differs, so the complete rule stays per model
                // (decision-16 rule 3; decision-8's 8080 resolution).
                0x00 => (*r.a, *r.flags) = F::and(a, operand),
                // XRA/XRI: XOR, AC/CY cleared: 98-153B Ch. 4 p. 4-9;
                // 205775-002 §5.6.3 p. 5-10. Second witness:
                // 9800301-04 Ch. 3 pp. 3-66 to 3-68.
                0x08 => {
                    *r.a = a ^ operand;
                    *r.flags = F::result(*r.a, false, false);
                }
                // ORA/ORI: OR, AC/CY cleared: 98-153B Ch. 4 p. 4-9;
                // 205775-002 §5.6.3 pp. 5-10/5-11. Second witness:
                // 9800301-04 Ch. 3 pp. 3-38 to 3-40.
                0x10 => {
                    *r.a = a | operand;
                    *r.flags = F::result(*r.a, false, false);
                }
                // CMP/CPI: subtraction flags, A unchanged, no input borrow:
                // 98-153B Ch. 4 pp. 4-9/4-10; 205775-002 §5.6.3 p. 5-11.
                // Second witness: 9800301-04 Ch. 3 pp. 3-12/3-13,
                // 3-16/3-17. FlagRules retains each model's ALU derivation.
                _ => *r.flags = F::subtract(a, operand, false).1,
            }
        }
        // RLC/RRC: circular rotation by one, outgoing bit also enters CY;
        // only documented CY changes: 98-153B Ch. 4 p. 4-10; 205775-002 §5.6.3
        // pp. 5-11/5-12. Second witness: 9800301-04 Ch. 3 p. 3-49,
        // pp. 3-53/3-54. Encodings and bit positions are on those pages.
        // RAL/RAR: old CY enters A, outgoing bit replaces CY, other flags
        // documented unchanged: 98-153B Ch. 4 p. 4-10; 205775-002 §5.6.3 p. 5-12.
        // Second witness: 9800301-04 Ch. 3 pp. 3-45/3-46.
        0x07 | 0x0F | 0x17 | 0x1F => {
            let old_a = *r.a;
            let old_carry = r.flags.carry();
            let (result, carry) = match opcode {
                0x07 => (old_a.rotate_left(1), old_a & 0x80 != 0),
                0x0F => (old_a.rotate_right(1), old_a & 1 != 0),
                0x17 => ((old_a << 1) | u8::from(old_carry), old_a & 0x80 != 0),
                _ => ((old_a >> 1) | (u8::from(old_carry) << 7), old_a & 1 != 0),
            };
            *r.a = result;
            r.flags.set_carry(carry);
            // decision-19: model-owned rotate flag effects.
            r.flags.rotate(opcode, old_a, old_carry, result, carry);
        }
        // CMA: complement A, flags unchanged: 98-153B Ch. 4 p. 4-10;
        // 205775-002 §5.6.3 p. 5-12; 9800301-04 Ch. 3 p. 3-11.
        0x2F => *r.a = !*r.a,
        // CMC/STC: complement/set only CY: 98-153B Ch. 4 p. 4-11;
        // 205775-002 §5.6.3 p. 5-12; 9800301-04 Ch. 3 pp. 3-12/3-63.
        0x3F => r.flags.set_carry(!r.flags.carry()),
        0x37 => r.flags.set_carry(true),
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
    // Sequential bytes: 98-153B Ch. 4 p. 4-2, Ch. 2 pp. 2-16/2-18;
    // 205775-002 §5.4 p. 5-3, Table 5-1 p. 5-19. Wrap is DERIVED from
    // 16-bit PC (98-153B Ch. 2 p. 2-2; 205775-002 §2.2.1 p. 2-2),
    // not an explicitly verified wrap edge. Completed writeback at this
    // boundary is decision-5 item 8; no extra bus read for overlap
    // (98-153B Ch. 2 note 9 p. 2-20; 205775-002 Table 5-1 p. 5-19).
    *r.pc = r.pc.wrapping_add(u16::from(length));
    // Both cost slots contain the same model-owned total (decision-5 item 7).
    Path::Taken
}
