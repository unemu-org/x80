//! Decision-20 §1 literal witnesses, independent of decoder/ALU helpers.
//! A ED70: Zilog 03-0029-01 (1977), Table 7.0-11 p. 54; UM008011-0816
//! rev. 11 pp. 296–297 (PDF +14). H operations/preservation: Young v0.91
//! (18 September 2005) §3.4 pp. 12–13 plus Rak z80test v1.2a .in_c
//! 1000–1006, .out_c_0 1070–1075, .neg_ 389–394, .reti_retn 1202–1207.
//! L ED71 emitted zero/cost: Young §8.11 p. 35; L alias costs: Young
//! §§8.5/8.10 pp. 30/34, UM pp. 176–177/288/290; L IM aliases and NOP
//! holes: Young §3.4 pp. 12–13, canonical IM UM pp. 184–186.
//! L IFF copy including RETI: Young §3.4 p. 13 / §5.3 pp. 20–21 /
//! §8.10 p. 34; hardware conflict remains unverified (PROVENANCE §1).

#![forbid(unsafe_code)]
mod support;
use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, MemoryAccess};

fn cpu_at(pc: u16) -> Z80 {
    Z80::new(
        Registers {
            a: 0x81,
            flags: Flags::from_byte(0xD7),
            b: 0xAB,
            c: 0xCD,
            d: 0x12,
            e: 0x34,
            h: 0x56,
            l: 0x78,
            a_alternate: 0x91,
            flags_alternate: Flags::from_byte(0x83),
            b_alternate: 0x21,
            c_alternate: 0x32,
            d_alternate: 0x43,
            e_alternate: 0x54,
            h_alternate: 0x65,
            l_alternate: 0x76,
            ix: 0x1357,
            iy: 0x2468,
            sp: 0x9000,
            pc,
            i: 0xA5,
            r: 0xFF,
        },
        Latches {
            iff1: true,
            iff2: false,
            interrupt_mode: InterruptMode::Mode2,
            ..Latches::default()
        },
    )
}
fn read(address: u16, value: u8) -> Event {
    Event::Read {
        address,
        kind: MemoryAccess::OpcodeFetch,
        value,
    }
}
fn fetch(pc: u16, op: u8) -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
        read(pc, 0xED),
        read(pc.wrapping_add(1), op),
    ]
}
fn retire(cpu: &mut Z80, bus: &mut RecordingBus, cycles: u64) {
    let result = cpu.step(bus);
    assert_eq!(
        (result.steps, result.cycles, result.exit),
        (1, cycles, Exit::Budget)
    );
}

#[test]
fn ed70_discards_input_uses_full_bc_and_sets_eight_stored_flags() {
    // A/H decision-20 §1; TM p. 54, UM pp. 296–297; Rak .in_c.
    // Decision-20 §4 H: Young §8.11 p. 35 / Rak .in_c, full Y/X.
    for (value, base_flags) in [
        (0x00, 0x44),
        (0x80, 0x80),
        (0x81, 0x84),
        (0x7F, 0x00),
        (0x03, 0x04),
        (0xBF, 0x80),
        (0xFF, 0x84),
    ] {
        for carry in [0, 1] {
            let mut cpu = cpu_at(0xFFFF);
            cpu.registers_mut().flags = Flags::from_byte(0xD6 | carry);
            let mut expected = cpu.clone();
            expected.latches_mut().wz = 0xABCE;
            expected.registers_mut().pc = 1;
            expected.registers_mut().r = 0x81;
            expected.registers_mut().flags = Flags::from_byte(base_flags | carry | (value & 0x28));
            let mut bus = RecordingBus::new();
            bus.load(0xFFFF, &[0xED, 0x70]);
            bus.inputs.push_back(value);
            let mut trace = fetch(0xFFFF, 0x70);
            trace.push(Event::Input16 {
                port: 0xABCD,
                value,
            });
            retire(&mut cpu, &mut bus, 12);
            assert_eq!(cpu, expected);
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn ed71_emits_library_zero_at_full_bc_preserving_all_state() {
    // L output/length/cost; H CPU preservation: decision-20 §1;
    // Young §3.4 p. 13 / §8.11 p. 35; Rak .out_c_0 has no bus capture.
    let mut cpu = cpu_at(0x1234);
    let mut expected = cpu.clone();
    expected.latches_mut().wz = 0xABCE;
    expected.registers_mut().pc = 0x1236;
    expected.registers_mut().r = 0x81;
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0xED, 0x71]);
    let mut trace = fetch(0x1234, 0x71);
    trace.push(Event::Output16 {
        port: 0xABCD,
        value: 0x00,
    });
    retire(&mut cpu, &mut bus, 12);
    assert_eq!(cpu, expected);
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn every_neg_encoding_has_literal_results_flags_and_alias_cost() {
    // H operation, L alias cost: decision-20 §1; Young §3.4 pp. 12–13,
    // §8.5 p. 30; Rak .neg_; UM pp. 176–177.
    for op in [0x44, 0x4C, 0x54, 0x5C, 0x64, 0x6C, 0x74, 0x7C] {
        for (a, result, f) in [
            (0x00, 0x00, 0x42),
            (0x01, 0xFF, 0x93),
            (0x7F, 0x81, 0x93),
            (0x80, 0x80, 0x87),
            (0xFF, 0x01, 0x13),
        ] {
            let mut cpu = cpu_at(0x1234);
            cpu.registers_mut().a = a;
            let mut expected = cpu.clone();
            expected.registers_mut().a = result;
            expected.registers_mut().flags = Flags::from_byte(f | (result & 0x28));
            expected.registers_mut().pc = 0x1236;
            expected.registers_mut().r = 0x81;
            let mut bus = RecordingBus::new();
            bus.load(0x1234, &[0xED, op]);
            retire(&mut cpu, &mut bus, 8);
            assert_eq!(cpu, expected, "{op:02X} {a:02X}");
            assert_eq!(bus.take_log(), fetch(0x1234, op));
        }
    }
}

#[test]
fn all_ed_returns_pop_wrapping_stack_and_copy_unequal_iffs_including_reti() {
    // H return/preservation, L alias timing/IFF mapping: decision-20 §1;
    // Young §§3.4/5.3/8.10 pp. 12–13/20–21/34; Rak .reti_retn.
    // ED4D alone remains the peripheral RETI signature via its fetches.
    for op in [0x45, 0x4D, 0x55, 0x5D, 0x65, 0x6D, 0x75, 0x7D] {
        for (iff1, iff2) in [(false, true), (true, false)] {
            let mut cpu = cpu_at(0x1234);
            cpu.registers_mut().sp = 0xFFFF;
            cpu.latches_mut().iff1 = iff1;
            cpu.latches_mut().iff2 = iff2;
            let mut expected = cpu.clone();
            expected.latches_mut().wz = 0x2468;
            expected.registers_mut().pc = 0x2468;
            expected.registers_mut().sp = 1;
            expected.registers_mut().r = 0x81;
            expected.latches_mut().iff1 = iff2;
            let mut bus = RecordingBus::new();
            bus.load(0x1234, &[0xED, op]);
            bus.load(0xFFFF, &[0x68, 0x24]);
            let mut trace = fetch(0x1234, op);
            trace.extend([
                Event::Read {
                    address: 0xFFFF,
                    kind: MemoryAccess::Stack,
                    value: 0x68,
                },
                Event::Read {
                    address: 0,
                    kind: MemoryAccess::Stack,
                    value: 0x24,
                },
            ]);
            retire(&mut cpu, &mut bus, 14);
            assert_eq!(cpu, expected);
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn all_im_encodings_select_literal_modes_preserving_iffs_and_registers() {
    // L aliases/assignment/timing: decision-20 §1, Young §3.4 pp. 12–13;
    // canonical A UM pp. 184–186. Rak .im_n does not observe IM.
    for (op, mode) in [
        (0x46, InterruptMode::Mode0),
        (0x4E, InterruptMode::Mode0),
        (0x66, InterruptMode::Mode0),
        (0x6E, InterruptMode::Mode0),
        (0x56, InterruptMode::Mode1),
        (0x76, InterruptMode::Mode1),
        (0x5E, InterruptMode::Mode2),
        (0x7E, InterruptMode::Mode2),
    ] {
        let mut cpu = cpu_at(0x1234);
        let mut expected = cpu.clone();
        expected.registers_mut().pc = 0x1236;
        expected.registers_mut().r = 0x81;
        expected.latches_mut().interrupt_mode = mode;
        let mut bus = RecordingBus::new();
        bus.load(0x1234, &[0xED, op]);
        retire(&mut cpu, &mut bus, 8);
        assert_eq!(cpu, expected);
        assert_eq!(bus.take_log(), fetch(0x1234, op));
    }
}

// Decision-20 §1 derived map, Young §3.4 pp. 12–13. Literal operation slots
// and totals, separate from implementation patterns. A documented operation
// costs; L alias/ED71 costs. ED63/6B remain four-byte loads.
const OPERATIONS: &[(u8, u64)] = &[
    (0x40, 12),
    (0x41, 12),
    (0x42, 15),
    (0x43, 20),
    (0x44, 8),
    (0x45, 14),
    (0x46, 8),
    (0x47, 9),
    (0x48, 12),
    (0x49, 12),
    (0x4A, 15),
    (0x4B, 20),
    (0x4C, 8),
    (0x4D, 14),
    (0x4E, 8),
    (0x4F, 9),
    (0x50, 12),
    (0x51, 12),
    (0x52, 15),
    (0x53, 20),
    (0x54, 8),
    (0x55, 14),
    (0x56, 8),
    (0x57, 9),
    (0x58, 12),
    (0x59, 12),
    (0x5A, 15),
    (0x5B, 20),
    (0x5C, 8),
    (0x5D, 14),
    (0x5E, 8),
    (0x5F, 9),
    (0x60, 12),
    (0x61, 12),
    (0x62, 15),
    (0x63, 20),
    (0x64, 8),
    (0x65, 14),
    (0x66, 8),
    (0x67, 18),
    (0x68, 12),
    (0x69, 12),
    (0x6A, 15),
    (0x6B, 20),
    (0x6C, 8),
    (0x6D, 14),
    (0x6E, 8),
    (0x6F, 18),
    (0x70, 12),
    (0x71, 12),
    (0x72, 15),
    (0x73, 20),
    (0x74, 8),
    (0x75, 14),
    (0x76, 8),
    (0x78, 12),
    (0x79, 12),
    (0x7A, 15),
    (0x7B, 20),
    (0x7C, 8),
    (0x7D, 14),
    (0x7E, 8),
    (0xA0, 16),
    (0xA1, 16),
    (0xA2, 16),
    (0xA3, 16),
    (0xA8, 16),
    (0xA9, 16),
    (0xAA, 16),
    (0xAB, 16),
    (0xB0, 21),
    (0xB1, 21),
    (0xB2, 21),
    (0xB3, 21),
    (0xB8, 21),
    (0xB9, 21),
    (0xBA, 21),
    (0xBB, 21),
];

#[test]
fn exhaustive_terminal_map_executes_78_operations_and_178_nops() {
    assert_eq!(OPERATIONS.len(), 78);
    let mut holes = 0;
    for op in 0..=255 {
        let mut cpu = cpu_at(0x1234);
        let mut bus = RecordingBus::new();
        bus.load(0x1234, &[0xED, op, 0x00, 0x80]);
        bus.inputs.push_back(0xBF);
        if let Some(&(_, cost)) = OPERATIONS.iter().find(|&&(key, _)| key == op) {
            retire(&mut cpu, &mut bus, cost);
        } else {
            // L NOP map/access/retirement: decision-20 §1, Young §3.4
            // pp. 12–13. Full state equality detects register/flag changes.
            assert!(matches!(op, 0x00..=0x3F | 0x77 | 0x7F | 0x80..=0x9F |
                0xA4..=0xA7 | 0xAC..=0xAF | 0xB4..=0xB7 | 0xBC..=0xFF));
            holes += 1;
            let mut expected = cpu.clone();
            expected.registers_mut().pc = 0x1236;
            expected.registers_mut().r = 0x81;
            let memory = bus.memory.clone();
            retire(&mut cpu, &mut bus, 8);
            assert_eq!(cpu, expected, "{op:02X}");
            assert_eq!(bus.memory, memory);
            assert_eq!(bus.take_log(), fetch(0x1234, op));
        }
    }
    assert_eq!(holes, 178);
}

#[test]
fn nop_holes_consume_prefix_valued_terminals_wrap_pc_and_retire_once() {
    // L decision-20 §1 / Young §3.4 pp. 12–13; boundary API mapping.
    for op in [0x00, 0x77, 0x7F, 0xCB, 0xDD, 0xED, 0xFD, 0xFF] {
        for (pc, next) in [(0xFFFE, 0), (0xFFFF, 1)] {
            let mut cpu = cpu_at(pc);
            cpu.latches_mut().ei_delay = true;
            cpu.latches_mut().parity_note_pending = true;
            let mut expected = cpu.clone();
            expected.registers_mut().pc = next;
            expected.registers_mut().r = 0x81;
            expected.latches_mut().ei_delay = false;
            expected.latches_mut().parity_note_pending = false;
            let mut bus = RecordingBus::new();
            bus.load(pc, &[0xED, op, 0x76]);
            let report = cpu.run(
                &mut bus,
                Budget {
                    steps: 1,
                    cycles: u64::MAX,
                },
            );
            assert_eq!(
                (report.steps, report.cycles, report.exit),
                (1, 8, Exit::Budget)
            );
            assert_eq!(cpu, expected);
            assert_eq!(bus.take_log(), fetch(pc, op));
        }
    }
}
