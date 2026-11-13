//! Decision-20 §§2–3 literal witnesses; no decoder or ALU helpers.
//! A canonical operations: Zilog UM008011-0816 rev. 11 (August 2016),
//! pp. 213–260; 03-0029-01 (1977) Tables 7.0-7/8 pp. 50–51.
//! H SLL/copies/BIT aliases: Young v0.91 (2005-09-18) §§3.1/3.5–3.6
//! pp. 11/13–14, §§8.7–8.8 pp. 32–33; Rak z80test v1.2a (2023-12-02)
//! .slia_a/.slia_r testtable 81–100, .sro_xyd_r 671–676,
//! .bit_n_xyd_ 833–838, .set_n_xyd_/.res_n_xyd_ 903–908.
//! L new costs/access/ROM copy: same Young sections and §6.1 p. 23;
//! no independent timing or write-ignoring-device capture. Pins/limits:
//! PROVENANCE §1 and decision-20 §§3–4. H shifted-byte Y/X and BIT S/PV:
//! Young §4.1 p. 15 / §8.7 p. 32; Rak .sro_xyd*, .bit_n_*. L register
//! BIT written-key gap; §5 L retained WZ / indexed WZ; H bits13/11 projection.

#![forbid(unsafe_code)]
mod support;
use support::{Event, RecordingBus};
use x80_core::z80::{Flags, Inputs, Registers, Z80};
use x80_core::{Bus, Control, Exit, MemoryAccess};
use MemoryAccess::{Data, OpcodeFetch as Op, OperandFetch as Arg};

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}
fn write(address: u16, value: u8) -> Event {
    Event::Write {
        address,
        kind: Data,
        value,
    }
}
fn polls() -> Vec<Event> {
    vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ]
}
fn set_register(r: &mut Registers, code: u8, value: u8) {
    match code {
        0 => r.b = value,
        1 => r.c = value,
        2 => r.d = value,
        3 => r.e = value,
        4 => r.h = value,
        5 => r.l = value,
        6 => {}
        7 => r.a = value,
        _ => unreachable!(),
    }
}
fn fixture() -> (Z80, RecordingBus) {
    let mut cpu = Z80::default();
    *cpu.registers_mut() = Registers {
        a: 0xA7,
        b: 0xB0,
        c: 0xC1,
        d: 0xD2,
        e: 0xE3,
        h: 0x64,
        l: 0x75,
        ix: 0x2001,
        iy: 0x3001,
        pc: 0xFFFE,
        r: 0xFF,
        flags: Flags::from_byte(0xD7),
        ..Registers::default()
    };
    (cpu, RecordingBus::new())
}
fn retire<S: Bus + Inputs>(cpu: &mut Z80, bus: &mut S, cycles: u64) {
    let report = cpu.step(bus);
    assert_eq!(
        (report.steps, report.cycles, report.exit),
        (1, cycles, Exit::Budget)
    );
}

// Literal output/F for old=81h, incoming C=1, in RLC/RRC/RL/RR/SLA/SRA/SLL/SRL order.
const SHIFT_81: [(u8, u8); 8] = [
    (0x03, 0x05),
    (0xC0, 0x85),
    (0x03, 0x05),
    (0xC0, 0x85),
    (0x02, 0x01),
    (0xC0, 0x85),
    (0x03, 0x05),
    (0x40, 0x01),
];
const RESET_81: [u8; 8] = [0x80, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0x01];
const SET_81: [u8; 8] = [0x81, 0x83, 0x85, 0x89, 0x91, 0xA1, 0xC1, 0x81];
// Decision-20 §4 H S/PV; L register Y/X; §5 BIT (HL) uses retained WZ.
const BIT_81: [u8; 8] = [0x11, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x91];

#[test]
fn all_256_cb_terminals_have_literal_results_flags_access_refresh_and_costs() {
    // A existing forms / H SLL / L SLL costs: citations in module header.
    for opcode in 0u8..=255 {
        let (mut cpu, mut bus) = fixture();
        let code = opcode & 7;
        set_register(cpu.registers_mut(), code, 0x81);
        if code == 6 {
            cpu.registers_mut().set_hl(0x8181);
        }
        bus.load(0x8181, &[0x81]);
        bus.load(0xFFFE, &[0xCB, opcode]);
        let mut expected = cpu.clone();
        expected.latches_mut().q = opcode < 0x80;
        expected.registers_mut().pc = 0;
        expected.registers_mut().r = 0x81;
        let bit = usize::from((opcode >> 3) & 7);
        let (value, flags) = match opcode >> 6 {
            0 => SHIFT_81[bit],
            1 => (0x81, BIT_81[bit]),
            2 => (RESET_81[bit], 0xD7),
            _ => (SET_81[bit], 0xD7),
        };
        expected.registers_mut().flags = Flags::from_byte(flags);
        if opcode >> 6 != 1 {
            set_register(expected.registers_mut(), code, value);
        }
        let mut trace = polls();
        trace.extend([read(0xFFFE, Op, 0xCB), read(0xFFFF, Op, opcode)]);
        if code == 6 {
            trace.push(read(0x8181, Data, 0x81));
            if opcode >> 6 != 1 {
                trace.push(write(0x8181, value));
            }
        }
        let cycles = if code != 6 {
            8
        } else if opcode >> 6 == 1 {
            12
        } else {
            15
        };
        retire(&mut cpu, &mut bus, cycles);
        assert_eq!(cpu, expected, "CB {opcode:02X}");
        assert_eq!(bus.memory[0x8181], if code == 6 { value } else { 0x81 });
        assert_eq!(bus.take_log(), trace, "CB {opcode:02X}");
    }
}

#[test]
fn sll_every_register_and_hl_literal_carry_sign_zero_and_parity() {
    // H operation/flags and L 8/15 costs: decision-20 §3, Y §3.1/§8.7,
    // R .slia_a/.slia_r. SLL can never set Z because result bit 0 is one.
    for (old, value, flags) in [
        (0x00, 0x01, 0x00),
        (0x80, 0x01, 0x01),
        (0x40, 0x81, 0x84),
        (0x7F, 0xFF, 0xAC),
        (0xFF, 0xFF, 0xAD),
        (0x01, 0x03, 0x04),
    ] {
        for code in 0..8 {
            for incoming in [0x00, 0xD7] {
                let (mut cpu, mut bus) = fixture();
                set_register(cpu.registers_mut(), code, old);
                if code == 6 {
                    cpu.registers_mut().set_hl(0x8181);
                }
                cpu.registers_mut().flags = Flags::from_byte(incoming);
                bus.load(0x8181, &[old]);
                bus.load(0xFFFE, &[0xCB, 0x30 + code]);
                let mut expected = cpu.clone();
                expected.latches_mut().q = true;
                expected.registers_mut().pc = 0;
                expected.registers_mut().r = 0x81;
                set_register(expected.registers_mut(), code, value);
                expected.registers_mut().flags = Flags::from_byte(flags);
                retire(&mut cpu, &mut bus, if code == 6 { 15 } else { 8 });
                assert_eq!(cpu, expected);
                assert!(!cpu.registers().flags.zero);
                if code == 6 {
                    assert_eq!(bus.memory[0x8181], value);
                }
            }
        }
    }
}

#[test]
fn all_256_ddcb_and_fdcb_terminals_copy_ordinary_registers_or_alias_bit() {
    // H copies/aliases, A documented forms, L costs/trace: decision-20 §3.
    for (prefix, address) in [(0xDD, 0x2000), (0xFD, 0x3000)] {
        for opcode in 0u8..=255 {
            let (mut cpu, mut bus) = fixture();
            bus.load(0xFFFE, &[prefix, 0xCB, 0xFF, opcode]);
            bus.load(address, &[0x81]);
            let mut expected = cpu.clone();
            expected.latches_mut().q = opcode < 0x80;
            expected.latches_mut().wz = address;
            expected.registers_mut().pc = 2;
            expected.registers_mut().r = 0x81;
            let bit = usize::from((opcode >> 3) & 7);
            let (value, flags) = match opcode >> 6 {
                0 => SHIFT_81[bit],
                1 => (0x81, BIT_81[bit] | 0x20), // §4: EA high=20/30, Y set.
                2 => (RESET_81[bit], 0xD7),
                _ => (SET_81[bit], 0xD7),
            };
            expected.registers_mut().flags = Flags::from_byte(flags);
            if opcode >> 6 != 1 {
                set_register(expected.registers_mut(), opcode & 7, value);
            }
            let mut trace = polls();
            trace.extend([
                read(0xFFFE, Op, prefix),
                read(0xFFFF, Op, 0xCB),
                read(0, Arg, 0xFF),
                read(1, Arg, opcode),
                read(address, Data, 0x81),
            ]);
            if opcode >> 6 != 1 {
                trace.push(write(address, value));
            }
            retire(&mut cpu, &mut bus, if opcode >> 6 == 1 { 20 } else { 23 });
            assert_eq!(cpu, expected, "{prefix:02X} CB FF {opcode:02X}");
            assert_eq!(bus.memory[usize::from(address)], value);
            assert_eq!(bus.take_log(), trace);
            if opcode >> 6 == 1 {
                let (mut canonical, mut ram) = fixture();
                ram.load(0xFFFE, &[prefix, 0xCB, 0xFF, (opcode & 0xF8) | 6]);
                ram.load(address, &[0x81]);
                retire(&mut canonical, &mut ram, 20);
                assert_eq!(cpu, canonical, "BIT alias {opcode:02X}");
            }
        }
    }
}

// A device records a write but deliberately retains its original byte.
struct ReadOnly(RecordingBus);
impl Bus for ReadOnly {
    fn read(&mut self, address: u16, kind: MemoryAccess) -> u8 {
        self.0.read(address, kind)
    }
    fn write(&mut self, address: u16, value: u8, kind: MemoryAccess) {
        assert_eq!(kind, Data);
        self.0.log.push(write(address, value));
    }
    fn input(&mut self, _: u8) -> u8 {
        panic!("unexpected input")
    }
    fn output(&mut self, _: u8, _: u8) {
        panic!("unexpected output")
    }
}
impl Control for ReadOnly {
    fn stop_requested(&mut self) -> bool {
        self.0.stop_requested()
    }
}
impl Inputs for ReadOnly {
    fn nmi(&mut self) -> bool {
        Inputs::nmi(&mut self.0)
    }
    fn int(&mut self) -> bool {
        Inputs::int(&mut self.0)
    }
}

#[test]
fn write_ignoring_device_still_copies_computed_result_without_reread() {
    // L decision-20 §3: Y §§3.5–3.6 pp. 13–14, R only tests writable RAM.
    for prefix in [0xDD, 0xFD] {
        for (opcode, result, flags) in [(0x34, 0x03, 0x05), (0x85, 0x80, 0xD7), (0xCF, 0x83, 0xD7)]
        {
            let (mut cpu, ram) = fixture();
            cpu.registers_mut().ix = 0x2001;
            cpu.registers_mut().iy = 0x2001;
            let mut bus = ReadOnly(ram);
            bus.0.load(0xFFFE, &[prefix, 0xCB, 0xFF, opcode]);
            bus.0.load(0x2000, &[0x81]);
            let mut expected = cpu.clone();
            expected.latches_mut().q = opcode < 0x80;
            expected.latches_mut().wz = 0x2000;
            expected.registers_mut().pc = 2;
            expected.registers_mut().r = 0x81;
            set_register(expected.registers_mut(), opcode & 7, result);
            expected.registers_mut().flags = Flags::from_byte(flags);
            retire(&mut cpu, &mut bus, 23);
            assert_eq!(cpu, expected);
            assert_eq!(bus.0.memory[0x2000], 0x81);
            let mut trace = polls();
            trace.extend([
                read(0xFFFE, Op, prefix),
                read(0xFFFF, Op, 0xCB),
                read(0, Arg, 0xFF),
                read(1, Arg, opcode),
                read(0x2000, Data, 0x81),
                write(0x2000, result),
            ]);
            assert_eq!(bus.0.take_log(), trace);
        }
    }
}

#[test]
fn repeated_selectors_into_indexed_sll_use_last_index_one_boundary_and_extra_m1_costs() {
    // L decision-20 §2 mapping/chains; H SLL/copy §3, sources above.
    let (mut cpu, mut bus) = fixture();
    bus.load(0xFFFE, &[0xDD, 0xFD, 0xDD, 0xFD, 0xCB, 0xFF, 0x35]);
    bus.load(0x3000, &[0x81]);
    let mut expected = cpu.clone();
    expected.latches_mut().q = true;
    expected.latches_mut().wz = 0x3000;
    expected.registers_mut().pc = 5;
    expected.registers_mut().r = 0x84;
    expected.registers_mut().l = 3;
    expected.registers_mut().flags = Flags::from_byte(5);
    retire(&mut cpu, &mut bus, 35);
    assert_eq!(cpu, expected);
    assert_eq!(bus.memory[0x3000], 3);
    let mut trace = polls();
    trace.extend([
        read(0xFFFE, Op, 0xDD),
        read(0xFFFF, Op, 0xFD),
        read(0, Op, 0xDD),
        read(1, Op, 0xFD),
        read(2, Op, 0xCB),
        read(3, Arg, 0xFF),
        read(4, Arg, 0x35),
        read(0x3000, Data, 0x81),
        write(0x3000, 3),
    ]);
    assert_eq!(bus.take_log(), trace);
}
