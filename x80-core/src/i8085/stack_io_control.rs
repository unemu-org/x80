//! The 8085 Stack, I/O and Machine Control group (excluding RIM/SIM).
//!
//! Intel 205775-002 (January 1983), §5.6.5, pp. 5-15 to 5-17, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Design: decision-10, decision-11; decision-12 rules 6, 7 and 9;
//! decision-5 item 3 supplies the memory access kinds and eight-bit ports.
//! Bus cycle sequences: Table 5-1 p. 5-19 (Table 5-2 p. 5-20 pins bytes),
//! §2.3.1-§2.3.3 pp. 2-7 to 2-13. Source hashes: PROVENANCE.md §1.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::{Bus, MemoryAccess};

/// Complete group list, in per-instruction page order.
/// PUSH/POP allow B, D and H pairs, with PSW as a separate entry
/// (205775-002 pp. 5-15/5-16). RIM/SIM live in their own module
/// under decision-10.
pub(super) const ENCODINGS: &[Encoding] = &[
    // PUSH B: 205775-002 p. 5-15.
    Encoding::new(b"11000101", 1, States::single(12)),
    // PUSH D: 205775-002 p. 5-15.
    Encoding::new(b"11010101", 1, States::single(12)),
    // PUSH H: 205775-002 p. 5-15.
    Encoding::new(b"11100101", 1, States::single(12)),
    // PUSH PSW: 205775-002 p. 5-15.
    Encoding::new(b"11110101", 1, States::single(12)),
    // POP B: 205775-002 p. 5-15.
    Encoding::new(b"11000001", 1, States::single(10)),
    // POP D: 205775-002 p. 5-15.
    Encoding::new(b"11010001", 1, States::single(10)),
    // POP H: 205775-002 p. 5-15.
    Encoding::new(b"11100001", 1, States::single(10)),
    // POP PSW: 205775-002 p. 5-16.
    Encoding::new(b"11110001", 1, States::single(10)),
    // XTHL: 205775-002 p. 5-16.
    Encoding::new(b"11100011", 1, States::single(16)),
    // SPHL: 205775-002 p. 5-16.
    Encoding::new(b"11111001", 1, States::single(6)),
    // IN port: 205775-002 p. 5-16.
    Encoding::new(b"11011011", 2, States::single(10)),
    // OUT port: 205775-002 p. 5-16.
    Encoding::new(b"11010011", 2, States::single(10)),
    // EI: 205775-002 p. 5-17.
    Encoding::new(b"11111011", 1, States::single(4)),
    // DI: 205775-002 p. 5-17.
    Encoding::new(b"11110011", 1, States::single(4)),
    // HLT: 205775-002 p. 5-17.
    Encoding::new(b"01110110", 1, States::single(5)),
    // NOP: 205775-002 p. 5-17.
    Encoding::new(b"00000000", 1, States::single(4)),
];

/// Executes an encoding that the opcode map assigns to this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Path {
    let registers = cpu.registers_mut();
    match opcode {
        // 205775-002 §5.6.5 p. 5-15: high at SP-1, low at SP-2,
        // SP -= 2, flags unchanged; Table 5-1 p. 5-19: SWW, 12 states.
        // 9800301-04 pp. 3-44/3-45 gives the same byte order. Timing
        // disagreement: p. 3-44 says "11 (13 on 8085)" for PUSH rp;
        // the primary's entry/table and the second witness's App. A p. A-2
        // say 12. Follow the primary per the supervisor ruling.
        // PUSH PSW stores A then five flags; undefined bits follow the
        // decision-11 library image, not an 8085 hardware claim.
        0xC5 | 0xD5 | 0xE5 | 0xF5 => {
            let word = match opcode {
                0xC5 => registers.bc(),
                0xD5 => registers.de(),
                0xE5 => registers.hl(),
                _ => registers.psw(),
            };
            let [high, low] = word.to_be_bytes();
            system.write(registers.sp.wrapping_sub(1), high, MemoryAccess::Stack);
            system.write(registers.sp.wrapping_sub(2), low, MemoryAccess::Stack);
            registers.sp = registers.sp.wrapping_sub(2);
        }
        // 205775-002 pp. 5-15/5-16; 9800301-04 p. 3-43: low from
        // SP, high from SP+1, SP += 2; only PSW restores the five flags,
        // discarding bits 5/3/1. Table 5-1: FRR, 10 states.
        0xC1 | 0xD1 | 0xE1 | 0xF1 => {
            let low = system.read(registers.sp, MemoryAccess::Stack);
            let high = system.read(registers.sp.wrapping_add(1), MemoryAccess::Stack);
            let word = u16::from_be_bytes([high, low]);
            match opcode {
                0xC1 => registers.set_bc(word),
                0xD1 => registers.set_de(word),
                0xE1 => registers.set_hl(word),
                _ => registers.set_psw(word),
            }
            registers.sp = registers.sp.wrapping_add(2);
        }
        // 205775-002 p. 5-16; 9800301-04 p. 3-69: exchange L/[SP],
        // H/[SP+1], leave SP/flags unchanged; Table 5-1: FRRWW, 16 states.
        // Unverified bus detail: these pages give RRWW but not the address
        // order within each pair. Model reads SP then SP+1, writes SP+1
        // then SP; witnesses pin this choice, not a stated hardware fact.
        0xE3 => {
            let low = system.read(registers.sp, MemoryAccess::Stack);
            let high = system.read(registers.sp.wrapping_add(1), MemoryAccess::Stack);
            system.write(
                registers.sp.wrapping_add(1),
                registers.h,
                MemoryAccess::Stack,
            );
            system.write(registers.sp, registers.l, MemoryAccess::Stack);
            registers.h = high;
            registers.l = low;
        }
        // 205775-002 p. 5-16; 9800301-04 p. 3-61: SP <- HL, no flags;
        // Table 5-1: S (six-state fetch), no data accesses.
        0xF9 => registers.sp = registers.hl(),
        // 205775-002 p. 5-16; 9800301-04 pp. 3-24/3-25, 3-41:
        // IN loads A, OUT sends A, byte 2 is the port, no flags changed.
        // Table 5-1: FRI/FRO; §2.3.2 p. 2-12 and §2.3.3 p. 2-13:
        // the I/O cycle is third, after the second instruction byte.
        0xDB | 0xD3 => {
            let port = system.read(registers.pc.wrapping_add(1), MemoryAccess::OperandFetch);
            if opcode == 0xDB {
                registers.a = system.input(port);
            } else {
                system.output(port, registers.a);
            }
        }
        // 205775-002 p. 5-17; 9800301-04 pp. 3-22/3-23: EI enables
        // following the next instruction; DI disables immediately. Both
        // are F, four states (Table 5-1), and preserve registers/flags.
        // Decision-12 rule 6's unverified reading suppresses all acceptance,
        // including TRAP, at the next boundary after either instruction.
        // Each re-arms the delay, including unverified EI;DI/EI;EI/DI;EI.
        0xFB | 0xF3 => {
            let latches = cpu.latches_mut();
            latches.interrupt_enable = opcode == 0xFB;
            latches.ei_delay = true;
        }
        // 205775-002 p. 5-17: HLT, five states; Table 5-1: FB (bus idle
        // has no bus call). §2.3.5 p. 2-16 and 9800301-04 p. 3-24:
        // stop until valid interrupt/reset; PC points to the next byte.
        // The boundary loop supplies Halt with no fetch (rule 7).
        0x76 => cpu.latches_mut().halted = true,
        // 205775-002 p. 5-17; 9800301-04 App. A p. A-4: NOP, F,
        // four states, no operation or flag changes.
        0x00 => {}
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
    // Sequential PC: 205775-002 §5.4 p. 5-3. PC/SP wrap is derived
    // from their 16-bit widths (§2.2.2 p. 2-2), not explicitly stated.
    let registers = cpu.registers_mut();
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    Path::Untaken
}
