//! Model-neutral additions: decision-14 rule 3 and decision-15 rules 4/13.
//! These witness API choices; model refusal semantics retain their witnesses in
//! i8080_coverage, i8080_interrupts, i8085_coverage and i8085_interrupts_intr.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8080::I8080;
use x80_core::i8085::I8085;
use x80_core::{Bus, Exit, MemoryAccess, Report, Unsupported};

/// An existing embedder implementing only the required Bus methods.
#[derive(Default)]
struct RequiredBus {
    log: Vec<Event>,
}

impl Bus for RequiredBus {
    fn read(&mut self, _address: u16, _kind: MemoryAccess) -> u8 {
        0
    }

    fn write(&mut self, _address: u16, _value: u8, _kind: MemoryAccess) {}

    fn input(&mut self, port: u8) -> u8 {
        self.log.push(Event::Input { port, value: 0x56 });
        0x56
    }

    fn output(&mut self, port: u8, value: u8) {
        self.log.push(Event::Output { port, value });
    }
}

#[test]
fn default_input16_forwards_only_the_low_byte() {
    let mut bus = RequiredBus::default();
    assert_eq!(bus.input16(0xAB12), 0x56);
    assert_eq!(
        bus.log,
        [Event::Input {
            port: 0x12,
            value: 0x56,
        }]
    );
}

#[test]
fn default_output16_forwards_only_the_low_byte() {
    let mut bus = RequiredBus::default();
    bus.output16(0xCD34, 0x78);
    assert_eq!(
        bus.log,
        [Event::Output {
            port: 0x34,
            value: 0x78,
        }]
    );
}

#[test]
fn overridden_port_methods_record_full_addresses_and_distinct_events() {
    let mut bus = RecordingBus::new();
    bus.inputs.extend([0x56, 0x9A]);
    assert_eq!(bus.input16(0xAB12), 0x56);
    bus.output16(0xCD34, 0x78);
    assert_eq!(bus.input(0x12), 0x9A);
    bus.output(0x34, 0xBC);
    assert!(bus.inputs.is_empty());
    assert_eq!(
        bus.take_log(),
        [
            Event::Input16 {
                port: 0xAB12,
                value: 0x56,
            },
            Event::Output16 {
                port: 0xCD34,
                value: 0x78,
            },
            Event::Input {
                port: 0x12,
                value: 0x9A,
            },
            Event::Output {
                port: 0x34,
                value: 0xBC,
            },
        ]
    );
}

fn assert_single_byte_refusal(report: Report, acknowledge: bool) -> Unsupported {
    assert_eq!((report.steps, report.cycles), (0, 0));
    let Exit::Unsupported(unsupported) = report.exit else {
        panic!("expected Unsupported, got {:?}", report.exit);
    };
    assert_eq!(unsupported.pc, 0x1234);
    assert_eq!(unsupported.opcode, 0x08);
    assert_eq!(unsupported.acknowledge, acknowledge);
    assert_eq!(unsupported.preceding, [0, 0, 0]);
    assert_eq!(unsupported.preceding_len, 0);
    assert_eq!(unsupported.preceding(), &[]);
    unsupported
}

#[test]
fn i8080_undocumented_byte_has_no_preceding_bytes() {
    let mut cpu = I8080::default();
    cpu.registers_mut().pc = 0x1234;
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0x08]);
    assert_single_byte_refusal(cpu.step(&mut bus), false);
}

#[test]
fn i8080_rejected_acknowledge_has_no_preceding_bytes() {
    let mut cpu = I8080::default();
    cpu.registers_mut().pc = 0x1234;
    cpu.latches_mut().interrupt_enable = true;
    let mut bus = RecordingBus::new();
    bus.interrupt = true;
    bus.acknowledges.push_back(0x08);
    assert_single_byte_refusal(cpu.step(&mut bus), true);
}

#[test]
fn i8085_has_no_undocumented_opcode_refusal() {
    let mut cpu = I8085::default();
    cpu.registers_mut().pc = 0x1234;
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0x08]);
    // Decision-18 §§2–4 admits all 256 bytes: 08h now executes DSUB.
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 10,
            exit: Exit::Budget,
        }
    );
    assert_eq!(cpu.registers().pc, 0x1235);
}

#[test]
fn i8085_rejected_acknowledge_has_no_preceding_bytes() {
    let mut cpu = I8085::default();
    cpu.registers_mut().pc = 0x1234;
    cpu.latches_mut().interrupt_enable = true;
    let mut bus = RecordingBus::new();
    bus.intr = true;
    bus.acknowledges.push_back(0x08);
    assert_single_byte_refusal(cpu.step(&mut bus), true);
}

#[test]
fn preceding_accessor_returns_used_bytes_and_clamps_invalid_lengths() {
    let mut cpu = I8080::default();
    cpu.registers_mut().pc = 0x1234;
    let mut bus = RecordingBus::new();
    bus.load(0x1234, &[0x08]);
    let mut unsupported = assert_single_byte_refusal(cpu.step(&mut bus), false);
    // Synthetic diagnostic bytes, not an encoding of any model.
    for (preceding, preceding_len, expected) in [
        ([0, 0, 0], 0, &[][..]),
        ([0x12, 0, 0], 1, &[0x12][..]),
        ([0x12, 0x34, 0], 2, &[0x12, 0x34][..]),
        ([0x12, 0x34, 0x56], 3, &[0x12, 0x34, 0x56][..]),
    ] {
        unsupported.preceding = preceding;
        unsupported.preceding_len = preceding_len;
        assert_eq!(unsupported.preceding(), expected);
    }
    for preceding_len in 4..=u8::MAX {
        unsupported.preceding_len = preceding_len;
        assert_eq!(unsupported.preceding(), &[0x12, 0x34, 0x56]);
    }
}
