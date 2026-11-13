//! Shared Branch executor (decision-16 rules 1–6; plan §2.2/2.3).
//!
//! Primaries: Intel 98-153B (September 1975), Ch. 4 pp. 4-11/4-12;
//! Intel 205775-002 (January 1983), §5.6.4 pp. 5-13 to 5-15.
//! Second witness: Intel 9800301-04 (May 1981), Ch. 3 entries below.
//! Independent per-model branch and interrupt witnesses stay unchanged.

use super::access::{condition, operand, push_and_jump, read_address};
use super::{FlagRules, View};
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

/// Model-owned bus facts (decision-16 rule 4), cited at each model's constant.
/// The primaries differ: 98-153B Ch. 2 pp. 2-18/2-19, notes 12/13 p. 2-20;
/// 205775-002 §2.3.1 p. 2-7 and Table 5-1 p. 5-19. No default here.
pub(crate) struct BranchFacts {
    pub untaken_operand_reads: u8,
}

/// Executes a model-admitted branch after opcode fetch; lengths/costs stay
/// model-owned (decision-16 rules 1/2/5). Flags are unchanged: 98-153B Ch. 4
/// Branch Group p. 4-11; 205775-002 §5.6.4 p. 5-13.
pub(crate) fn execute<S: Bus, F: FlagRules>(
    r: &mut View<'_, F>,
    system: &mut S,
    opcode: u8,
    length: u8,
    facts: &BranchFacts,
) -> Path {
    // Sequential/return PC is after the complete instruction: 98-153B Ch. 4
    // pp. 4-11/4-12; 205775-002 §5.6.4 pp. 5-13/5-14. Wrapping is DERIVED
    // from 16-bit PC/SP widths (98-153B Ch. 2 p. 2-2;
    // 205775-002 §2.2.1/§2.2.3 p. 2-2), not an explicit wrap-edge account.
    let next_pc = r.pc.wrapping_add(u16::from(length));
    match opcode {
        // JMP/Jcond encodings and CCC bits 5..3: 98-153B Ch. 4 p. 4-11;
        // 205775-002 §5.6.4 p. 5-13. Taken operand order is in read_address;
        // untaken reads come only from the model's facts (decision-16 rule 4).
        // Second witness: 9800301-04 Ch. 3 pp. 3-26 to 3-32.
        0xC3 | 0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA => {
            if opcode == 0xC3 || condition(r, (opcode >> 3) & 7) {
                *r.pc = read_address(system, *r.pc);
                Path::Taken
            } else {
                skip_address(r, system, facts);
                *r.pc = next_pc;
                Path::Untaken
            }
        }
        // CALL/Ccond encodings and CCC bits 5..3, pushing next PC only if
        // taken: 98-153B Ch. 4 pp. 4-11/4-12; 205775-002 §5.6.4 pp. 5-13/
        // 5-14. Second witness: 9800301-04 Ch. 3 pp. 3-8 to 3-11, 3-14
        // to 3-18. Untaken bus effects remain model-supplied.
        0xCD | 0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => {
            if opcode == 0xCD || condition(r, (opcode >> 3) & 7) {
                let target = read_address(system, *r.pc);
                push_and_jump(r, system, next_pc, target);
                Path::Taken
            } else {
                skip_address(r, system, facts);
                *r.pc = next_pc;
                Path::Untaken
            }
        }
        // RET/Rcond encodings and CCC bits 5..3; pop low at SP then high at
        // SP+1, incrementing SP after each read, or no reads if false:
        // 98-153B Ch. 4 p. 4-12, Ch. 2 pp. 2-18/2-19, notes 14/15 p. 2-20;
        // 205775-002 §5.6.4 p. 5-14, Table 5-1 p. 5-19. Second witness:
        // 9800301-04 Ch. 3 pp. 3-47/3-48, 3-50 to 3-53, 3-55/3-56.
        // Stack access kind is the library mapping of decision-5 item 3.
        0xC9 | 0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8 => {
            if opcode == 0xC9 || condition(r, (opcode >> 3) & 7) {
                let low = system.read(*r.sp, MemoryAccess::Stack);
                *r.sp = r.sp.wrapping_add(1);
                let high = system.read(*r.sp, MemoryAccess::Stack);
                *r.sp = r.sp.wrapping_add(1);
                *r.pc = u16::from_le_bytes([low, high]);
                Path::Taken
            } else {
                *r.pc = next_pc;
                Path::Untaken
            }
        }
        // RST encodings, next PC and vector = 8*NNN (bits 5..3):
        // 98-153B Ch. 4 p. 4-12; 205775-002 §5.6.4 p. 5-14.
        // Second witness: 9800301-04 Ch. 3 pp. 3-54/3-55.
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            push_and_jump(r, system, next_pc, u16::from(opcode & 0x38));
            Path::Taken
        }
        // PCHL encoding and PCH=H, PCL=L, with no memory read:
        // 98-153B Ch. 4 p. 4-12, Ch. 2 p. 2-18 (only M1);
        // 205775-002 §5.6.4 p. 5-15, Table 5-1 p. 5-19 (S only).
        // Second witness: 9800301-04 Ch. 3 p. 3-42.
        0xE9 => {
            *r.pc = u16::from_be_bytes([*r.h, *r.l]);
            Path::Taken
        }
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
}

// Stream operands start at PC+1: 98-153B Ch. 2 pp. 2-18/2-19, note 6
// p. 2-20; 205775-002 §2.3.1 p. 2-7. The read count is model-owned;
// no high-byte read is inferred from instruction length (decision-16 rule 4).
fn skip_address<S: Bus, F>(r: &View<'_, F>, system: &mut S, facts: &BranchFacts) {
    for offset in 1..=facts.untaken_operand_reads {
        operand(r, system, u16::from(offset));
    }
}
