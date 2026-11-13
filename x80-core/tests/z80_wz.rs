//! Decision-20 §5 complete L exact-latch table, §§4/7 and decision-15 state.
//! Written key: pinned Boo-boo/Kladov MEMPTR (©2006), instruction headings
//! named by each witness. R = Rak z80test v1.2a; only WZ13/11 projection
//! observations are H (Young v0.91 §4.1 p. 15; R idea.asm 124–127).
//! Every expected word/flag below is literal, independent of CPU helpers.
//! Exact words, wrap edges, public state/reset and interrupt extensions are
//! L library rules, not whole-word silicon measurements. In particular,
//! continuing INxR follows the accepted replacement, never M's falsified rule.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, MemoryAccess};

fn fixture(bytes: &[u8]) -> (Z80, RecordingBus) {
    let mut bus = RecordingBus::new();
    bus.load(0x1000, bytes);
    bus.inputs.extend([0xBF; 4]);
    let cpu = Z80::new(
        Registers {
            pc: 0x1000,
            sp: 0x8000,
            a: 0x28,
            b: 2,
            c: 0xFF,
            d: 0x50,
            h: 0x40,
            ix: 0x27FF,
            iy: 0x2801,
            ..Registers::default()
        },
        Latches {
            wz: 0x1357,
            ..Latches::default()
        },
    );
    (cpu, bus)
}

/// Observe the latch twice through BIT (HL), independently of HL, memory
/// Y/X and old flags. This runs after every table-row class below.
fn observe(cpu: &mut Z80, bus: &mut RecordingBus, word: u16, yx: u8) {
    assert_eq!(cpu.latches().wz, word);
    bus.load(0x6000, &[0xCB, 0x46, 0xCB, 0x46]);
    bus.memory[0x7000] = 0x28;
    cpu.registers_mut().pc = 0x6000;
    cpu.registers_mut().set_hl(0x7000);
    cpu.registers_mut().flags = Flags::from_byte(0x29);
    cpu.latches_mut().iff1 = false;
    cpu.latches_mut().ei_delay = false;
    cpu.latches_mut().halted = false;
    bus.nmi = false;
    bus.int = false;
    for _ in 0..2 {
        assert_eq!(cpu.step(bus).exit, Exit::Budget);
        assert_eq!(cpu.registers().flags.to_byte(), 0x55 | yx);
        assert_eq!(cpu.latches().wz, word);
    }
}

fn retire(cpu: &mut Z80, bus: &mut RecordingBus, cycles: u64) {
    let report = cpu.step(bus);
    assert_eq!(
        (report.steps, report.cycles, report.exit),
        (1, cycles, Exit::Budget)
    );
}

#[test]
fn accumulator_reads_address_plus_one_including_ffff() {
    // §5 row 1; M LD A,(addr)/(rp); R .ld_a_mem/.ld_a_rr page limit.
    for (bytes, address, word, yx, cycles) in [
        (&[0x3A, 0xFF, 0x27][..], 0x27FF, 0x2800, 0x28, 13),
        (&[0x3A, 0xFF, 0xFF], 0xFFFF, 0x0000, 0, 13),
        (&[0x0A], 0x27FF, 0x2800, 0x28, 7),
        (&[0x1A], 0xFFFF, 0x0000, 0, 7),
    ] {
        let (mut cpu, mut bus) = fixture(bytes);
        cpu.registers_mut().set_bc(address);
        cpu.registers_mut().set_de(address);
        bus.memory[usize::from(address)] = 0xA5;
        retire(&mut cpu, &mut bus, cycles);
        assert_eq!(cpu.registers().a, 0xA5);
        observe(&mut cpu, &mut bus, word, yx);
    }
}

#[test]
fn accumulator_stores_route_a_high_and_suppress_low_byte_carry() {
    // §5 row 2; M LD (addr)/(rp),A; R .ld_mem_a/.ld_rr_a insufficient.
    for (bytes, address, a, word, yx, cycles) in [
        (&[0x32, 0xFF, 0x27][..], 0x27FF, 0x28, 0x2800, 0x28, 13),
        (&[0x32, 0xFF, 0xFF], 0xFFFF, 0x07, 0x0700, 0, 13),
        (&[0x02], 0xFFFF, 0x27, 0x2700, 0x20, 7),
        (&[0x12], 0x08FF, 0x08, 0x0800, 8, 7),
    ] {
        let (mut cpu, mut bus) = fixture(bytes);
        cpu.registers_mut().a = a;
        cpu.registers_mut().set_bc(address);
        cpu.registers_mut().set_de(address);
        retire(&mut cpu, &mut bus, cycles);
        assert_eq!(bus.memory[usize::from(address)], a);
        observe(&mut cpu, &mut bus, word, yx);
    }
}

#[test]
fn pair_memory_all_pairs_aliases_and_indexes_use_nn_plus_one() {
    // §5 row 3; M LD rp,(addr)/LD (addr),rp; R .ld_*_mem/.ld_mem_*.
    for (head, cycles) in [
        (&[0x2A][..], 16),
        (&[0x22], 16),
        (&[0xED, 0x4B], 20),
        (&[0xED, 0x5B], 20),
        (&[0xED, 0x6B], 20),
        (&[0xED, 0x7B], 20),
        (&[0xED, 0x43], 20),
        (&[0xED, 0x53], 20),
        (&[0xED, 0x63], 20),
        (&[0xED, 0x73], 20),
        (&[0xDD, 0x2A], 20),
        (&[0xFD, 0x2A], 20),
        (&[0xDD, 0x22], 20),
        (&[0xFD, 0x22], 20),
    ] {
        for (tail, word, yx) in [([0xFF, 0x27], 0x2800, 0x28), ([0xFF, 0xFF], 0, 0)] {
            let bytes = [head, &tail].concat();
            let (mut cpu, mut bus) = fixture(&bytes);
            retire(&mut cpu, &mut bus, cycles);
            observe(&mut cpu, &mut bus, word, yx);
        }
    }
}

#[test]
fn stack_exchange_uses_new_stack_word_with_sp_wrap() {
    // §5 row 4; M EX (SP),rp; R .ex_sp_hl/.ex_sp_xy limited patterns.
    for (bytes, cycles) in [(&[0xE3][..], 19), (&[0xDD, 0xE3], 23), (&[0xFD, 0xE3], 23)] {
        let (mut cpu, mut bus) = fixture(bytes);
        cpu.registers_mut().sp = 0xFFFF;
        bus.memory[0xFFFF] = 0xA5;
        bus.memory[0] = 0x28;
        retire(&mut cpu, &mut bus, cycles);
        observe(&mut cpu, &mut bus, 0x28A5, 0x28);
    }
}

#[test]
fn pair_arithmetic_uses_old_destination_plus_one_not_result_or_old_high() {
    // §5 row 5; M ADD/ADC/SBC rp1,rp2; R arithmetic carry discrimination.
    for (bytes, cycles) in [
        (&[0x09][..], 11),
        (&[0x29], 11),
        (&[0xDD, 0x09], 15),
        (&[0xFD, 0x29], 15),
        (&[0xED, 0x4A], 15),
        (&[0xED, 0x42], 15),
    ] {
        for (old, word, yx) in [(0x27FF, 0x2800, 0x28), (0xFFFF, 0, 0)] {
            let (mut cpu, mut bus) = fixture(bytes);
            cpu.registers_mut().set_hl(old);
            cpu.registers_mut().ix = old;
            cpu.registers_mut().iy = old;
            cpu.registers_mut().set_bc(0x1234);
            retire(&mut cpu, &mut bus, cycles);
            observe(&mut cpu, &mut bus, word, yx);
        }
    }
}

#[test]
fn nibble_rotates_use_hl_plus_one() {
    // §5 row 6; M RLD/RRD; R .rld/.rrd has no +1 carry stress.
    for op in [0x67, 0x6F] {
        for (hl, word, yx) in [(0x27FF, 0x2800, 0x28), (0xFFFF, 0, 0)] {
            let (mut cpu, mut bus) = fixture(&[0xED, op]);
            cpu.registers_mut().set_hl(hl);
            retire(&mut cpu, &mut bus, 18);
            observe(&mut cpu, &mut bus, word, yx);
        }
    }
}

#[test]
fn taken_relative_return_aliases_and_restart_use_destination_pc() {
    // §5 row 7; M JR/DJNZ/RET/RETI/RST; R has no RST capture.
    for (bytes, pc, b, stack, target, yx, cycles) in [
        (&[0x18, 0xFE][..], 0x2800, 2, 0, 0x2800, 0x28, 12),
        (&[0x20, 0xFE], 0x0800, 2, 0, 0x0800, 8, 12),
        (&[0x10, 0xFE], 0x2000, 2, 0, 0x2000, 0x20, 13),
        (&[0x18, 0xFD], 0xFFFF, 2, 0, 0xFFFE, 0x28, 12),
        (&[0xC9], 0x1000, 2, 0x28A5, 0x28A5, 0x28, 10),
        (&[0xC0], 0x1000, 2, 0x08A5, 0x08A5, 8, 11),
        (&[0xFF], 0x1000, 2, 0, 0x0038, 0, 11),
    ] {
        let (mut cpu, mut bus) = fixture(&[]);
        cpu.registers_mut().pc = pc;
        cpu.registers_mut().b = b;
        bus.load(pc, bytes);
        bus.load(0x8000, &u16::to_le_bytes(stack));
        retire(&mut cpu, &mut bus, cycles);
        assert_eq!(cpu.registers().pc, target);
        observe(&mut cpu, &mut bus, target, yx);
    }
    for op in [0x45, 0x4D, 0x55, 0x5D, 0x65, 0x6D, 0x75, 0x7D] {
        let (mut cpu, mut bus) = fixture(&[0xED, op]);
        bus.load(0x8000, &[0xA5, 0x28]);
        retire(&mut cpu, &mut bus, 14);
        observe(&mut cpu, &mut bus, 0x28A5, 0x28);
    }
}

#[test]
fn immediate_jump_and_call_set_target_even_untaken() {
    // §5 row 8; M JP/CALL; R fixed targets cannot confirm all word bits.
    for (op, flags, pc, cycles) in [
        (0xC3, 0, 0x28A5, 10),
        (0xC2, 0, 0x28A5, 10),
        (0xC2, 0x40, 0x1003, 10),
        (0xCD, 0, 0x28A5, 17),
        (0xC4, 0, 0x28A5, 17),
        (0xC4, 0x40, 0x1003, 10),
    ] {
        let (mut cpu, mut bus) = fixture(&[op, 0xA5, 0x28]);
        cpu.registers_mut().flags = Flags::from_byte(flags);
        retire(&mut cpu, &mut bus, cycles);
        assert_eq!(cpu.registers().pc, pc);
        observe(&mut cpu, &mut bus, 0x28A5, 0x28);
    }
}

#[test]
fn immediate_input_uses_old_a_full_port_plus_one() {
    // §5 row 9; M IN A,(n); R .in_a_n only n=FEh.
    for (a, word, yx) in [(0x27, 0x2800, 0x28), (0xFF, 0, 0)] {
        let (mut cpu, mut bus) = fixture(&[0xDB, 0xFF]);
        cpu.registers_mut().a = a;
        retire(&mut cpu, &mut bus, 11);
        assert_eq!(cpu.registers().a, 0xBF);
        observe(&mut cpu, &mut bus, word, yx);
    }
}

#[test]
fn register_io_all_codes_uses_bc_before_input_replaces_b_or_c() {
    // §5 row 10; M names IN A,(C)/OUT (C),A; other codes L completion.
    for op in [
        0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78, 0x41, 0x49, 0x51, 0x59, 0x61, 0x69, 0x71,
        0x79,
    ] {
        for (bc, word, yx) in [(0x27FF, 0x2800, 0x28), (0xFFFF, 0, 0)] {
            let (mut cpu, mut bus) = fixture(&[0xED, op]);
            cpu.registers_mut().set_bc(bc);
            retire(&mut cpu, &mut bus, 12);
            observe(&mut cpu, &mut bus, word, yx);
        }
    }
}

#[test]
fn immediate_output_keeps_a_high_without_carry() {
    // §5 row 11; M OUT (n),A; R .out_n_a never n=FFh.
    for (a, word, yx) in [(0x27, 0x2700, 0x20), (0xFF, 0xFF00, 0x28)] {
        let (mut cpu, mut bus) = fixture(&[0xD3, 0xFF]);
        cpu.registers_mut().a = a;
        retire(&mut cpu, &mut bus, 11);
        observe(&mut cpu, &mut bus, word, yx);
    }
}

#[test]
fn continuing_transfer_uses_this_ed_plus_one_and_terminating_preserves() {
    // §5 rows 12/13; M LDIR/LDDR; R .ldir/.lddr/.ldir_nop/.lddr_nop.
    for op in [0xB0, 0xB8] {
        for (pc, bytes, word, yx) in [
            (0x27FF, &[0xED, op][..], 0x2800, 0x28),
            (0xFFFF, &[0xED, op], 0, 0),
            (0x27FD, &[0xDD, 0xFD, 0xED, op], 0x2800, 0x28),
        ] {
            let (mut cpu, mut bus) = fixture(&[]);
            cpu.registers_mut().pc = pc;
            cpu.registers_mut().set_bc(2);
            bus.load(pc, bytes);
            retire(&mut cpu, &mut bus, if bytes.len() == 4 { 29 } else { 21 });
            assert_eq!(cpu.latches().wz, word);
            // Termination refetches ED only, without the leading selectors.
            retire(&mut cpu, &mut bus, 16);
            observe(&mut cpu, &mut bus, word, yx);
        }
        let (mut cpu, mut bus) = fixture(&[0xED, op]);
        cpu.registers_mut().set_bc(1);
        cpu.latches_mut().wz = 0x28FF;
        retire(&mut cpu, &mut bus, 16);
        observe(&mut cpu, &mut bus, 0x28FF, 0x28);
    }
}

#[test]
fn compare_and_terminating_repeat_adjust_old_wz_with_wrap() {
    // §5 row 14; M CPI/CPD/CPIR/CPDR; R is not full 14-bit recovery.
    for (op, old, word, yx) in [
        (0xA1, 0x27FF, 0x2800, 0x28),
        (0xA1, 0xFFFF, 0, 0),
        (0xA9, 0x2800, 0x27FF, 0x20),
        (0xA9, 0, 0xFFFF, 0x28),
        (0xB1, 0x27FF, 0x2800, 0x28),
        (0xB9, 0x2800, 0x27FF, 0x20),
    ] {
        let (mut cpu, mut bus) = fixture(&[0xED, op]);
        cpu.registers_mut().set_bc(1);
        cpu.latches_mut().wz = old;
        retire(&mut cpu, &mut bus, 16);
        observe(&mut cpu, &mut bus, word, yx);
    }
    // A match terminates even with nonzero decremented BC.
    for (op, word, yx) in [(0xB1, 0x2800, 0x28), (0xB9, 0x27FE, 0x20)] {
        let (mut cpu, mut bus) = fixture(&[0xED, op]);
        cpu.latches_mut().wz = 0x27FF;
        bus.memory[0x4000] = 0x28;
        retire(&mut cpu, &mut bus, 16);
        observe(&mut cpu, &mut bus, word, yx);
    }
}

#[test]
fn continuing_compare_overrides_old_wz_delta_with_ed_plus_one() {
    // §5 row 15; M CPIR/CPDR; R .cpir/.cpdr differential projections.
    for op in [0xB1, 0xB9] {
        for (pc, word, yx) in [(0x27FF, 0x2800, 0x28), (0xFFFF, 0, 0)] {
            let (mut cpu, mut bus) = fixture(&[]);
            cpu.registers_mut().pc = pc;
            cpu.registers_mut().set_bc(2);
            bus.load(pc, &[0xED, op]);
            retire(&mut cpu, &mut bus, 21);
            observe(&mut cpu, &mut bus, word, yx);
        }
    }
}

#[test]
fn input_blocks_and_terminating_repeats_use_bc_before_decrement() {
    // §5 row 16; M INI/IND/INIR/INDR; R final groups hide prior WZ.
    for (op, bc, word, yx) in [
        (0xA2, 0x27FF, 0x2800, 0x28),
        (0xA2, 0xFFFF, 0, 0),
        (0xAA, 0x2800, 0x27FF, 0x20),
        (0xAA, 0, 0xFFFF, 0x28),
        (0xB2, 0x01FF, 0x0200, 0),
        (0xBA, 0x0100, 0x00FF, 0),
    ] {
        let (mut cpu, mut bus) = fixture(&[0xED, op]);
        cpu.registers_mut().set_bc(bc);
        retire(&mut cpu, &mut bus, 16);
        observe(&mut cpu, &mut bus, word, yx);
    }
}

#[test]
fn continuing_input_replacement_uses_ed_plus_one_not_memptr_bc_rule() {
    // §5 row 17 REPLACEMENT; R .inir_nop/.indr_nop F3B1BE2F vs
    // falsified M equation 0A537B63; whole word remains L.
    for op in [0xB2, 0xBA] {
        for (pc, bytes, word, yx) in [
            (0x27FF, &[0xED, op][..], 0x2800, 0x28),
            (0xFFFF, &[0xED, op], 0, 0),
            (0x27FD, &[0xDD, 0xFD, 0xED, op], 0x2800, 0x28),
        ] {
            let (mut cpu, mut bus) = fixture(&[]);
            cpu.registers_mut().pc = pc;
            cpu.registers_mut().set_bc(0x0234);
            bus.load(pc, bytes);
            retire(&mut cpu, &mut bus, if bytes.len() == 4 { 29 } else { 21 });
            observe(&mut cpu, &mut bus, word, yx);
        }
    }
}

#[test]
fn output_blocks_use_decremented_bc_on_every_iteration() {
    // §5 row 18; M OUTI/OUTD/OTIR/OTDR; R .otir/.otdr cannot see
    // continuing PC override because final iteration overwrites the latch.
    for (op, bc, cycles, word, yx) in [
        (0xA3, 0x28FF, 16, 0x2800, 0x28),
        (0xA3, 0x00FF, 16, 0, 0),
        (0xAB, 0x2900, 16, 0x27FF, 0x20),
        (0xAB, 0x0100, 16, 0xFFFF, 0x28),
        (0xB3, 0x28FF, 21, 0x2800, 0x28),
        (0xBB, 0x2900, 21, 0x27FF, 0x20),
        (0xB3, 0x01FF, 16, 0x0100, 0),
        (0xBB, 0x0100, 16, 0xFFFF, 0x28),
    ] {
        let (mut cpu, mut bus) = fixture(&[0xED, op]);
        cpu.registers_mut().set_bc(bc);
        retire(&mut cpu, &mut bus, cycles);
        observe(&mut cpu, &mut bus, word, yx);
    }
}

#[test]
fn every_indexed_memory_class_sets_effective_address_before_bit() {
    // §5 row 19; M indexed-address heading; R page-local indexed groups.
    for prefix in [0xDD, 0xFD] {
        for tail in [
            &[0x46, 1][..],
            &[0x66, 1],
            &[0x74, 1],
            &[0x36, 1, 0xA5],
            &[0x34, 1],
            &[0x35, 1],
            &[0x86, 1],
            &[0xCB, 1, 0x00],
            &[0xCB, 1, 0x46],
            &[0xCB, 1, 0x80],
            &[0xCB, 1, 0xC0],
        ] {
            let bytes = [&[prefix], tail].concat();
            let (mut cpu, mut bus) = fixture(&bytes);
            cpu.registers_mut().ix = 0x27FF;
            cpu.registers_mut().iy = 0x27FF;
            assert_eq!(cpu.step(&mut bus).exit, Exit::Budget);
            observe(&mut cpu, &mut bus, 0x2800, 0x28);
        }
        // Signed displacement wraps in both directions.
        for (base, d, word, yx) in [(0xFFFF, 1, 0, 0), (0, 0xFF, 0xFFFF, 0x28)] {
            let (mut cpu, mut bus) = fixture(&[prefix, 0xCB, d, 0x40]);
            cpu.registers_mut().ix = base;
            cpu.registers_mut().iy = base;
            retire(&mut cpu, &mut bus, 20);
            observe(&mut cpu, &mut bus, word, yx);
        }
    }
}

#[test]
fn successful_interrupts_set_actual_handler_destination() {
    // §5 row 20; M interrupt CALL; NMI/IM0/1/2 completion L, R no vectors.
    for (nmi, mode, ack, target, yx, cycles) in [
        (true, InterruptMode::Mode0, 0, 0x0066, 0, 11),
        (false, InterruptMode::Mode0, 0xEF, 0x0028, 0, 13),
        (false, InterruptMode::Mode1, 0, 0x0038, 0, 13),
        (false, InterruptMode::Mode2, 0xFE, 0x28A5, 0x28, 19),
    ] {
        let (mut cpu, mut bus) = fixture(&[]);
        cpu.latches_mut().iff1 = true;
        cpu.latches_mut().interrupt_mode = mode;
        cpu.registers_mut().i = 0xFF;
        bus.nmi = nmi;
        bus.int = !nmi;
        bus.acknowledges.push_back(ack);
        bus.load(0xFFFE, &[0xA5, 0x28]);
        retire(&mut cpu, &mut bus, cycles);
        assert_eq!(cpu.registers().pc, target);
        observe(&mut cpu, &mut bus, target, yx);
    }
}

#[test]
fn unlisted_forms_and_untaken_relative_returns_preserve_wz() {
    // §5 preservation sentence and M exclusions; L full-path extension.
    for bytes in [
        &[0][..],
        &[0xDD, 0],
        &[0xFD, 0xED, 0],
        &[0xED, 0xA0],
        &[0xED, 0xA8],
        &[0xC1],
        &[0xF1],
        &[0xDD, 0xE1],
        &[0xEB],
        &[0xD9],
        &[0xF9],
        &[0xDD, 0xF9],
        &[0xFD, 0xF9],
        &[0x03],
        &[0x0B],
        &[0xDD, 0x23],
        &[0xFD, 0x2B],
        &[0xE9],
        &[0xDD, 0xE9],
        &[0xFD, 0xE9],
        &[0x28, 0x7F],
        &[0x10, 0x7F],
        &[0xC8],
        &[0x21, 0, 0x28],
        &[0xDD, 0x21, 0, 0x28],
        &[0xCB, 0x46],
        &[0xCB, 0x86],
        &[0xCB, 6],
    ] {
        let (mut cpu, mut bus) = fixture(bytes);
        cpu.latches_mut().wz = 0x28A5;
        cpu.registers_mut().b = 1; // DJNZ untaken.
        assert_eq!(cpu.step(&mut bus).exit, Exit::Budget);
        observe(&mut cpu, &mut bus, 0x28A5, 0x28);
    }
}

#[test]
fn default_reset_budget_stop_refusal_and_snapshot_continuation() {
    // §5/§7 state contract: L public/default/reset/transaction policy,
    // UM008011-0816 RESET p. 6 / TM 03-0029-01 RESET p. 9 omit WZ.
    assert_eq!(Z80::default().latches().wz, 0);
    let (mut cpu, mut bus) = fixture(&[0x09, 0xCB, 0x46]);
    cpu.latches_mut().wz = 0x28A5;
    cpu.reset();
    assert_eq!(cpu.latches().wz, 0x28A5);
    cpu.registers_mut().pc = 0x1000;
    let before = cpu.clone();
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
    assert_eq!(cpu, before);
    assert!(bus.log.is_empty());
    bus.stop = true;
    assert_eq!(cpu.step(&mut bus).exit, Exit::Stop);
    assert_eq!(cpu, before);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(true),
            Event::NmiPoll(false),
            Event::IntPoll(false)
        ]
    );
    bus.stop = false;
    for (mode, ack) in [(InterruptMode::Mode0, 0), (InterruptMode::Mode2, 1)] {
        cpu.latches_mut().iff1 = true;
        cpu.latches_mut().interrupt_mode = mode;
        let before = cpu.clone();
        bus.int = true;
        bus.acknowledges.push_back(ack);
        let report = cpu.step(&mut bus);
        assert!(matches!(report.exit, Exit::Unsupported(_)));
        assert_eq!((report.steps, report.cycles), (0, 0));
        assert_eq!(cpu, before);
        assert_eq!(
            bus.take_log(),
            [
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(true),
                Event::Read {
                    address: 0x1000,
                    kind: MemoryAccess::InterruptAcknowledge,
                    value: ack
                },
            ]
        );
    }
    bus.int = false;
    cpu.latches_mut().iff1 = false;
    cpu.registers_mut().set_hl(0x27FF);
    retire(&mut cpu, &mut bus, 11);
    assert_eq!(cpu.latches().wz, 0x2800);
    let mut restored = Z80::new(*cpu.registers(), *cpu.latches());
    let mut cloned = cpu.clone();
    let mut other_bus = RecordingBus::new();
    other_bus.memory.clone_from(&bus.memory);
    let mut clone_bus = RecordingBus::new();
    clone_bus.memory.clone_from(&bus.memory);
    bus.take_log();
    retire(&mut cpu, &mut bus, 12);
    retire(&mut restored, &mut other_bus, 12);
    retire(&mut cloned, &mut clone_bus, 12);
    assert_eq!(cpu, restored);
    assert_eq!(cpu, cloned);
    assert_eq!(bus.log, other_bus.log);
    assert_eq!(bus.log, clone_bus.log);
    assert_eq!(cpu.registers().flags.to_byte() & 0x28, 0x28);
}
