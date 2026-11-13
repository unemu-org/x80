//! Ownership, construction, reset and input witnesses (decision-15 rules 1/12).
//!
//! Zilog UM008011-0816 (revision 11, August 2016), RESET p. 6 and Table 1
//! p. 18 (PDF 20/32): clear PC, I, R, both IFFs; select IM 0. HALT p. 181
//! (PDF 195): reset leaves halt. Second witness: 03-0029-01 (1977), RESET
//! p. 9 and Fig. 8.0-1 p. 56 (PDF 15/62). Pages read as rendered images.
//! Preservation, bookkeeping, Default and poll order are library choices.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, Inputs, InterruptMode, Latches, Registers, Z80};
use x80_core::Control;

const NO_FLAGS: Flags = Flags {
    sign: false,
    zero: false,
    half_carry: false,
    parity_overflow: false,
    subtract: false,
    carry: false,
};
const ALL_FLAGS: Flags = Flags {
    sign: true,
    zero: true,
    half_carry: true,
    parity_overflow: true,
    subtract: true,
    carry: true,
};

const ZERO_REGISTERS: Registers = Registers {
    a: 0,
    flags: NO_FLAGS,
    b: 0,
    c: 0,
    d: 0,
    e: 0,
    h: 0,
    l: 0,
    a_alternate: 0,
    flags_alternate: NO_FLAGS,
    b_alternate: 0,
    c_alternate: 0,
    d_alternate: 0,
    e_alternate: 0,
    h_alternate: 0,
    l_alternate: 0,
    ix: 0,
    iy: 0,
    sp: 0,
    pc: 0,
    i: 0,
    r: 0,
};
const RESET_LATCHES: Latches = Latches {
    iff1: false,
    iff2: false,
    interrupt_mode: InterruptMode::Mode0,
    ei_delay: false,
    halted: false,
    nmi_asserted: false,
    parity_note_pending: false,
};

// Every register, flag and latch differs from its construction default.
const DIRTY_REGISTERS: Registers = Registers {
    a: 0xA1,
    flags: ALL_FLAGS,
    b: 0xB2,
    c: 0xC3,
    d: 0xD4,
    e: 0xE5,
    h: 0xF6,
    l: 0x07,
    a_alternate: 0x18,
    flags_alternate: ALL_FLAGS,
    b_alternate: 0x29,
    c_alternate: 0x3A,
    d_alternate: 0x4B,
    e_alternate: 0x5C,
    h_alternate: 0x6D,
    l_alternate: 0x7E,
    ix: 0x8091,
    iy: 0x92A3,
    sp: 0xB4C5,
    pc: 0xD6E7,
    i: 0xF8,
    r: 0x89,
};
const DIRTY_LATCHES: Latches = Latches {
    iff1: true,
    iff2: true,
    interrupt_mode: InterruptMode::Mode2,
    ei_delay: true,
    halted: true,
    nmi_asserted: true,
    parity_note_pending: true,
};

#[test]
fn default_is_a_library_construction_value() {
    let cpu = Z80::default();
    assert_eq!(*cpu.registers(), ZERO_REGISTERS);
    assert_eq!(*cpu.latches(), RESET_LATCHES);
    assert_eq!(Registers::default(), ZERO_REGISTERS);
    assert_eq!(Flags::default(), NO_FLAGS);
    assert_eq!(Latches::default(), RESET_LATCHES);
    assert_eq!(InterruptMode::default(), InterruptMode::Mode0);
}

#[test]
fn new_and_const_accessors_preserve_supplied_state() {
    const CPU: Z80 = Z80::new(DIRTY_REGISTERS, DIRTY_LATCHES);
    const PC: u16 = CPU.registers().pc;
    const IFF2: bool = CPU.latches().iff2;
    assert_eq!(PC, 0xD6E7);
    assert_eq!(u8::from(IFF2), 1);
    assert_eq!(*CPU.registers(), DIRTY_REGISTERS);
    assert_eq!(*CPU.latches(), DIRTY_LATCHES);
}

#[test]
fn mutable_accessors_change_owned_state() {
    let mut cpu = Z80::default();
    *cpu.registers_mut() = DIRTY_REGISTERS;
    *cpu.latches_mut() = DIRTY_LATCHES;
    cpu.latches_mut().interrupt_mode = InterruptMode::Mode1;
    assert_eq!(*cpu.registers(), DIRTY_REGISTERS);
    assert_eq!(
        *cpu.latches(),
        Latches {
            interrupt_mode: InterruptMode::Mode1,
            ..DIRTY_LATCHES
        }
    );
}

#[test]
fn reset_clears_the_documented_state_and_leaves_halt() {
    // UM RESET p. 6, Table 1 p. 18, HALT p. 181; TM pp. 9/56.
    for mode in [InterruptMode::Mode1, InterruptMode::Mode2] {
        let mut cpu = Z80::new(
            DIRTY_REGISTERS,
            Latches {
                interrupt_mode: mode,
                ..DIRTY_LATCHES
            },
        );
        cpu.reset();
        assert_eq!(cpu.registers().pc, 0x0000);
        assert_eq!(cpu.registers().i, 0x00);
        assert_eq!(cpu.registers().r, 0x00);
        assert!(!cpu.latches().iff1);
        assert!(!cpu.latches().iff2);
        assert_eq!(cpu.latches().interrupt_mode, InterruptMode::Mode0);
        assert!(!cpu.latches().halted);
    }
}

#[test]
fn reset_preserves_both_register_sets_flags_sp_and_indexes_by_library_choice() {
    let mut cpu = Z80::new(DIRTY_REGISTERS, DIRTY_LATCHES);
    cpu.reset();
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0,
            i: 0,
            r: 0,
            ..DIRTY_REGISTERS
        }
    );
}

#[test]
fn reset_clears_bookkeeping_by_library_choice_and_is_idempotent() {
    let mut cpu = Z80::new(DIRTY_REGISTERS, DIRTY_LATCHES);
    cpu.reset();
    assert_eq!(*cpu.latches(), RESET_LATCHES);
    let once = cpu.clone();
    cpu.reset();
    assert_eq!(cpu, once);
    // No bus or input argument exists: reset makes no calls or polls.
}

struct Silent;
impl Control for Silent {}
impl Inputs for Silent {}

#[test]
fn default_inputs_request_no_stop_nmi_or_int() {
    // API defaults, not hardware pin bias; decision-15 rule 1.
    let mut silent = Silent;
    assert!(!silent.stop_requested());
    assert!(!silent.nmi());
    assert!(!silent.int());
}

fn poll<S: Inputs>(system: &mut S) -> [bool; 3] {
    [system.stop_requested(), system.nmi(), system.int()]
}

#[test]
fn recording_bus_logs_independent_nmi_int_scripts_and_fallbacks() {
    // Fixture witness only: Z9 will implement CPU boundary polling.
    // Asserted true means pin Low (UM p. 6; TM §3 p. 8, PDF 20/14).
    let mut bus = RecordingBus::new();
    assert_eq!(poll(&mut bus), [false, false, false]);
    bus.stop = true;
    bus.nmi = true;
    bus.int = true;
    bus.nmi_levels.extend([false, true]);
    bus.int_levels.extend([true, false]);
    bus.interrupt_levels.push_back(true);
    bus.intr_levels.push_back(false);
    assert_eq!(poll(&mut bus), [true, false, true]);
    assert_eq!(poll(&mut bus), [true, true, false]);
    assert_eq!(poll(&mut bus), [true, true, true]);
    assert!(x80_core::i8080::Inputs::interrupt_requested(&mut bus));
    assert!(!x80_core::i8085::Inputs::intr(&mut bus));
    assert!(bus.nmi_levels.is_empty());
    assert!(bus.int_levels.is_empty());
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::NmiPoll(false),
            Event::IntPoll(false),
            Event::StopPoll(true),
            Event::NmiPoll(false),
            Event::IntPoll(true),
            Event::StopPoll(true),
            Event::NmiPoll(true),
            Event::IntPoll(false),
            Event::StopPoll(true),
            Event::NmiPoll(true),
            Event::IntPoll(true),
            Event::InterruptPoll(true),
            Event::IntrPoll(false),
        ]
    );
}
