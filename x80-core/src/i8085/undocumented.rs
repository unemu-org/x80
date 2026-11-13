//! The ten admitted 8085 extensions (decision-18 §§2–4; decisions 16/19).
//!
//! S = documented by second source (CA80C85B), unverified for Intel 8085:
//! Tundra 9000000.MD302.01 (©1996, layout 13 June 1997), Table 2-10
//! p. 2-16/PDF 40, Fig. 2-9 p. 2-15/PDF 39; completed/reconciled by
//! Dehnhardt/Sorensen, Electronics, 18 January 1979, p. 145/PDF 2 (W).
//! Independent encoding key D = die analysis (8085; exact part unverified):
//! Zima decode ROM rows 22/25/23/24/8/11/32/12, respectively (DI/EI = 1).
//! Source revisions, dated captures and hashes: PROVENANCE §1, decision-18 §1.
//! D does not independently establish every operation, timing or flag effect.
//! L = adopted library rule, exact hardware effect unverified. All bus
//! sequences/kinds below are L mappings of decision-18 §3, not pin traces.
//! Width wrapping, unsigned immediates and register preservation are L (§2).

use super::opcodes::{Encoding, Instruction, Path, States};
use super::{Flags, I8085};
use crate::intel::access::{operand, push_and_jump, read_address};
use crate::{Bus, MemoryAccess};

// S lengths/states, including second-source conditional path interpretation:
// decision-18 §2, T Table 2-10 p. 2-16/W p. 145. Single totals fill both slots.
pub(super) const ENCODINGS: &[Encoding] = &[
    Encoding::new(b"00001000", 1, States::single(10)), // DSUB; D ROM row 22.
    Encoding::new(b"00010000", 1, States::single(7)),  // ARHL; D row 25.
    Encoding::new(b"00011000", 1, States::single(10)), // RDEL; D row 23.
    Encoding::new(b"00101000", 2, States::single(10)), // LDHI; D row 24.
    Encoding::new(b"00111000", 2, States::single(10)), // LDSI; D row 24.
    Encoding::new(
        b"11001011",
        1,
        States {
            untaken: 6,
            taken: 12,
        },
    ), // RSTV; D row 8.
    Encoding::new(b"11011001", 1, States::single(10)), // SHLX; D row 11.
    Encoding::new(
        b"11011101",
        3,
        States {
            untaken: 7,
            taken: 10,
        },
    ), // JNK; D row 32.
    Encoding::new(b"11101101", 1, States::single(10)), // LHLX; D row 12.
    Encoding::new(
        b"11111101",
        3,
        States {
            untaken: 7,
            taken: 10,
        },
    ), // JK; D row 32.
];

pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Path {
    let mut r = super::view(cpu.registers_mut());
    let next_pc = r.pc.wrapping_add(u16::from(instruction.length));
    match opcode {
        0x08 => {
            // S operation/all-seven mask/V, silicon-corrected K: decision-18
            // §§2/4, T Table 2-10/Fig. 2-9, W p. 145. D/S combined Z:
            // Shirriff FC storage discussion (July 2013). S/P/AC/CY follow
            // the explicit L low-then-high byte-step rule, not a die trace.
            let left = u16::from_be_bytes([*r.h, *r.l]);
            let right = u16::from_be_bytes([*r.b, *r.c]);
            let borrow = u16::from(*r.l < *r.c);
            let low = r.l.wrapping_sub(*r.c);
            let high = r.h.wrapping_sub(*r.b).wrapping_sub(borrow as u8);
            let result = u16::from_be_bytes([high, low]);
            let sign = high & 0x80 != 0;
            let overflow = (left ^ right) & (left ^ result) & 0x8000 != 0;
            *r.flags = Flags {
                sign,
                zero: result == 0,
                auxiliary_carry: u16::from(*r.h & 15) >= u16::from(*r.b & 15) + borrow,
                parity: high.count_ones() & 1 == 0,
                carry: u16::from(*r.h) < u16::from(*r.b) + borrow,
                overflow,
                k: overflow ^ sign,
            };
            [*r.h, *r.l] = [high, low];
        }
        0x10 => {
            // S arithmetic shift/sign fill and CY, K/S/Z/AC/P preservation:
            // decision-18 §§2/4, T Table 2-10/W p. 145. V=0 is L, following
            // Shirriff ALU store_v_flag/right-shift carries (July 2013).
            let old = u16::from_be_bytes([*r.h, *r.l]);
            [*r.h, *r.l] = ((old >> 1) | (old & 0x8000)).to_be_bytes();
            r.flags.carry = old & 1 != 0;
            r.flags.overflow = false;
        }
        0x18 => {
            // S DE+DE+old CY, old D7 -> CY, signed word V; preserve K and
            // S/Z/AC/P: decision-18 §§2/4, T Table 2-10/Fig. 2-9, W p. 145.
            let old = u16::from_be_bytes([*r.d, *r.e]);
            let result = (old << 1) | u16::from(r.flags.carry);
            [*r.d, *r.e] = result.to_be_bytes();
            r.flags.carry = old & 0x8000 != 0;
            r.flags.overflow = (old ^ result) & 0x8000 != 0;
        }
        0x28 | 0x38 => {
            // S base+byte -> DE, all flags preserved: decision-18 §§2/4,
            // T Table 2-10/W p. 145; CY also D, FC temporary carry latch.
            // L zero extension/wrap and OperandFetch mapping (§§2/3).
            let base = if opcode == 0x28 {
                u16::from_be_bytes([*r.h, *r.l])
            } else {
                *r.sp
            };
            let offset = operand(&r, system, 1);
            [*r.d, *r.e] = base.wrapping_add(u16::from(offset)).to_be_bytes();
        }
        0xCB if r.flags.overflow => {
            // S V condition/vector 0040h: decision-18 §2, T/W, D ROM row 8.
            // L next-PC, Stack high-before-low mapping (§3), interrupt-latch
            // preservation/ordinary retirement (§5). No acknowledge or INTE
            // change; the boundary loop alone retires the EI delay.
            push_and_jump(&mut r, system, next_pc, 0x0040);
            return Path::Taken;
        }
        0xCB => {
            *r.pc = next_pc; // S sequential path; L no extra accesses (§3).
            return Path::Untaken;
        }
        0xD9 | 0xED => {
            // S byte placements and flag preservation: decision-18 §§2/4,
            // T Table 2-10/W p. 145 (SHLX completion/LHLX reconciliation).
            // L low-before-high Data accesses, even when DE=SP (§3).
            let address = u16::from_be_bytes([*r.d, *r.e]);
            let high_address = address.wrapping_add(1);
            if opcode == 0xD9 {
                system.write(address, *r.l, MemoryAccess::Data);
                system.write(high_address, *r.h, MemoryAccess::Data);
            } else {
                *r.l = system.read(address, MemoryAccess::Data);
                *r.h = system.read(high_address, MemoryAccess::Data);
            }
        }
        0xDD | 0xFD => {
            // S polarity after text reconciliation: decision-18 §2,
            // T JNUI/JUI/W p. 145. L operand reads (§3): both bytes when
            // taken, only byte 2 when untaken, then skip the high byte.
            if r.flags.k == (opcode == 0xFD) {
                *r.pc = read_address(system, *r.pc);
                return Path::Taken;
            }
            operand(&r, system, 1);
            *r.pc = next_pc;
            return Path::Untaken;
        }
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
    *r.pc = next_pc;
    Path::Taken
}
