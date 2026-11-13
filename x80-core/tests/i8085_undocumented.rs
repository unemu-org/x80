//! Decision-18 §§2–5 independent literal witnesses; no core decode/ALU helpers.
//!
//! S = documented by second source (CA80C85B), unverified for Intel 8085:
//! Tundra 9000000.MD302.01 (©1996, layout 13 June 1997), Table 2-10
//! p. 2-16/PDF 40, Fig. 2-9 p. 2-15/PDF 39; W = Dehnhardt/Sorensen,
//! Electronics, 18 January 1979, p. 145/PDF 2, with decision-18's rulings.
//! D = die analysis (8085; exact part unverified): Zima decode ROM rows
//! 22/25/23/24/8/11/32/12 (DI/EI = 1); Shirriff FC combined-zero storage.
//! Captures/revisions/hashes are pinned in PROVENANCE §1/decision-18 §1.
//! L = adopted library rule, exact hardware effect unverified: all traces,
//! access kinds, width wraps, unsigned offsets and non-destination preservation.
//! S totals are T-states, including the second-source path interpretation;
//! traces do not claim physical bus order or one call per machine cycle.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Exit, MemoryAccess, Report};

// Independent test-only decoding; no crate PSW conversion supplies expectations.
// S/D image positions: decision-18 §4, T Fig. 2-9, KV/FC bus drivers.
fn flags_from_image(image: u8) -> Flags {
    Flags {
        sign: image & 0x80 != 0,
        zero: image & 0x40 != 0,
        k: image & 0x20 != 0,
        auxiliary_carry: image & 0x10 != 0,
        parity: image & 0x04 != 0,
        overflow: image & 0x02 != 0,
        carry: image & 0x01 != 0,
    }
}

fn fixture(pc: u16, bytes: &[u8], flags: u8) -> (I8085, RecordingBus) {
    let cpu = I8085::new(
        Registers {
            a: 0xA6,
            b: 0x12,
            c: 0x34,
            d: 0x56,
            e: 0x78,
            h: 0x9A,
            l: 0xBC,
            sp: 0x8100,
            pc,
            flags: flags_from_image(flags),
        },
        Latches {
            interrupt_enable: true,
            ei_delay: true,
            rst7_5_pending: true,
            sod: true,
            trap_saved_interrupt_enable: true,
            trap_rim_pending: true,
            ..Latches::default()
        },
    );
    let mut bus = RecordingBus::new();
    bus.load(pc, bytes);
    (cpu, bus)
}

// L boundary polls and fetch mapping (decision-12, decision-18 §3).
fn fetch(pc: u16, opcode: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::TrapPoll(false),
        Event::Rst7_5Poll(false),
        Event::Rst6_5Poll(false),
        Event::Rst5_5Poll(false),
        Event::IntrPoll(false),
        read(pc, MemoryAccess::OpcodeFetch, opcode),
    ]
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

fn completes(cpu: &mut I8085, bus: &mut RecordingBus, states: u64, mut expected: I8085) {
    // L ordinary retirement expires the EI delay, never interrupt entry (§5).
    expected.latches_mut().ei_delay = false;
    assert_eq!(
        cpu.step(bus),
        Report {
            steps: 1,
            cycles: states,
            exit: Exit::Budget
        }
    );
    assert_eq!(*cpu, expected);
}

#[test]
fn dsub_word_result_and_all_seven_literal_flags() {
    // S DSUB/10 states/all-seven mask/V/K (silicon-corrected K); D/S whole
    // Z; L high-byte S/P/AC and final borrow CY, ignoring incoming CY (§4).
    // 0100-00FF discriminates combined Z and high-byte parity; 1000-0001
    // discriminates propagated low borrow in AC. 0000-FF01 widens B+borrow.
    for (hl, bc, result, image) in [
        (0x1234, 0x0102, 0x1132, 0x14),
        (0x0100, 0x0001, 0x00FF, 0x14),
        (0x0100, 0x00FF, 0x0001, 0x14),
        (0x1000, 0x0001, 0x0FFF, 0x04),
        (0x0000, 0x0001, 0xFFFF, 0xA5),
        (0x0000, 0xFF01, 0x00FF, 0x05),
        (0x0000, 0x0000, 0x0000, 0x54),
        (0x1234, 0x1234, 0x0000, 0x54),
        (0x8000, 0x8000, 0x0000, 0x54),
        (0x8000, 0x0001, 0x7FFF, 0x22),
        (0x7FFF, 0xFFFF, 0x8000, 0x93),
        (0x8000, 0x7FFF, 0x0001, 0x26),
        (0x7F00, 0xFF01, 0x7FFF, 0x01),
    ] {
        for incoming in [0x00, 0xF7] {
            let (mut cpu, mut bus) = fixture(0xFFFF, &[0x08], incoming);
            cpu.registers_mut().set_hl(hl);
            cpu.registers_mut().set_bc(bc);
            let memory = bus.memory.clone();
            let mut expected = cpu.clone();
            expected.registers_mut().set_hl(result);
            expected.registers_mut().flags = flags_from_image(image);
            expected.registers_mut().pc = 0; // L derived width wrap (§2).
            completes(&mut cpu, &mut bus, 10, expected);
            assert_eq!(bus.take_log(), fetch(0xFFFF, 0x08));
            assert_eq!(bus.memory, memory);
        }
    }
}

#[test]
fn arhl_sign_fill_low_bit_carry_cleared_v_and_preserved_other_flags() {
    // S operation/7 states/CY and preservation; L V=0 die-derived override
    // (decision-18 §4, ALU store_v_flag/right-shift constant carries).
    for (hl, result, carry) in [
        (0x8001, 0xC000, 1),
        (0x0001, 0x0000, 1),
        (0x0002, 0x0001, 0),
        (0xFFFF, 0xFFFF, 1),
        (0x7F00, 0x3F80, 0),
        (0x0100, 0x0080, 0),
    ] {
        for incoming in (0..=0xF7).filter(|image| image & 8 == 0) {
            let (mut cpu, mut bus) = fixture(0xFFFF, &[0x10], incoming);
            cpu.registers_mut().set_hl(hl);
            let memory = bus.memory.clone();
            let mut expected = cpu.clone();
            expected.registers_mut().set_hl(result);
            expected.registers_mut().flags = flags_from_image((incoming & 0xF4) | carry);
            expected.registers_mut().pc = 0; // L wrap.
            completes(&mut cpu, &mut bus, 7, expected);
            assert_eq!(bus.take_log(), fetch(0xFFFF, 0x10));
            assert_eq!(bus.memory, memory);
        }
    }
}

#[test]
fn rdel_carry_in_out_and_word_overflow_preserve_k_and_documented_latches() {
    // S operation/10 states/CY/V and K/S/Z/AC/P preservation (§§2/4).
    // V is overflow of DE+DE+old CY, including the propagated low carry.
    for (de, carry_in, result, cv) in [
        (0x0000, 0, 0x0000, 0),
        (0x0000, 1, 0x0001, 0),
        (0x00FF, 1, 0x01FF, 0),
        (0x3FFF, 1, 0x7FFF, 0),
        (0x4000, 0, 0x8000, 2),
        (0x7FFF, 1, 0xFFFF, 2),
        (0x8000, 0, 0x0000, 3),
        (0xBFFF, 1, 0x7FFF, 3),
        (0xC000, 0, 0x8000, 1),
        (0xFFFF, 1, 0xFFFF, 1),
    ] {
        for preserved in (0..=0xF4).filter(|image| image & 0x0B == 0) {
            for old_v in [0, 2] {
                let (mut cpu, mut bus) = fixture(0xFFFF, &[0x18], preserved | old_v | carry_in);
                cpu.registers_mut().set_de(de);
                let memory = bus.memory.clone();
                let mut expected = cpu.clone();
                expected.registers_mut().set_de(result);
                expected.registers_mut().flags = flags_from_image(preserved | cv);
                expected.registers_mut().pc = 0; // L wrap.
                completes(&mut cpu, &mut bus, 10, expected);
                assert_eq!(bus.take_log(), fetch(0xFFFF, 0x18));
                assert_eq!(bus.memory, memory);
            }
        }
    }
}

#[test]
fn ldhi_ldsi_unsigned_byte_two_and_wrapped_sum_preserve_all_flags_and_base() {
    // S base+byte -> DE/2 bytes/10 states/no flags; D additionally CY
    // preservation (FC temporary latch). L unsigned 80h/FFh, sum/PC wrap
    // and OperandFetch; exact hardware edge behavior unverified (§§2–4).
    for opcode in [0x28, 0x38] {
        for (base, offset, result) in [
            (0x12FF, 0x01, 0x1300),
            (0x1200, 0x80, 0x1280),
            (0x1200, 0xFF, 0x12FF),
            (0xFFFF, 0x01, 0x0000),
            (0xFF80, 0xFF, 0x007F),
            (0x1234, 0x00, 0x1234),
        ] {
            for flags in [0, 0xF7, 0x22, 0x80] {
                let (mut cpu, mut bus) = fixture(0xFFFF, &[opcode, offset], flags);
                if opcode == 0x28 {
                    cpu.registers_mut().set_hl(base);
                } else {
                    cpu.registers_mut().sp = base;
                }
                let memory = bus.memory.clone();
                let mut expected = cpu.clone();
                expected.registers_mut().set_de(result);
                expected.registers_mut().pc = 1;
                completes(&mut cpu, &mut bus, 10, expected);
                let mut trace = fetch(0xFFFF, opcode);
                trace.push(read(0, MemoryAccess::OperandFetch, offset));
                assert_eq!(bus.take_log(), trace);
                assert_eq!(bus.memory, memory);
            }
        }
    }
}

#[test]
fn shlx_lhlx_low_before_high_data_even_at_sp_and_wrapped_address() {
    // S placements/DE preservation/one byte/10 states/no flags (§§2/4).
    // L Data order/kind/exact calls (§3), L address and PC wraps (§2).
    for opcode in [0xD9, 0xED] {
        for (de, high_at) in [(0x8100, 0x8101), (0xFFFF, 0)] {
            for flags in [0, 0xF7, 0x22, 0x80] {
                let (mut cpu, mut bus) = fixture(0x23FF, &[opcode], flags);
                cpu.registers_mut().set_de(de);
                cpu.registers_mut().sp = de; // Numeric equality must not imply Stack.
                cpu.registers_mut().set_hl(0xB64D);
                bus.load(de, &[0x72, 0xC3]);
                let mut memory = bus.memory.clone();
                let mut expected = cpu.clone();
                expected.registers_mut().pc = 0x2400;
                let mut trace = fetch(0x23FF, opcode);
                if opcode == 0xD9 {
                    trace.extend([
                        write(de, MemoryAccess::Data, 0x4D),
                        write(high_at, MemoryAccess::Data, 0xB6),
                    ]);
                    memory[usize::from(de)] = 0x4D;
                    memory[usize::from(high_at)] = 0xB6;
                } else {
                    trace.extend([
                        read(de, MemoryAccess::Data, 0x72),
                        read(high_at, MemoryAccess::Data, 0xC3),
                    ]);
                    expected.registers_mut().set_hl(0xC372);
                }
                completes(&mut cpu, &mut bus, 10, expected);
                assert_eq!(bus.take_log(), trace);
                assert_eq!(bus.memory, memory);
            }
        }
        // L PC wrap without data/code aliasing.
        let (mut cpu, mut bus) = fixture(0xFFFF, &[opcode], 0xF7);
        bus.load(0x5678, &[0x4D, 0xB6]);
        let mut expected = cpu.clone();
        expected.registers_mut().pc = 0;
        let mut trace = fetch(0xFFFF, opcode);
        if opcode == 0xED {
            expected.registers_mut().set_hl(0xB64D);
            trace.extend([
                read(0x5678, MemoryAccess::Data, 0x4D),
                read(0x5679, MemoryAccess::Data, 0xB6),
            ]);
        } else {
            trace.extend([
                write(0x5678, MemoryAccess::Data, 0xBC),
                write(0x5679, MemoryAccess::Data, 0x9A),
            ]);
        }
        completes(&mut cpu, &mut bus, 10, expected);
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn rstv_both_paths_next_pc_stack_wrap_vector_and_ordinary_latch_retirement() {
    // S V condition/0040h/1 byte/6 or 12 states/no flags (§§2/4).
    // L saved next PC, high-before-low Stack, no acknowledge/dummy calls,
    // INTE/source-latch preservation and ordinary EI-delay retirement (§§3/5).
    for (pc, next, sp, high_at, low_at) in [
        (0x23FF, 0x2400u16, 0x8100, 0x80FF, 0x80FE),
        (0xFFFF, 0x0000, 0x0000, 0xFFFF, 0xFFFE),
        (0x2345, 0x2346, 0x0001, 0x0000, 0xFFFF),
    ] {
        for (flags, taken) in [(0x00, false), (0xF5, false), (0x02, true), (0xF7, true)] {
            let (mut cpu, mut bus) = fixture(pc, &[0xCB], flags);
            cpu.registers_mut().sp = sp;
            let mut memory = bus.memory.clone();
            let mut expected = cpu.clone();
            let mut trace = fetch(pc, 0xCB);
            if taken {
                let [high, low] = next.to_be_bytes();
                trace.extend([
                    write(high_at, MemoryAccess::Stack, high),
                    write(low_at, MemoryAccess::Stack, low),
                ]);
                memory[usize::from(high_at)] = high;
                memory[usize::from(low_at)] = low;
                expected.registers_mut().pc = 0x0040;
                expected.registers_mut().sp = low_at;
            } else {
                expected.registers_mut().pc = next;
            }
            completes(&mut cpu, &mut bus, if taken { 12 } else { 6 }, expected);
            assert_eq!(bus.take_log(), trace);
            assert_eq!(bus.memory, memory);
        }
    }
}

#[test]
fn jnk_jk_both_polarities_only_low_operand_when_untaken_with_pc_wrap() {
    // S K polarity (not S or V), length 3, totals 7/10 and preserved flags.
    // L taken reads both address bytes; untaken consumes byte 2 only and
    // skips byte 3, even over FFFFh (§§2–4). Literal traces pin every path.
    for opcode in [0xDD, 0xFD] {
        for (flags, jnk, jk) in [
            (0x00, true, false),
            (0xA2, false, true),
            (0x82, true, false),
            (0x20, false, true),
        ] {
            let taken = if opcode == 0xDD { jnk } else { jk };
            for (pc, low_at, high_at, next) in [
                (0x23FF, 0x2400, 0x2401, 0x2402),
                (0xFFFE, 0xFFFF, 0x0000, 0x0001),
                (0xFFFF, 0x0000, 0x0001, 0x0002),
            ] {
                let (mut cpu, mut bus) = fixture(pc, &[opcode, 0x4D, 0xB6], flags);
                let memory = bus.memory.clone();
                let mut expected = cpu.clone();
                expected.registers_mut().pc = if taken { 0xB64D } else { next };
                let mut trace = fetch(pc, opcode);
                trace.push(read(low_at, MemoryAccess::OperandFetch, 0x4D));
                if taken {
                    trace.push(read(high_at, MemoryAccess::OperandFetch, 0xB6));
                }
                completes(&mut cpu, &mut bus, if taken { 10 } else { 7 }, expected);
                assert_eq!(bus.take_log(), trace);
                assert_eq!(bus.memory, memory);
            }
        }
    }
}
