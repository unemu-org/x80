//! Decision-20 §6 literal witnesses. Source revisions/hashes: PROVENANCE §1.
//! Q pattern A is B8039/H (Banks' written 8039 report / Rak post-CCF
//! acquisition), not universal NMOS behavior. Classification, DD/FD clearing,
//! interrupt/default/reset and snapshot rules are L bookkeeping.
//! Repeat LDxR/INxR observed projections are H; arbitrary PC, CPxR/OTxR and
//! the complete I/O equation domain are L following Banks' pinned equations.
//! LD A,I/R: UM008011-0816 rev.11 pp.94–95 (A note); decision-15 rule10's
//! boundary interpretation is L. EI/NMI rules7–8 stay L; no new anomaly.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, Report};

fn setup(pc: u16, bytes: &[u8]) -> (Z80, RecordingBus) {
    let mut bus = RecordingBus::new();
    bus.load(pc, bytes);
    bus.load(0x8000, &[0x28, 0]);
    bus.inputs.extend([0x28; 4]);
    let cpu = Z80::new(
        Registers {
            pc,
            sp: 0x8000,
            h: 0x40,
            d: 0x50,
            b: 2,
            ix: 0x4000,
            iy: 0x4000,
            ..Registers::default()
        },
        Latches::default(),
    );
    (cpu, bus)
}

#[test]
fn scf_ccf_after_every_modifying_and_nonmodifying_class() {
    // L universal class table: decision-20 §6. B's POP exception and R's
    // post-CCF groups support distinctions, not a complete silicon table.
    // Include every byte ALU family and accumulator/CB/nibble rotate class,
    // pair operations, all block classes, and both special-register loads.
    let modifying: &[&[u8]] = &[
        &[0x80],
        &[0x88],
        &[0x90],
        &[0x98],
        &[0xA0],
        &[0xA8],
        &[0xB0],
        &[0xB8],
        &[0xC6, 0],
        &[0xCE, 0],
        &[0xD6, 0],
        &[0xDE, 0],
        &[0xE6, 0],
        &[0xEE, 0],
        &[0xF6, 0],
        &[0xFE, 0],
        &[0x04],
        &[0x05],
        &[0x27],
        &[0x2F],
        &[0xED, 0x44],
        &[0xED, 0x7C],
        &[0x07],
        &[0x0F],
        &[0x17],
        &[0x1F],
        &[0xCB, 0],
        &[0xCB, 8],
        &[0xCB, 0x10],
        &[0xCB, 0x18],
        &[0xCB, 0x20],
        &[0xCB, 0x28],
        &[0xCB, 0x30],
        &[0xCB, 0x38],
        &[0xED, 0x6F],
        &[0xED, 0x67],
        &[0xCB, 0x40],
        &[0x09],
        &[0xED, 0x4A],
        &[0xED, 0x42],
        &[0xED, 0x57],
        &[0xED, 0x5F],
        &[0xED, 0x40],
        &[0xED, 0x70],
        &[0xED, 0xA0],
        &[0xED, 0xA8],
        &[0xED, 0xA1],
        &[0xED, 0xA9],
        &[0xED, 0xB0],
        &[0xED, 0xB8],
        &[0xED, 0xB1],
        &[0xED, 0xB9],
        &[0xED, 0xA2],
        &[0xED, 0xAA],
        &[0xED, 0xA3],
        &[0xED, 0xAB],
        &[0xED, 0xB2],
        &[0xED, 0xBA],
        &[0xED, 0xB3],
        &[0xED, 0xBB],
        &[0x37],
        &[0x3F],
        &[0xDD, 0x84],
        &[0xFD, 0x24],
        &[0xDD, 0x09],
        &[0xFD, 0xCB, 0, 0x46],
        &[0xDD, 0xCB, 0, 0x30],
    ];
    let nonmodifying: &[&[u8]] = &[
        &[0x41],
        &[0x06, 0x28],
        &[0x01, 0, 0],
        &[0xED, 0x47],
        &[0xED, 0x4F],
        &[0xF1],
        &[0x08],
        &[0xD9],
        &[0xEB],
        &[0xE3],
        &[0xF5],
        &[0xC1],
        &[0xCB, 0x80],
        &[0xCB, 0xC0],
        &[0x03],
        &[0x0B],
        &[0xC3, 0, 0x30],
        &[0x18, 0],
        &[0x10, 0],
        &[0xCD, 0, 0x30],
        &[0xC9],
        &[0xC7],
        &[0xED, 0x45],
        &[0xED, 0x4D],
        &[0xD3, 0],
        &[0xED, 0x41],
        &[0xED, 0x71],
        &[0xDB, 0],
        &[0],
        &[0xED, 0],
        &[0xED, 0xED],
        &[0x76],
        &[0xF3],
        &[0xFB],
        &[0xED, 0x46],
        &[0xED, 0x56],
        &[0xED, 0x5E],
        &[0xDD, 0x21, 0, 0],
        &[0xFD, 0x23],
        &[0xDD, 0xCB, 0, 0x86],
        &[0xFD, 0],
    ];
    for (cases, expected_q, scf, ccf) in [
        (modifying, true, 0xC5, 0xD4),
        (nonmodifying, false, 0xED, 0xFC),
    ] {
        for bytes in cases {
            for initial_q in [false, true] {
                for (probe, expected_f) in [(0x37, scf), (0x3F, ccf)] {
                    let (mut cpu, mut bus) = setup(0x1000, bytes);
                    cpu.latches_mut().q = initial_q;
                    assert_eq!(cpu.step(&mut bus).steps, 1, "{bytes:02X?}");
                    assert_eq!(cpu.latches().q, expected_q, "{bytes:02X?}");
                    // Supply identical probe operands without changing Q. This
                    // isolates the predecessor class from its arithmetic result.
                    cpu.registers_mut().pc = 0x3000;
                    cpu.registers_mut().a = 0;
                    cpu.registers_mut().flags = Flags::from_byte(0xFF);
                    cpu.latches_mut().halted = false;
                    bus.load(0x3000, &[probe]);
                    cpu.step(&mut bus);
                    assert_eq!(
                        cpu.registers().flags.to_byte(),
                        expected_f,
                        "{bytes:02X?} -> {probe:02X}"
                    );
                    assert!(cpu.latches().q);
                }
            }
        }
    }
}

#[test]
fn q_records_class_even_when_f_is_unchanged_and_pop_or_exchange_changes_f() {
    // CP 0 leaves literal F=42 unchanged, but is modifying. POP AF and
    // EX AF,AF' replace F=FF with 28 but are nonmodifying (L class rule).
    for (bytes, before, after, q) in [
        (&[0xFE, 0][..], 0x42, 0x42, true),
        (&[0xF1], 0xFF, 0x28, false),
        (&[0x08], 0xFF, 0x28, false),
    ] {
        let (mut cpu, mut bus) = setup(0x1000, bytes);
        cpu.registers_mut().flags = Flags::from_byte(before);
        cpu.registers_mut().flags_alternate = Flags::from_byte(0x28);
        cpu.latches_mut().q = !q;
        cpu.step(&mut bus);
        assert_eq!(cpu.registers().flags.to_byte(), after);
        assert_eq!(cpu.latches().q, q);
    }
}

#[test]
fn dd_fd_clear_predecessor_q_without_retiring_and_terminal_replaces_it() {
    // L prefix policy: B “SCF/CCF” literal update 9/7/2026, no independent
    // matching R vector. Prefixes poll/retire only as part of the instruction.
    for bytes in [&[0xDD, 0x37][..], &[0xFD, 0x3F], &[0xDD, 0xFD, 0x37]] {
        let (mut cpu, mut bus) = setup(0x1000, bytes);
        cpu.latches_mut().q = true;
        cpu.registers_mut().flags = Flags::from_byte(0x28);
        let report = cpu.step(&mut bus);
        assert_eq!(report.steps, 1);
        assert_eq!(report.cycles, if bytes.len() == 3 { 12 } else { 8 });
        assert_eq!(cpu.registers().flags.to_byte(), 0x29);
        assert!(cpu.latches().q);
        assert_eq!(
            bus.log
                .iter()
                .filter(|e| matches!(e, Event::StopPoll(_)))
                .count(),
            1
        );
    }
    for (bytes, q) in [
        (&[0xDD, 0xFD, 0x04][..], true),
        (&[0xFD, 0xDD, 0x41], false),
    ] {
        let (mut cpu, mut bus) = setup(0x1000, bytes);
        cpu.step(&mut bus);
        assert_eq!(cpu.latches().q, q);
    }
}

#[test]
fn q_complete_snapshots_stop_budget_unsupported_reset_and_prompt_halt() {
    assert!(!Z80::default().latches().q);
    for q in [false, true] {
        let (mut cpu, mut bus) = setup(0x1000, &[0x37]);
        cpu.latches_mut().q = q;
        cpu.latches_mut().wz = 0xABCD;
        let snapshot = cpu.clone();
        assert_eq!(Z80::new(*cpu.registers(), *cpu.latches()), snapshot);
        bus.stop = true;
        assert_eq!(cpu.step(&mut bus).exit, Exit::Stop);
        assert_eq!(cpu, snapshot);
        bus.log.clear();
        assert_eq!(
            cpu.run(
                &mut bus,
                Budget {
                    steps: 0,
                    cycles: 0
                }
            )
            .exit,
            Exit::Budget
        );
        assert_eq!(cpu, snapshot);
        assert!(bus.log.is_empty());
        bus.stop = false;
        for mode in [InterruptMode::Mode0, InterruptMode::Mode2] {
            cpu.latches_mut().iff1 = true;
            cpu.latches_mut().interrupt_mode = mode;
            bus.int = true;
            bus.acknowledges.push_back(1);
            let refused = cpu.clone();
            assert!(matches!(cpu.step(&mut bus).exit, Exit::Unsupported(_)));
            assert_eq!(cpu, refused);
        }
        bus.int = false;
        cpu.latches_mut().halted = true;
        let halted = cpu.clone();
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 0,
                cycles: 0,
                exit: Exit::Halt
            }
        );
        assert_eq!(cpu, halted); // No invented idle predecessor, even Q=true.
        cpu.reset();
        assert!(!cpu.latches().q);
        assert_eq!(cpu.latches().wz, 0xABCD);
    }
    let (mut cpu, mut bus) = setup(0x1000, &[0x76]);
    cpu.latches_mut().q = true;
    cpu.step(&mut bus);
    assert!(!cpu.latches().q); // HALT entry itself is nonmodifying.
}

#[test]
fn successful_nmi_and_each_int_mode_clear_q_before_handler_scf_ccf() {
    // L interrupt-entry classification and snapshot policy, decision-20 §6.
    for (nmi, mode, ack, target, cycles) in [
        (true, InterruptMode::Mode0, 0, 0x66, 11),
        (false, InterruptMode::Mode0, 0xCF, 8, 13),
        (false, InterruptMode::Mode1, 0, 0x38, 13),
        (false, InterruptMode::Mode2, 0, 0x3000, 19),
    ] {
        for (probe, f) in [(0x37, 0xED), (0x3F, 0xFC)] {
            let (mut cpu, mut bus) = setup(0x1000, &[0]);
            cpu.latches_mut().q = true;
            cpu.latches_mut().iff1 = true;
            cpu.latches_mut().interrupt_mode = mode;
            cpu.registers_mut().i = 0x20;
            cpu.registers_mut().flags = Flags::from_byte(0xFF);
            bus.load(0x2000, &[0, 0x30]);
            bus.load(target, &[probe]);
            bus.nmi = nmi;
            bus.int = !nmi;
            bus.acknowledges.push_back(ack);
            assert_eq!(cpu.step(&mut bus).cycles, cycles);
            assert_eq!(cpu.registers().pc, target);
            assert!(!cpu.latches().q);
            cpu.step(&mut bus);
            assert_eq!(cpu.registers().flags.to_byte(), f);
            assert!(cpu.latches().q);
        }
    }
}

#[test]
fn continuing_ldxr_cpxr_replace_pc_bits_but_termination_keeps_ordinary_flags() {
    // B “Interrupted block instructions”; R .ldir_nop/.lddr_nop (H).
    // CPxR/arbitrary p, including ED after ignored prefix/wrap, are L.
    for (op, continuing, terminating) in [
        (0xB0, 0xC5, 0xE1),
        (0xB8, 0xC5, 0xE1),
        (0xB1, 0x17, 0x33),
        (0xB9, 0x17, 0x33),
    ] {
        for (p, yx) in [
            (0, 0),
            (0x0800, 8),
            (0x2000, 0x20),
            (0x2800, 0x28),
            (0xFFFF, 0x28),
        ] {
            for (count, cycles, base) in [(2, 21, continuing), (1, 16, terminating)] {
                let (mut cpu, mut bus) = setup(p, &[0xED, op]);
                cpu.registers_mut().set_bc(count);
                cpu.registers_mut().a = if op & 1 == 0 { 0 } else { 0x10 };
                cpu.registers_mut().flags = Flags::from_byte(0xC1);
                bus.memory[0x4000] = if op & 1 == 0 { 2 } else { 8 };
                assert_eq!(cpu.step(&mut bus).cycles, cycles);
                assert_eq!(
                    cpu.registers().flags.to_byte(),
                    if count == 2 { base | yx } else { base }
                );
                assert_eq!(
                    cpu.registers().pc,
                    if count == 2 { p } else { p.wrapping_add(2) }
                );
                assert!(cpu.latches().q);
                let snapshot = cpu.clone();
                bus.stop = true;
                assert_eq!(cpu.step(&mut bus).exit, Exit::Stop);
                assert_eq!(cpu, snapshot);
            }
        }
    }
    for op in [0xB1, 0xB9] {
        let (mut cpu, mut bus) = setup(0x2800, &[0xED, op]);
        cpu.registers_mut().set_bc(2);
        cpu.registers_mut().a = 8;
        cpu.registers_mut().flags = Flags::from_byte(1);
        bus.memory[0x4000] = 8;
        assert_eq!(cpu.step(&mut bus).cycles, 16); // early match, BC remains 1
        assert_eq!(cpu.registers().flags.to_byte(), 0x47);
    }
    let (mut cpu, mut bus) = setup(0x27FF, &[0xDD, 0xED, 0xB0]);
    cpu.registers_mut().set_bc(2);
    assert_eq!(cpu.step(&mut bus).cycles, 25);
    assert_eq!(cpu.registers().pc, 0x2800);
    assert_eq!(cpu.registers().flags.to_byte(), 0x2C);
}

#[test]
fn continuing_inxr_otxr_literal_pv_h_branches_and_terminating_base_flags() {
    // B exact §6 equations. H only observed INxR domain / L complete
    // branch domain and OTxR. offset is C±1 for input, L_after for output.
    // Literal tuples: b_after, offset, data, continuing F at p=2800,
    // ordinary F (same b/data for nonrepeat) and terminating F (b_after=0).
    for op in [0xB2, 0xBA, 0xB3, 0xBB] {
        for (b, offset, data, continuing_f, ordinary_f, terminating_f) in [
            (0x10u8, 1u8, 0xFF, 0x3F, 0x13, 0x57), // C1/N1, H set
            (0x11, 1, 0xFF, 0x2F, 0x17, 0x57),     // C1/N1, H cleared
            (0x0F, 1, 0xFF, 0x2F, 0x1F, 0x57),
            (0x0E, 1, 0xFF, 0x2B, 0x1B, 0x57),    // no PV toggle
            (0x0F, 0x81, 0x7F, 0x3D, 0x1D, 0x55), // C1/N0, H set
            (0x10, 0x81, 0x7F, 0x2D, 0x11, 0x55), // C1/N0, H cleared
            (0x0E, 0x81, 0x7F, 0x2D, 0x19, 0x55),
            (0x11, 0x81, 0x7F, 0x29, 0x15, 0x55),
            (1, 0, 0x80, 0x2E, 2, 0x46),          // C0/N1, toggle
            (8, 0, 0x80, 0x2A, 0x0A, 0x46),       // C0/N1, no toggle
            (0x80, 0, 0x80, 0xAA, 0x82, 0x46),    // S retained, Z clear
            (0xFF, 1, 0xFF, 0xAF, 0xBF, 0x57),    // B=0 wraps, b-1
            (0xFF, 0x81, 0x7F, 0xBD, 0xBD, 0x55), // b+1 wraps
            (0xFF, 0, 0x80, 0xAA, 0xAE, 0x46),
            (1, 1, 0, 0x28, 4, 0x40), // C0/N0, toggle
        ] {
            for (opcode, count, cycles, expected) in [
                (op, b.wrapping_add(1), 21, continuing_f),
                (op & !0x10, b.wrapping_add(1), 16, ordinary_f),
                (op, 1, 16, terminating_f),
            ] {
                let (mut cpu, mut bus) = setup(0x2800, &[0xED, opcode]);
                let down = op & 8 != 0;
                cpu.registers_mut().b = count;
                cpu.registers_mut().c = if down {
                    offset.wrapping_add(1)
                } else {
                    offset.wrapping_sub(1)
                };
                if op & 1 != 0 {
                    cpu.registers_mut().l = if down {
                        offset.wrapping_add(1)
                    } else {
                        offset.wrapping_sub(1)
                    };
                }
                let address = cpu.registers().hl();
                bus.memory[usize::from(address)] = data;
                bus.inputs.clear();
                bus.inputs.push_back(data);
                cpu.registers_mut().flags = Flags::from_byte(0xFF);
                assert_eq!(cpu.step(&mut bus).cycles, cycles);
                assert_eq!(
                    cpu.registers().flags.to_byte(),
                    expected,
                    "ED {opcode:02X}, b={b:02X}, offset={offset:02X}, data={data:02X}"
                );
                assert_eq!(
                    cpu.registers().pc,
                    if cycles == 21 { 0x2800 } else { 0x2802 }
                );
                assert!(cpu.latches().q);
                let snapshot = cpu.clone();
                bus.stop = true;
                assert_eq!(cpu.step(&mut bus).exit, Exit::Stop);
                assert_eq!(cpu, snapshot);
            }
        }
    }
    // PC routing independent of B/data; prefixes before ED must not supply p.
    for (p, expected) in [
        (0u16, 6),
        (0x0800, 0x0E),
        (0x2000, 0x26),
        (0x2800, 0x2E),
        (0xFFFF, 0x2E),
    ] {
        for op in [0xB2, 0xBA, 0xB3, 0xBB] {
            let (mut cpu, mut bus) = setup(p.wrapping_sub(1), &[0xFD, 0xED, op]);
            cpu.registers_mut().b = 2;
            cpu.registers_mut().c = if op & 8 == 0 { 0xFF } else { 1 };
            cpu.registers_mut().l = if op & 8 == 0 { 0xFF } else { 1 };
            let address = cpu.registers().hl();
            bus.memory[usize::from(address)] = 0x80;
            bus.inputs.clear();
            bus.inputs.push_back(0x80);
            assert_eq!(cpu.step(&mut bus).cycles, 25);
            assert_eq!(cpu.registers().flags.to_byte(), expected);
            assert_eq!(cpu.registers().pc, p);
        }
    }
}

#[test]
fn self_overwrite_exposes_continuing_flags_even_when_next_repeat_is_nop() {
    // H observed LDxR/INxR repeat-boundary projections: R .ldir_nop/
    // .lddr_nop/.inir_nop/.indr_nop; L arbitrary p/complete input domain.
    for (op, expected, bc) in [
        (0xB0, 0x2C, 2),
        (0xB8, 0x2C, 2),
        (0xB2, 0x28, 0x0200),
        (0xBA, 0x28, 0x0202),
    ] {
        let (mut cpu, mut bus) = setup(0x2800, &[0xED, op]);
        cpu.registers_mut().set_bc(bc);
        if op & 2 == 0 {
            cpu.registers_mut().set_de(0x2800);
        } else {
            cpu.registers_mut().set_hl(0x2800);
        }
        bus.inputs.clear();
        bus.inputs.push_back(0); // overwrite ED with NOP
        assert_eq!(cpu.step(&mut bus).cycles, 21);
        assert_eq!(bus.memory[0x2800], 0);
        assert_eq!(cpu.registers().flags.to_byte(), expected);
        assert_eq!(cpu.registers().pc, 0x2800);
        assert!(cpu.latches().q);
        let snapshot = cpu.clone();
        bus.stop = true;
        assert_eq!(cpu.step(&mut bus).exit, Exit::Stop);
        assert_eq!(cpu, snapshot);
        bus.stop = false;
        assert_eq!(cpu.step(&mut bus).cycles, 4);
        assert!(!cpu.latches().q);
        assert_eq!(cpu.registers().flags.to_byte(), expected);
        assert_eq!(cpu.registers().pc, 0x2801);
    }
}

#[test]
fn all_eight_repeat_forms_step_budget_snapshots_resume_to_ordinary_termination() {
    // Boundary exposure is unconditional on interrupts; Budget preserves Q,
    // WZ and the completed flags. Independent literals for both cost paths.
    for (op, first_f, final_f) in [
        (0xB0, 0x2C, 0),
        (0xB8, 0x2C, 0),
        (0xB1, 0x2E, 2),
        (0xB9, 0x2E, 2),
        (0xB2, 0x28, 0x40),
        (0xBA, 0x28, 0x40),
        (0xB3, 0x28, 0x40),
        (0xBB, 0x28, 0x44),
    ] {
        let (mut cpu, mut bus) = setup(0x2800, &[0xED, op]);
        cpu.registers_mut()
            .set_bc(if op & 2 != 0 { 0x0200 } else { 2 });
        cpu.registers_mut().a = if op & 1 != 0 && op & 2 == 0 { 1 } else { 0 };
        // Input offset=1; output first L_after=1 / FFh, data=0.
        bus.inputs.clear();
        bus.inputs.extend([0, 0]);
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 21,
                exit: Exit::Budget
            }
        );
        assert_eq!(cpu.registers().flags.to_byte(), first_f, "ED {op:02X}");
        assert!(cpu.latches().q);
        let mut supplied = Z80::new(*cpu.registers(), *cpu.latches());
        let mut clone = cpu.clone();
        // Same memory is safe: data is zero and transfer/input writes zero.
        for machine in [&mut cpu, &mut supplied, &mut clone] {
            bus.inputs.push_back(0);
            assert_eq!(
                machine.step(&mut bus),
                Report {
                    steps: 1,
                    cycles: 16,
                    exit: Exit::Budget
                }
            );
            assert_eq!(machine.registers().flags.to_byte(), final_f, "ED {op:02X}");
            assert_eq!(machine.registers().pc, 0x2802);
            assert!(machine.latches().q);
        }
        assert_eq!(cpu, supplied);
        assert_eq!(cpu, clone);
    }
}

#[test]
fn ld_a_i_r_retains_iff2_source_and_l_pending_note_acceptance_window() {
    // A ordinary source + manufacturer interrupt note: U pp.94–95; ALPM
    // 03-0002-01 pp.48–49 omits R note. L mapping, not a z80test-confirmed
    // NMOS anomaly: successful next-boundary NMI/INT clears P/V only.
    for opcode in [0x57, 0x5F] {
        for iff2 in [false, true] {
            let (mut cpu, mut bus) = setup(0x1000, &[0xED, opcode]);
            cpu.registers_mut().i = 0x28;
            cpu.registers_mut().r = 0x26;
            cpu.registers_mut().flags = Flags::from_byte(1);
            cpu.latches_mut().iff2 = iff2;
            cpu.step(&mut bus);
            assert_eq!(cpu.registers().a, 0x28);
            assert_eq!(
                cpu.registers().flags.to_byte(),
                if iff2 { 0x2D } else { 0x29 }
            );
            assert!(cpu.latches().parity_note_pending);
        }
        for nmi in [false, true] {
            let (mut cpu, mut bus) = setup(0x1000, &[0xED, opcode, 0]);
            cpu.registers_mut().i = 0x28;
            cpu.registers_mut().r = 0x26;
            cpu.registers_mut().flags = Flags::from_byte(1);
            cpu.latches_mut().iff1 = true;
            cpu.latches_mut().iff2 = true;
            cpu.step(&mut bus);
            let pending = cpu.clone();
            bus.stop = true;
            assert_eq!(cpu.step(&mut bus).exit, Exit::Stop);
            assert_eq!(cpu, pending);
            bus.stop = false;
            assert_eq!(
                cpu.run(
                    &mut bus,
                    Budget {
                        steps: 0,
                        cycles: 1
                    }
                )
                .exit,
                Exit::Budget
            );
            assert_eq!(cpu, pending);
            // Refused INT cannot apply the note or Q=0 entry bookkeeping.
            bus.int = true;
            bus.acknowledges.push_back(0);
            assert!(matches!(cpu.step(&mut bus).exit, Exit::Unsupported(_)));
            assert_eq!(cpu, pending);
            cpu.latches_mut().interrupt_mode = InterruptMode::Mode1;
            bus.nmi = nmi;
            bus.int = !nmi;
            bus.acknowledges.push_back(0);
            cpu.step(&mut bus);
            assert_eq!(cpu.registers().flags.to_byte(), 0x29);
            assert!(!cpu.latches().parity_note_pending);
            assert!(!cpu.latches().q);
        }
        // Ineligible INT preserves P/V while the ordinary NOP expires note;
        // a later NMI cannot apply an already expired correction.
        let (mut cpu, mut bus) = setup(0x1000, &[0xED, opcode, 0]);
        cpu.registers_mut().i = 0x28;
        cpu.registers_mut().r = 0x26;
        cpu.latches_mut().iff2 = true;
        cpu.step(&mut bus);
        bus.int = true;
        cpu.step(&mut bus);
        assert_eq!(cpu.registers().flags.to_byte(), 0x2C);
        assert!(!cpu.latches().parity_note_pending);
        bus.nmi = true;
        cpu.step(&mut bus);
        assert_eq!(cpu.registers().flags.to_byte(), 0x2C);
    }
}

#[test]
fn nmi_during_ei_preserves_iff2_and_pending_delay_no_new_anomaly_admitted() {
    // L decision-15 rules7–8; U Table1 pp.18–19 / EI p.183; TM §8
    // pp.55–56. Banks' NMI page supplies no second chip key and explicitly
    // calls simulator IFF2=0 erroneous. No IFF2-clearing anomaly admitted.
    let (mut cpu, mut bus) = setup(0x1000, &[0xFB, 0]);
    bus.load(0x66, &[0xED, 0x45]); // RETN copies preserved IFF2 to IFF1
    cpu.step(&mut bus);
    assert!(cpu.latches().iff1 && cpu.latches().iff2 && cpu.latches().ei_delay);
    bus.nmi = true;
    bus.int = true;
    assert_eq!(cpu.step(&mut bus).cycles, 11);
    assert!(!cpu.latches().iff1);
    assert!(cpu.latches().iff2 && cpu.latches().ei_delay);
    assert_eq!(cpu.registers().pc, 0x66);
    assert_eq!((bus.memory[0x7FFE], bus.memory[0x7FFF]), (1, 0x10));
    assert_eq!(cpu.step(&mut bus).cycles, 14); // pending delay gates INT
    assert!(cpu.latches().iff1 && cpu.latches().iff2);
    assert!(!cpu.latches().ei_delay);
    assert_eq!(cpu.registers().pc, 0x1001);
    bus.acknowledges.push_back(0xCF);
    assert_eq!(cpu.step(&mut bus).cycles, 13);
    assert_eq!(cpu.registers().pc, 8);
}

#[test]
fn scf_ccf_pattern_a_literal_yx_union_and_old_carry_half_carry() {
    // B8039/H selected pattern A, decision-20 §6; B/S 8039 / R post-CCF.
    // Mix independently supplied A and F bits to distinguish OR from copy,
    // preserve S/Z/PV, and prove CCF H=old C for either predecessor class.
    for (q, a, before, scf, ccf) in [
        (false, 0, 0xFF, 0xED, 0xFC),
        (true, 0, 0xFF, 0xC5, 0xD4),
        (false, 8, 0xE5, 0xED, 0xFC),
        (true, 8, 0xE5, 0xCD, 0xDC),
        (false, 0x20, 0xCD, 0xED, 0xFC),
        (true, 0x20, 0xCD, 0xE5, 0xF4),
        (false, 0x28, 0xD7, 0xED, 0xFC),
        (true, 0x28, 0xD7, 0xED, 0xFC),
        (false, 0, 0xEE, 0xED, 0xED),
        (true, 0, 0xEE, 0xC5, 0xC5),
    ] {
        for (op, expected) in [(0x37, scf), (0x3F, ccf)] {
            let (mut cpu, mut bus) = setup(0x1000, &[op]);
            cpu.registers_mut().a = a;
            cpu.registers_mut().flags = Flags::from_byte(before);
            cpu.latches_mut().q = q;
            cpu.step(&mut bus);
            assert_eq!(cpu.registers().flags.to_byte(), expected);
            assert!(cpu.latches().q);
        }
    }
}

#[test]
fn interrupt_after_each_continuing_repeat_keeps_completed_flags_and_clears_q() {
    // Decision-20 §6: overrides belong to the five repeat states, before
    // interrupt selection, not to acceptance. H observed LDxR/INxR only;
    // L CPxR/OTxR, complete I/O formula and interrupt-entry Q bookkeeping.
    for (op, expected) in [
        (0xB0, 0x2C),
        (0xB8, 0x2C),
        (0xB1, 0x2E),
        (0xB9, 0x2E),
        (0xB2, 0x28),
        (0xBA, 0x28),
        (0xB3, 0x28),
        (0xBB, 0x28),
    ] {
        for nmi in [false, true] {
            let (mut cpu, mut bus) = setup(0x2800, &[0xED, op]);
            cpu.registers_mut()
                .set_bc(if op & 2 == 0 { 2 } else { 0x0200 });
            cpu.registers_mut().a = if op & 3 == 1 { 1 } else { 0 };
            cpu.latches_mut().iff1 = true;
            cpu.latches_mut().iff2 = true;
            cpu.latches_mut().interrupt_mode = InterruptMode::Mode1;
            bus.inputs.clear();
            bus.inputs.push_back(0);
            assert_eq!(cpu.step(&mut bus).cycles, 21);
            assert_eq!(cpu.registers().flags.to_byte(), expected);
            assert!(cpu.latches().q);
            bus.nmi = nmi;
            bus.int = !nmi;
            bus.acknowledges.push_back(0);
            assert_eq!(cpu.step(&mut bus).cycles, if nmi { 11 } else { 13 });
            assert_eq!(cpu.registers().flags.to_byte(), expected);
            assert!(!cpu.latches().q);
            assert_eq!(cpu.registers().pc, if nmi { 0x66 } else { 0x38 });
            assert_eq!((bus.memory[0x7FFE], bus.memory[0x7FFF]), (0, 0x28));
        }
    }
}
