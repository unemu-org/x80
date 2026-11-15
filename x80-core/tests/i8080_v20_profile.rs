//! V20 integration-profile witnesses through the public 8080 API (decision-9).
//!
//! NEC uPD70108/uPD70116 User's Manual, Stock No. 500350, October 1986:
//! pp. 8-1, 8-2, 8-4, 9-1, 12-176 and 12-177 (PDF 32, 33, 35, 38, 221,
//! 222), checked against rendered page images. These witness the adapter
//! boundary, not native-mode execution or V20 clocks. The profile uses the
//! Intel 8080 model; its instruction encodings, effects and T-states below
//! follow Intel 98-153B, September 1975, Ch. 4 pp. 4-4 to 4-6 and 4-11 to
//! 4-14; stack bus order follows Ch. 2 pp. 2-18 to 2-20.

use x80_core::i8080::{Flags, Inputs, Latches, Registers, I8080};
use x80_core::{Budget, Bus, Control, Exit, MemoryAccess, Report, Unsupported};

const UNLIMITED: Budget = Budget {
    steps: u64::MAX,
    cycles: u64::MAX,
};

const RUNNING: Latches = Latches {
    interrupt_enable: false,
    ei_delay: false,
    halted: false,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    Read(u16, MemoryAccess, u8),
    Write(u16, MemoryAccess, u8),
    Input(u8, u8),
    Output(u8, u8),
    StopPoll(bool),
    InterruptPoll(bool),
}

/// NEC p. 8-1 / decision-9: instruction bytes use PS; memory operands,
/// including the emulated stack, use DS0. Poison unused locations differently
/// so an incorrect access kind cannot silently read the same byte.
struct SegmentedBus {
    ps: Box<[u8; 0x1_0000]>,
    ds0: Box<[u8; 0x1_0000]>,
    log: Vec<Event>,
    stop: bool,
    stop_on_write: Option<u16>,
}

impl SegmentedBus {
    fn new() -> Self {
        Self {
            ps: Box::new([0xDD; 0x1_0000]),
            ds0: Box::new([0xA5; 0x1_0000]),
            log: Vec::new(),
            stop: false,
            stop_on_write: None,
        }
    }

    fn load_program(&mut self, pc: u16, bytes: &[u8]) {
        for (offset, &value) in bytes.iter().enumerate() {
            self.ps[usize::from(pc.wrapping_add(offset as u16))] = value;
        }
    }

    fn take_log(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.log)
    }
}

impl Bus for SegmentedBus {
    fn read(&mut self, address: u16, kind: MemoryAccess) -> u8 {
        let value = match kind {
            MemoryAccess::OpcodeFetch | MemoryAccess::OperandFetch => self.ps[usize::from(address)],
            MemoryAccess::Data | MemoryAccess::Stack => self.ds0[usize::from(address)],
            MemoryAccess::InterruptAcknowledge => panic!("V20 profile must not acknowledge INT"),
            _ => panic!("unexpected access kind: {kind:?}"),
        };
        self.log.push(Event::Read(address, kind, value));
        value
    }

    fn write(&mut self, address: u16, value: u8, kind: MemoryAccess) {
        assert!(matches!(kind, MemoryAccess::Data | MemoryAccess::Stack));
        self.ds0[usize::from(address)] = value;
        self.log.push(Event::Write(address, kind, value));
        if self.stop_on_write == Some(address) {
            self.stop = true;
        }
    }

    fn input(&mut self, port: u8) -> u8 {
        assert_eq!(port, 0x23);
        self.log.push(Event::Input(port, 0xA6));
        0xA6
    }

    fn output(&mut self, port: u8, value: u8) {
        self.log.push(Event::Output(port, value));
    }
}

impl Control for SegmentedBus {
    fn stop_requested(&mut self) -> bool {
        self.log.push(Event::StopPoll(self.stop));
        self.stop
    }
}

impl Inputs for SegmentedBus {
    fn interrupt_requested(&mut self) -> bool {
        // Native interrupt entry is the adapter's job (decision-9, NEC p. 8-4).
        self.log.push(Event::InterruptPoll(false));
        false
    }
}

fn registers_at(pc: u16) -> Registers {
    Registers {
        a: 0xA1,
        flags: Flags {
            sign: true,
            zero: false,
            auxiliary_carry: true,
            parity: false,
            carry: true,
        },
        b: 0xB2,
        c: 0xC3,
        d: 0xD4,
        e: 0xE5,
        h: 0xF6,
        l: 0x07,
        sp: 0x9000,
        pc,
    }
}

fn polls(stop: bool) -> [Event; 2] {
    [Event::StopPoll(stop), Event::InterruptPoll(false)]
}

/// Expected trace from literal instruction bytes and independently specified
/// data/stack/I/O events; never decodes an opcode using the crate.
fn instruction(trace: &mut Vec<Event>, pc: u16, bytes: &[u8], accesses: &[Event]) {
    trace.extend(polls(false));
    trace.push(Event::Read(pc, MemoryAccess::OpcodeFetch, bytes[0]));
    for (offset, &value) in bytes.iter().enumerate().skip(1) {
        trace.push(Event::Read(
            pc.wrapping_add(offset as u16),
            MemoryAccess::OperandFetch,
            value,
        ));
    }
    trace.extend_from_slice(accesses);
}

fn assert_native_escape_unsupported(bytes: &[u8]) {
    for pc in [0x2345, 0xFFFF] {
        let mut bus = SegmentedBus::new();
        bus.load_program(pc, bytes);
        let ps = bus.ps.clone();
        let ds0 = bus.ds0.clone();
        let registers = registers_at(pc);
        let latches = Latches {
            interrupt_enable: true,
            ei_delay: true,
            halted: false,
        };
        let mut cpu = I8080::new(registers, latches);

        for _ in 0..2 {
            let report = cpu.run(&mut bus, UNLIMITED);
            assert_eq!((report.steps, report.cycles), (0, 0));
            assert!(matches!(
                report.exit,
                Exit::Unsupported(Unsupported {
                    pc: at,
                    opcode: 0xED,
                    acknowledge: false,
                    ..
                }) if at == pc
            ));
            assert_eq!(*cpu.registers(), registers);
            assert_eq!(*cpu.latches(), latches);
            assert_eq!(bus.ps, ps);
            assert_eq!(bus.ds0, ds0);
            let mut trace = Vec::new();
            // Only the ED is fetched: the native core decodes it again with its
            // suffix (decision-9 amendment of 2026-11-15, option B).
            instruction(&mut trace, pc, &[0xED], &[]);
            assert_eq!(bus.take_log(), trace);
        }
    }
}

#[test]
fn retem_returns_retryable_unsupported_without_reading_fd() {
    // decision-9; NEC Ch. 12 RETEM, p. 12-177: ED FD.
    assert_native_escape_unsupported(&[0xED, 0xFD]);
}

#[test]
fn calln_returns_retryable_unsupported_without_reading_suffix_or_vector() {
    // decision-9; NEC Ch. 12 CALLN, p. 12-176: ED ED imm8.
    assert_native_escape_unsupported(&[0xED, 0xED, 0x40]);
}

#[test]
fn mixed_program_routes_instruction_bytes_to_ps_and_data_and_stack_to_ds0() {
    // decision-9; NEC p. 8-1: PS holds the program, DS0 all memory operands
    // including the stack. Intel Ch. 4 pages cited above supply the program's
    // 8080 semantics; its 226 T-states are not NEC clocks.
    use MemoryAccess::{Data, Stack};

    let mut bus = SegmentedBus::new();
    bus.load_program(
        0x0100,
        &[
            0x31, 0x00, 0x90, // 0100: LXI SP,9000
            0x3E, 0x11, // 0103: MVI A,11
            0x32, 0x03, 0x01, // 0105: STA 0103 (DS0 only)
            0x3A, 0x03, 0x01, // 0108: LDA 0103 (reads DS0, not code)
            0x21, 0x00, 0x40, // 010B: LXI H,4000
            0x36, 0x22, // 010E: MVI M,22
            0x46, // 0110: MOV B,M
            0x77, // 0111: MOV M,A
            0x86, // 0112: ADD M (11 + 11 = 22)
            0xC5, // 0113: PUSH B
            0xD1, // 0114: POP D
            0xCD, 0x00, 0x02, // 0115: CALL 0200
            0x2A, 0x00, 0x41, // 0118: LHLD 4100
            0x22, 0x00, 0x42, // 011B: SHLD 4200
            0x21, 0x78, 0x56, // 011E: LXI H,5678
            0xE3, // 0121: XTHL
            0xDB, 0x23, // 0122: IN 23
            0xD3, 0x24, // 0124: OUT 24
            0x76, // 0126: HLT
        ],
    );
    bus.load_program(0x0200, &[0x3E, 0x5A, 0xC9]); // MVI A,5A; RET
    bus.ds0[0x4100] = 0x34;
    bus.ds0[0x4101] = 0x12;
    bus.ds0[0x9000] = 0xBC;
    bus.ds0[0x9001] = 0x9A;
    let ps = bus.ps.clone();
    let mut expected_ds0 = bus.ds0.clone();
    for (address, value) in [
        (0x0103, 0x11),
        (0x4000, 0x11),
        (0x8FFF, 0x01),
        (0x8FFE, 0x18),
        (0x4200, 0x34),
        (0x4201, 0x12),
        (0x9000, 0x78),
        (0x9001, 0x56),
    ] {
        expected_ds0[address] = value;
    }
    let mut cpu = I8080::new(
        Registers {
            pc: 0x0100,
            ..Registers::default()
        },
        RUNNING,
    );

    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 21,
            cycles: 226,
            exit: Exit::Halt,
        }
    );
    assert_eq!(
        *cpu.registers(),
        Registers {
            a: 0xA6,
            b: 0x22,
            c: 0x00,
            d: 0x22,
            e: 0x00,
            h: 0x9A,
            l: 0xBC,
            flags: Flags {
                sign: false,
                zero: false,
                auxiliary_carry: false,
                parity: true,
                carry: false,
            },
            sp: 0x9000,
            pc: 0x0127,
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            halted: true,
            ..RUNNING
        }
    );
    assert_eq!(bus.ps, ps);
    assert_eq!(bus.ds0, expected_ds0);
    assert_eq!((bus.ps[0x0103], bus.ds0[0x0103]), (0x3E, 0x11));

    let mut trace = Vec::new();
    instruction(&mut trace, 0x0100, &[0x31, 0x00, 0x90], &[]);
    instruction(&mut trace, 0x0103, &[0x3E, 0x11], &[]);
    instruction(
        &mut trace,
        0x0105,
        &[0x32, 0x03, 0x01],
        &[Event::Write(0x0103, Data, 0x11)],
    );
    instruction(
        &mut trace,
        0x0108,
        &[0x3A, 0x03, 0x01],
        &[Event::Read(0x0103, Data, 0x11)],
    );
    instruction(&mut trace, 0x010B, &[0x21, 0x00, 0x40], &[]);
    instruction(
        &mut trace,
        0x010E,
        &[0x36, 0x22],
        &[Event::Write(0x4000, Data, 0x22)],
    );
    instruction(
        &mut trace,
        0x0110,
        &[0x46],
        &[Event::Read(0x4000, Data, 0x22)],
    );
    instruction(
        &mut trace,
        0x0111,
        &[0x77],
        &[Event::Write(0x4000, Data, 0x11)],
    );
    instruction(
        &mut trace,
        0x0112,
        &[0x86],
        &[Event::Read(0x4000, Data, 0x11)],
    );
    instruction(
        &mut trace,
        0x0113,
        &[0xC5],
        &[
            Event::Write(0x8FFF, Stack, 0x22),
            Event::Write(0x8FFE, Stack, 0x00),
        ],
    );
    instruction(
        &mut trace,
        0x0114,
        &[0xD1],
        &[
            Event::Read(0x8FFE, Stack, 0x00),
            Event::Read(0x8FFF, Stack, 0x22),
        ],
    );
    instruction(
        &mut trace,
        0x0115,
        &[0xCD, 0x00, 0x02],
        &[
            Event::Write(0x8FFF, Stack, 0x01),
            Event::Write(0x8FFE, Stack, 0x18),
        ],
    );
    instruction(&mut trace, 0x0200, &[0x3E, 0x5A], &[]);
    instruction(
        &mut trace,
        0x0202,
        &[0xC9],
        &[
            Event::Read(0x8FFE, Stack, 0x18),
            Event::Read(0x8FFF, Stack, 0x01),
        ],
    );
    instruction(
        &mut trace,
        0x0118,
        &[0x2A, 0x00, 0x41],
        &[
            Event::Read(0x4100, Data, 0x34),
            Event::Read(0x4101, Data, 0x12),
        ],
    );
    instruction(
        &mut trace,
        0x011B,
        &[0x22, 0x00, 0x42],
        &[
            Event::Write(0x4200, Data, 0x34),
            Event::Write(0x4201, Data, 0x12),
        ],
    );
    instruction(&mut trace, 0x011E, &[0x21, 0x78, 0x56], &[]);
    instruction(
        &mut trace,
        0x0121,
        &[0xE3],
        &[
            Event::Read(0x9000, Stack, 0xBC),
            Event::Read(0x9001, Stack, 0x9A),
            Event::Write(0x9001, Stack, 0x56),
            Event::Write(0x9000, Stack, 0x78),
        ],
    );
    instruction(
        &mut trace,
        0x0122,
        &[0xDB, 0x23],
        &[Event::Input(0x23, 0xA6)],
    );
    instruction(
        &mut trace,
        0x0124,
        &[0xD3, 0x24],
        &[Event::Output(0x24, 0xA6)],
    );
    instruction(&mut trace, 0x0126, &[0x76], &[]);
    trace.extend(polls(false));
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn push_psw_stores_the_v20_flag_layout_for_all_flag_combinations() {
    // decision-9; NEC Fig. 8-2, p. 8-1: S Z 0 AC 0 P 1 CY.
    // Intel 98-153B p. 4-13: PUSH PSW = F5, A at SP-1, flags at SP-2.
    // Literal flag bytes independently enumerate that diagram, CY fastest.
    let expected_bytes = [
        0x02, 0x03, 0x06, 0x07, 0x12, 0x13, 0x16, 0x17, 0x42, 0x43, 0x46, 0x47, 0x52, 0x53, 0x56,
        0x57, 0x82, 0x83, 0x86, 0x87, 0x92, 0x93, 0x96, 0x97, 0xC2, 0xC3, 0xC6, 0xC7, 0xD2, 0xD3,
        0xD6, 0xD7,
    ];
    for (bits, expected) in expected_bytes.into_iter().enumerate() {
        let mut bus = SegmentedBus::new();
        bus.load_program(0x0100, &[0xF5]);
        let ps = bus.ps.clone();
        let mut ds0 = bus.ds0.clone();
        ds0[0x8FFF] = 0xA1;
        ds0[0x8FFE] = expected;
        let registers = Registers {
            flags: Flags {
                sign: bits & 16 != 0,
                zero: bits & 8 != 0,
                auxiliary_carry: bits & 4 != 0,
                parity: bits & 2 != 0,
                carry: bits & 1 != 0,
            },
            ..registers_at(0x0100)
        };
        let mut cpu = I8080::new(registers, RUNNING);
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 1,
                cycles: 11,
                exit: Exit::Budget
            }
        );
        assert_eq!(
            *cpu.registers(),
            Registers {
                pc: 0x0101,
                sp: 0x8FFE,
                ..registers
            }
        );
        assert_eq!(*cpu.latches(), RUNNING);
        assert_eq!(bus.ps, ps);
        assert_eq!(bus.ds0, ds0);
        let mut trace = Vec::new();
        instruction(
            &mut trace,
            0x0100,
            &[0xF5],
            &[
                Event::Write(0x8FFF, MemoryAccess::Stack, 0xA1),
                Event::Write(0x8FFE, MemoryAccess::Stack, expected),
            ],
        );
        assert_eq!(bus.take_log(), trace);
    }
}

#[test]
fn host_stop_returns_complete_boundary_state_and_the_next_run_continues() {
    // decision-9; NEC p. 8-4, NMI or INT / Fig. 8-6: native interrupt
    // service belongs outside the 8080 core. Host Stop is the handoff;
    // this witnesses its boundary state, not a native interrupt frame.
    let mut bus = SegmentedBus::new();
    bus.load_program(
        0x0100,
        &[
            0x3E, 0x2A, // MVI A,2A
            0x32, 0x00, 0x40, // STA 4000, requests host stop during the write
            0x3E, 0x7C, // MVI A,7C
            0x32, 0x01, 0x40, // STA 4001
            0x76, // HLT
        ],
    );
    bus.stop_on_write = Some(0x4000);
    let ps = bus.ps.clone();
    let mut ds0 = bus.ds0.clone();
    ds0[0x4000] = 0x2A;
    let registers = registers_at(0x0100);
    let latches = Latches {
        interrupt_enable: true,
        ..RUNNING
    };
    let mut cpu = I8080::new(registers, latches);

    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 2,
            cycles: 20,
            exit: Exit::Stop
        }
    );
    assert_eq!(
        *cpu.registers(),
        Registers {
            a: 0x2A,
            pc: 0x0105,
            ..registers
        }
    );
    assert_eq!(*cpu.latches(), latches);
    assert_eq!(bus.ps, ps);
    assert_eq!(bus.ds0, ds0);
    let mut trace = Vec::new();
    instruction(&mut trace, 0x0100, &[0x3E, 0x2A], &[]);
    instruction(
        &mut trace,
        0x0102,
        &[0x32, 0x00, 0x40],
        &[Event::Write(0x4000, MemoryAccess::Data, 0x2A)],
    );
    trace.extend(polls(true));
    assert_eq!(bus.take_log(), trace);

    bus.stop = false;
    bus.stop_on_write = None;
    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 3,
            cycles: 27,
            exit: Exit::Halt
        }
    );
    assert_eq!(
        *cpu.registers(),
        Registers {
            a: 0x7C,
            pc: 0x010B,
            ..registers
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            halted: true,
            ..latches
        }
    );
    ds0[0x4001] = 0x7C;
    assert_eq!(bus.ps, ps);
    assert_eq!(bus.ds0, ds0);
    let mut trace = Vec::new();
    instruction(&mut trace, 0x0105, &[0x3E, 0x7C], &[]);
    instruction(
        &mut trace,
        0x0107,
        &[0x32, 0x01, 0x40],
        &[Event::Write(0x4001, MemoryAccess::Data, 0x7C)],
    );
    instruction(&mut trace, 0x010A, &[0x76], &[]);
    trace.extend(polls(false));
    assert_eq!(bus.take_log(), trace);
}

#[test]
fn adapter_can_release_hlt_standby_with_ie_clear_and_resume_after_hlt() {
    // decision-9; NEC pp. 8-2 and 9-1, INT with interrupts disabled:
    // resume emulation at the instruction after HLT, without native service.
    // The adapter handles that INT and clears the public halted latch;
    // the core's own INT input remains low throughout.
    let mut bus = SegmentedBus::new();
    bus.load_program(0x3000, &[0x76, 0x3E, 0x5A, 0x76]); // HLT; MVI A,5A; HLT
    let ps = bus.ps.clone();
    let ds0 = bus.ds0.clone();
    let registers = registers_at(0x3000);
    let mut cpu = I8080::new(registers, RUNNING);
    let halted = Latches {
        halted: true,
        ..RUNNING
    };

    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 1,
            cycles: 7,
            exit: Exit::Halt
        }
    );
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x3001,
            ..registers
        }
    );
    assert_eq!(*cpu.latches(), halted);
    let mut trace = Vec::new();
    instruction(&mut trace, 0x3000, &[0x76], &[]);
    trace.extend(polls(false));
    assert_eq!(bus.take_log(), trace);

    // Still halted until the adapter releases it; no following byte is read.
    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 0,
            cycles: 0,
            exit: Exit::Halt
        }
    );
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x3001,
            ..registers
        }
    );
    assert_eq!(*cpu.latches(), halted);
    assert_eq!(bus.take_log(), polls(false));
    assert_eq!(bus.ps, ps);
    assert_eq!(bus.ds0, ds0);

    cpu.latches_mut().halted = false;
    assert_eq!(*cpu.latches(), RUNNING);
    assert_eq!(
        cpu.run(&mut bus, UNLIMITED),
        Report {
            steps: 2,
            cycles: 14,
            exit: Exit::Halt
        }
    );
    assert_eq!(
        *cpu.registers(),
        Registers {
            a: 0x5A,
            pc: 0x3004,
            ..registers
        }
    );
    assert_eq!(*cpu.latches(), halted);
    assert_eq!(bus.ps, ps);
    assert_eq!(bus.ds0, ds0);
    let mut trace = Vec::new();
    instruction(&mut trace, 0x3001, &[0x3E, 0x5A], &[]);
    instruction(&mut trace, 0x3003, &[0x76], &[]);
    trace.extend(polls(false));
    assert_eq!(bus.take_log(), trace);
}
