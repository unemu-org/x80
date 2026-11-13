//! Shared Stack/I/O subset (decision-16 rules 3/5/6; plan §2.2/§2.6).
//!
//! Primaries: Intel 98-153B (September 1975), Ch. 4 pp. 4-13/4-14;
//! Intel 205775-002 (January 1983), §5.6.5 pp. 5-15/5-16.
//! Second witness: Intel 9800301-04 (May 1981), Ch. 3 entries below.
//! Independent witnesses remain in each model's stack_io_control,
//! registers, timing and coverage integration tests, unchanged.
//! PSW images and XTHL address order remain model-owned; MemoryAccess
//! kinds and eight-bit ports follow decision-5 item 3.

use super::access::operand;
use super::{FlagRules, View};
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

/// Model-supplied XTHL address order (decision-16 rule 6).
/// Offsets index the L/H bytes at SP/SP+1: 98-153B Ch. 4 p. 4-14;
/// 205775-002 §5.6.5 p. 5-16. Each model supplies a permutation of
/// these offsets for reads and writes, retaining its evidential status.
pub(crate) struct XthlOrder {
    pub reads: [u16; 2],
    pub writes: [u16; 2],
}

/// Executes only the shared subset; model wrappers handle control first.
pub(crate) fn execute<S: Bus, F: FlagRules>(
    r: &mut View<'_, F>,
    system: &mut S,
    opcode: u8,
    length: u8,
    xthl_order: XthlOrder,
) -> Path {
    match opcode {
        // PUSH rp/PSW: high at SP-1, low at SP-2, then SP -= 2;
        // flags unchanged. Encodings/pair selection and PSW A/flags order:
        // 98-153B Ch. 4 pp. 4-3/4-13, Ch. 2 pp. 2-18/2-19, note 16
        // p. 2-20; 205775-002 §5.2 p. 5-1, §5.6.5 p. 5-15,
        // §2.2.3 p. 2-2, Table 5-1 p. 5-19 (SWW).
        // Second witness: 9800301-04 Ch. 3 pp. 3-44/3-45.
        // FlagRules supplies the model's PSW image (decision-16 rule 3).
        0xC5 | 0xD5 | 0xE5 | 0xF5 => {
            let [high, low] = match opcode {
                0xC5 => [*r.b, *r.c],
                0xD5 => [*r.d, *r.e],
                0xE5 => [*r.h, *r.l],
                _ => [*r.a, r.flags.to_byte()],
            };
            system.write(r.sp.wrapping_sub(1), high, MemoryAccess::Stack);
            system.write(r.sp.wrapping_sub(2), low, MemoryAccess::Stack);
            *r.sp = r.sp.wrapping_sub(2);
        }
        // POP rp/PSW: low from SP, high from SP+1, then SP += 2;
        // only PSW restores flags. Encodings, pair selection and A/flags:
        // 98-153B Ch. 4 pp. 4-3/4-13, Ch. 2 pp. 2-18/2-19, note 15
        // p. 2-20; 205775-002 §5.2 p. 5-1, §5.6.5 pp. 5-15/5-16,
        // Table 5-1 p. 5-19 (FRR). Second witness: 9800301-04 Ch. 3
        // p. 3-43. FlagRules retains the model-owned filler-bit handling.
        0xC1 | 0xD1 | 0xE1 | 0xF1 => {
            let low = system.read(*r.sp, MemoryAccess::Stack);
            let high = system.read(r.sp.wrapping_add(1), MemoryAccess::Stack);
            match opcode {
                0xC1 => [*r.b, *r.c] = [high, low],
                0xD1 => [*r.d, *r.e] = [high, low],
                0xE1 => [*r.h, *r.l] = [high, low],
                _ => {
                    *r.a = high;
                    *r.flags = F::from_byte(low);
                }
            }
            *r.sp = r.sp.wrapping_add(2);
        }
        // XTHL: L <-> [SP], H <-> [SP+1], SP/flags unchanged, RRWW:
        // 98-153B Ch. 4 p. 4-14, Ch. 2 pp. 2-18/2-19, notes 15/16
        // p. 2-20; 205775-002 §5.6.5 p. 5-16, Table 5-1 p. 5-19.
        // Second witness: 9800301-04 Ch. 3 p. 3-69. Address order within
        // RR/WW is model-supplied: the 8085 order remains unverified
        // (decision-16 rule 6). Register replacement follows all accesses.
        0xE3 => {
            let old = [*r.l, *r.h];
            let mut new = [0; 2];
            for offset in xthl_order.reads {
                new[usize::from(offset)] =
                    system.read(r.sp.wrapping_add(offset), MemoryAccess::Stack);
            }
            for offset in xthl_order.writes {
                system.write(
                    r.sp.wrapping_add(offset),
                    old[usize::from(offset)],
                    MemoryAccess::Stack,
                );
            }
            [*r.l, *r.h] = new;
        }
        // SPHL: SP <- HL, no flags or memory data access:
        // 98-153B Ch. 4 p. 4-14, Ch. 2 p. 2-16;
        // 205775-002 §5.6.5 p. 5-16, Table 5-1 p. 5-19 (S).
        // Second witness: 9800301-04 Ch. 3 p. 3-61.
        0xF9 => *r.sp = u16::from_be_bytes([*r.h, *r.l]),
        // IN/OUT: byte 2 is the port, fetched before I/O; IN loads A,
        // OUT sends A; no flags changed. Encodings and transfer sequence:
        // 98-153B Ch. 4 p. 4-14, Ch. 2 p. 2-9 Figs. 2-6/2-7,
        // pp. 2-18/2-19; 205775-002 §5.6.5 p. 5-16,
        // §§2.3.2/2.3.3 pp. 2-12/2-13, Table 5-1 p. 5-19 (FRI/FRO).
        // Second witness: 9800301-04 Ch. 3 pp. 3-24/3-25, 3-41.
        0xDB | 0xD3 => {
            let port = operand(r, system, 1);
            if opcode == 0xDB {
                *r.a = system.input(port);
            } else {
                system.output(port, *r.a);
            }
        }
        _ => unreachable!("model wrapper delegates only shared Stack/I/O encodings"),
    }
    // Sequential PC: 98-153B Ch. 4 p. 4-2, Ch. 2 pp. 2-16/2-18;
    // 205775-002 §5.4 p. 5-3. PC/SP/address wrap is DERIVED from the
    // 16-bit widths (98-153B Ch. 2 p. 2-2; 205775-002 §§2.2.2/2.2.3
    // p. 2-2), not an explicitly verified wrap edge (decision-16 rule 6).
    *r.pc = r.pc.wrapping_add(u16::from(length));
    // Library choice: every delegated entry uses States::single, so both
    // Path slots have the same model-owned cost (decision-16 rule 5).
    Path::Untaken
}
