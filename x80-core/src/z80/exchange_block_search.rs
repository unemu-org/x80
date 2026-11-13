//! Exchange, Block Transfer, and Search Group (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 123–143.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-3, p. 46
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).
//! All encoded facts were read on rendered page images; source hashes match
//! PROVENANCE §1. Independent witnesses: tests/z80_exchange_block_search.rs.
//! Transfer P/V follows the decremented count: stated for LDIR (UM p. 133),
//! derived for intermediate LDDR iterations (PROVENANCE §1). UM LDDR's
//! flag list omits C; TM p. 46 explicitly preserves it. No flags are printed
//! unknown in this group. One iteration retires per step, with
//! terminating/repeating costs selected below (decision-15 rule 11).

use super::cpu::{Decoded, IndexRegister, Table};
use super::opcodes::{Encoding, Instruction, States};
use super::Z80;
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

// Repeats use terminating/repeating slots (decision-15 rule 11).

pub(super) const MAIN: &[Encoding] = &[
    // UM EX DE,HL, p. 124.
    Encoding::new(b"11101011", 1, States::single(4)),
    // UM EX AF,AF', p. 125.
    Encoding::new(b"00001000", 1, States::single(4)),
    // UM EXX, p. 126.
    Encoding::new(b"11011001", 1, States::single(4)),
    // UM EX (SP),HL, p. 127.
    Encoding::new(b"11100011", 1, States::single(19)),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM EX (SP),IX/IY, pp. 128–129.
    Encoding::new(b"11100011", 2, States::single(23)),
];

pub(super) const ED: &[Encoding] = &[
    // UM LDI, p. 130.
    Encoding::new(b"10100000", 2, States::single(16)),
    // UM LDIR, p. 132.
    Encoding::new(
        b"10110000",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM LDD, p. 134.
    Encoding::new(b"10101000", 2, States::single(16)),
    // UM LDDR, p. 136.
    Encoding::new(
        b"10111000",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM CPI, p. 138.
    Encoding::new(b"10100001", 2, States::single(16)),
    // UM CPIR, p. 139; termination wording: PROVENANCE §1.
    Encoding::new(
        b"10110001",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM CPD, p. 141.
    Encoding::new(b"10101001", 2, States::single(16)),
    // UM CPDR, p. 142; termination wording: PROVENANCE §1.
    Encoding::new(
        b"10111001",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
];

/// Final executor seam (decision-13 rule 4). PC is after prefix/key bytes;
/// read remaining operands through Z80::fetch_operand. INDEX d is executor-
/// owned; INDEX_CB d is in Decoded. Return the complete instruction's path.
/// Repeating blocks rewind PC by two and return Path::Taken (rule 11).
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    system: &mut S,
    _instruction: &Instruction,
    decoded: &Decoded,
) -> Path {
    // The dispatcher admits only this group's listed table/key combinations.
    if decoded.table == Table::Ed {
        return block(cpu, system, decoded.opcode);
    }
    if decoded.opcode == 0xE3 {
        exchange_stack(cpu, system, decoded.index);
        return Path::Untaken;
    }
    let r = cpu.registers_mut();
    match decoded.opcode {
        // UM EX DE,HL p. 124; TM Table 7.0-3 p. 46. Flags untouched.
        0xEB => {
            core::mem::swap(&mut r.d, &mut r.h);
            core::mem::swap(&mut r.e, &mut r.l);
        }
        // UM EX AF,AF' p. 125; TM p. 46. Exchange F as stored, without
        // interpreting it as an arithmetic result (decision-14 rule 5).
        0x08 => {
            core::mem::swap(&mut r.a, &mut r.a_alternate);
            core::mem::swap(&mut r.flags, &mut r.flags_alternate);
        }
        // UM EXX p. 126; TM p. 46. AF/AF' and flags untouched.
        0xD9 => {
            core::mem::swap(&mut r.b, &mut r.b_alternate);
            core::mem::swap(&mut r.c, &mut r.c_alternate);
            core::mem::swap(&mut r.d, &mut r.d_alternate);
            core::mem::swap(&mut r.e, &mut r.e_alternate);
            core::mem::swap(&mut r.h, &mut r.h_alternate);
            core::mem::swap(&mut r.l, &mut r.l_alternate);
        }
        _ => {} // No other main/index keys are dispatched here.
    }
    Path::Untaken
}

// UM EX (SP),HL/IX/IY pp. 127–129; TM Table 7.0-3 p. 46: low byte
// exchanges with (SP), high with (SP+1); SP and all flags are unaffected.
// UNVERIFIED library order: read SP, read SP+1, write SP+1, write SP.
// The cited entries establish byte mapping, not temporal bus order. Capture
// both old register bytes and both reads before writes, including aliases.
// Stack is the decision-15 rule 5 API mapping for these SP-based accesses.
// SP+1 wrapping is DERIVED from the 16-bit address/register width.
fn exchange_stack<S: Bus>(cpu: &mut Z80, system: &mut S, index: Option<IndexRegister>) {
    let r = cpu.registers_mut();
    let old = match index {
        Some(IndexRegister::Ix) => r.ix,
        Some(IndexRegister::Iy) => r.iy,
        None => r.hl(),
    };
    let high_address = r.sp.wrapping_add(1);
    let low = system.read(r.sp, MemoryAccess::Stack);
    let high = system.read(high_address, MemoryAccess::Stack);
    let [old_low, old_high] = old.to_le_bytes();
    system.write(high_address, old_high, MemoryAccess::Stack);
    system.write(r.sp, old_low, MemoryAccess::Stack);
    let value = u16::from_le_bytes([low, high]);
    match index {
        Some(IndexRegister::Ix) => r.ix = value,
        Some(IndexRegister::Iy) => r.iy = value,
        None => r.set_hl(value),
    }
    // L: decision-20 §5; MEMPTR (©2006), EX (SP),rp.
    cpu.latches_mut().wz = value;
}

// UM LDI pp. 130–131, LDIR pp. 132–133, LDD pp. 134–135, LDDR
// pp. 136–137, CPI p. 138, CPIR pp. 139–140, CPD p. 141, CPDR
// pp. 142–143; second witness TM Table 7.0-3 p. 46. Encoding bits select
// compare/transfer, decrement/increment and repeat; no other ED keys enter.
fn block<S: Bus>(cpu: &mut Z80, system: &mut S, opcode: u8) -> Path {
    let compare = opcode & 1 != 0;
    let repeating = opcode & 0x10 != 0;
    // DERIVED modulo-16-bit +/-1 for pointers and decrement for BC.
    let delta = if opcode & 0x08 != 0 { u16::MAX } else { 1 };
    let old_wz = cpu.latches().wz;
    let r = cpu.registers_mut();
    let value = system.read(r.hl(), MemoryAccess::Data);
    if !compare {
        // The entries order the transfer before updating either pointer.
        system.write(r.de(), value, MemoryAccess::Data);
        r.set_de(r.de().wrapping_add(delta));
    }
    r.set_hl(r.hl().wrapping_add(delta));
    let count = r.bc().wrapping_sub(1);
    r.set_bc(count);
    if compare {
        // UM pp. 138/140/141/142–143: S/Z/H from A-(HL), P/V from
        // BC-1, N set, C unaffected. Bit-4 borrow is DERIVED as a low-
        // nibble comparison; A and the memory byte are not changed.
        let result = r.a.wrapping_sub(value);
        r.flags.sign = result & 0x80 != 0;
        r.flags.zero = result == 0;
        r.flags.half_carry = (r.a & 0x0F) < (value & 0x0F);
        r.flags.parity_overflow = count != 0;
        r.flags.subtract = true;
    } else {
        // UM pp. 130/133/134: H/N reset, P/V from BC-1, S/Z/C unaffected.
        // P/V's counter rule is STATED for LDIR (p. 133), DERIVED for
        // intermediate LDDR iterations from LDIR/LDD. UM LDDR p. 137's
        // reset and TM p. 46's zero entries describe completion, when
        // BC=0 (maintainer-accepted ruling, 2026-11-13).
        // LDDR's omitted C is explicitly unaffected in TM p. 46.
        // See PROVENANCE §1 for the source discrepancy and ruling.
        r.flags.half_carry = false;
        r.flags.parity_overflow = count != 0;
        r.flags.subtract = false;
    }
    // H ordinary Y/X: decision-20 §4; Young v0.91 §4.2 p. 16;
    // Rak .ldi… .lddr 912–938 / .cpi… .cpdr 954–980.
    // Compute ordinary flags before §6's continuing-step overrides.
    let t = if compare {
        r.a.wrapping_sub(value)
            .wrapping_sub(u8::from(r.flags.half_carry))
    } else {
        r.a.wrapping_add(value)
    };
    r.flags.y = t & 0x02 != 0;
    r.flags.x = t & 0x08 != 0;
    // Decision-15 rule 11: CPIR/CPDR stop on count zero OR match. UM
    // pp. 139/142 termination sentences and TM p. 46 agree. PROVENANCE §1
    // records the UM timing-heading AND and CPDR repeat-sentence equality
    // misprints; the 21-state headings correctly require no match. PC-2,
    // Taken=21; termination means Untaken=16. PC wrap is DERIVED.
    if repeating && count != 0 && (!compare || !r.flags.zero) {
        r.pc = r.pc.wrapping_sub(2);
        // H LDxR observed projections / L CPxR and arbitrary p routing:
        // decision-20 §6; Banks “Interrupted block instructions” (wiki
        // 49e542d); Rak .ldir_nop/.lddr_nop. p is this iteration's ED.
        r.flags.set_yx((r.pc >> 8) as u8);
        // L: decision-20 §5; MEMPTR (©2006), continuing LDxR/CPxR.
        // PC now names this iteration's ED, even after ignored prefixes.
        cpu.latches_mut().wz = cpu.registers().pc.wrapping_add(1);
        Path::Taken
    } else {
        if compare {
            // L: decision-20 §5; MEMPTR (©2006), CPI/CPD/CPIR/CPDR.
            cpu.latches_mut().wz = old_wz.wrapping_add(delta);
        }
        Path::Untaken
    }
}
