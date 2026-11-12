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
