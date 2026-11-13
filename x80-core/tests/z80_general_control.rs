//! General arithmetic witnesses: decisions 13–15. Zilog UM008011-0816,
//! rev. 11 (August 2016), CPL p. 175, NEG pp. 176–177, CCF p. 178,
//! SCF p. 179 (PDF = printed + 14); 03-0029-01 (1977), Table 7.0-5
//! p. 48 (PDF 54). Read on rendered images; CCF rechecked at 200 dpi.
//! All instruction witnesses fail before landing with Unsupported, zero
//! steps/cycles on the historical base. Decision-20 §1 now admits NEG aliases.
//! No implementation ALU is used.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

fn cpu_at(a: u8, flags: u8) -> Z80 {
    Z80::new(
        Registers {
            a,
            flags: Flags::from_byte(flags),
            b: 0x12,
            c: 0x34,
            d: 0x56,
            e: 0x78,
            h: 0x9A,
            l: 0xBC,
            a_alternate: 0xDE,
            flags_alternate: Flags::from_byte(0x42),
            b_alternate: 0xF0,
            c_alternate: 0x13,
            d_alternate: 0x25,
            e_alternate: 0x37,
            h_alternate: 0x49,
            l_alternate: 0x5B,
            ix: 0x6789,
            iy: 0xABCD,
            sp: 0xDEF0,
            pc: 0xFFFF,
            i: 0xF8,
            r: 0xFF,
        },
        Latches {
            iff1: true,
            iff2: false,
            interrupt_mode: InterruptMode::Mode2,
            ei_delay: true,
            halted: false,
            nmi_asserted: true,
            parity_note_pending: true,
        },
    )
}

fn check(bytes: &[u8], a: u8, flags: u8, result: u8, result_flags: u8) {
    let mut cpu = cpu_at(a, flags);
    let mut expected = cpu.clone();
    // DERIVED PC wrap from its 16-bit width (UM p. 3). One/two M1s
    // preserve R bit 7 and wrap its low seven bits (decision-15 rules 5–6).
    let (pc, r, cycles) = if bytes.len() == 1 {
        (0, 0x80, 4)
    } else {
        (1, 0x81, 8)
    };
    expected.registers_mut().pc = pc;
    expected.registers_mut().r = r;
    expected.registers_mut().a = result;
    expected.registers_mut().flags = Flags::from_byte(result_flags);
    expected.latches_mut().ei_delay = false;
    expected.latches_mut().nmi_asserted = false;
    expected.latches_mut().parity_note_pending = false;
    let mut bus = RecordingBus::new();
    bus.load(0xFFFF, bytes);
    let memory = bus.memory.clone();
    let mut trace = vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ];
    for (&value, offset) in bytes.iter().zip(0u16..) {
        trace.push(Event::Read {
            address: 0xFFFFu16.wrapping_add(offset),
            kind: MemoryAccess::OpcodeFetch,
            value,
        });
    }
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles,
            exit: Exit::Budget
        }
    );
    assert_eq!(cpu, expected, "{bytes:02X?}, A={a:02X}, F={flags:02X}");
    assert_eq!(cpu.registers().flags.to_byte() & 0xD7, result_flags);
    // Separate library claim: F bits 5/3 are zero (decision-14 rule 5).
    assert_eq!(cpu.registers().flags.to_byte() & 0x28, 0);
    assert_eq!(bus.memory, memory);
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn cpl_sets_h_and_n_preserving_s_z_pv_c() {
    // UM p. 175 example A=B4 -> 4B; TM p. 48. Literal F expectations
    // exercise each changed flag from both values and every preserved flag.
    for (a, result) in [(0xB4, 0x4B), (0x00, 0xFF), (0xFF, 0x00)] {
        for (before, after) in [(0x00, 0x12), (0xD7, 0xD7), (0x85, 0x97), (0x52, 0x52)] {
            check(&[0x2F], a, before, result, after);
        }
    }
}

#[test]
fn ccf_copies_previous_carry_to_h_despite_secondary_unknown_cell() {
    // UM p. 178 explicitly copies old C to H; TM Table 7.0-5 p. 48
    // prints H=X. Reuse PROVENANCE §1's H-definedness resolution;
    // decision-14 rule 6 keeps H in the D7h comparison mask.
    for (before, after) in [
        (0x00, 0x01),
        (0x12, 0x01),
        (0x01, 0x10),
        (0x13, 0x10),
        (0xC6, 0xC5),
        (0xD7, 0xD4),
        (0x85, 0x94),
        (0x52, 0x41),
    ] {
        check(&[0x3F], 0xA5, before, 0xA5, after);
    }
}

#[test]
fn scf_sets_c_resets_h_n_and_preserves_s_z_pv() {
    // UM p. 179; TM p. 48. Old C never enters another flag.
    for (before, after) in [
        (0x00, 0x01),
        (0xD7, 0xC5),
        (0x12, 0x01),
        (0x85, 0x85),
        (0x52, 0x41),
    ] {
        check(&[0x37], 0x5A, before, 0x5A, after);
    }
}

#[test]
fn neg_literal_zero_borrow_overflow_and_manual_example() {
    // UM p. 176: P/V iff old A=80h, C iff old A!=00h, H is borrow
    // from bit 4; p. 177 example 98h -> 68h. TM p. 48: A <- 0-A.
    // Include nonzero results with even parity but no overflow (02,98).
    for (a, result, flags) in [
        (0x00, 0x00, 0x42),
        (0x01, 0xFF, 0x93),
        (0x02, 0xFE, 0x93),
        (0x0F, 0xF1, 0x93),
        (0x10, 0xF0, 0x83),
        (0x7F, 0x81, 0x93),
        (0x80, 0x80, 0x87),
        (0x81, 0x7F, 0x13),
        (0x98, 0x68, 0x13),
        (0xF0, 0x10, 0x03),
        (0xFF, 0x01, 0x13),
    ] {
        for before in [0x00, 0xD7] {
            check(&[0xED, 0x44], a, before, result, flags);
        }
    }
}

// Decision-20 §1 replaces the former NEG-alias refusals with H operations
// and L 8-state alias costs (Young v0.91 §3.4 pp. 12–13 / §8.5 p. 30;
// Rak .neg_ lines 389–394); literal witnesses: z80_undocumented_ed.rs.
#[test]
fn undocumented_neg_aliases_execute_the_literal_neg_case() {
    // H operation/L alias cost: decision-20 §1; Young v0.91 §3.4
    // pp. 12–13 / §8.5 p. 30; Rak .neg_ lines 389–394.
    for opcode in [0x4C, 0x54, 0x5C, 0x64, 0x6C, 0x74, 0x7C] {
        check(&[0xED, opcode], 0x80, 0xD7, 0x80, 0x87);
    }
}
