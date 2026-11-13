//! Shared Arithmetic executor (decision-16 rules 1–3/5/6; plan §2.2).
//!
//! Primaries: Intel 98-153B (September 1975), Ch. 4 pp. 4-6 to 4-8;
//! Intel 205775-002 (January 1983), §5.6.2 pp. 5-6 to 5-9.
//! Second witness: Intel 9800301-04 (May 1981), Ch. 3 entries below.
//! Independent arithmetic and DAA witnesses remain separate for each model.
//! ALU flags stay behind FlagRules; DAA carry derivations stay model-owned.

use super::access::{operand, read_operand, read_pair, write_operand, write_pair};
use super::{FlagRules, View};
use crate::decode::Path;
use crate::Bus;

/// Model-owned derived DAA carry policy (decision-16 rule 6).
/// Neither 98-153B Ch. 4 pp. 4-2/4-8 nor 205775-002 §§5.5/5.6.2
/// pp. 5-3/5-9 explicitly settles incoming CY retention or the low step's
/// bit-7 carry propagation.
/// Each model's constant retains its derivation and independent witnesses.
pub(crate) struct DaaCarry {
    pub retain_incoming: bool,
    pub propagate_low: bool,
}

/// Executes a model-admitted opcode after fetch; lengths/costs stay per model.
pub(crate) fn execute<S: Bus, F: FlagRules>(
    r: &mut View<'_, F>,
    system: &mut S,
    opcode: u8,
    length: u8,
    daa_carry: DaaCarry,
) -> Path {
    match opcode {
        // DAA encoding and two-step correction: low digit >9 or AC adds 6,
        // then adjusted high digit >9 or CY adds 6 to that digit:
        // 98-153B Ch. 4 p. 4-8; 205775-002 §5.6.2 p. 5-9.
        // Second witness: 9800301-04 Ch. 3 p. 3-19. Carry retention and
        // propagation of step 1's bit-7 carry are DERIVED per model; consume
        // the model's policy rather than claim a both-primary statement.
        0x27 => {
            let old_a = *r.a;
            let correction = if *r.a & 0x0F > 9 || r.flags.auxiliary_carry() {
                6
            } else {
                0
            };
            let (low, low_flags) = F::add(*r.a, correction, false);
            let low_carry = daa_carry.propagate_low && low_flags.carry();
            let carry_for_high = r.flags.carry() || low_carry;
            let high_correction = if low >> 4 > 9 || carry_for_high {
                0x60
            } else {
                0
            };
            let (value, high_carry) = if high_correction != 0 {
                low.overflowing_add(0x60)
            } else {
                (low, false)
            };
            let carry = (daa_carry.retain_incoming && r.flags.carry()) || low_carry || high_carry;
            *r.a = value;
            // All flags affected; S/Z/P from the final byte, AC from the
            // low step's bit-3 carry: 98-153B Ch. 4 pp. 4-2/4-3/4-8;
            // 205775-002 §§5.5/5.6.2 pp. 5-3/5-9. Second witness:
            // 9800301-04 Ch. 3 p. 3-19's 9BH -> A1H -> 01H keeps AC=1
            // through the high adjustment; Ch. 1 pp. 1-10/1-11.
            *r.flags = F::result(value, low_flags.auxiliary_carry(), carry);
            // decision-19: model-owned effects of the combined correction.
            r.flags.daa(old_a, correction + high_correction, value);
        }
        // ADD/ADI/ADC/ACI/SUB/SUI/SBB/SBI encodings, source bits 2..0,
        // operation bits 4..3 and CY input only for ADC/ACI/SBB/SBI:
        // 98-153B Ch. 4 pp. 4-6/4-7; 205775-002 §§5.2/5.6.2
        // pp. 5-1, 5-6 to 5-8. Sample A before replacing it, including
        // source A. Second witness: 9800301-04 Ch. 3 pp. 3-2 to 3-5,
        // 3-56 to 3-58, 3-63 to 3-65. ALU facts stay model-owned.
        0x80..=0x9F | 0xC6 | 0xCE | 0xD6 | 0xDE => {
            let operand = if opcode & 0xC0 == 0xC0 {
                // Byte 2 at PC+1: 98-153B Ch. 2 pp. 2-16/2-17,
                // note 6 p. 2-20; 205775-002 §§5.4/5.6.2 pp. 5-3,
                // 5-6 to 5-8, Table 5-1 p. 5-19. OperandFetch is the
                // library distinction of decision-5 item 3.
                operand(r, system, 1)
            } else {
                read_operand(r, system, opcode & 7)
            };
            let operation = (opcode >> 3) & 3;
            let input = operation & 1 != 0 && r.flags.carry();
            (*r.a, *r.flags) = if operation & 2 != 0 {
                F::subtract(*r.a, operand, input)
            } else {
                F::add(*r.a, operand, input)
            };
        }
        // INR/DCR encodings, DDD bits 5..3, +/-1 with CY preserved:
        // 98-153B Ch. 4 pp. 4-3/4-7/4-8; 205775-002 §§5.2/5.6.2
        // pp. 5-1/5-8. M reads then writes the same HL address:
        // 98-153B Ch. 2 pp. 2-16/2-17; 205775-002 Table 5-1 p. 5-19.
        // Second witness: 9800301-04 Ch. 3 pp. 3-20/3-21/3-25.
        // AC is model-owned, including each subtraction derivation.
        _ if opcode & 0xC6 == 0x04 => {
            let destination = (opcode >> 3) & 7;
            let operand = read_operand(r, system, destination);
            let (value, mut flags) = if opcode & 1 == 0 {
                F::add(operand, 1, false)
            } else {
                F::subtract(operand, 1, false)
            };
            flags.set_carry(r.flags.carry());
            write_operand(r, system, destination, value);
            *r.flags = flags;
        }
        // INX/DCX encodings, RP bits 5..4, 16-bit pair +/-1, no documented flags:
        // 98-153B Ch. 4 pp. 4-3/4-8; 205775-002 §§5.2/5.6.2
        // pp. 5-1/5-9. No memory access beyond fetch: 98-153B Ch. 2
        // pp. 2-16/2-17; 205775-002 Table 5-1 p. 5-19 (S only).
        // Second witness: 9800301-04 Ch. 3 pp. 3-22/3-26. Wrapping is
        // DERIVED from 16-bit pairs (98-153B Ch. 2 pp. 2-2/2-3;
        // 205775-002 §2.2.2 p. 2-2), not an explicit wrap-edge account.
        _ if opcode & 0xC7 == 0x03 => {
            let pair = (opcode >> 4) & 3;
            let value = read_pair(r, pair);
            let result = if opcode & 8 == 0 {
                value.wrapping_add(1)
            } else {
                value.wrapping_sub(1)
            };
            write_pair(r, pair, result);
            // decision-19: model-owned pair-step flag effects.
            r.flags.pair_step(value, result, opcode & 8 != 0);
        }
        // DAD encoding, RP bits 5..4, HL <- HL + rp, documented CY updated:
        // 98-153B Ch. 4 pp. 4-3/4-8; 205775-002 §§5.2/5.6.2
        // pp. 5-1/5-9. Internal cycles make no memory reference:
        // 98-153B Ch. 2 note 8 p. 2-20; 205775-002 Table 5-1 p. 5-19
        // (FBB), Figs. 2-10/2-12 pp. 2-8/2-9. Second witness:
        // 9800301-04 Ch. 3 p. 3-20; 231718-001 Tables 3/4, handbook
        // p. 16-20. Wrapped HL is DERIVED from the widths cited above.
        _ if opcode & 0xCF == 0x09 => {
            let operand = read_pair(r, (opcode >> 4) & 3);
            let left = u16::from_be_bytes([*r.h, *r.l]);
            let (value, carry) = left.overflowing_add(operand);
            [*r.h, *r.l] = value.to_be_bytes();
            r.flags.set_carry(carry);
            // decision-19: model-owned word-add flag effects.
            r.flags.dad(left, operand, value);
        }
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
    // Sequential instruction bytes: 98-153B Ch. 4 p. 4-2, Ch. 2
    // pp. 2-16/2-17; 205775-002 §5.4 p. 5-3, Table 5-1 p. 5-19.
    // PC wrap is DERIVED from 16-bit PC (98-153B Ch. 2 p. 2-2;
    // 205775-002 §2.2.1 p. 2-2). Completed writeback at this boundary is
    // decision-5 item 8; overlap adds no bus read (98-153B Ch. 2 note 9
    // p. 2-20; 205775-002 Table 5-1 p. 5-19).
    *r.pc = r.pc.wrapping_add(u16::from(length));
    // Both cost slots have the same model-owned total (decision-5 item 7).
    Path::Taken
}
