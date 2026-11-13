//! Jump Group witnesses: decision-13 rules 2–5; decisions 14–15.
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), Jump Group
//! pp. 262–279 (PDF = printed + 14). Second witness: 03-0029-01 (1977),
//! Table 7.0-9 p. 52 (PDF 58). Encodings, flags and costs were read on
//! rendered pages; hashes match PROVENANCE.md §1. The sources share lineage.
//! Expectations below are literal transcriptions or test-side condition
//! tables, independent of the executor and its opcode map.
//!
//! Poll/access kinds are API choices (decision-15 rules 1/5). R accounting:
//! rule 6, UM Registers p. 3; TM §2 p. 4 and §3 p. 7. PC wraps are derived
//! from its 16-bit width (UM p. 2), B wrap from its 8-bit width (UM pp. 2–3).
//! All eight flag bits are unaffected, so no unknown flags need masking. Every
//! test varies all 64 stored flag images and checks the complete CPU state,
//! memory and ordered bus trace. Decision-20 §4 H preservation: Young
//! §§8.9–8.10 pp. 33–34 / Rak flow groups; RST universal extension is L.
//!
//! Fail-before: with jump::LANDED false on the supplied base, all nine
//! witnesses report Unsupported, retire zero steps/cycles and stop before
//! operand reads (prefix/key reads only for DD/FD E9).

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ARG: MemoryAccess = MemoryAccess::OperandFetch;

fn flag_images() -> impl Iterator<Item = u8> {
    0..=255
}

fn registers(pc: u16, image: u8) -> Registers {
    Registers {
        a: 0xA1,
        flags: Flags::from_byte(image),
        b: 0xB2,
        c: 0xC3,
        d: 0xD4,
        e: 0xE5,
        h: 0xF6,
        l: 0x07,
        a_alternate: 0x18,
        flags_alternate: Flags::from_byte(0x42),
        b_alternate: 0x29,
        c_alternate: 0x3A,
        d_alternate: 0x4B,
        e_alternate: 0x5C,
        h_alternate: 0x6D,
        l_alternate: 0x7E,
        ix: 0x8091,
        iy: 0x92A3,
        sp: 0xDEF0,
        pc,
        i: 0xF8,
        r: 0xFF,
    }
}

fn latches() -> Latches {
    Latches {
        q: false,
        wz: 0,
        iff1: true,
        iff2: true,
        interrupt_mode: InterruptMode::Mode2,
        ei_delay: true,
        halted: false,
        nmi_asserted: true,
        parity_note_pending: true,
    }
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

fn fetch(pc: u16, opcode: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
        read(pc, OP, opcode),
    ]
}

fn check(
    mut bus: RecordingBus,
    initial: Registers,
    expected: Registers,
    cycles: u64,
    trace: Vec<Event>,
    wz: u16,
) {
    let memory = bus.memory.clone();
    let mut cpu = Z80::new(initial, latches());
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles,
            exit: Exit::Budget,
        },
        "initial {initial:?}"
    );
    assert_eq!(*cpu.registers(), expected, "initial {initial:?}");
    assert_eq!(
        cpu.registers().flags.to_byte(),
        initial.flags.to_byte(),
        "all eight flag bits unaffected (decision-20 §4)"
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            wz,
            ei_delay: false,
            nmi_asserted: false,
            parity_note_pending: false,
            ..latches()
        }
    );
    assert_eq!(bus.log, trace, "initial {initial:?}");
    assert!(bus.memory == memory, "unexpected memory change");
}

#[test]
fn jp_nn_reads_low_then_high_and_wraps_operand_addresses() {
    // UM JP nn p. 262: C3, low operand first, 10 states, flags none.
    // TM Table 7.0-9 p. 52 agrees. Address wraps below are derived.
    for image in flag_images() {
        for (pc, low_at, high_at, low, high, target) in [
            (0x23FE, 0x23FF, 0x2400, 0xB6, 0x4D, 0x4DB6),
            (0xFFFE, 0xFFFF, 0x0000, 0x00, 0x00, 0x0000),
            (0xFFFF, 0x0000, 0x0001, 0xFF, 0xFF, 0xFFFF),
        ] {
            let initial = registers(pc, image);
            let mut bus = RecordingBus::new();
            bus.load(pc, &[0xC3, low, high, 0xA5]);
            let mut trace = fetch(pc, 0xC3);
            trace.extend([read(low_at, ARG, low), read(high_at, ARG, high)]);
            check(
                bus,
                initial,
                Registers {
                    pc: target,
                    r: 0x80,
                    ..initial
                },
                10,
                trace,
                target,
            );
        }
    }
}

// UM JP cc,nn pp. 263–264; TM Table 7.0-9 p. 52. Literal opcode,
// relevant F-bit mask, required bit value; not the executor's cc decoder.
const JP_CONDITIONS: [(u8, u8, bool); 8] = [
    (0xC2, 0x40, false), // NZ
    (0xCA, 0x40, true),  // Z
    (0xD2, 0x01, false), // NC
    (0xDA, 0x01, true),  // C; compact UM p. 59's D8 is a known misprint.
    (0xE2, 0x04, false), // PO
    (0xEA, 0x04, true),  // PE
    (0xF2, 0x80, false), // P
    (0xFA, 0x80, true),  // M
];

#[test]
fn every_jp_condition_reads_both_operands_in_ten_states_on_both_paths() {
    // UM pp. 263–264: false continues sequentially; one unqualified
    // 3-cycle / 10 (4,3,3) entry covers both paths. With the two printed
    // operands, derive both reads even untaken. TM p. 52 independently
    // gives three bytes, three cycles and ten states without a path split.
    for (opcode, mask, required) in JP_CONDITIONS {
        for image in flag_images() {
            let taken = (image & mask != 0) == required;
            for (pc, low_at, high_at, sequential) in [
                (0x23FE, 0x23FF, 0x2400, 0x2401),
                (0xFFFE, 0xFFFF, 0x0000, 0x0001),
                (0xFFFF, 0x0000, 0x0001, 0x0002),
            ] {
                let initial = registers(pc, image);
                let mut bus = RecordingBus::new();
                bus.load(pc, &[opcode, 0xB6, 0x4D, 0xA5]);
                let mut trace = fetch(pc, opcode);
                trace.extend([read(low_at, ARG, 0xB6), read(high_at, ARG, 0x4D)]);
                check(
                    bus,
                    initial,
                    Registers {
                        pc: if taken { 0x4DB6 } else { sequential },
                        r: 0x80,
                        ..initial
                    },
                    10,
                    trace,
                    0x4DB6,
                );
            }
        }
    }
}

#[test]
fn jr_uses_signed_displacement_after_the_instruction_including_derived_wraps() {
    // UM JR e pp. 265–266: "measured from the address of the instruction
    // op code", range -126..+129; assembler adjusts for the "twice
    // incremented PC". The encoded byte is e-2, hence a signed -128..127
    // added to the post-instruction PC. TM p. 52's note agrees explicitly.
    // The 0480 + 03 -> 0485 case is the primary's printed example.
    for image in flag_images() {
        for (pc, operand_at, displacement, target) in [
            (0x0480, 0x0481, 0x03, 0x0485),
            (0x0480, 0x0481, 0xFA, 0x047C),
            (0x0480, 0x0481, 0x00, 0x0482),
            (0x0480, 0x0481, 0xFE, 0x0480),
            (0x0480, 0x0481, 0x7F, 0x0501),
            (0x0480, 0x0481, 0x80, 0x0402),
            (0xFFFD, 0xFFFE, 0x01, 0x0000),
            (0x0000, 0x0001, 0x80, 0xFF82),
            (0xFFFE, 0xFFFF, 0xFF, 0xFFFF),
            (0xFFFF, 0x0000, 0x7F, 0x0080),
        ] {
            let initial = registers(pc, image);
            let mut bus = RecordingBus::new();
            bus.load(pc, &[0x18, displacement, 0xA5]);
            let mut trace = fetch(pc, 0x18);
            trace.push(read(operand_at, ARG, displacement));
            check(
                bus,
                initial,
                Registers {
                    pc: target,
                    r: 0x80,
                    ..initial
                },
                12,
                trace,
                target,
            );
        }
    }
}

const JR_CONDITIONS: [(u8, u8, bool); 4] = [
    (0x38, 0x01, true),  // C: UM p. 267
    (0x30, 0x01, false), // NC: UM p. 269
    (0x28, 0x40, true),  // Z: UM p. 271
    (0x20, 0x40, false), // NZ: UM p. 273
];

#[test]
fn every_jr_condition_fetches_displacement_on_both_seven_and_twelve_state_paths() {
    // UM pp. 267/269/271/273 and TM p. 52: false 7, true 12; flags none.
    // Test all flags independently, both displacement extrema and zero,
    // with derived operand-address and positive/negative target wraps.
    for (opcode, mask, required) in JR_CONDITIONS {
        for image in flag_images() {
            let taken = (image & mask != 0) == required;
            for (pc, operand_at, sequential, displacement, target) in [
                (0xFFFE, 0xFFFF, 0x0000, 0x00, 0x0000),
                (0xFFFE, 0xFFFF, 0x0000, 0x7F, 0x007F),
                (0xFFFE, 0xFFFF, 0x0000, 0x80, 0xFF80),
                (0xFFFF, 0x0000, 0x0001, 0xFF, 0x0000),
                (0xFFFD, 0xFFFE, 0xFFFF, 0x01, 0x0000),
            ] {
                let initial = registers(pc, image);
                let mut bus = RecordingBus::new();
                bus.load(pc, &[opcode, displacement, 0xA5]);
                let mut trace = fetch(pc, opcode);
                trace.push(read(operand_at, ARG, displacement));
                check(
                    bus,
                    initial,
                    Registers {
                        pc: if taken { target } else { sequential },
                        r: 0x80,
                        ..initial
                    },
                    if taken { 12 } else { 7 },
                    trace,
                    if taken { target } else { 0 },
                );
            }
        }
    }
}

#[test]
fn jp_hl_loads_the_pair_without_reading_memory_at_the_target() {
    // UM p. 275 / TM p. 52: E9, PC <- HL, one byte, four states.
    // PROVENANCE's existing compact-JP row resolves UM p. 59's EB typo.
    for image in flag_images() {
        for (h, l, target) in [(0x4D, 0xB6, 0x4DB6), (0, 0, 0), (0xFF, 0xFF, 0xFFFF)] {
            let initial = Registers {
                h,
                l,
                ..registers(0xFFFF, image)
            };
            let mut bus = RecordingBus::new();
            bus.load(target, &[0x5A, 0xA5]);
            bus.load(0xFFFF, &[0xE9, 0x3C]);
            check(
                bus,
                initial,
                Registers {
                    pc: target,
                    r: 0x80,
                    ..initial
                },
                4,
                fetch(0xFFFF, 0xE9),
                0,
            );
        }
    }
}

#[test]
fn jp_ix_and_iy_use_the_selected_pair_and_two_opcode_fetches() {
    // UM pp. 276–277 / TM p. 52: DD E9 / FD E9, PC <- IX/IY,
    // two bytes, eight states. Derived prefix/key address wrap at FFFF.
    for image in flag_images() {
        for (prefix, ix, iy, target) in [
            (0xDD, 0x4DB6, 0x92A3, 0x4DB6),
            (0xFD, 0x8091, 0x4DB6, 0x4DB6),
            (0xDD, 0x0000, 0x92A3, 0x0000),
            (0xFD, 0x8091, 0x0000, 0x0000),
            (0xDD, 0xFFFF, 0x92A3, 0xFFFF),
            (0xFD, 0x8091, 0xFFFF, 0xFFFF),
        ] {
            let initial = Registers {
                ix,
                iy,
                ..registers(0xFFFF, image)
            };
            let mut bus = RecordingBus::new();
            bus.load(target, &[0x5A, 0xA5]);
            bus.load(0xFFFF, &[prefix, 0xE9, 0x3C]);
            let mut trace = fetch(0xFFFF, prefix);
            trace.push(read(0x0000, OP, 0xE9));
            check(
                bus,
                initial,
                Registers {
                    pc: target,
                    r: 0x81,
                    ..initial
                },
                8,
                trace,
                0,
            );
        }
    }
}

#[test]
fn djnz_decrements_b_before_testing_it_and_preserves_every_flag() {
    // UM pp. 278–279 / TM p. 52: B <- B-1; branch if new B != 0.
    // Zero (not F.Z) terminates in 8 states; nonzero takes 13. B=0 wraps
    // to FF (derived) and branches. No arithmetic flag is changed.
    for image in flag_images() {
        for (b, next_b, pc, cycles) in [
            (0x00, 0xFF, 0x047C, 13),
            (0x01, 0x00, 0x0482, 8),
            (0x02, 0x01, 0x047C, 13),
            (0x80, 0x7F, 0x047C, 13),
            (0xFF, 0xFE, 0x047C, 13),
        ] {
            let initial = Registers {
                b,
                ..registers(0x0480, image)
            };
            let mut bus = RecordingBus::new();
            bus.load(0x0480, &[0x10, 0xFA, 0xA5]);
            let mut trace = fetch(0x0480, 0x10);
            trace.push(read(0x0481, ARG, 0xFA));
            check(
                bus,
                initial,
                Registers {
                    b: next_b,
                    pc,
                    r: 0x80,
                    ..initial
                },
                cycles,
                trace,
                if next_b != 0 { pc } else { 0 },
            );
        }
    }
}

#[test]
fn djnz_signed_extrema_and_derived_pc_wraps_on_both_paths() {
    // UM p. 278 uses the same twice-incremented PC wording as JR;
    // TM p. 52 agrees. All target/address wraps below are derived.
    for image in flag_images() {
        for (start, operand_at, displacement, target, sequential) in [
            (0xFFFE, 0xFFFF, 0x00, 0x0000, 0x0000),
            (0xFFFE, 0xFFFF, 0x80, 0xFF80, 0x0000),
            (0xFFFF, 0x0000, 0x7F, 0x0080, 0x0001),
            (0xFFFD, 0xFFFE, 0x01, 0x0000, 0xFFFF),
        ] {
            for (b, next_b, pc, cycles) in [(2, 1, target, 13), (1, 0, sequential, 8)] {
                let initial = Registers {
                    b,
                    ..registers(start, image)
                };
                let mut bus = RecordingBus::new();
                bus.load(start, &[0x10, displacement, 0xA5]);
                let mut trace = fetch(start, 0x10);
                trace.push(read(operand_at, ARG, displacement));
                check(
                    bus,
                    initial,
                    Registers {
                        b: next_b,
                        pc,
                        r: 0x80,
                        ..initial
                    },
                    cycles,
                    trace,
                    if next_b != 0 { pc } else { 0 },
                );
            }
        }
    }
}

#[test]
fn jr_nc_zero_displacement_continues_and_minus_two_repeats() {
    // Source disagreement pin: UM p. 270 prints 30 00 at 0480 as JR NC,$,
    // ending at 0480. UM p. 269's twice-incremented-PC rule and TM Table
    // 7.0-9 p. 52's explicit PC+2 note instead give 0482 for byte 00;
    // FE is required for a self jump. Both images reread at 200 dpi with
    // enlarged crops. Follow the agreeing instruction rule and TM note.
    // The root PROVENANCE row is handed to the supervisor (outside this
    // worker's writable scope); this test pins both bytes/results now.
    for image in flag_images() {
        let taken = image & 1 == 0;
        for (displacement, target) in [(0x00, 0x0482), (0xFE, 0x0480)] {
            let initial = registers(0x0480, image);
            let mut bus = RecordingBus::new();
            bus.load(0x0480, &[0x30, displacement, 0xA5]);
            let mut trace = fetch(0x0480, 0x30);
            trace.push(read(0x0481, ARG, displacement));
            check(
                bus,
                initial,
                Registers {
                    pc: if taken { target } else { 0x0482 },
                    r: 0x80,
                    ..initial
                },
                if taken { 12 } else { 7 },
                trace,
                if taken { target } else { 0 },
            );
        }
    }
}
