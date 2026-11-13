//! Input and Output Group (decision-13 rule 2; decision-15 rules 4/11).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 294–316.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-11, p. 54
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::Decoded;
use super::opcodes::{Encoding, Instruction, States};
use super::Z80;
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

// Repeats use terminating/repeating slots (decision-15 rule 11).

pub(super) const MAIN: &[Encoding] = &[
    // UM IN A,(n), p. 295.
    Encoding::new(b"11011011", 2, States::single(11)),
    // UM OUT (n),A, p. 306.
    Encoding::new(b"11010011", 2, States::single(11)),
];

pub(super) const ED: &[Encoding] = &[
    // A: decision-20 §1; TM Table 7.0-11 p. 54 includes r=110,
    // flag-only input, 2 bytes/12 states; UM pp. 296–297 flag rules.
    Encoding::new(b"01RRR000", 2, States::single(12)),
    // L: decision-20 §1; Young v0.91 §3.4 p. 13 / §8.11 p. 35
    // extends UM pp. 307–308 with r=110: zero output, 2/12.
    Encoding::new(b"01RRR001", 2, States::single(12)),
    // UM INI, p. 298.
    Encoding::new(b"10100010", 2, States::single(16)),
    // UM INIR, p. 300.
    Encoding::new(
        b"10110010",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM IND, p. 302.
    Encoding::new(b"10101010", 2, States::single(16)),
    // UM INDR, p. 304.
    Encoding::new(
        b"10111010",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM OUTI, p. 309.
    Encoding::new(b"10100011", 2, States::single(16)),
    // UM OTIR, pp. 311–312.
    Encoding::new(
        b"10110011",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM OUTD, p. 313.
    Encoding::new(b"10101011", 2, States::single(16)),
    // UM OTDR, pp. 315–316.
    Encoding::new(
        b"10111011",
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
    let opcode = decoded.opcode;
    match opcode {
        // UM IN A,(n) p. 295 / OUT (n),A p. 306; TM Table 7.0-11
        // p. 54: A supplies A8–A15, n A0–A7, flags unchanged. Input
        // samples the port BEFORE replacing A (decision-15 rule 4).
        0xDB | 0xD3 => {
            let n = cpu.fetch_operand(system);
            let registers = cpu.registers_mut();
            let port = u16::from_be_bytes([registers.a, n]);
            if opcode == 0xDB {
                registers.a = system.input16(port);
            } else {
                system.output16(port, registers.a);
            }
            // L: decision-20 §5; MEMPTR (©2006), IN A,(n)/OUT (n),A.
            cpu.latches_mut().wz = if opcode == 0xDB {
                port.wrapping_add(1)
            } else {
                u16::from_be_bytes([(port >> 8) as u8, n.wrapping_add(1)])
            };
        }
        // UM IN r,(C) pp. 296–297; TM p. 54: address from old BC,
        // even for IN B/IN C. S/Z/parity reflect input; H/N reset,
        // C unchanged. H reset is also explicit in 03-0002-01 p. 255
        // (TM marks H affected: PROVENANCE §1, IN H notation).
        // A/H: decision-20 §1; TM p. 54 flag-only r=110; Young
        // v0.91 §3.4 pp. 12–13; Rak v1.2a .in_c lines 1000–1006.
        // H Y/X: decision-20 §4; Young v0.91 §8.11 p. 35;
        // Rak .in_r_c/.in_c at BFh; arbitrary input follows written rule.
        0x40 | 0x48 | 0x50 | 0x58 | 0x60 | 0x68 | 0x70 | 0x78 => {
            // L completion: decision-20 §5; MEMPTR (©2006) names IN A,(C);
            // all codes including ED70 use BC before register replacement.
            cpu.latches_mut().wz = cpu.registers().bc().wrapping_add(1);
            let registers = cpu.registers_mut();
            let value = system.input16(registers.bc());
            match opcode {
                0x40 => registers.b = value,
                0x48 => registers.c = value,
                0x50 => registers.d = value,
                0x58 => registers.e = value,
                0x60 => registers.h = value,
                0x68 => registers.l = value,
                0x70 => {}
                _ => registers.a = value,
            }
            registers.flags.set_yx(value);
            registers.flags.sign = value & 0x80 != 0;
            registers.flags.zero = value == 0;
            registers.flags.half_carry = false;
            registers.flags.parity_overflow = value.count_ones().is_multiple_of(2);
            registers.flags.subtract = false;
        }
        // UM OUT (C),r pp. 307–308; TM p. 54: BC address, selected
        // register data, no flags. Decision-20 §1: H preservation
        // (Rak .out_c_0 lines 1070–1075), L emitted zero/length/cost
        // (Young v0.91 §3.4 p. 13 / §8.11 p. 35), not bus-confirmed.
        0x41 | 0x49 | 0x51 | 0x59 | 0x61 | 0x69 | 0x71 | 0x79 => {
            let registers = cpu.registers();
            let value = match opcode {
                0x41 => registers.b,
                0x49 => registers.c,
                0x51 => registers.d,
                0x59 => registers.e,
                0x61 => registers.h,
                0x69 => registers.l,
                0x71 => 0,
                _ => registers.a,
            };
            system.output16(registers.bc(), value);
            // L completion: decision-20 §5; MEMPTR (©2006), OUT (C),A;
            // extended to every register code and ED71.
            cpu.latches_mut().wz = cpu.registers().bc().wrapping_add(1);
        }
        // UM INI p. 298, INIR pp. 300–301, IND p. 302, INDR
        // pp. 304–305; OUTI p. 309, OTIR pp. 311–312, OUTD p. 313,
        // OTDR pp. 315–316; TM Table 7.0-11 p. 54. Input precedes
        // memory write and B decrement; output follows memory read
        // and B decrement. Thus input uses old B, output new B on
        // A8–A15 (decision-15 rule 4). HL changes after the transfer.
        0xA2 | 0xB2 | 0xAA | 0xBA | 0xA3 | 0xB3 | 0xAB | 0xBB => {
            let registers = cpu.registers_mut();
            let address = registers.hl();
            let old_bc = registers.bc();
            let data = if opcode & 1 == 0 {
                let value = system.input16(registers.bc());
                system.write(address, value, MemoryAccess::Data);
                registers.b = registers.b.wrapping_sub(1);
                value
            } else {
                let value = system.read(address, MemoryAccess::Data);
                registers.b = registers.b.wrapping_sub(1);
                system.output16(registers.bc(), value);
                value
            };
            // Wrapping is derived from 8-bit B / 16-bit HL and PC
            // (UM Registers p. 3) and the cited entry equations.
            registers.set_hl(if opcode & 0x08 == 0 {
                address.wrapping_add(1)
            } else {
                address.wrapping_sub(1)
            });
            // H observable flags, L unexercised extension: decision-20 §4;
            // Young v0.91 §4.3 pp. 16–17 / §8.11 p. 35; Rak .ini/.ind/
            // .outi/.outd/.inir/.indr/.otir/.otdr (input restricted to BFh).
            // Supersedes UM's N=1/C-unaffected and unknown S/H/PV entries.
            let offset = if opcode & 1 == 0 {
                registers
                    .c
                    .wrapping_add(if opcode & 8 == 0 { 1 } else { 0xFF })
            } else {
                registers.l
            };
            let k = u16::from(data) + u16::from(offset);
            registers.flags.set_yx(registers.b);
            registers.flags.sign = registers.b & 0x80 != 0;
            registers.flags.zero = registers.b == 0;
            registers.flags.subtract = data & 0x80 != 0;
            registers.flags.half_carry = k > 0xFF;
            registers.flags.carry = k > 0xFF;
            registers.flags.parity_overflow =
                ((k as u8 & 7) ^ registers.b).count_ones().is_multiple_of(2);
            // Repeating descriptions explicitly rewind PC by two while
            // B != 0. One iteration per step is decision-15 rule 11;
            // Taken selects 21 states, termination selects 16.
            let continuing = opcode & 0x10 != 0 && registers.b != 0;
            let bc = if opcode & 1 == 0 {
                old_bc
            } else {
                registers.bc()
            };
            let delta = if opcode & 8 == 0 { 1 } else { u16::MAX };
            // L: decision-20 §5; MEMPTR (©2006), block I/O headings.
            // Continuing INxR REPLACEMENT uses this iteration's ED+1.
            // M's BC-derived continuing rule is falsified (Rak .inir_nop/
            // .indr_nop); output keeps decremented BC±1 on every iteration.
            let wz = if continuing && opcode & 1 == 0 {
                registers.pc.wrapping_sub(1)
            } else {
                bc.wrapping_add(delta)
            };
            if continuing {
                registers.pc = registers.pc.wrapping_sub(2);
                // H discriminated INxR observations / L OTxR, arbitrary p
                // and full formula domain: decision-20 §6; Banks
                // “Interrupted block instructions” (wiki 49e542d), exact
                // PV/H equations; Rak .inir_nop/.indr_nop. Apply on every
                // extra-five-state path, before Budget/Stop can observe it.
                registers.flags.set_yx((registers.pc >> 8) as u8);
                let b = registers.b;
                let parity_arg = if registers.flags.carry {
                    if data & 0x80 != 0 {
                        registers.flags.half_carry = b & 0x0F == 0;
                        b.wrapping_sub(1)
                    } else {
                        registers.flags.half_carry = b & 0x0F == 0x0F;
                        b.wrapping_add(1)
                    }
                } else {
                    b
                };
                registers.flags.parity_overflow ^= !(parity_arg & 7).count_ones().is_multiple_of(2);
                cpu.latches_mut().wz = wz;
                return Path::Taken;
            }
            cpu.latches_mut().wz = wz;
        }
        // Only the encodings listed above are assigned to this executor.
        _ => {}
    }
    Path::Untaken
}
