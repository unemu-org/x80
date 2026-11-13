//! Shared access helpers (decision-16 rules 2/5/6; plan §2.2/§2.6).
//!
//! Primaries: Intel 98-153B (September 1975), Intel 205775-002 (January
//! 1983). Second witness: Intel 9800301-04 (May 1981), Ch. 3 entries below.
//! Independent execution witnesses remain in each model's data-transfer,
//! arithmetic, logical, branch and interrupt integration tests, unchanged.
//! MemoryAccess kinds are the library distinction of decision-5 item 3.

use super::{FlagRules, View};
use crate::{Bus, MemoryAccess};

// Instruction bytes follow the opcode: 98-153B Ch. 2 pp. 2-16/2-17,
// note 6 p. 2-20; 205775-002 §5.3 p. 5-2, Table 5-1 p. 5-19 and
// §§2.3.1/2.3.2 pp. 2-7 to 2-10. PC+offset wrapping is DERIVED from
// the 16-bit width (98-153B Ch. 2 p. 2-2; 205775-002 §2.2.1 p. 2-2),
// not an explicitly verified wrap edge. Design: decision-5 item 3.
pub(crate) fn operand<S: Bus, F>(r: &View<'_, F>, system: &mut S, offset: u16) -> u8 {
    system.read(r.pc.wrapping_add(offset), MemoryAccess::OperandFetch)
}

// Low byte 2, high byte 3, read in that order before any data access:
// 98-153B Ch. 4 LXI/direct transfers pp. 4-4/4-5, Ch. 2 pp. 2-16/2-17;
// 205775-002 §5.6.1 p. 5-5, Table 5-1 p. 5-19.
// Second witness: 9800301-04 Ch. 3 LXI p. 3-35, LDA pp. 3-32/3-33.
// Design: decision-5 item 3; decision-16 rule 5. No PC advancement here.
pub(crate) fn operand_word<S: Bus, F>(r: &View<'_, F>, system: &mut S) -> u16 {
    read_address(system, *r.pc)
}

// DDD/SSS register codes, M at HL: 98-153B Ch. 4 pp. 4-3/4-4,
// Ch. 2 p. 2-16 and note 6 p. 2-20; 205775-002 §5.2 p. 5-1,
// §5.6.1 p. 5-4, §5.6.2 pp. 5-6 to 5-8, §5.6.3 pp. 5-10/5-11,
// Table 5-1 p. 5-19. Second witness: 9800301-04 Ch. 3 pp. 3-3/3-4,
// 3-36/3-37. Callers supply a masked three-bit field (111 is A).
// Design: decision-16 rule 5; decision-5 item 3.
pub(crate) fn read_location<S: Bus, F>(r: &View<'_, F>, system: &mut S, code: u8) -> u8 {
    match code {
        0 => *r.b,
        1 => *r.c,
        2 => *r.d,
        3 => *r.e,
        4 => *r.h,
        5 => *r.l,
        6 => system.read(u16::from_be_bytes([*r.h, *r.l]), MemoryAccess::Data),
        _ => *r.a,
    }
}

// Same destination codes and HL address: 98-153B Ch. 4 pp. 4-3/4-4,
// INR/DCR pp. 4-7/4-8; Ch. 2 pp. 2-16/2-17, note 7 p. 2-20;
// 205775-002 §5.2 p. 5-1, §5.6.1 p. 5-4, §5.6.2 p. 5-8,
// Table 5-1 p. 5-19, §§2.3.2/2.3.3 pp. 2-10 to 2-12.
// Second witness: 9800301-04 Ch. 3 MOV/MVI pp. 3-36 to 3-38,
// INR p. 3-25, DCR pp. 3-20/3-21. Design: decision-5 item 3.
pub(crate) fn write_location<S: Bus, F>(r: &mut View<'_, F>, system: &mut S, code: u8, value: u8) {
    match code {
        0 => *r.b = value,
        1 => *r.c = value,
        2 => *r.d = value,
        3 => *r.e = value,
        4 => *r.h = value,
        5 => *r.l = value,
        6 => system.write(u16::from_be_bytes([*r.h, *r.l]), value, MemoryAccess::Data),
        _ => *r.a = value,
    }
}

// Both executor vocabularies name the same operation (decision-16 rule 5).
pub(crate) use read_location as read_operand;
pub(crate) use write_location as write_operand;

// RP codes and high/low order, including SP: 98-153B Ch. 4 RP table
// p. 4-3, INX/DCX/DAD p. 4-8; 205775-002 §5.2 p. 5-1,
// §5.6.2 p. 5-9, Table 5-1 p. 5-19. Second witness: 9800301-04
// Ch. 3 DAD/DCX/INX pp. 3-20/3-22/3-26. Design: decision-16 rule 5.
pub(crate) fn read_pair<F>(r: &View<'_, F>, pair: u8) -> u16 {
    match pair {
        0 => u16::from_be_bytes([*r.b, *r.c]),
        1 => u16::from_be_bytes([*r.d, *r.e]),
        2 => u16::from_be_bytes([*r.h, *r.l]),
        _ => *r.sp,
    }
}

// RP destinations/order: 98-153B Ch. 4 p. 4-3, INX/DCX p. 4-8;
// 205775-002 §5.2 p. 5-1, §5.6.2 p. 5-9. Second witness:
// 9800301-04 Ch. 3 DCX/INX pp. 3-22/3-26. Design: decision-16 rule 5.
pub(crate) fn write_pair<F>(r: &mut View<'_, F>, pair: u8, value: u16) {
    let [high, low] = value.to_be_bytes();
    match pair {
        0 => [*r.b, *r.c] = [high, low],
        1 => [*r.d, *r.e] = [high, low],
        2 => [*r.h, *r.l] = [high, low],
        _ => *r.sp = value,
    }
}

// CCC (caller extracts bits 5..3): 98-153B Ch. 4 condition table p. 4-11,
// Ch. 2 note 17 p. 2-20; 205775-002 §5.6.4 condition table p. 5-13.
// Second witness: 9800301-04 Ch. 3 Jcond pp. 3-26 to 3-32.
// AC is neither tested nor changed. Design: decision-16 rules 3/5.
pub(crate) fn condition<F: FlagRules>(r: &View<'_, F>, ccc: u8) -> bool {
    let flags = *r.flags;
    match ccc {
        0 => !flags.zero(),
        1 => flags.zero(),
        2 => !flags.carry(),
        3 => flags.carry(),
        4 => !flags.parity(),
        5 => flags.parity(),
        6 => !flags.sign(),
        _ => flags.sign(),
    }
}

// Read byte 2 low, then byte 3 high: 98-153B Ch. 2 JMP/Jcc/CALL/Ccc
// pp. 2-18/2-19, note 6 p. 2-20, Ch. 4 pp. 4-11/4-12;
// 205775-002 §2.3.1 p. 2-7, §5.6.4 pp. 5-13/5-14, Table 5-1 p. 5-19
// (taken paths). Second witness: 9800301-04 Ch. 3 CALL pp. 3-8/3-9,
// JMP p. 3-28. Untaken-path reads and PC advancement remain model-owned.
// Wrap is DERIVED from 16-bit PC (98-153B Ch. 2 p. 2-2;
// 205775-002 §2.2.1 p. 2-2). Design: decision-16 rules 4/5.
pub(crate) fn read_address<S: Bus>(system: &mut S, pc: u16) -> u16 {
    let low = system.read(pc.wrapping_add(1), MemoryAccess::OperandFetch);
    let high = system.read(pc.wrapping_add(2), MemoryAccess::OperandFetch);
    u16::from_le_bytes([low, high])
}

// High at SP-1, then low at SP-2, then transfer control:
// 98-153B Ch. 4 CALL/RST pp. 4-11/4-12, Ch. 2 pp. 2-18/2-19
// (CALL M4/M5, RST M2/M3), note 16 p. 2-20 identifies Stack;
// 205775-002 §5.6.4 pp. 5-13/5-14, §2.2.3 p. 2-2, Fig. 2-18 p. 2-14,
// Table 5-1 p. 5-19. Second witness: 9800301-04 Ch. 3 pp. 3-8/3-9,
// 3-54/3-55. SP wrapping is DERIVED from its 16-bit width (98-153B Ch. 2
// p. 2-2; 205775-002 §2.2.3 p. 2-2), not an explicit zero-crossing account.
// Callers supply return PC and target, including interrupt paths; no fetch
// advancement or vector selection here (decision-16 rule 5; decision-12
// rules 4/5; decision-5 item 4). Flags remain untouched (98-153B Ch. 4
// Branch Group p. 4-11; 205775-002 §5.6.4 p. 5-13).
pub(crate) fn push_and_jump<S: Bus, F>(
    r: &mut View<'_, F>,
    system: &mut S,
    return_pc: u16,
    target: u16,
) {
    let [high, low] = return_pc.to_be_bytes();
    *r.sp = r.sp.wrapping_sub(1);
    system.write(*r.sp, high, MemoryAccess::Stack);
    *r.sp = r.sp.wrapping_sub(1);
    system.write(*r.sp, low, MemoryAccess::Stack);
    *r.pc = target;
}
