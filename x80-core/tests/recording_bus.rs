//! The 8080 recording test bus, driven only through the `Bus`, `Control` and
//! `i8080::Inputs` traits, and the default input-line methods (decision-5
//! items 3 and 4).

mod support;

use support::{Event, RecordingBus};
use x80_core::i8080::Inputs;
use x80_core::{Bus, Control, MemoryAccess};

/// Calls every trait method once, as generic code in the core would.
fn drive<S: Bus + Inputs>(system: &mut S) -> [u8; 5] {
    let stop = system.stop_requested();
    let interrupt = system.interrupt_requested();
    let opcode = system.read(0x0100, MemoryAccess::OpcodeFetch);
    let operand = system.read(0x0101, MemoryAccess::OperandFetch);
    system.write(0x80FF, 0x5A, MemoryAccess::Stack);
    let stacked = system.read(0x80FF, MemoryAccess::Stack);
    system.write(0x2000, 0xC3, MemoryAccess::Data);
    let input = system.input(0x10);
    system.output(0x11, 0x99);
    let acknowledged = system.read(0x0100, MemoryAccess::InterruptAcknowledge);
    assert!(!stop);
    assert!(interrupt);
    [opcode, operand, stacked, input, acknowledged]
}

#[test]
fn records_every_call_in_order_with_scripted_responses() {
    let mut bus = RecordingBus::new();
    bus.load(0x0100, &[0xDB, 0x10]);
    bus.inputs.push_back(0x42);
    bus.acknowledges.push_back(0xFF);
    bus.interrupt = true;

    let results = drive(&mut bus);

    assert_eq!(results, [0xDB, 0x10, 0x5A, 0x42, 0xFF]);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::InterruptPoll(true),
            Event::Read {
                address: 0x0100,
                kind: MemoryAccess::OpcodeFetch,
                value: 0xDB,
            },
            Event::Read {
                address: 0x0101,
                kind: MemoryAccess::OperandFetch,
                value: 0x10,
            },
            Event::Write {
                address: 0x80FF,
                kind: MemoryAccess::Stack,
                value: 0x5A,
            },
            Event::Read {
                address: 0x80FF,
                kind: MemoryAccess::Stack,
                value: 0x5A,
            },
            Event::Write {
                address: 0x2000,
                kind: MemoryAccess::Data,
                value: 0xC3,
            },
            Event::Input {
                port: 0x10,
                value: 0x42,
            },
            Event::Output {
                port: 0x11,
                value: 0x99,
            },
            Event::Read {
                address: 0x0100,
                kind: MemoryAccess::InterruptAcknowledge,
                value: 0xFF,
            },
        ]
    );
    assert!(bus.log.is_empty());
    assert_eq!(bus.memory[0x2000], 0xC3);
    // The acknowledge read did not come from memory.
    assert_eq!(bus.memory[0x0100], 0xDB);
    assert!(bus.inputs.is_empty());
    assert!(bus.acknowledges.is_empty());
}

#[test]
fn load_wraps_at_the_top_of_memory() {
    let mut bus = RecordingBus::new();
    bus.load(0xFFFF, &[0x11, 0x22]);
    assert_eq!(bus.memory[0xFFFF], 0x11);
    assert_eq!(bus.memory[0x0000], 0x22);
    assert!(bus.log.is_empty());
}

#[test]
#[should_panic(expected = "unscripted input from port 0x10")]
fn unscripted_input_panics() {
    RecordingBus::new().input(0x10);
}

#[test]
#[should_panic(expected = "unscripted interrupt acknowledge at 0x0038")]
fn unscripted_acknowledge_panics() {
    RecordingBus::new().read(0x0038, MemoryAccess::InterruptAcknowledge);
}

/// An embedder value that relies on the default input-line methods.
struct Silent;

impl Control for Silent {}
impl Inputs for Silent {}

#[test]
fn default_input_lines_request_no_stop_and_no_interrupt() {
    // decision-5, Consequences: "the defaults request no stop and no interrupt".
    let mut silent = Silent;
    assert!(!silent.stop_requested());
    assert!(!silent.interrupt_requested());
}

impl x80_core::i8085::Inputs for Silent {}

#[test]
fn default_8085_inputs_are_low_and_sod_notification_is_a_no_op() {
    use x80_core::i8085::Inputs;
    // decision-12: defaults are part of the API, not hardware line bias.
    let mut silent = Silent;
    assert!(!silent.stop_requested());
    assert!(!silent.trap());
    assert!(!silent.rst7_5());
    assert!(!silent.rst6_5());
    assert!(!silent.rst5_5());
    assert!(!silent.intr());
    assert!(!silent.sid());
    silent.sod_changed(true);
    silent.sod_changed(false);
}

/// Drive the decision-12 rule 1 order through the trait. This tests the bus
/// fixture, not CPU boundary execution (which is not implemented yet).
fn drive_8085<S: x80_core::i8085::Inputs>(system: &mut S) -> [bool; 7] {
    [
        system.stop_requested(),
        system.trap(),
        system.rst7_5(),
        system.rst6_5(),
        system.rst5_5(),
        system.intr(),
        system.sid(),
    ]
}

#[test]
fn records_8085_scripts_then_fallbacks_and_every_sod_notification() {
    use x80_core::i8085::Inputs;
    let mut bus = RecordingBus::new();
    bus.stop = true;
    bus.trap = true;
    bus.rst6_5 = true;
    bus.intr = true;
    bus.trap_levels.extend([false, true]);
    bus.rst7_5_levels.extend([true, false]);
    bus.rst6_5_levels.extend([false, true]);
    bus.rst5_5_levels.extend([true, false]);
    bus.intr_levels.extend([false, true]);
    bus.sid_levels.extend([true, false]);

    assert_eq!(
        drive_8085(&mut bus),
        [true, false, true, false, true, false, true]
    );
    assert_eq!(
        drive_8085(&mut bus),
        [true, true, false, true, false, true, false]
    );
    assert_eq!(
        drive_8085(&mut bus),
        [true, true, false, true, false, true, false]
    );
    bus.sod_changed(true);
    bus.sod_changed(true); // Enabled SIM writes notify even at the same level.
    bus.sod_changed(false);
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(true),
            Event::TrapPoll(false),
            Event::Rst7_5Poll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(true),
            Event::IntrPoll(false),
            Event::SidPoll(true),
            Event::StopPoll(true),
            Event::TrapPoll(true),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(true),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(true),
            Event::SidPoll(false),
            Event::StopPoll(true),
            Event::TrapPoll(true),
            Event::Rst7_5Poll(false),
            Event::Rst6_5Poll(true),
            Event::Rst5_5Poll(false),
            Event::IntrPoll(true),
            Event::SidPoll(false),
            Event::SodChanged(true),
            Event::SodChanged(true),
            Event::SodChanged(false),
        ]
    );
}

#[test]
fn input_scripts_are_independent_between_models_and_rim_reads() {
    use x80_core::i8085::Inputs;
    let mut bus = RecordingBus::new();
    bus.interrupt_levels.extend([true, false]);
    bus.intr_levels.extend([false, true]);
    bus.sid_levels.push_back(true);
    bus.rst6_5_levels.extend([false, true]);
    bus.rst5_5_levels.extend([true, false]);
    assert!(x80_core::i8080::Inputs::interrupt_requested(&mut bus));
    assert!(!bus.intr());
    assert!(!x80_core::i8080::Inputs::interrupt_requested(&mut bus));
    assert!(bus.intr());
    // rule 8's in-instruction SID / RST 6.5 / RST 5.5 order consumes
    // independent scripts, including when level methods are read again.
    assert!(bus.sid());
    assert!(!bus.rst6_5());
    assert!(bus.rst5_5());
    assert!(bus.rst6_5());
    assert!(!bus.rst5_5());
    assert_eq!(
        bus.take_log(),
        [
            Event::InterruptPoll(true),
            Event::IntrPoll(false),
            Event::InterruptPoll(false),
            Event::IntrPoll(true),
            Event::SidPoll(true),
            Event::Rst6_5Poll(false),
            Event::Rst5_5Poll(true),
            Event::Rst6_5Poll(true),
            Event::Rst5_5Poll(false),
        ]
    );
}
