//! Decision-20 §4 full-byte witnesses; expected images are literal and never
//! produced by core helpers. Source pins and A/H/L scope: PROVENANCE §1,
//! "Z80 undocumented rulings". Y = Young v0.91 (2005), R = Rak z80test v1.2a.
//! H means independently discriminated observations, not universal silicon
//! coverage; L includes the explicitly missing BIT-register written key.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, Report};

fn registers() -> Registers {
    Registers {
        pc: 0x1000,
        sp: 0x8000,
        h: 0x40,
        ix: 0x27FF,
        iy: 0x2801,
        ..Registers::default()
    }
}

fn run(bytes: &[u8], registers: Registers, data: u8, input: u8) -> (Z80, RecordingBus) {
    let mut bus = RecordingBus::new();
    bus.load(0x1000, bytes);
    bus.memory[0x4000] = data;
    bus.memory[0x2800] = data;
    bus.inputs.push_back(input);
    let mut cpu = Z80::new(registers, Latches::default());
    assert_eq!(cpu.step(&mut bus).exit, Exit::Budget);
    (cpu, bus)
}

#[test]
fn full_f_and_f_prime_input_exchange_pop_push_and_reset() {
    // H transfers: decision-20 §4; Y §2.3 p. 8 / §§8.2–8.3 pp. 28–29;
    // R .selftest 297–302, .poppush_af 1218–1223, .ex_af_af.
    // Independent storage/public input and reset preservation are L library
    // choices (decision-15 reset); this is not a hardware reset assertion.
    assert_eq!(Z80::default().registers().af(), 0);
    assert_eq!(Z80::default().registers().af_alternate(), 0);
    for f in 0..=255u8 {
        let mut r = registers();
        r.set_af(0xA500 | u16::from(f));
        r.set_af_alternate(0x5A00 | u16::from(!f));
        assert_eq!(r.af(), 0xA500 | u16::from(f));
        assert_eq!(r.af_alternate(), 0x5A00 | u16::from(!f));
        let mut reset = Z80::new(r, Latches::default());
        reset.reset();
        assert_eq!(reset.registers().flags.to_byte(), f);
        assert_eq!(reset.registers().flags_alternate.to_byte(), !f);
        let (mut cpu, mut bus) = run(&[0x08, 0xF5, 0xF1], r, 0, 0);
        assert_eq!(cpu.registers().af(), 0x5A00 | u16::from(!f));
        assert_eq!(cpu.registers().af_alternate(), 0xA500 | u16::from(f));
        assert_eq!(cpu.step(&mut bus).steps, 1);
        assert_eq!(bus.memory[0x7FFE], !f);
        assert_eq!(bus.memory[0x7FFF], 0x5A);
        bus.memory[0x7FFE] = f;
        assert_eq!(cpu.step(&mut bus).steps, 1);
        assert_eq!(cpu.registers().af(), 0x5A00 | u16::from(f));
        cpu.reset();
        assert_eq!(cpu.registers().flags.to_byte(), f);
        assert_eq!(cpu.registers().flags_alternate.to_byte(), f);
    }
    for (y, x, f) in [
        (false, false, 0),
        (false, true, 8),
        (true, false, 32),
        (true, true, 40),
    ] {
        let r = Registers {
            flags: Flags {
                y,
                x,
                ..Flags::default()
            },
            ..registers()
        };
        assert_eq!(
            Z80::new(r, Latches::default()).registers().flags.to_byte(),
            f
        );
    }
}

#[test]
fn byte_alu_cp_inc_dec_neg_daa_and_cpl_literal_images() {
    // H: decision-20 §4; Y §2.3 p. 8 / §§8.4–8.5 p. 30, §4.7 p. 18;
    // R .add_a_n… .or_n/.cp_n, .inc_*/.dec_*, .neg/.neg_, .daa/.cpl.
    // DAA witnesses Y/X only in addition to the already accepted correction.
    for (bytes, a, before, after_a, after_f) in [
        (&[0xC6, 8][..], 0x20, 0, 0x28, 0x28),
        (&[0xCE, 8], 0x1F, 1, 0x28, 0x38),
        (&[0xD6, 8], 0x30, 0, 0x28, 0x3A),
        (&[0xDE, 8], 0x31, 1, 0x28, 0x3A),
        (&[0xE6, 0x28], 0x3F, 0, 0x28, 0x3C),
        (&[0xEE, 8], 0x20, 0, 0x28, 0x2C),
        (&[0xF6, 8], 0x20, 0, 0x28, 0x2C),
        (&[0xFE, 0x28], 0x30, 0, 0x30, 0x3A), // operand 28, result 08
        (&[0xFE, 8], 0x30, 0, 0x30, 0x1A),    // operand 08, result 28
        (&[0x3C], 0x27, 0, 0x28, 0x28),
        (&[0x3D], 0x29, 0, 0x28, 0x2A),
        (&[0xED, 0x44], 0xD8, 0, 0x28, 0x3B),
        (&[0x27], 0x22, 0x10, 0x28, 0x2C),
        (&[0x2F], 0xD7, 1, 0x28, 0x3B),
        (&[0xC6, 0], 0, 0xFF, 0, 0x40), // clear stale Y/X
    ] {
        let (cpu, _) = run(
            bytes,
            Registers {
                a,
                flags: Flags::from_byte(before),
                ..registers()
            },
            0,
            0,
        );
        assert_eq!(
            (cpu.registers().a, cpu.registers().flags.to_byte()),
            (after_a, after_f),
            "{bytes:02X?}"
        );
    }
}

#[test]
fn accumulator_cb_indexed_cb_and_nibble_rotates_choose_their_result() {
    // H: decision-20 §4; Y §§8.5/8.7 pp. 30/32; R .rlca… .rra,
    // .rlc_*… .srl_*, .sro_xyd*, .rld/.rrd (new A, not memory result).
    for (opcode, a) in [(7, 0x14), (0x0F, 0x50), (0x17, 0x14), (0x1F, 0x50)] {
        let (cpu, _) = run(&[opcode], Registers { a, ..registers() }, 0, 0);
        assert_eq!(
            (cpu.registers().a, cpu.registers().flags.to_byte()),
            (0x28, 0x28)
        );
    }
    for (op, value, result, f) in [
        (0x00, 0x14, 0x28, 0x2C),
        (0x08, 0x50, 0x28, 0x2C),
        (0x10, 0x14, 0x28, 0x2C),
        (0x18, 0x50, 0x28, 0x2C),
        (0x20, 0x14, 0x28, 0x2C),
        (0x28, 0xD0, 0xE8, 0xAC),
        (0x30, 0x14, 0x29, 0x28),
        (0x38, 0x50, 0x28, 0x2C),
    ] {
        let (cpu, _) = run(
            &[0xCB, op],
            Registers {
                b: value,
                ..registers()
            },
            0,
            0,
        );
        assert_eq!(
            (cpu.registers().b, cpu.registers().flags.to_byte()),
            (result, f)
        );
        for bytes in [[0xDD, 0xCB, 1, op], [0xFD, 0xCB, 0xFF, op]] {
            let (cpu, bus) = run(&bytes, registers(), value, 0);
            assert_eq!(
                (
                    cpu.registers().b,
                    bus.memory[0x2800],
                    cpu.registers().flags.to_byte()
                ),
                (result, result, f)
            );
        }
    }
    for (op, memory) in [(0x6F, 0x80), (0x67, 8)] {
        let (cpu, bus) = run(
            &[0xED, op],
            Registers {
                a: 0x20,
                ..registers()
            },
            memory,
            0,
        );
        assert_eq!(
            (
                cpu.registers().a,
                bus.memory[0x4000],
                cpu.registers().flags.to_byte()
            ),
            (0x28, 0, 0x2C)
        );
    }
}

#[test]
fn pair_arithmetic_uses_high_result_byte() {
    // H: decision-20 §4; Y §4.6 p. 17 / §8.6 p. 31; R .add_hl_rr/
    // .add_ix_rr/.add_iy_rr/.adc_hl_rr/.sbc_hl_rr 768–801.
    for (bytes, left, right, result, f) in [
        (&[0x09][..], 0x2700, 0x0108, 0x2808, 0x28),
        (&[0xDD, 0x09], 0x2700, 0x0108, 0x2808, 0x28),
        (&[0xFD, 0x09], 0x2700, 0x0108, 0x2808, 0x28),
        (&[0xED, 0x4A], 0x2700, 0x0108, 0x2808, 0x28),
        (&[0xED, 0x42], 0x2900, 0x00F8, 0x2808, 0x2A),
        (&[0xED, 0x4A], 0x0020, 8, 0x0028, 0), // low byte must not supply Y/X
    ] {
        let mut r = registers();
        r.set_hl(left);
        r.ix = left;
        r.iy = left;
        r.set_bc(right);
        let (cpu, _) = run(bytes, r, 0, 0);
        let pair = match bytes[0] {
            0xDD => cpu.registers().ix,
            0xFD => cpu.registers().iy,
            _ => cpu.registers().hl(),
        };
        assert_eq!((pair, cpu.registers().flags.to_byte()), (result, f));
    }
}

#[test]
fn special_loads_and_all_c_port_inputs_use_loaded_byte() {
    // H: decision-20 §4; Y §§8.1/8.11 pp. 27/35; R .ld_a_i/.ld_a_r
    // 1436–1448, .in_r_c/.in_c (observed BFh); full-input rule follows Y.
    for (op, i, r) in [(0x57, 0x28, 0), (0x5F, 0, 0x26)] {
        let (cpu, _) = run(
            &[0xED, op],
            Registers {
                i,
                r,
                flags: Flags::from_byte(1),
                ..registers()
            },
            0,
            0,
        );
        assert_eq!(
            (cpu.registers().a, cpu.registers().flags.to_byte()),
            (0x28, 0x29)
        );
    }
    for op in [0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78] {
        for (input, f) in [(0xBF, 0xA9), (0x28, 0x2D), (0, 0x45)] {
            let (cpu, _) = run(
                &[0xED, op],
                Registers {
                    flags: Flags::from_byte(0xFF),
                    ..registers()
                },
                0,
                input,
            );
            assert_eq!(
                cpu.registers().flags.to_byte(),
                f,
                "ED {op:02X}, {input:02X}"
            );
        }
    }
}

#[test]
fn bit_register_l_written_key_gap_and_all_forms_s_pv() {
    // L Y/X: decision-20 §4 explicitly selects original register despite
    // Y §4.1 p. 15's masked-operand statement; R .bit_n_a/.bit_n_r
    // 805–824 discriminates it, without supplying the missing written key.
    // H other flags: same section / R .bit_n_*; C retained.
    for (op, b, f) in [
        (0x40, 0x28, 0x7D),
        (0x78, 0xA8, 0xB9),
        (0x78, 0x28, 0x7D),
        (0x40, 1, 0x11),
    ] {
        let (cpu, _) = run(
            &[0xCB, op],
            Registers {
                b,
                flags: Flags::from_byte(0x87),
                ..registers()
            },
            0,
            0,
        );
        assert_eq!(cpu.registers().flags.to_byte(), f);
    }
    for bytes in [[0xDD, 0xCB, 1, 0x46], [0xFD, 0xCB, 0xFF, 0x40]] {
        // H address-vs-data: Y §4.1 p. 15 / M indexed-address entry;
        // R .bit_n_xyd*. Interim L: direct EA=2800 until §5 WZ.
        let (cpu, _) = run(
            &bytes,
            Registers {
                flags: Flags::from_byte(1),
                ..registers()
            },
            0,
            0,
        );
        assert_eq!(cpu.registers().flags.to_byte(), 0x7D);
    }
    for (op, data, f) in [(0x7E, 0x80, 0xB9), (0x7E, 0, 0x7D), (0x46, 1, 0x39)] {
        let (cpu, _) = run(
            &[0xDD, 0xCB, 1, op],
            Registers {
                flags: Flags::from_byte(1),
                ..registers()
            },
            data,
            0,
        );
        assert_eq!(cpu.registers().flags.to_byte(), f);
    }
}

#[test]
fn interim_l_bit_hl_preserves_yx_until_x80_mn3_11_retained_wz() {
    // Interim L only: decision-20 §4 staging; §5 / x80-mn3.11 MUST update
    // this witness to retained WZ bits13/11. Y §4.1 p. 15 / M opening,
    // R .bit_n_hl and memptr idea.asm 124–127: neither memory nor HL Y/X.
    for (before, expected) in [(0, 0x54), (8, 0x5C), (0x20, 0x74), (0x28, 0x7C)] {
        let (cpu, _) = run(
            &[0xCB, 0x46],
            Registers {
                flags: Flags::from_byte(before),
                ..registers()
            },
            0x28,
            0,
        );
        assert_eq!(cpu.registers().flags.to_byte(), expected);
    }
}

#[test]
fn transfer_and_search_t_bit_one_maps_to_y_bit_five() {
    // H: decision-20 §4; Y §4.2 p. 16; R .ldi… .lddr 912–938 /
    // .cpi… .cpdr 954–980. Terminating repeats only; no §6 PC override.
    for op in [0xA0, 0xA8, 0xB0, 0xB8] {
        for (a, data, f) in [
            (0, 2, 0x20),
            (0, 8, 8),
            (1, 1, 0x20),
            (0xFF, 3, 0x20),
            (0, 0x20, 0),
        ] {
            let (cpu, _) = run(
                &[0xED, op],
                Registers {
                    a,
                    c: 1,
                    d: 0x50,
                    flags: Flags::from_byte(0x28),
                    ..registers()
                },
                data,
                0,
            );
            assert_eq!(
                cpu.registers().flags.to_byte(),
                f,
                "ED {op:02X}, {a:02X}/{data:02X}"
            );
        }
    }
    for op in [0xA1, 0xA9, 0xB1, 0xB9] {
        for (a, data, f) in [
            (2, 0, 0x22),
            (8, 0, 0x0A),
            (0x10, 7, 0x1A),
            // Decision-20 §4; Y §4.2 p. 16: half borrow makes t=07h,
            // giving F=32h rather than the unadjusted t=08h's F=1Ah.
            (0x10, 0x08, 0x32),
            (0x10, 0x0D, 0x32),
            (0, 0, 0x42),
        ] {
            let (cpu, _) = run(
                &[0xED, op],
                Registers {
                    a,
                    c: 1,
                    flags: Flags::from_byte(0x28),
                    ..registers()
                },
                data,
                0,
            );
            assert_eq!(
                cpu.registers().flags.to_byte(),
                f,
                "ED {op:02X}, {a:02X}/{data:02X}"
            );
        }
    }
}

#[test]
fn all_eight_block_io_formulas_literal_carry_half_parity_and_port_order() {
    // H discriminated flags / L unexercised full-input extension:
    // decision-20 §4; Y §4.3 pp. 16–17 / §8.11 p. 35; R .ini/.ind/
    // .outi/.outd/.inir/.indr/.otir/.otdr (Spectrum input BFh only).
    // A port ordering retained: decision-15 rule 4; UM pp. 298/309,
    // TM Table 7.0-11 p. 54. Input old B, output decremented B.
    // Each row gives literal input C / output L_after, data and expected F;
    // all four repeat opcodes terminate (B=1). Ordinary nonrepeat cases
    // additionally expose b=28/FF, even/odd parity, k=FF/100/101.
    for op in [0xA2, 0xAA, 0xA3, 0xAB, 0xB2, 0xBA, 0xB3, 0xBB] {
        let input = op & 1 == 0;
        let down = op & 8 != 0;
        for (offset, data, expected_f) in [
            (0u8, 0, 0x44),
            (0, 0x80, 0x46),
            (1, 0xFE, 0x42),
            (1, 0xFF, 0x57),
            (2, 0xFF, 0x53),
        ] {
            let c = if input {
                if down {
                    offset + 1
                } else {
                    offset.wrapping_sub(1)
                }
            } else {
                0x33
            };
            let l = if down {
                offset.wrapping_add(1)
            } else {
                offset.wrapping_sub(1)
            };
            let mut r = Registers {
                b: 1,
                c,
                h: 0x40,
                l,
                flags: Flags::from_byte(0xFF),
                ..registers()
            };
            if input {
                r.set_hl(0x4000);
            }
            let address = r.hl();
            let mut bus = RecordingBus::new();
            bus.load(0x1000, &[0xED, op]);
            bus.memory[usize::from(address)] = data;
            bus.inputs.push_back(data);
            let mut cpu = Z80::new(r, Latches::default());
            assert_eq!(
                cpu.step(&mut bus),
                Report {
                    steps: 1,
                    cycles: 16,
                    exit: Exit::Budget
                }
            );
            assert_eq!(
                cpu.registers().flags.to_byte(),
                expected_f,
                "ED {op:02X}, offset={offset}, data={data:02X}"
            );
            let event = if input {
                Event::Input16 {
                    port: 0x0100 | u16::from(c),
                    value: data,
                }
            } else {
                Event::Output16 {
                    port: 0x0033,
                    value: data,
                }
            };
            assert!(bus.log.contains(&event));
        }
    }
    for (b, offset, data, f) in [(0x29, 0u8, 0, 0x2C), (0x29, 1, 0, 0x28), (0, 1, 0xFF, 0xBF)] {
        for op in [0xA2, 0xAA, 0xA3, 0xAB] {
            let c = if op & 8 != 0 {
                offset + 1
            } else {
                offset.wrapping_sub(1)
            };
            let l = if op & 8 != 0 {
                offset + 1
            } else {
                offset.wrapping_sub(1)
            };
            let r = Registers {
                b,
                c,
                l,
                ..registers()
            };
            let mut bus = RecordingBus::new();
            bus.load(0x1000, &[0xED, op]);
            bus.memory[usize::from(r.hl())] = data;
            bus.inputs.push_back(data);
            let mut cpu = Z80::new(r, Latches::default());
            cpu.step(&mut bus);
            assert_eq!(cpu.registers().flags.to_byte(), f);
        }
    }
}

#[test]
fn interim_l_scf_ccf_use_unconditional_a_until_x80_mn3_16_q() {
    // Interim L: decision-20 §4 staging; Y §8.5 p. 30 unconditional A.
    // §6 B/S/R predecessor-dependent selection is deferred to x80-mn3.16.
    for (op, a, before, expected) in [
        (0x37, 0x28, 0, 0x29),
        (0x3F, 0x28, 1, 0x38),
        (0x37, 0, 0x28, 1),
        (0x3F, 0, 0x28, 1),
    ] {
        let (cpu, _) = run(
            &[op],
            Registers {
                a,
                flags: Flags::from_byte(before),
                ..registers()
            },
            0,
            0,
        );
        assert_eq!(cpu.registers().flags.to_byte(), expected);
    }
}

#[test]
fn remaining_classes_preserve_full_flags() {
    // H for ordinary tested classes, L universal preservation: decision-20
    // §4; Y §§8.1–8.11 pp. 26–35 / R corresponding load/flow/RES/SET groups.
    // Holes, ignored-prefix cases and HALT are L, not inferred ALU activity.
    for bytes in [
        &[0][..],
        &[0xED, 0],
        &[0xDD, 0],
        &[0x41],
        &[0x03],
        &[0x0B],
        &[0xED, 0x47],
        &[0xED, 0x4F],
        &[0xCB, 0x80],
        &[0xCB, 0xC0],
        &[0xDB, 0x33],
        &[0xD3, 0x33],
        &[0xED, 0x41],
        &[0xED, 0x71],
        &[0xF3],
        &[0xFB],
        &[0xED, 0x56],
        &[0xC3, 0, 0x20],
        &[0xCD, 0, 0x20],
        &[0xC9],
        &[0xC7],
        &[0xD9],
        &[0xEB],
        &[0xE3],
        &[0xC5],
        &[0xC1],
        &[0xDD, 0xCB, 1, 0x86],
    ] {
        for f in [0x28, 0xD7, 0xFF] {
            let (cpu, _) = run(
                bytes,
                Registers {
                    flags: Flags::from_byte(f),
                    ..registers()
                },
                0,
                0,
            );
            assert_eq!(cpu.registers().flags.to_byte(), f, "{bytes:02X?}");
        }
    }
}

#[test]
fn interrupt_entry_and_halt_preserve_full_f_and_f_prime_by_library_rule() {
    // L universal preservation: decision-20 §4 / Young §§8.1–8.11
    // pp. 26–35; Rak ordinary preservation groups do not inject interrupts
    // or HALT. No independent hardware acquisition is claimed for these.
    for nmi in [false, true] {
        let r = Registers {
            flags: Flags::from_byte(0xFF),
            flags_alternate: Flags::from_byte(0x28),
            ..registers()
        };
        let mut cpu = Z80::new(
            r,
            Latches {
                iff1: true,
                interrupt_mode: InterruptMode::Mode1,
                ..Latches::default()
            },
        );
        let mut bus = RecordingBus::new();
        bus.nmi = nmi;
        bus.int = !nmi;
        bus.acknowledges.push_back(0);
        assert_eq!(cpu.step(&mut bus).steps, 1);
        assert_eq!(cpu.registers().flags.to_byte(), 0xFF);
        assert_eq!(cpu.registers().flags_alternate.to_byte(), 0x28);
    }
    let mut bus = RecordingBus::new();
    bus.load(0x1000, &[0x76]);
    let mut cpu = Z80::new(
        Registers {
            flags: Flags::from_byte(0xFF),
            ..registers()
        },
        Latches::default(),
    );
    // Decision-15 boundary priority: one-step budget wins immediately after
    // HALT retirement; the next boundary exposes prompt Halt without a step.
    assert_eq!(cpu.step(&mut bus).exit, Exit::Budget);
    assert!(cpu.latches().halted);
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 0,
            cycles: 0,
            exit: Exit::Halt
        }
    );
    assert_eq!(cpu.registers().flags.to_byte(), 0xFF);
}
