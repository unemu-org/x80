//! Independent exchange, block-transfer and search witnesses.
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), Exchange, Block
//! Transfer, and Search Group, pp. 124–143 (PDF = printed + 14).
//! Second witness: 03-0029-01 (1977), Table 7.0-3 p. 46 (PDF 52).
//! All encoded facts were read on rendered page images; hashes match
//! PROVENANCE §1. These related manuals are not hardware measurements.
//! Design: decision-13 rules 2–5, decision-14 rules 5–6, decision-15
//! rules 1/5/6/11. No flags in this group are printed unknown; compare
//! all eight bits (decision-20 §4 H: Young §4.2 p. 16 / Rak block groups).
//! UM LDDR p. 137 omits C; its preservation is explicit in TM p. 46.
//! Transfer P/V follows the decremented count on every iteration: stated
//! for LDIR (UM p. 133), derived for intermediate LDDR iterations from
//! LDIR/LDD. LDDR's reset and TM's zero entries describe completion;
//! supervisor ruling accepted by the maintainer (2026-11-13), PROVENANCE §1.
//!
//! MEASURED fail-before on base 5f5f6e8: all 18 tests fail at their first
//! retirement assertion with Unsupported, 0 steps and 0 cycles, because the
//! group is not landed. Refusal precedes data/stack access. Expectations
//! below never call executor or pair/flag helpers.
//! MEASURED correction fail-before: the LDIR, LDDR, mid-block budget and
//! repeating-transfer wrap tests fail against the pre-fix executor because
//! an intermediate iteration clears P/V instead of setting it (14 pass,
//! 4 fail). Source readings and corrected ruling are recorded above.
//!
//! EX (SP) trace order is an UNVERIFIED library choice: read SP, read SP+1,
//! write SP+1, write SP. UM pp. 127–129 and TM p. 46 specify the byte
//! mapping, but do not establish that temporal order. All four accesses
//! are Stack under decision-15 rule 5 and the group brief.
//! All 16-bit wraps below are DERIVED from register/address width and the
//! entries' +/-1 or PC-2 operations, not separately measured hardware facts.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, MemoryAccess, Report};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const DATA: MemoryAccess = MemoryAccess::Data;
const STACK: MemoryAccess = MemoryAccess::Stack;

// Independent Table 21 p. 65 transcription, not Flags::from_byte.
fn flags(image: u8) -> Flags {
    Flags {
        sign: image & 0x80 != 0,
        zero: image & 0x40 != 0,
        y: image & 0x20 != 0,
        x: image & 0x08 != 0,
        half_carry: image & 0x10 != 0,
        parity_overflow: image & 0x04 != 0,
        subtract: image & 0x02 != 0,
        carry: image & 0x01 != 0,
    }
}

fn initial() -> Registers {
    Registers {
        a: 0x90,
        flags: flags(0xD7),
        b: 0x12,
        c: 0x34,
        d: 0x56,
        e: 0x78,
        h: 0x9A,
        l: 0xBC,
        a_alternate: 0x81,
        flags_alternate: flags(0x42),
        b_alternate: 0x23,
        c_alternate: 0x45,
        d_alternate: 0x67,
        e_alternate: 0x89,
        h_alternate: 0xAB,
        l_alternate: 0xCD,
        ix: 0x1357,
        iy: 0x2468,
        sp: 0xDEF0,
        pc: 0x2000,
        i: 0xF8,
        r: 0xFF,
    }
}

fn latches() -> Latches {
    Latches {
        iff1: true,
        iff2: true,
        interrupt_mode: InterruptMode::Mode2,
        ..Latches::default()
    }
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

fn write(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Write {
        address,
        kind,
        value,
    }
}

fn report(steps: u64, cycles: u64) -> Report {
    Report {
        steps,
        cycles,
        exit: Exit::Budget,
    }
}

fn trace(fetches: &[(u16, u8)], accesses: &[Event]) -> Vec<Event> {
    // Format the caller's literal expectations only: include every poll
    // and fetch, followed by every expected access, without filtering the
    // actual bus log or deriving addresses/values from CPU results.
    let mut events = vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ];
    events.extend(
        fetches
            .iter()
            .map(|&(address, value)| read(address, OP, value)),
    );
    events.extend_from_slice(accesses);
    events
}

fn witness(
    before: Registers,
    fetches: &[(u16, u8)],
    seed: &[(u16, u8)],
    after: Registers,
    cycles: u64,
    accesses: &[Event],
    wz: u16,
) {
    let mut bus = RecordingBus::new();
    for &(address, value) in fetches.iter().chain(seed) {
        bus.memory[usize::from(address)] = value;
    }
    let mut memory = bus.memory.clone();
    for &access in accesses {
        if let Event::Write { address, value, .. } = access {
            memory[usize::from(address)] = value;
        }
    }
    let mut cpu = Z80::new(before, latches());
    assert_eq!(cpu.step(&mut bus), report(1, cycles), "{fetches:02X?}");
    assert_eq!(*cpu.registers(), after, "{fetches:02X?}");
    assert_eq!(*cpu.latches(), Latches { wz, ..latches() });
    // Decision-20 §4: full F/F' compared with Registers above.
    assert_eq!(bus.memory, memory, "unexpected memory effect");
    assert_eq!(bus.take_log(), trace(fetches, accesses));
}

#[test]
fn ex_de_hl() {
    // UM p. 124; TM Table 7.0-3 p. 46: EB, 4, no flag change.
    for image in [0, 0xD7] {
        let mut before = initial();
        before.flags = flags(image);
        let mut after = before;
        after.d = 0x9A;
        after.e = 0xBC;
        after.h = 0x56;
        after.l = 0x78;
        after.pc = 0x2001;
        after.r = 0x80;
        witness(before, &[(0x2000, 0xEB)], &[], after, 4, &[], 0);
    }
}

#[test]
fn ex_af_af_alternate() {
    // UM p. 125; TM p. 46: 08, 4. Swap the stored flags as well as A.
    for (active, alternate) in [(0xD7, 0), (0, 0xD7), (0x95, 0x42)] {
        let mut before = initial();
        before.flags = flags(active);
        before.flags_alternate = flags(alternate);
        let mut after = before;
        after.a = 0x81;
        after.a_alternate = 0x90;
        after.flags = flags(alternate);
        after.flags_alternate = flags(active);
        after.pc = 0x2001;
        after.r = 0x80;
        witness(before, &[(0x2000, 0x08)], &[], after, 4, &[], 0);
    }
}

#[test]
fn exx_exchanges_only_bc_de_hl_banks() {
    // UM p. 126; TM p. 46: D9, 4; AF/AF', indexes and SP untouched.
    for (image, alternate) in [(0, 0xD7), (0xD7, 0)] {
        let mut before = initial();
        before.flags = flags(image);
        // Exercise preservation of both flag banks from both polarities;
        // distinct images also expose an accidental AF/AF' exchange.
        before.flags_alternate = flags(alternate);
        let mut after = before;
        after.b = 0x23;
        after.c = 0x45;
        after.d = 0x67;
        after.e = 0x89;
        after.h = 0xAB;
        after.l = 0xCD;
        after.b_alternate = 0x12;
        after.c_alternate = 0x34;
        after.d_alternate = 0x56;
        after.e_alternate = 0x78;
        after.h_alternate = 0x9A;
        after.l_alternate = 0xBC;
        after.pc = 0x2001;
        after.r = 0x80;
        witness(before, &[(0x2000, 0xD9)], &[], after, 4, &[], 0);
    }
}

#[test]
fn ex_sp_hl_uses_stack_and_keeps_sp() {
    // UM p. 127; TM p. 46: E3, 19. Full ordered trace pins the
    // unverified choice documented above; both flag polarities survive.
    for image in [0, 0xD7] {
        let mut before = initial();
        before.flags = flags(image);
        let mut after = before;
        after.h = 0x22;
        after.l = 0x11;
        after.pc = 0x2001;
        after.r = 0x80;
        witness(
            before,
            &[(0x2000, 0xE3)],
            &[(0xDEF0, 0x11), (0xDEF1, 0x22)],
            after,
            19,
            &[
                read(0xDEF0, STACK, 0x11),
                read(0xDEF1, STACK, 0x22),
                write(0xDEF1, STACK, 0x9A),
                write(0xDEF0, STACK, 0xBC),
            ],
            0x2211,
        );
    }
}

#[test]
fn ex_sp_ix_and_iy_select_the_correct_pair() {
    // UM pp. 128/129; TM p. 46: DD/FD E3, 23, no flag changes.
    for (prefix, low, high) in [(0xDD, 0x57, 0x13), (0xFD, 0x68, 0x24)] {
        for image in [0, 0xD7] {
            let mut before = initial();
            before.flags = flags(image);
            let mut after = before;
            if prefix == 0xDD {
                after.ix = 0x2211;
            } else {
                after.iy = 0x2211;
            }
            after.pc = 0x2002;
            after.r = 0x81;
            witness(
                before,
                &[(0x2000, prefix), (0x2001, 0xE3)],
                &[(0xDEF0, 0x11), (0xDEF1, 0x22)],
                after,
                23,
                &[
                    read(0xDEF0, STACK, 0x11),
                    read(0xDEF1, STACK, 0x22),
                    write(0xDEF1, STACK, high),
                    write(0xDEF0, STACK, low),
                ],
                0x2211,
            );
        }
    }
}

#[test]
fn ex_sp_all_forms_wrap_sp_plus_one_and_opcode_fetches() {
    // UM pp. 127–129; TM p. 46. DERIVED: SP+1 and PC wrap at 16 bits.
    for (prefix, low, high, cycles) in [
        (None, 0xBC, 0x9A, 19),
        (Some(0xDD), 0x57, 0x13, 23),
        (Some(0xFD), 0x68, 0x24, 23),
    ] {
        let mut before = initial();
        before.sp = 0xFFFF;
        before.pc = 0xFFFE;
        // For prefixed forms the stack low byte aliases the fetched E3.
        // Writes occur only after both instruction bytes have been fetched.
        let fetches = if let Some(prefix) = prefix {
            vec![(0xFFFE, prefix), (0xFFFF, 0xE3)]
        } else {
            vec![(0xFFFE, 0xE3)]
        };
        let stack_low = if prefix.is_some() { 0xE3 } else { 0x11 };
        let mut after = before;
        match prefix {
            None => {
                after.h = 0x22;
                after.l = 0x11;
                after.pc = 0xFFFF;
                after.r = 0x80;
            }
            Some(0xDD) => {
                after.ix = 0x22E3;
                after.pc = 0;
                after.r = 0x81;
            }
            _ => {
                after.iy = 0x22E3;
                after.pc = 0;
                after.r = 0x81;
            }
        }
        witness(
            before,
            &fetches,
            &[(0xFFFF, stack_low), (0, 0x22)],
            after,
            cycles,
            &[
                read(0xFFFF, STACK, stack_low),
                read(0, STACK, 0x22),
                write(0, STACK, high),
                write(0xFFFF, STACK, low),
            ],
            if prefix.is_some() { 0x22E3 } else { 0x2211 },
        );
    }
}

fn single_transfer(opcode: u8, next_e: u8, next_l: u8) {
    // UM LDI p. 130 / LDD p. 134; TM Table 7.0-3 p. 46.
    // Literal input/output flag rows exercise S/Z/C preservation and both
    // H/N/PV polarities. A transfer always happens, including BC=0.
    for (count, next_b, next_c, flag_rows) in [
        (
            2_u16,
            0,
            1,
            &[(0x00, 0x04), (0xD7, 0xC5), (0x95, 0x85), (0x42, 0x44)][..],
        ),
        (
            1,
            0,
            0,
            &[(0x00, 0x00), (0xD7, 0xC1), (0x95, 0x81), (0x42, 0x40)][..],
        ),
        // DERIVED: 0000h-1 = FFFFh; 0100h-1 = 00FFh.
        (0, 0xFF, 0xFF, &[(0x00, 0x04), (0xD7, 0xC5)][..]),
        (0x0100, 0, 0xFF, &[(0x00, 0x04), (0xD7, 0xC5)][..]),
    ] {
        for &(input, output) in flag_rows {
            let mut before = initial();
            [before.b, before.c] = count.to_be_bytes();
            before.flags = flags(input);
            let mut after = before;
            after.b = next_b;
            after.c = next_c;
            after.e = next_e;
            after.l = next_l;
            after.flags = flags(output | 8); // §4 H: 90h+88h=18h, X=1.
            after.pc = 0x2002;
            after.r = 0x81;
            witness(
                before,
                &[(0x2000, 0xED), (0x2001, opcode)],
                &[(0x9ABC, 0x88), (0x5678, 0x66)],
                after,
                16,
                &[read(0x9ABC, DATA, 0x88), write(0x5678, DATA, 0x88)],
                0,
            );
        }
    }
}

#[test]
fn ldi_one_transfer_and_counter_flags() {
    single_transfer(0xA0, 0x79, 0xBD);
}

#[test]
fn ldd_one_transfer_and_counter_flags() {
    single_transfer(0xA8, 0x77, 0xBB);
}

fn searches(opcode: u8, next_l: u8, repeating: bool) {
    // UM CPI p. 138, CPIR pp. 139–140, CPD p. 141, CPDR pp. 142–143;
    // TM Table 7.0-3 p. 46. Result is A-(HL), with H from bit-4 borrow;
    // P/V is the decremented count, not parity or arithmetic overflow.
    // Last two fields are literal flag images with C=0 / C=1.
    for (a, value, count, next_b, next_c, repeats, clear_c, set_c) in [
        (0x90, 0x01, 2_u16, 0, 1, true, 0x96, 0x97),
        (0x90, 0x01, 1, 0, 0, false, 0x92, 0x93), // exhaustion, no match
        (0x90, 0x90, 2, 0, 1, false, 0x46, 0x47), // early match, count remains
        (0x90, 0x90, 1, 0, 0, false, 0x42, 0x43), // both stop conditions
        (0x80, 0x01, 1, 0, 0, false, 0x12, 0x13), // overflow is not P/V
        (0x00, 0x80, 2, 0, 1, true, 0x86, 0x87),  // negative, no half borrow
        (0x31, 0x10, 2, 0, 1, true, 0x06, 0x07),  // positive, no half borrow
        (0x00, 0x01, 0, 0xFF, 0xFF, true, 0x96, 0x97), // DERIVED BC wrap
        (0x90, 0x90, 0, 0xFF, 0xFF, false, 0x46, 0x47), // match after wrap
        (0x90, 0x90, 0x0100, 0, 0xFF, false, 0x46, 0x47),
    ] {
        for (input, output) in [(0, clear_c), (0xD7, set_c)] {
            let mut before = initial();
            before.a = a;
            [before.b, before.c] = count.to_be_bytes();
            before.flags = flags(input);
            let mut after = before;
            after.b = next_b;
            after.c = next_c;
            after.l = next_l;
            // Decision-20 §4 H: Young §4.2 p. 16 / Rak .cpi… .cpdr.
            let t = a
                .wrapping_sub(value)
                .wrapping_sub(u8::from(output & 0x10 != 0));
            after.flags = flags(output | ((t & 2) << 4) | (t & 8));
            let repeats = repeating && repeats;
            after.pc = if repeats { 0x2000 } else { 0x2002 };
            after.r = 0x81;
            witness(
                before,
                &[(0x2000, 0xED), (0x2001, opcode)],
                &[(0x9ABC, value)],
                after,
                if repeats { 21 } else { 16 },
                &[read(0x9ABC, DATA, value)],
                if repeats {
                    0x2001
                } else if opcode & 8 == 0 {
                    1
                } else {
                    0xFFFF
                },
            );
        }
    }
}

#[test]
fn cpi_compare_flags_and_count_are_independent() {
    searches(0xA1, 0xBD, false);
}

#[test]
fn cpd_compare_flags_and_count_are_independent() {
    searches(0xA9, 0xBB, false);
}

#[test]
fn cpir_repeats_or_stops_on_either_match_or_exhaustion() {
    // Pins PROVENANCE §1's existing conjunction-heading disagreement:
    // UM p. 139 prose OR, timing heading AND; TM p. 46 timing OR.
    searches(0xB1, 0xBD, true);
}

#[test]
fn cpdr_repeats_or_stops_on_either_match_or_exhaustion() {
    // Same existing heading disagreement for UM p. 142 / TM p. 46.
    // Also pins the newly recorded repeat-sentence equality misprint:
    // UM p. 142's termination sentence and 21-state heading, and TM p. 46,
    // agree on stopping for a match and repeating only without a match.
    searches(0xB9, 0xBB, true);
}

#[test]
fn single_transfers_wrap_both_data_pointers() {
    // UM LDI p. 130, LDD p. 134; TM p. 46. DERIVED 16-bit pointer wraps.
    for (opcode, address, next_h, next_l, next_d, next_e) in [
        (0xA0, 0xFFFF_u16, 0, 0, 0, 0),
        (0xA8, 0, 0xFF, 0xFF, 0xFF, 0xFF),
    ] {
        let mut before = initial();
        before.b = 0;
        before.c = 1;
        [before.h, before.l] = address.to_be_bytes();
        [before.d, before.e] = address.to_be_bytes();
        let mut after = before;
        after.c = 0;
        after.h = next_h;
        after.l = next_l;
        after.d = next_d;
        after.e = next_e;
        after.flags = flags(0xC1);
        after.pc = 0x2002;
        after.r = 0x81;
        witness(
            before,
            &[(0x2000, 0xED), (0x2001, opcode)],
            &[(address, 0xA5)],
            after,
            16,
            &[read(address, DATA, 0xA5), write(address, DATA, 0xA5)],
            0,
        );
    }
}

fn repeating_transfer(opcode: u8, next_e: u8, next_l: u8) {
    // UM LDIR pp. 132–133, LDDR pp. 136–137; TM p. 46.
    // P/V follows BC after decrement: STATED for LDIR on UM p. 133;
    // DERIVED for intermediate LDDR iterations from LDIR/LDD (p. 134).
    // UM LDDR p. 137's reset and TM's zero entries describe completion
    // (maintainer-accepted ruling, 2026-11-13; PROVENANCE §1).
    for (count, next_b, next_c, pc, cycles) in [
        (2_u16, 0, 1, 0x2000, 21),
        (1, 0, 0, 0x2002, 16),
        (0, 0xFF, 0xFF, 0x2000, 21), // DERIVED BC wrap
        (0x0100, 0, 0xFF, 0x2000, 21),
    ] {
        for (input, repeating_output, final_output) in [
            (0x00, 0x04, 0x00),
            (0xD7, 0xC5, 0xC1),
            (0x95, 0x85, 0x81),
            (0x42, 0x44, 0x40),
        ] {
            let mut before = initial();
            [before.b, before.c] = count.to_be_bytes();
            before.flags = flags(input);
            let mut after = before;
            after.b = next_b;
            after.c = next_c;
            after.e = next_e;
            after.l = next_l;
            after.flags = flags(if count == 1 {
                final_output | 8
            } else {
                repeating_output | 8
            });
            after.pc = pc;
            after.r = 0x81;
            witness(
                before,
                &[(0x2000, 0xED), (0x2001, opcode)],
                &[(0x9ABC, 0x88), (0x5678, 0x66)],
                after,
                cycles,
                &[read(0x9ABC, DATA, 0x88), write(0x5678, DATA, 0x88)],
                if count == 1 { 0 } else { 0x2001 },
            );
        }
    }

    // Step the same BC=2 instruction twice: P/V=1 at BC=1, then P/V=0
    // at BC=0. Preserve all other state and pin each complete bus trace.
    let mut before = initial();
    before.b = 0;
    before.c = 2;
    before.flags = flags(0xD3); // P/V starts clear, H/N start set.
    let mut cpu = Z80::new(before, latches());
    let mut bus = RecordingBus::new();
    bus.load(0x2000, &[0xED, opcode]);
    bus.memory[0x9ABC] = 0x88;
    let source2 = u16::from_be_bytes([0x9A, next_l]);
    let destination2 = u16::from_be_bytes([0x56, next_e]);
    bus.memory[usize::from(source2)] = 0x36;
    let mut memory = bus.memory.clone();
    let (final_e, final_l) = if opcode == 0xB0 {
        (0x7A, 0xBE)
    } else {
        (0x76, 0xBA)
    };
    for (count, e, l, image, pc, r, cycles, source, destination, value) in [
        (
            1, next_e, next_l, 0xCD, 0x2000, 0x81, 21, 0x9ABC, 0x5678, 0x88,
        ),
        (
            0,
            final_e,
            final_l,
            0xE1,
            0x2002,
            0x83,
            16,
            source2,
            destination2,
            0x36,
        ),
    ] {
        assert_eq!(cpu.step(&mut bus), report(1, cycles));
        let mut after = before;
        after.c = count;
        after.e = e;
        after.l = l;
        after.flags = flags(image);
        after.pc = pc;
        after.r = r;
        assert_eq!(*cpu.registers(), after);
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: 0x2001,
                ..latches()
            }
        );
        memory[usize::from(destination)] = value;
        assert_eq!(bus.memory, memory);
        assert_eq!(
            bus.take_log(),
            trace(
                &[(0x2000, 0xED), (0x2001, opcode)],
                &[read(source, DATA, value), write(destination, DATA, value)],
            )
        );
    }
}

#[test]
fn ldir_sets_pv_until_final_iteration() {
    repeating_transfer(0xB0, 0x79, 0xBD);
}

#[test]
fn lddr_sets_pv_until_final_iteration_derived() {
    repeating_transfer(0xB8, 0x77, 0xBB);
}

#[test]
fn every_repeating_form_resumes_after_a_mid_block_step_budget() {
    // UM LDIR pp. 132–133, LDDR pp. 136–137, CPIR pp. 139–140,
    // CPDR pp. 142–143; TM p. 46. decision-15 rule 11: one iteration
    // per step, repoll and refetch both bytes, 21+21 then 16 on resume.
    for (
        opcode,
        transfer,
        source2,
        source3,
        source_end,
        destination2,
        destination3,
        destination_end,
    ) in [
        (
            0xB0, true, 0x9ABD_u16, 0x9ABE_u16, 0xBF, 0x5679, 0x567A_u16, 0x7B,
        ),
        (0xB8, true, 0x9ABB, 0x9ABA, 0xB9, 0x5677, 0x5676, 0x75),
        (0xB1, false, 0x9ABD, 0x9ABE, 0xBF, 0x5679, 0x567A, 0x78),
        (0xB9, false, 0x9ABB, 0x9ABA, 0xB9, 0x5677, 0x5676, 0x78),
    ] {
        let mut before = initial();
        before.b = 0;
        before.c = 3;
        let mut cpu = Z80::new(before, latches());
        let mut bus = RecordingBus::new();
        bus.load(0x2000, &[0xED, opcode]);
        let values = if transfer {
            [0x88, 0x36, 0xA5]
        } else {
            [0x01, 0x11, 0x90]
        };
        for (address, value) in [
            (0x9ABC, values[0]),
            (source2, values[1]),
            (source3, values[2]),
        ] {
            bus.memory[usize::from(address)] = value;
        }
        let mut memory = bus.memory.clone();
        let first = if transfer {
            vec![read(0x9ABC, DATA, 0x88), write(0x5678, DATA, 0x88)]
        } else {
            vec![read(0x9ABC, DATA, 0x01)]
        };
        let second = if transfer {
            vec![read(source2, DATA, 0x36), write(destination2, DATA, 0x36)]
        } else {
            vec![read(source2, DATA, 0x11)]
        };
        if transfer {
            memory[0x5678] = 0x88;
            memory[usize::from(destination2)] = 0x36;
        }
        let budget = Budget {
            steps: 2,
            cycles: u64::MAX,
        };
        assert_eq!(cpu.run(&mut bus, budget), report(2, 42));
        let mut middle = before;
        middle.c = 1;
        [middle.h, middle.l] = source3.to_be_bytes();
        if transfer {
            [middle.d, middle.e] = destination3.to_be_bytes();
        }
        middle.flags = flags(if transfer { 0xE5 } else { 0x3F }); // §4 H t-bit rule.
        middle.r = 0x83;
        assert_eq!(*cpu.registers(), middle);
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: 0x2001,
                ..latches()
            }
        );
        assert_eq!(bus.memory, memory);
        let mut events = trace(&[(0x2000, 0xED), (0x2001, opcode)], &first);
        events.extend(trace(&[(0x2000, 0xED), (0x2001, opcode)], &second));
        assert_eq!(bus.take_log(), events);

        assert_eq!(
            cpu.run(
                &mut bus,
                Budget {
                    steps: 1,
                    cycles: u64::MAX
                }
            ),
            report(1, 16)
        );
        let mut after = middle;
        after.c = 0;
        after.l = source_end;
        after.e = destination_end;
        after.flags = flags(if transfer { 0xC1 } else { 0x43 });
        after.pc = 0x2002;
        after.r = 0x85;
        assert_eq!(*cpu.registers(), after);
        assert_eq!(
            *cpu.latches(),
            Latches {
                wz: if transfer {
                    0x2001
                } else if opcode == 0xB1 {
                    0x2002
                } else {
                    0x2000
                },
                ..latches()
            }
        );
        let third = if transfer {
            memory[usize::from(destination3)] = 0xA5;
            vec![read(source3, DATA, 0xA5), write(destination3, DATA, 0xA5)]
        } else {
            vec![read(source3, DATA, 0x90)]
        };
        assert_eq!(bus.memory, memory);
        assert_eq!(
            bus.take_log(),
            trace(&[(0x2000, 0xED), (0x2001, opcode)], &third)
        );
    }
}

#[test]
fn repeating_transfers_wrap_pointers_and_rewind_pc_across_zero() {
    // UM pp. 132–133/136–137; TM p. 46; decision-15 rule 11.
    // DERIVED: fetch at FFFF/0000, rewind 0001-2=FFFF, pointer +/-1.
    for (opcode, address, next) in [(0xB0, 0xFFFF_u16, 0_u16), (0xB8, 0, 0xFFFF)] {
        let mut before = initial();
        before.pc = 0xFFFF;
        before.b = 0;
        before.c = 2;
        [before.h, before.l] = address.to_be_bytes();
        [before.d, before.e] = address.to_be_bytes();
        let value = if address == 0xFFFF { 0xED } else { 0xB8 };
        let mut after = before;
        after.c = 1;
        [after.h, after.l] = next.to_be_bytes();
        [after.d, after.e] = next.to_be_bytes();
        after.flags = flags(0xCD); // Decision-20 §4: t=7D/48, X set.
        after.r = 0x81;
        witness(
            before,
            &[(0xFFFF, 0xED), (0, opcode)],
            &[],
            after,
            21,
            &[read(address, DATA, value), write(address, DATA, value)],
            0,
        );
    }
}

#[test]
fn all_search_forms_wrap_hl_and_pc() {
    // UM pp. 138–143; TM p. 46; decision-15 rule 11. DERIVED HL and
    // post-fetch PC wrap, including PC-2 back across zero on repetition.
    for (opcode, address, next_h, next_l, pc, cycles) in [
        (0xA1, 0xFFFF_u16, 0, 0, 1, 16),
        (0xA9, 0, 0xFF, 0xFF, 1, 16),
        (0xB1, 0xFFFF, 0, 0, 0xFFFF, 21),
        (0xB9, 0, 0xFF, 0xFF, 0xFFFF, 21),
    ] {
        let mut before = initial();
        before.pc = 0xFFFF;
        before.a = 0;
        before.b = 0;
        before.c = 2;
        [before.h, before.l] = address.to_be_bytes();
        // The data read aliases either ED or the terminal operation byte.
        let value = if address == 0xFFFF { 0xED } else { opcode };
        let mut after = before;
        after.c = 1;
        after.h = next_h;
        after.l = next_l;
        // 00-ED = 13h; 00-A9 = 57h; 00-B9 = 47h: H,PV,N,C set.
        after.flags = flags(0x37); // Decision-20 §4: t=12/56/46, Y set.
        after.pc = pc;
        after.r = 0x81;
        witness(
            before,
            &[(0xFFFF, 0xED), (0, opcode)],
            &[],
            after,
            cycles,
            &[read(address, DATA, value)],
            if opcode == 0xA1 {
                1
            } else if opcode == 0xA9 {
                0xFFFF
            } else {
                0
            },
        );
    }
}
