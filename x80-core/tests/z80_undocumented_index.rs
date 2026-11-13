//! Decision-20 §2 witnesses, independent of private maps/helpers.
//!
//! H: Young, The Undocumented Z80 Documented v0.91 (2005-09-18),
//! §3.2 pp. 11–12 / §§8.1/8.4 pp. 26/30; Rak z80test v1.2a (2023-12-02),
//! .inc_x/.dec_x, .ld_x_n/.ld_x_x, .alo_a_x/.alo_a_y.
//! A/H memory exceptions: UM008011-0816 rev. 11 pp. 78–85;
//! Young §3.2; Rak .ld_r_xyd/.ld_xyd_r. L: all new costs, general ignoring,
//! arbitrary chains, access/boundary mapping (Young §§3.2/3.7 pp. 11–14,
//! §6.1 p. 23; no independent timing/chain/interrupt acquisition).
//! Source hashes and specimen limits remain in PROVENANCE §1 / decision-20.
//! Six defined flag bits are asserted here; Y/X await decision-20 §4.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, Inputs, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Bus, Control, Exit, MemoryAccess};
use MemoryAccess::{OpcodeFetch as Op, OperandFetch as Arg};

fn fixture(prefix: u8, opcode: u8) -> (Z80, RecordingBus) {
    let (ix, iy) = if prefix == 0xDD {
        (0x3000, 0x4567)
    } else {
        (0x4567, 0x3000)
    };
    let cpu = Z80::new(
        Registers {
            pc: 0x1000,
            sp: 0x9000,
            a: 0x41,
            b: 0x11,
            c: 0x22,
            d: 0x22,
            e: 0x33,
            h: 0x12,
            l: 0x34,
            ix,
            iy,
            flags: Flags::from_byte(1),
            ..Registers::default()
        },
        Latches::default(),
    );
    let mut bus = RecordingBus::new();
    bus.load(0x1000, &[prefix, opcode, 0x01, 0x20]);
    bus.load(0x2001, &[0x9A, 0xBC]);
    bus.load(0x3001, &[0x81]);
    bus.load(0x9000, &[0x9A, 0xBC]);
    bus.inputs.push_back(0x42);
    (cpu, bus)
}

fn selected(cpu: &Z80, prefix: u8) -> u16 {
    if prefix == 0xDD {
        cpu.registers().ix
    } else {
        cpu.registers().iy
    }
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

// Literal full 16x16 classification: I ignored, H half substitution,
// M displaced memory with ordinary H/L, W documented word/index, P dispatch.
// Decision-20 §2: H substitution, A/H exceptions, L general ignoring.
const MAP: [&[u8; 16]; 16] = [
    b"IIIIIIIIIWIIIIII",
    b"IIIIIIIIIWIIIIII",
    b"IWWWHHHIIWWWHHHI",
    b"IIIIMMMIIWIIIIII",
    b"IIIIHHMIIIIIHHMI",
    b"IIIIHHMIIIIIHHMI",
    b"HHHHHHMHHHHHHHMH",
    b"MMMMMMIMIIIIHHMI",
    b"IIIIHHMIIIIIHHMI",
    b"IIIIHHMIIIIIHHMI",
    b"IIIIHHMIIIIIHHMI",
    b"IIIIHHMIIIIIHHMI",
    b"IIIIIIIIIIIPIIII",
    b"IIIIIIIIIIIIIPII",
    b"IWIWIWIIIWIIIPII",
    b"IIIIIIIIIWIIIPII",
];

#[test]
fn complete_main_substitution_and_exclusion_map_for_both_selectors() {
    let mut counts = [0; 5];
    for prefix in [0xDD, 0xFD] {
        for op in 0..=255u8 {
            let class = MAP[usize::from(op >> 4)][usize::from(op & 15)];
            let (mut cpu, mut bus) = fixture(prefix, op);
            let before = cpu.clone();
            match class {
                b'I' => {
                    // The L rule retains the independently witnessed MAIN
                    // instruction, including its own immediates/flow/ports.
                    // Same terminal address makes relative PCs/stack writes
                    // identical; only prefix fetch, R and +4 states differ.
                    let mut main = before.clone();
                    main.registers_mut().pc = 0x1001;
                    let (_, mut main_bus) = fixture(prefix, op);
                    let expected = main.step(&mut main_bus);
                    let actual = cpu.step(&mut bus);
                    assert_eq!(
                        (actual.exit, actual.steps, actual.cycles),
                        (expected.exit, 1, expected.cycles + 4),
                        "{prefix:02X} {op:02X}"
                    );
                    main.registers_mut().r = main.registers().r.wrapping_add(1);
                    assert_eq!(cpu, main, "ignored {prefix:02X} {op:02X}");
                    assert_eq!(bus.memory, main_bus.memory);
                    let mut expected_trace = main_bus.take_log();
                    expected_trace.insert(3, read(0x1000, Op, prefix));
                    assert_eq!(bus.take_log(), expected_trace);
                    counts[0] += 1;
                }
                b'H' => {
                    counts[1] += 1;
                } // literal results in next witnesses
                b'M' => {
                    let report = cpu.step(&mut bus);
                    assert_eq!((report.exit, report.steps), (Exit::Budget, 1));
                    assert_eq!(cpu.registers().pc, if op == 0x36 { 0x1004 } else { 0x1003 });
                    assert_eq!(
                        report.cycles,
                        if [0x34, 0x35].contains(&op) { 23 } else { 19 }
                    );
                    assert_eq!(selected(&cpu, prefix), 0x3000);
                    let trace = bus.take_log();
                    assert_eq!(
                        &trace[3..6],
                        &[
                            read(0x1000, Op, prefix),
                            read(0x1001, Op, op),
                            read(0x1002, Arg, 1)
                        ]
                    );
                    match op {
                        0x66 => assert_eq!((cpu.registers().h, cpu.registers().l), (0x81, 0x34)),
                        0x6E => assert_eq!((cpu.registers().h, cpu.registers().l), (0x12, 0x81)),
                        0x74 => assert_eq!(bus.memory[0x3001], 0x12),
                        0x75 => assert_eq!(bus.memory[0x3001], 0x34),
                        _ => assert_eq!((cpu.registers().h, cpu.registers().l), (0x12, 0x34)),
                    }
                    counts[2] += 1;
                }
                b'W' => {
                    let report = cpu.step(&mut bus);
                    let (word, states) = match op {
                        0x09 => (0x4122, 15),
                        0x19 => (0x5233, 15),
                        0x21 => (0x2001, 14),
                        0x22 => (0x3000, 20),
                        0x23 => (0x3001, 10),
                        0x29 => (0x6000, 15),
                        0x2A => (0xBC9A, 20),
                        0x2B => (0x2FFF, 10),
                        0x39 => (0xC000, 15),
                        0xE1 => (0xBC9A, 14),
                        0xE3 => (0xBC9A, 23),
                        0xE5 => (0x3000, 15),
                        0xE9 => (0x3000, 8),
                        0xF9 => (0x3000, 10),
                        _ => panic!("bad literal word map {op:02X}"),
                    };
                    assert_eq!(
                        (report.exit, report.steps, report.cycles),
                        (Exit::Budget, 1, states)
                    );
                    assert_eq!(selected(&cpu, prefix), word, "{prefix:02X} {op:02X}");
                    assert_eq!(cpu.registers().hl(), 0x1234);
                    counts[3] += 1;
                }
                b'P' => {
                    // Literal dispatch continuations: CB d=1, documented
                    // RLC memory=06; ED consumes 01 as a hole; DD/FD chain
                    // continues to 01 (LD BC,nn), never a retirement itself.
                    bus.load(0x1002, &[0x01, 0x06, 0x20]);
                    assert_eq!(cpu.step(&mut bus).exit, Exit::Budget);
                    counts[4] += 1;
                }
                _ => panic!("bad map class"),
            }
        }
    }
    assert_eq!(counts, [334, 92, 50, 28, 8]);
}

#[test]
fn all_register_ld_combinations_substitute_halves_including_cross_and_self_loads() {
    // H operation / L costs: decision-20 §2, Young §8.1 p. 26;
    // Rak .ld_x_x and .ld_x_n. Literal values distinguish ordinary H/L,
    // selected halves, other index and A/B/C/D/E.
    for prefix in [0xDD, 0xFD] {
        for (dst, field_dst) in [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 7)] {
            for (src, field_src) in [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 7)] {
                let (mut cpu, mut bus) = fixture(prefix, 0x40 + 8 * field_dst + field_src);
                if prefix == 0xDD {
                    cpu.registers_mut().ix = 0x7F80;
                } else {
                    cpu.registers_mut().iy = 0x7F80;
                }
                let mut expected = [0x11, 0x22, 0x22, 0x33, 0x7F, 0x80, 0x41];
                expected[dst] = expected[src];
                let report = cpu.step(&mut bus);
                let r = cpu.registers();
                let word = selected(&cpu, prefix);
                assert_eq!(
                    [r.b, r.c, r.d, r.e, (word >> 8) as u8, word as u8, r.a],
                    expected
                );
                assert_eq!(
                    (r.hl(), r.flags.to_byte(), r.pc, r.r),
                    (0x1234, 1, 0x1002, 2)
                );
                assert_eq!(
                    (report.steps, report.cycles, report.exit),
                    (1, 8, Exit::Budget)
                );
                assert_eq!(if prefix == 0xDD { r.iy } else { r.ix }, 0x4567);
            }
        }
        for (op, word) in [(0x26, 0x0180), (0x2E, 0x7F01)] {
            let (mut cpu, mut bus) = fixture(prefix, op);
            if prefix == 0xDD {
                cpu.registers_mut().ix = 0x7F80;
            } else {
                cpu.registers_mut().iy = 0x7F80;
            }
            let report = cpu.step(&mut bus);
            assert_eq!(
                (selected(&cpu, prefix), cpu.registers().hl()),
                (word, 0x1234)
            );
            assert_eq!(
                (report.steps, report.cycles, cpu.registers().pc),
                (1, 11, 0x1003)
            );
            assert_eq!(
                &bus.log[3..],
                &[
                    read(0x1000, Op, prefix),
                    read(0x1001, Op, op),
                    read(0x1002, Arg, 1)
                ]
            );
        }
    }
}

#[test]
fn index_half_inc_dec_and_all_alu_operations_have_literal_results_and_flags() {
    // H operation/flags, L 8 states: decision-20 §2, Young §8.4 p. 30;
    // Rak .inc_x/.dec_x and .alo_a_x/.alo_a_y. Incoming C=1.
    for prefix in [0xDD, 0xFD] {
        for (op, word, a, flags) in [
            (0x24, 0x8080, 0x41, 0x95),
            (0x25, 0x7E80, 0x41, 0x03),
            (0x2C, 0x7F81, 0x41, 0x81),
            (0x2D, 0x7F7F, 0x41, 0x17),
            (0x84, 0x7F80, 0xC0, 0x94),
            (0x85, 0x7F80, 0xC1, 0x80),
            (0x8C, 0x7F80, 0xC1, 0x94),
            (0x8D, 0x7F80, 0xC2, 0x80),
            (0x94, 0x7F80, 0xC2, 0x93),
            (0x95, 0x7F80, 0xC1, 0x87),
            (0x9C, 0x7F80, 0xC1, 0x93),
            (0x9D, 0x7F80, 0xC0, 0x87),
            (0xA4, 0x7F80, 0x41, 0x14),
            (0xA5, 0x7F80, 0x00, 0x54),
            (0xAC, 0x7F80, 0x3E, 0x00),
            (0xAD, 0x7F80, 0xC1, 0x80),
            (0xB4, 0x7F80, 0x7F, 0x00),
            (0xB5, 0x7F80, 0xC1, 0x80),
            (0xBC, 0x7F80, 0x41, 0x93),
            (0xBD, 0x7F80, 0x41, 0x87),
        ] {
            let (mut cpu, mut bus) = fixture(prefix, op);
            if prefix == 0xDD {
                cpu.registers_mut().ix = 0x7F80;
            } else {
                cpu.registers_mut().iy = 0x7F80;
            }
            let report = cpu.step(&mut bus);
            assert_eq!(
                (
                    selected(&cpu, prefix),
                    cpu.registers().a,
                    cpu.registers().flags.to_byte()
                ),
                (word, a, flags),
                "{prefix:02X} {op:02X}"
            );
            assert_eq!(cpu.registers().hl(), 0x1234);
            assert_eq!(
                (report.steps, report.cycles, report.exit),
                (1, 8, Exit::Budget)
            );
        }
    }
}

#[test]
fn last_selector_ed_cancellation_indexed_cb_halt_and_selection_expiry() {
    // L chains/access/cost, A documented indexed-CB operation: decision-20 §2;
    // Young §3.7 p. 14/§6.1 p. 23; UM RLC pp. 217–218 / HALT p. 181;
    // TM Tables 7.0-7/6 pp. 50/49. No independent chain silicon key.
    for (bytes, ix, iy) in [
        (&[0xDD, 0xFD, 0x21, 0x78, 0x56][..], 0x3000, 0x5678),
        (&[0xFD, 0xDD, 0x21, 0x78, 0x56][..], 0x5678, 0x4567),
    ] {
        let (mut cpu, mut bus) = fixture(0xDD, 0);
        bus.load(0x1000, bytes);
        assert_eq!(cpu.step(&mut bus).cycles, 18);
        assert_eq!(
            (
                cpu.registers().ix,
                cpu.registers().iy,
                cpu.registers().pc,
                cpu.registers().r
            ),
            (ix, iy, 0x1005, 3)
        );
        bus.load(0x1005, &[0x26, 0xAA]);
        assert_eq!(cpu.step(&mut bus).cycles, 7);
        assert_eq!(
            (cpu.registers().h, cpu.registers().ix, cpu.registers().iy),
            (0xAA, ix, iy)
        );
    }
    let (mut cpu, mut bus) = fixture(0xDD, 0);
    bus.load(0x1000, &[0xFD, 0xDD, 0xED, 0x44]);
    assert_eq!(cpu.step(&mut bus).cycles, 16);
    assert_eq!(
        (
            cpu.registers().a,
            cpu.registers().flags.to_byte(),
            cpu.registers().pc,
            cpu.registers().r
        ),
        (0xBF, 0x93, 0x1004, 4)
    );
    assert_eq!((cpu.registers().ix, cpu.registers().iy), (0x3000, 0x4567));
    // All 256 ED operations cancel selection, including HL operations and
    // ED terminals whose byte value happens to be DD/FD/CB/ED.
    for op in 0..=255 {
        let (mut cpu, mut bus) = fixture(0xDD, 0xFD);
        bus.load(0x1002, &[0xED, op, 0x01, 0x20]);
        let (mut expected, mut plain) = fixture(0xDD, 0);
        expected.registers_mut().pc = 0x1002;
        expected.registers_mut().r = 2;
        plain.load(0x1000, &[0xDD, 0xFD, 0xED, op, 0x01, 0x20]);
        let a = cpu.step(&mut bus);
        let b = expected.step(&mut plain);
        assert_eq!(cpu, expected, "ED {op:02X}");
        assert_eq!((a.exit, a.steps, a.cycles), (b.exit, 1, b.cycles + 8));
        assert!(bus.memory == plain.memory, "ED {op:02X} memory");
    }
    let (mut cpu, mut bus) = fixture(0xDD, 0);
    bus.load(0x1000, &[0xFD, 0xDD, 0xCB, 0x01, 0x06]);
    assert_eq!(cpu.step(&mut bus).cycles, 27);
    assert_eq!(
        (bus.memory[0x3001], cpu.registers().r, cpu.registers().pc),
        (3, 3, 0x1005)
    );
    assert_eq!(
        &bus.log[3..8],
        &[
            read(0x1000, Op, 0xFD),
            read(0x1001, Op, 0xDD),
            read(0x1002, Op, 0xCB),
            read(0x1003, Arg, 1),
            read(0x1004, Arg, 6)
        ]
    );
    for prefix in [0xDD, 0xFD] {
        let (mut cpu, mut bus) = fixture(prefix, 0x76);
        assert_eq!(cpu.step(&mut bus).cycles, 8);
        assert!(cpu.latches().halted);
        assert_eq!(
            (cpu.registers().pc, cpu.registers().r, bus.log.len()),
            (0x1002, 2, 5)
        );
        assert_eq!(cpu.step(&mut bus).exit, Exit::Halt);
    }
}

#[test]
fn long_chain_wraps_pc_and_r_with_one_overshooting_retirement() {
    // L width/access rule: decision-20 §2; Young §6.1 p. 23 supplies R
    // counts only. Literal 300 selectors + NOP = 301 fetches / 1204 states.
    let (mut cpu, mut bus) = fixture(0xDD, 0);
    cpu.registers_mut().pc = 0xFF80;
    cpu.registers_mut().r = 0xFE;
    let mut bytes = vec![0xDD; 300];
    bytes[299] = 0xFD;
    bytes.push(0);
    bus.load(0xFF80, &bytes);
    let report = cpu.run(
        &mut bus,
        Budget {
            steps: 99,
            cycles: 255,
        },
    );
    assert_eq!(
        (report.exit, report.steps, report.cycles),
        (Exit::Budget, 1, 1204)
    );
    assert_eq!((cpu.registers().pc, cpu.registers().r), (0x00AD, 0xAB));
    let mut expected = vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ];
    for offset in 0..300u16 {
        expected.push(read(
            0xFF80u16.wrapping_add(offset),
            Op,
            if offset == 299 { 0xFD } else { 0xDD },
        ));
    }
    expected.push(read(0x00AC, Op, 0));
    assert_eq!(bus.take_log(), expected);
}

#[test]
fn repeating_block_refetch_starts_at_ed_after_extra_selectors() {
    // L chain extension: decision-20 §2 / Young §§3.7/6.1 pp. 14/23;
    // A iteration semantics: UM LDIR p. 132 / TM Table 7.0-3 p. 46.
    let (mut cpu, mut bus) = fixture(0xDD, 0);
    cpu.registers_mut().set_bc(2);
    cpu.registers_mut().set_hl(0x3001);
    cpu.registers_mut().set_de(0x5000);
    bus.load(0x1000, &[0xDD, 0xFD, 0xED, 0xB0]);
    assert_eq!(cpu.step(&mut bus).cycles, 29);
    assert_eq!((cpu.registers().pc, cpu.registers().r), (0x1002, 4));
    bus.take_log();
    assert_eq!(cpu.step(&mut bus).cycles, 16);
    assert_eq!(
        (cpu.registers().pc, cpu.registers().r, cpu.registers().bc()),
        (0x1004, 6, 0)
    );
    assert_eq!(
        &bus.log[3..5],
        &[read(0x1002, Op, 0xED), read(0x1003, Op, 0xB0)]
    );
}

// Decision-20 §3 removes the indexed-CB refusal-chain fixture; no fetched
// refusal remains. z80_undocumented_cb.rs now pins a retiring copy chain;
// z80_interrupts_int.rs retains interrupt-only payload/rollback witnesses.

// Requests become asserted during the first prefix fetch, rather than on a
// later scripted poll. The full trace proves there is no mid-chain polling.
struct AssertDuringFetch {
    bus: RecordingBus,
    nmi: bool,
    int: bool,
    stop: bool,
}
impl Bus for AssertDuringFetch {
    fn read(&mut self, address: u16, kind: MemoryAccess) -> u8 {
        let value = self.bus.read(address, kind);
        if address == 0x1000 && kind == Op {
            self.bus.nmi = self.nmi;
            self.bus.int = self.int;
            self.bus.stop = self.stop;
        }
        value
    }
    fn write(&mut self, a: u16, v: u8, k: MemoryAccess) {
        self.bus.write(a, v, k);
    }
    fn input(&mut self, p: u8) -> u8 {
        self.bus.input(p)
    }
    fn output(&mut self, p: u8, v: u8) {
        self.bus.output(p, v);
    }
}
impl Control for AssertDuringFetch {
    fn stop_requested(&mut self) -> bool {
        self.bus.stop_requested()
    }
}
impl Inputs for AssertDuringFetch {
    fn nmi(&mut self) -> bool {
        self.bus.nmi()
    }
    fn int(&mut self) -> bool {
        self.bus.int()
    }
}

#[test]
fn requests_asserted_during_chain_wait_for_terminal_boundary_and_ei_delay() {
    // L instruction-boundary mapping: decision-20 §2; Young §5.5 pp. 21–22
    // reports INT only, explicitly no NMI; no independent chain injection.
    for (nmi, int, stop) in [
        (true, true, false),
        (false, true, false),
        (false, false, true),
    ] {
        let (mut cpu, mut bus) = fixture(0xDD, 0);
        bus.load(0x1000, &[0xDD, 0xFD, 0xDD, 0x00]);
        bus.acknowledges.push_back(0xFF);
        cpu.latches_mut().iff1 = true;
        cpu.latches_mut().iff2 = true;
        cpu.latches_mut().interrupt_mode = InterruptMode::Mode1;
        let mut system = AssertDuringFetch {
            bus,
            nmi,
            int,
            stop,
        };
        let report = cpu.step(&mut system);
        assert_eq!(
            (report.exit, report.steps, report.cycles),
            (Exit::Budget, 1, 16)
        );
        assert_eq!((cpu.registers().pc, cpu.registers().r), (0x1004, 4));
        assert_eq!(
            system.bus.take_log(),
            vec![
                Event::StopPoll(false),
                Event::NmiPoll(false),
                Event::IntPoll(false),
                read(0x1000, Op, 0xDD),
                read(0x1001, Op, 0xFD),
                read(0x1002, Op, 0xDD),
                read(0x1003, Op, 0)
            ]
        );
        let report = cpu.step(&mut system);
        if stop {
            assert_eq!(
                (report.exit, report.steps, report.cycles),
                (Exit::Stop, 0, 0)
            );
        } else {
            assert_eq!(
                (report.exit, report.steps, report.cycles, cpu.registers().pc),
                (
                    Exit::Budget,
                    1,
                    if nmi { 11 } else { 13 },
                    if nmi { 0x66 } else { 0x38 }
                )
            );
        }
    }
    let (mut cpu, mut bus) = fixture(0xDD, 0);
    bus.load(0x1000, &[0xFB, 0xDD, 0xFD, 0x00]);
    assert_eq!(cpu.step(&mut bus).cycles, 4);
    bus.int = true;
    bus.acknowledges.push_back(0xFF);
    assert_eq!(cpu.step(&mut bus).cycles, 12);
    assert_eq!(
        (cpu.registers().pc, cpu.latches().ei_delay),
        (0x1004, false)
    );
    assert_eq!(cpu.step(&mut bus).cycles, 13);
}

#[test]
fn ignored_exchange_prefixes_keep_ordinary_hl_and_alternate_registers() {
    // L ignored selectors: decision-20 §2; Young v0.91 §3.2 p. 11.
    // A exchanges: UM EX DE,HL p. 124 / EXX p. 126; TM Table 7.0-3 p. 46.
    for prefix in [0xDD, 0xFD] {
        for op in [0xEB, 0xD9] {
            let (mut cpu, mut bus) = fixture(prefix, op);
            let r = cpu.registers_mut();
            r.set_bc_alternate(0x5566);
            r.set_de_alternate(0x7788);
            r.set_hl_alternate(0xA1B2);
            assert_eq!(cpu.step(&mut bus).cycles, 8);
            let r = cpu.registers();
            if op == 0xEB {
                assert_eq!((r.de(), r.hl(), r.hl_alternate()), (0x1234, 0x2233, 0xA1B2));
            } else {
                assert_eq!((r.bc(), r.de(), r.hl()), (0x5566, 0x7788, 0xA1B2));
                assert_eq!(
                    (r.bc_alternate(), r.de_alternate(), r.hl_alternate()),
                    (0x1122, 0x2233, 0x1234)
                );
            }
            assert_eq!(
                (selected(&cpu, prefix), r.flags.to_byte(), r.pc, r.r),
                (0x3000, 1, 0x1002, 2)
            );
        }
    }
}

#[test]
fn prefixed_ld_a_i_parity_note_survives_until_the_next_interrupt_boundary() {
    // L chain/boundary rule: decision-20 §2; Young v0.91 §§3.7/5.5/6.1
    // pp. 14/21–23. LD A,I/interrupt note: decision-15 rule 10 (unverified),
    // UM pp. 18/94; ALPM 03-0002-01 (January 1978) p. 48.
    let (mut cpu, mut bus) = fixture(0xDD, 0);
    cpu.registers_mut().i = 0x80;
    cpu.latches_mut().iff1 = true;
    cpu.latches_mut().iff2 = true;
    bus.load(0x1000, &[0xDD, 0xFD, 0xED, 0x57]);
    let mut system = AssertDuringFetch {
        bus,
        nmi: true,
        int: true,
        stop: false,
    };
    let report = cpu.step(&mut system);
    assert_eq!(
        (report.steps, report.cycles, report.exit),
        (1, 17, Exit::Budget)
    );
    assert_eq!(
        (
            cpu.registers().a,
            cpu.registers().flags.to_byte(),
            cpu.registers().r
        ),
        (0x80, 0x85, 4)
    );
    assert!(cpu.latches().parity_note_pending);
    assert_eq!(
        system
            .bus
            .log
            .iter()
            .filter(|e| matches!(e, Event::NmiPoll(_)))
            .count(),
        1
    );
    assert_eq!(cpu.step(&mut system).cycles, 11);
    assert_eq!(cpu.registers().flags.to_byte(), 0x81);
    assert!(!cpu.latches().parity_note_pending);
}
