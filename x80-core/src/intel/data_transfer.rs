//! Shared Data Transfer executor (decision-16 rules 1/2/5/6; plan §2.2/§2.6).
//!
//! Primaries: Intel 98-153B (September 1975), Ch. 4 pp. 4-3 to 4-5;
//! Intel 205775-002 (January 1983), §§5.2/5.6.1 pp. 5-1, 5-4 to 5-6.
//! Second witness: Intel 9800301-04 (May 1981), Ch. 3 entries cited below.
//! Independent witnesses remain in tests/i8080_data_transfer.rs and
//! tests/i8085_data_transfer.rs. Encodings and state totals stay per model.
//! Access kinds are the library distinction of decision-5 item 3.

use super::access::{operand, operand_word, read_location, write_location, write_pair};
use super::View;
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

/// Executes an already fetched opcode dispatched to this group by a model.
pub(crate) fn execute<S: Bus, F>(
    r: &mut View<'_, F>,
    system: &mut S,
    opcode: u8,
    length: u8,
) -> Path {
    match opcode {
        // MOV and DDD/SSS fields: 98-153B Ch. 4 pp. 4-3/4-4,
        // Ch. 2 p. 2-16; 205775-002 §§5.2/5.6.1 pp. 5-1/5-4,
        // Table 5-1 p. 5-19. Read the source at original HL before writing
        // the destination. Second witness: 9800301-04 Ch. 3 pp. 3-36/3-37.
        // HLT exclusion: 98-153B Ch. 4 p. 4-14; 205775-002 Table 5-1 p. 5-19.
        0x40..=0x7F if opcode != 0x76 => {
            let value = read_location(r, system, opcode & 7);
            write_location(r, system, (opcode >> 3) & 7, value);
        }
        // MVI: byte 2 before an optional HL write; 98-153B Ch. 4 p. 4-4,
        // Ch. 2 pp. 2-16/2-17; 205775-002 §5.6.1 p. 5-4,
        // Table 5-1 p. 5-19 (FR/FRW). DDD: 98-153B Ch. 4 p. 4-3;
        // 205775-002 §5.2 p. 5-1. Second: 9800301-04 Ch. 3 pp. 3-37/3-38.
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
            let value = operand(r, system, 1);
            write_location(r, system, (opcode >> 3) & 7, value);
        }
        // LXI: RP selects BC/DE/HL/SP, byte 2 low then byte 3 high;
        // 98-153B Ch. 4 pp. 4-3/4-4, Ch. 2 pp. 2-16/2-17;
        // 205775-002 §§5.2/5.6.1 pp. 5-1/5-5, Table 5-1 p. 5-19 (FRR).
        // Second witness: 9800301-04 Ch. 3 p. 3-35.
        0x01 | 0x11 | 0x21 | 0x31 => {
            let value = operand_word(r, system);
            write_pair(r, (opcode >> 4) & 3, value);
        }
        // LDA/STA/LHLD/SHLD: fetch low/high address before data, L before H;
        // 98-153B Ch. 4 p. 4-5, Ch. 2 pp. 2-16/2-17 (M2..M5);
        // 205775-002 §5.6.1 p. 5-5, Table 5-1 p. 5-19
        // (FRRR/FRRW/FRRRR/FRRWW). Second: 9800301-04 Ch. 3
        // pp. 3-32 to 3-34, 3-58/3-59, 3-61/3-62.
        0x3A | 0x32 | 0x2A | 0x22 => {
            let address = operand_word(r, system);
            match opcode {
                0x3A => *r.a = system.read(address, MemoryAccess::Data),
                0x32 => system.write(address, *r.a, MemoryAccess::Data),
                0x2A => {
                    *r.l = system.read(address, MemoryAccess::Data);
                    *r.h = system.read(address.wrapping_add(1), MemoryAccess::Data);
                }
                _ => {
                    system.write(address, *r.l, MemoryAccess::Data);
                    system.write(address.wrapping_add(1), *r.h, MemoryAccess::Data);
                }
            }
            // Address+1: 98-153B Ch. 4 p. 4-5, Ch. 2 p. 2-17;
            // 205775-002 §5.6.1 p. 5-5. FFFFh -> 0000h is DERIVED from
            // 16-bit addresses (98-153B Ch. 4 p. 4-3; 205775-002 §5.2
            // p. 5-1), not an explicitly verified wrap edge.
        }
        // LDAX/STAX: BC or DE only, one data read/write; 98-153B Ch. 4
        // p. 4-5, Ch. 2 p. 2-16; 205775-002 §5.6.1 pp. 5-5/5-6,
        // Table 5-1 p. 5-19 (FR/FW). Pair order: 98-153B Ch. 4 p. 4-3;
        // 205775-002 §5.2 p. 5-1. Second: 9800301-04 Ch. 3 pp. 3-33/3-62.
        0x0A | 0x1A | 0x02 | 0x12 => {
            let address = if opcode & 0x10 == 0 {
                u16::from_be_bytes([*r.b, *r.c])
            } else {
                u16::from_be_bytes([*r.d, *r.e])
            };
            if opcode & 0x08 != 0 {
                *r.a = system.read(address, MemoryAccess::Data);
            } else {
                system.write(address, *r.a, MemoryAccess::Data);
            }
        }
        // XCHG: H <-> D, L <-> E, no memory access; 98-153B Ch. 4
        // p. 4-5, Ch. 2 p. 2-16 (M1 only); 205775-002 §5.6.1 p. 5-6,
        // Table 5-1 p. 5-19 (F). Second: 9800301-04 Ch. 3 p. 3-65.
        0xEB => {
            core::mem::swap(r.h, r.d);
            core::mem::swap(r.l, r.e);
        }
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
    // Sequential instruction bytes: 98-153B Ch. 4 p. 4-2, Ch. 2
    // pp. 2-16/2-17; 205775-002 §2.2.1 p. 2-2, §5.3 p. 5-2.
    // PC wrap is DERIVED from its 16-bit width (98-153B Ch. 4 p. 4-3;
    // 205775-002 §2.2.1 p. 2-2), not an explicitly verified wrap edge.
    // Flags untouched: 98-153B Ch. 4 p. 4-4; 205775-002 §5.6.1 p. 5-4.
    *r.pc = r.pc.wrapping_add(u16::from(length));
    // Both cost slots are model-owned single totals (decision-5 item 7).
    Path::Taken
}
