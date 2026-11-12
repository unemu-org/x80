//! The 8080 Stack, I/O and Machine Control Group (98-153B pp. 4-13 and 4-14).
//!
//! NOP is the only instruction that executes. Every other opcode of the group
//! still returns `Unsupported` (decision-5 item 7) until its bead lands.
//! Expected values are literals.

mod support;

use support::{assert_unsupported, sample_registers, Event, RecordingBus};
use x80_core::i8080::{Latches, I8080};
use x80_core::{Budget, Exit, MemoryAccess, Report};

/// Interrupts enabled, no EI delay pending, not halted.
const LATCHES: Latches = Latches {
    interrupt_enable: true,
    ei_delay: false,
    halted: false,
};

fn nop_fetch(pc: u16) -> [Event; 3] {
    [
        Event::StopPoll(false),
        Event::InterruptPoll(false),
        Event::Read {
            address: pc,
            kind: MemoryAccess::OpcodeFetch,
            value: 0x00,
        },
    ]
}

#[test]
fn nop_takes_four_states_and_only_advances_pc() {
    // 98-153B p. 4-14: NOP is 00000000, "Cycles: 1, States: 4"; "No operation
    // is performed. The registers and flags are unaffected."
    let mut bus = RecordingBus::new();
    bus.load(0x1233, &[0x5A, 0x00, 0xA5]);
    let memory = bus.memory.clone();
    let mut cpu = I8080::new(sample_registers(0x1234), LATCHES);

    let report = cpu.step(&mut bus);

    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: 4,
            exit: Exit::Budget,
        }
    );
    assert_eq!(*cpu.registers(), sample_registers(0x1235));
    assert_eq!(*cpu.latches(), LATCHES);
    assert!(bus.memory == memory, "memory changed");
    assert_eq!(bus.take_log(), nop_fetch(0x1234));
}

#[test]
fn nop_wraps_pc_from_ffff_to_0000() {
    // PC is a 16-bit register (98-153B p. 2-2). Memory is all zero, so 0x0000
    // also holds a NOP.
    let mut bus = RecordingBus::new();
    let mut cpu = I8080::new(sample_registers(0xFFFF), LATCHES);

    let report = cpu.run(
        &mut bus,
        Budget {
            steps: 2,
            cycles: u64::MAX,
        },
    );

    assert_eq!(
        report,
        Report {
            steps: 2,
            cycles: 8,
            exit: Exit::Budget,
        }
    );
    assert_eq!(*cpu.registers(), sample_registers(0x0001));
    assert_eq!(
        bus.take_log(),
        [nop_fetch(0xFFFF), nop_fetch(0x0000)].concat()
    );
}

#[test]
fn the_other_opcodes_of_the_group_are_not_implemented_yet() {
    // PUSH B, D, H, PSW; POP B, D, H, PSW (p. 4-13); XTHL, SPHL, IN, OUT,
    // EI, DI and HLT (p. 4-14).
    let opcodes = [
        0xC5, 0xD5, 0xE5, 0xF5, 0xC1, 0xD1, 0xE1, 0xF1, 0xE3, 0xF9, 0xDB, 0xD3, 0xFB, 0xF3, 0x76,
    ];
    for opcode in opcodes {
        assert_unsupported(opcode, 0x2345);
    }
}
