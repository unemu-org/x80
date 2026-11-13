//! Decision-18 §4 literal V/K witnesses, independent of production ALU helpers.
//!
//! D = die analysis (8085; exact part unverified); S = documented by second
//! source (CA80C85B), unverified for Intel 8085; L = adopted library rule,
//! exact hardware effect unverified. Pins/lineages: PROVENANCE §1.
//! W pp. 144–145 / Tundra 9000000.MD302.01 Fig. 2-9 p. 2-15 supply
//! arithmetic/image rules; KV circuit and notes [4]/[5], FC storage and ALU
//! control lists are one independent die origin. Each test states its label.

mod support;

use support::RecordingBus;
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::Exit;

fn step(opcode: u8, mut registers: Registers, operand: u8) -> (Registers, RecordingBus) {
    registers.pc = 0x2000;
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[opcode, operand, 0x30]);
    bus.memory[usize::from(registers.hl())] = operand;
    bus.memory[usize::from(registers.sp)] = operand;
    bus.memory[usize::from(registers.sp.wrapping_add(1))] = 0x5A;
    if opcode == 0xDB {
        bus.inputs.push_back(0x69);
    }
    let mut cpu = I8085::new(registers, Latches::default());
    let report = cpu.step(&mut bus);
    assert_eq!(report.steps, 1, "opcode={opcode:02X}");
    assert_eq!(report.exit, Exit::Budget, "opcode={opcode:02X}");
    (*cpu.registers(), bus)
}

fn initial(v: bool, k: bool, carry: bool) -> Registers {
    Registers {
        h: 0x40,
        sp: 0x6000,
        flags: Flags {
            overflow: v,
            k,
            carry,
            ..Flags::default()
        },
        ..Registers::default()
    }
}

fn vk(r: &Registers) -> (bool, bool) {
    (r.flags.overflow, r.flags.k)
}

#[test]
fn arithmetic_and_compare_all_sources_and_immediates() {
    // D/S: decision-18 §4 ADD/ADC/SUB/SBB/CMP rows, W/T arithmetic
    // definitions and KV circuit/note [5]. CMP uses temporary result sign.
    // Literals: base, immediate, A, operand, CY, result, V, K.
    for (base, immediate, a, operand, carry, result, v, k) in [
        (0x80, 0xC6, 0x7F, 1, false, 0x80, true, false),
        (0x80, 0xC6, 0x80, 0x80, false, 0, true, true),
        (0x80, 0xC6, 0xFF, 1, true, 0, false, false),
        (0x88, 0xCE, 0x7F, 0, true, 0x80, true, false),
        (0x88, 0xCE, 0x80, 0xFF, true, 0x80, false, true),
        (0x88, 0xCE, 0x7F, 0x7F, true, 0xFF, true, false),
        (0x90, 0xD6, 0x80, 1, false, 0x7F, true, true),
        (0x90, 0xD6, 0, 1, true, 0xFF, false, true),
        (0x98, 0xDE, 0x80, 0, true, 0x7F, true, true),
        (0x98, 0xDE, 0x7F, 0xFF, true, 0x7F, false, false),
        (0xB8, 0xFE, 0x80, 1, true, 0x7F, true, true),
        (0xB8, 0xFE, 0x7F, 0x80, true, 0xFF, true, false),
        (0xB8, 0xFE, 0, 1, true, 0xFF, false, true),
    ] {
        for source in 0..=8 {
            if source == 7 && a != operand {
                continue; // A is its own operand, witnessed separately below.
            }
            for old in [false, true] {
                let mut r = initial(old, old, carry);
                r.a = a;
                match source {
                    0 => r.b = operand,
                    1 => r.c = operand,
                    2 => r.d = operand,
                    3 => r.e = operand,
                    4 => r.h = operand,
                    5 => r.l = operand,
                    _ => {}
                }
                let opcode = if source == 8 {
                    immediate
                } else {
                    base + source
                };
                let (after, _) = step(opcode, r, operand);
                assert_eq!(
                    vk(&after),
                    (v, k),
                    "op={opcode:02X} A={a:02X} rhs={operand:02X}"
                );
                assert_eq!(after.a, if base == 0xB8 { a } else { result });
            }
        }
    }
    for (opcode, a, carry, result, v, k) in [
        (0x87, 0x80, false, 0, true, true),
        (0x8F, 0x7F, true, 0xFF, true, false),
        (0x97, 0x80, true, 0, false, false),
        (0x9F, 0x80, true, 0xFF, false, true),
        (0xBF, 0x80, true, 0x80, false, false),
    ] {
        let mut r = initial(!v, !k, carry);
        r.a = a;
        let (after, _) = step(opcode, r, 0);
        assert_eq!((after.a, vk(&after)), (result, (v, k)));
    }
}

#[test]
fn inr_dcr_all_destinations() {
    // D/S derived +/-1 specialisation: decision-18 §4, W/T and KV note [4].
    for (base, old, result, v, k) in [
        (0x04, 0x7F, 0x80, true, false),
        (0x04, 0xFF, 0, false, false),
        (0x04, 0x80, 0x81, false, true),
        (0x05, 0x80, 0x7F, true, true),
        (0x05, 0, 0xFF, false, true),
        (0x05, 1, 0, false, false),
    ] {
        for destination in 0..8 {
            for carry in [false, true] {
                let mut r = initial(!v, !k, carry);
                match destination {
                    0 => r.b = old,
                    1 => r.c = old,
                    2 => r.d = old,
                    3 => r.e = old,
                    4 => r.h = old,
                    5 => r.l = old,
                    7 => r.a = old,
                    _ => {}
                }
                let address = r.hl();
                let (after, bus) = step(base + destination * 8, r, old);
                assert_eq!(vk(&after), (v, k));
                assert_eq!(after.flags.carry, carry);
                let actual = [
                    after.b,
                    after.c,
                    after.d,
                    after.e,
                    after.h,
                    after.l,
                    bus.memory[usize::from(address)],
                    after.a,
                ][usize::from(destination)];
                assert_eq!(actual, result);
            }
        }
    }
}

#[test]
fn pair_increment_decrement_wraps_and_clears_k_otherwise() {
    // K D/S: W p. 144/T Fig. 2-9, KV incrementer/ALU carry_to_k_flag.
    // V preservation L: decision-18 §4, no independent exact effect.
    for (base, old, result, k) in [
        (0x03, 0xFFFF, 0, true),
        (0x03, 0x7FFF, 0x8000, false),
        (0x03, 0x00FF, 0x0100, false),
        (0x0B, 0, 0xFFFF, true),
        (0x0B, 0x8000, 0x7FFF, false),
        (0x0B, 0x0100, 0x00FF, false),
    ] {
        for pair in 0..4 {
            for v in [false, true] {
                for old_k in [false, true] {
                    let mut r = initial(v, old_k, true);
                    match pair {
                        0 => r.set_bc(old),
                        1 => r.set_de(old),
                        2 => r.set_hl(old),
                        _ => r.sp = old,
                    }
                    let (after, _) = step(base + pair * 16, r, 0);
                    assert_eq!(vk(&after), (v, k));
                    assert_eq!(
                        [after.bc(), after.de(), after.hl(), after.sp][usize::from(pair)],
                        result
                    );
                    assert_eq!(
                        (
                            after.flags.sign,
                            after.flags.zero,
                            after.flags.auxiliary_carry,
                            after.flags.parity,
                            after.flags.carry
                        ),
                        (false, false, false, false, true)
                    );
                }
            }
        }
    }
}

#[test]
fn logical_clears_v_and_uses_result_sign_including_xor_conflict() {
    // L following explicit die reading: decision-18 §4, KV note [4]/ALU
    // constant carries/enables. XRA 80,80 distinguishes the rejected W/T K.
    for (base, immediate, a, operand, result, k) in [
        (0xA0, 0xE6, 0x80, 0x80, 0x80, true),
        (0xA0, 0xE6, 0x80, 0x7F, 0, false),
        (0xA8, 0xEE, 0x80, 0x80, 0, false),
        (0xA8, 0xEE, 0, 0x80, 0x80, true),
        (0xB0, 0xF6, 0, 0x80, 0x80, true),
        (0xB0, 0xF6, 0x80, 0x80, 0x80, true),
        (0xB0, 0xF6, 1, 2, 3, false),
    ] {
        for source in 0..=8 {
            if source == 7 && a != operand {
                continue;
            }
            let mut r = initial(true, !k, true);
            r.a = a;
            match source {
                0 => r.b = operand,
                1 => r.c = operand,
                2 => r.d = operand,
                3 => r.e = operand,
                4 => r.h = operand,
                5 => r.l = operand,
                _ => {}
            }
            let opcode = if source == 8 {
                immediate
            } else {
                base + source
            };
            let (after, _) = step(opcode, r, operand);
            assert_eq!((after.a, vk(&after)), (result, (false, k)));
        }
    }
}

#[test]
fn rotates_update_v_but_preserve_independent_k_and_szap() {
    // L: decision-18 §4; KV note [4]/ALU V controls. Right V=0;
    // left V is signed A+A (+old CY only for RAL), independent of S latch.
    for (opcode, a, carry, result, v) in [
        (0x07, 0x40, false, 0x80, true),
        (0x07, 0x80, false, 1, true),
        (0x07, 0xC0, false, 0x81, false),
        (0x17, 0x7F, true, 0xFF, true),
        (0x17, 0xFF, true, 0xFF, false),
        (0x0F, 0x80, false, 0x40, false),
        (0x1F, 0x80, true, 0xC0, false),
    ] {
        for k in [false, true] {
            for sign in [false, true] {
                let mut r = initial(!v, k, carry);
                r.a = a;
                r.flags.sign = sign;
                r.flags.zero = true;
                r.flags.auxiliary_carry = true;
                r.flags.parity = true;
                let (after, _) = step(opcode, r, 0);
                assert_eq!((after.a, vk(&after)), (result, (v, k)));
                assert_eq!(
                    (
                        after.flags.sign,
                        after.flags.zero,
                        after.flags.auxiliary_carry,
                        after.flags.parity
                    ),
                    (sign, true, true, true)
                );
            }
        }
    }
}

#[test]
fn daa_overflow_uses_combined_correction() {
    // L: decision-18 §4 exact combined ACT rule; ALU accumulator/ACT and
    // KV V circuit. 20+60 and 7A+06 refute the restricted top-digit reading.
    for (a, carry, ac, result, v, k) in [
        (0x20, true, false, 0x80, true, false),
        (0x7A, false, false, 0x80, true, false),
        (0x80, false, false, 0x80, false, true),
        (0x00, false, false, 0, false, false),
        (0x14, true, true, 0x7A, false, false),
        (0x1A, true, false, 0x80, true, false),
        (0xFA, false, false, 0x60, false, false),
        (0x9A, false, false, 0, false, false),
    ] {
        let mut r = initial(!v, !k, carry);
        r.a = a;
        r.flags.auxiliary_carry = ac;
        let (after, _) = step(0x27, r, 0);
        assert_eq!((after.a, vk(&after)), (result, (v, k)), "A={a:02X}");
    }
}

#[test]
fn dad_signed_word_overflow_includes_low_byte_carry_and_preserves_k() {
    // V S / die-reading reconciliation: decision-18 §4, W/T 16-bit V
    // and KV note [4] explicit DAD take precedence over omitted enable list.
    // K preservation L: no exact DAD K statement.
    for pair in 0..4 {
        let cases: &[(u16, u16, u16, bool)] = if pair == 2 {
            &[
                (0x4000, 0x4000, 0x8000, true),
                (0x8000, 0x8000, 0, true),
                (0xFFFF, 0xFFFF, 0xFFFE, false),
            ]
        } else {
            &[
                (0x7FFF, 1, 0x8000, true),
                (0x8000, 0xFFFF, 0x7FFF, true),
                (0xFFFF, 1, 0, false),
                (0x1234, 0x0102, 0x1336, false),
            ]
        };
        for &(hl, operand, result, v) in cases {
            for k in [false, true] {
                let mut r = initial(!v, k, false);
                r.set_hl(hl);
                match pair {
                    0 => r.set_bc(operand),
                    1 => r.set_de(operand),
                    2 => {}
                    _ => r.sp = operand,
                }
                let (after, _) = step(0x09 + pair * 16, r, 0);
                assert_eq!((after.hl(), vk(&after)), (result, (v, k)));
            }
        }
    }
}

#[test]
fn psw_images_push_pop_and_independent_loaded_flags() {
    // Image D/S W p. 145/T Fig. 2-9, KV bus/FC storage. PUSH preservation L.
    // POP/set_psw L following KV databus loads; discard only bit 3, and
    // never impose K=V XOR S on arbitrary loaded combinations (§4).
    for (input, output, v, k) in [
        (0x00, 0x00, false, false),
        (0x02, 0x02, true, false),
        (0x20, 0x20, false, true),
        (0x22, 0x22, true, true),
        (0x08, 0, false, false),
        (0xFF, 0xF7, true, true),
        (0x80, 0x80, false, false),
    ] {
        let mut r = initial(false, false, false);
        r.set_psw(0x5A00 | u16::from(input));
        assert_eq!(r.psw(), 0x5A00 | u16::from(output));
        assert_eq!(vk(&r), (v, k));
        let (pushed, bus) = step(0xF5, r, 0);
        assert_eq!((bus.memory[0x5FFE], bus.memory[0x5FFF]), (output, 0x5A));
        assert_eq!(pushed.flags, r.flags);
        let (popped, _) = step(0xF1, initial(!v, !k, true), input);
        assert_eq!(popped.psw(), 0x5A00 | u16::from(output));
        assert_eq!(vk(&popped), (v, k));
    }
}

#[test]
fn default_clears_and_reset_preserves_hidden_flags() {
    // L construction/reset rules, decision-18 §4 / decision-12 rule 9.
    assert_eq!(
        (Flags::default().overflow, Flags::default().k),
        (false, false)
    );
    assert_eq!(Registers::default().psw(), 0);
    for v in [false, true] {
        for k in [false, true] {
            let mut r = initial(v, k, true);
            r.pc = 0x1234;
            let mut cpu = I8085::new(r, Latches::default());
            cpu.reset();
            r.pc = 0;
            assert_eq!(*cpu.registers(), r);
        }
    }
}

#[test]
fn remaining_documented_instructions_preserve_v_k() {
    // L unstated-effect policy, decision-18 §4. Exercise every encoding in
    // each preserving family, including both condition paths via flag bits.
    let mut opcodes = vec![
        0x00, 0x2F, 0x37, 0x3F, 0x20, 0x30, 0xF3, 0xFB, 0x76, 0xC3, 0xCD, 0xC9, 0xE3, 0xE9, 0xEB,
        0xF9, 0xD3, 0xDB, 0x02, 0x12, 0x0A, 0x1A, 0x22, 0x2A, 0x32, 0x3A,
    ];
    opcodes.extend((0x40..=0x7F).filter(|op| *op != 0x76));
    for code in 0..8 {
        opcodes.extend([
            0x06 + code * 8,
            0xC0 + code * 8,
            0xC2 + code * 8,
            0xC4 + code * 8,
            0xC7 + code * 8,
        ]);
    }
    for pair in 0..4 {
        opcodes.extend([0x01 + pair * 16, 0xC5 + pair * 16]);
    }
    opcodes.extend([0xC1, 0xD1, 0xE1]); // POP PSW has its own load rule.
    for opcode in opcodes {
        for v in [false, true] {
            for k in [false, true] {
                for documented in [false, true] {
                    let mut r = initial(v, k, documented);
                    r.flags.sign = documented;
                    r.flags.zero = documented;
                    r.flags.parity = documented;
                    let (after, _) = step(opcode, r, 0);
                    assert_eq!(vk(&after), (v, k), "opcode={opcode:02X}");
                }
            }
        }
    }
}

#[test]
fn ordinary_interrupt_entry_preserves_v_k() {
    // L unstated-effect policy, decision-18 §4: all external sources.
    for (source, vector) in [0x24, 0x3C, 0x34, 0x2C, 0, 0x1234].into_iter().enumerate() {
        for v in [false, true] {
            for k in [false, true] {
                let mut bus = RecordingBus::new();
                let latches = Latches {
                    interrupt_enable: true,
                    mask5_5: false,
                    mask6_5: false,
                    mask7_5: false,
                    ..Latches::default()
                };
                match source {
                    0 => bus.trap = true,
                    1 => bus.rst7_5 = true,
                    2 => bus.rst6_5 = true,
                    3 => bus.rst5_5 = true,
                    4 => {
                        bus.intr = true;
                        bus.acknowledges.push_back(0xC7);
                    }
                    _ => {
                        bus.intr = true;
                        bus.acknowledges.extend([0xCD, 0x34, 0x12]);
                    }
                }
                let mut cpu = I8085::new(initial(v, k, true), latches);
                assert_eq!(cpu.step(&mut bus).steps, 1);
                assert_eq!(cpu.registers().pc, vector);
                assert_eq!(vk(cpu.registers()), (v, k));
            }
        }
    }
}
