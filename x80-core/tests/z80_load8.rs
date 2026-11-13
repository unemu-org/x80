//! Decision-20 §4 replaces zero Y/X and former unknown preservation here.
//! H class rules: Young v0.91 §§4.1–4.3 pp. 15–17 / §§8.1–8.11 pp. 26–35;
//! Rak z80test v1.2a corresponding load/ALU/BIT/block-I/O groups, pinned in
//! PROVENANCE §1. SCF/CCF use decision-20 §6 Q; parity-window rules stay L.
//! 8-bit load witnesses, decisions 13–15. Primary: Zilog UM008011-0816,
//! rev. 11 (August 2016), 8-Bit Load Group pp. 70–97 (PDF = printed + 14).
//! Second cross-read: 03-0029-01 (1977), Table 7.0-1 p. 44 (PDF 50).
//! Expectations come from rendered source pages, not executor helpers.
//! All tests fail before landing with Unsupported and zero retired steps.
//! Access kinds/polls and parity-note lifetime are library contracts
//! (decision-15 rules 1,5,6,10); address wraps below are derived from the
//! 16-bit register widths (UM pp. 2–3) and signed displacement, not explicit
//! wrap text. Source SHA-256 values match PROVENANCE.md §1.

mod support;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Exit, MemoryAccess, Report};

const OP: MemoryAccess = MemoryAccess::OpcodeFetch;
const ARG: MemoryAccess = MemoryAccess::OperandFetch;
const DATA: MemoryAccess = MemoryAccess::Data;

fn cpu_at(pc: u16, flags: u8) -> Z80 {
    Z80::new(
        Registers {
            a: 0xA1,
            flags: Flags::from_byte(flags),
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
            i: 0x38,
            r: 0xFF,
        },
        Latches {
            q: false,
            wz: 0,
            iff1: false,
            iff2: true,
            interrupt_mode: InterruptMode::Mode2,
            ei_delay: true,
            halted: false,
            nmi_asserted: true,
            parity_note_pending: true,
        },
    )
}

fn after(cpu: &Z80, pc: u16, r: u8) -> Z80 {
    let mut expected = cpu.clone();
    expected.registers_mut().pc = pc;
    expected.registers_mut().r = r;
    expected.latches_mut().q = false;
    expected.latches_mut().ei_delay = false;
    expected.latches_mut().nmi_asserted = false;
    expected.latches_mut().parity_note_pending = false;
    expected
}

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
        kind: DATA,
        value,
    }
}

fn check(
    cpu: &mut Z80,
    bus: &mut RecordingBus,
    expected: &Z80,
    memory: &[u8],
    cycles: u64,
    accesses: &[Event],
) {
    assert_eq!(
        cpu.step(bus),
        Report {
            steps: 1,
            cycles,
            exit: Exit::Budget
        }
    );
    assert_eq!(cpu, expected);
    assert_eq!(bus.memory, memory);
    let mut trace = vec![
        Event::StopPoll(false),
        Event::NmiPoll(false),
        Event::IntPoll(false),
    ];
    trace.extend_from_slice(accesses);
    assert_eq!(bus.take_log(), trace);
}

// Literal register fields and seed values: UM p. 71; TM p. 44.
// Setters name actual fields; no production register accessor is used.
type RegisterWitness = (u8, u8, fn(&mut Registers, u8));
const REGS: [RegisterWitness; 7] = [
    (0, 0xB2, |r, v| r.b = v),
    (1, 0xC3, |r, v| r.c = v),
    (2, 0xD4, |r, v| r.d = v),
    (3, 0xE5, |r, v| r.e = v),
    (4, 0xF6, |r, v| r.h = v),
    (5, 0x07, |r, v| r.l = v),
    (7, 0xA1, |r, v| r.a = v),
];

#[test]
fn register_matrix_including_self_loads_preserves_all_flags() {
    // UM p. 71: 01 r r', 4 states, no flags. Derived expansion: 7 x 7.
    // H's field is 100, also pinning the existing compact-heading erratum.
    for (dst, _, set) in REGS {
        for (src, value, _) in REGS {
            for flags in [0, 0xD7] {
                let opcode = 0x40 | (dst << 3) | src;
                let mut cpu = cpu_at(0xFFFF, flags);
                let mut expected = after(&cpu, 0, 0x80);
                set(expected.registers_mut(), value);
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[opcode]);
                let memory = bus.memory.clone();
                check(
                    &mut cpu,
                    &mut bus,
                    &expected,
                    &memory,
                    4,
                    &[read(0xFFFF, OP, opcode)],
                );
            }
        }
    }
}

#[test]
fn immediate_register_loads_fetch_operand_across_pc_wrap() {
    // UM pp. 72–73: 00 r 110, 7 states, no flags; PC wrap derived.
    for ((_, _, set), opcode) in REGS
        .into_iter()
        .zip([0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x3E])
    {
        for flags in [0, 0xD7] {
            let mut cpu = cpu_at(0xFFFF, flags);
            let mut expected = after(&cpu, 1, 0x80);
            set(expected.registers_mut(), 0x5A);
            let mut bus = RecordingBus::new();
            bus.load(0xFFFF, &[opcode, 0x5A]);
            let memory = bus.memory.clone();
            check(
                &mut cpu,
                &mut bus,
                &expected,
                &memory,
                7,
                &[read(0xFFFF, OP, opcode), read(0, ARG, 0x5A)],
            );
        }
    }
}

#[test]
fn hl_reads_capture_address_before_loading_h_or_l() {
    // UM p. 74: 01 r 110, 7 states; no flags.
    for ((_, _, set), opcode) in REGS
        .into_iter()
        .zip([0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x7E])
    {
        for flags in [0, 0xD7] {
            let mut cpu = cpu_at(0x1000, flags);
            let mut expected = after(&cpu, 0x1001, 0x80);
            set(expected.registers_mut(), 0x59);
            let mut bus = RecordingBus::new();
            bus.load(0x1000, &[opcode]);
            bus.load(0xF607, &[0x59]);
            let memory = bus.memory.clone();
            check(
                &mut cpu,
                &mut bus,
                &expected,
                &memory,
                7,
                &[read(0x1000, OP, opcode), read(0xF607, DATA, 0x59)],
            );
        }
    }
}

#[test]
fn hl_register_stores_preserve_registers_and_flags() {
    // UM pp. 79–80: 01 110 r, 7 states; no flags.
    for ((_, value, _), opcode) in REGS
        .into_iter()
        .zip([0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77])
    {
        for flags in [0, 0xD7] {
            let mut cpu = cpu_at(0x1000, flags);
            let expected = after(&cpu, 0x1001, 0x80);
            let mut bus = RecordingBus::new();
            bus.load(0x1000, &[opcode]);
            bus.load(0xF607, &[0x99]);
            let mut memory = bus.memory.clone();
            memory[0xF607] = value;
            check(
                &mut cpu,
                &mut bus,
                &expected,
                &memory,
                7,
                &[read(0x1000, OP, opcode), write(0xF607, value)],
            );
        }
    }
}

#[test]
fn hl_immediate_store_has_operand_then_data_access() {
    // UM p. 85: 36 n, 10 states, no flags; PC wrap derived.
    for flags in [0, 0xD7] {
        let mut cpu = cpu_at(0xFFFF, flags);
        let expected = after(&cpu, 1, 0x80);
        let mut bus = RecordingBus::new();
        bus.load(0xFFFF, &[0x36, 0x28]);
        let mut memory = bus.memory.clone();
        memory[0xF607] = 0x28;
        check(
            &mut cpu,
            &mut bus,
            &expected,
            &memory,
            10,
            &[
                read(0xFFFF, OP, 0x36),
                read(0, ARG, 0x28),
                write(0xF607, 0x28),
            ],
        );
    }
}

// Literal signed-address witnesses for both index registers: -128, -1,
// zero, +127. Underflow/overflow are DERIVED 16-bit wraps (UM pp. 75–87).
const INDEX_ADDRESSES: [(u16, u8, u16); 4] = [
    (0x0040, 0x80, 0xFFC0),
    (0xFFFE, 0xFF, 0xFFFD),
    (0x2345, 0x00, 0x2345),
    (0xFFC0, 0x7F, 0x003F),
];

fn index_cpu(prefix: u8, base: u16, flags: u8) -> Z80 {
    let mut cpu = cpu_at(0xFFFE, flags);
    if prefix == 0xDD {
        cpu.registers_mut().ix = base;
    } else {
        cpu.registers_mut().iy = base;
    }
    cpu
}

#[test]
fn indexed_reads_use_signed_displacements_and_real_h_l() {
    // UM pp. 75–78: DD/FD 01 r 110 d, 19 states; no flags.
    for prefix in [0xDD, 0xFD] {
        for (base, d, address) in INDEX_ADDRESSES {
            for ((_, _, set), opcode) in REGS
                .into_iter()
                .zip([0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x7E])
            {
                for flags in [0, 0xD7] {
                    let mut cpu = index_cpu(prefix, base, flags);
                    let mut expected = after(&cpu, 1, 0x81);
                    expected.latches_mut().wz = address;
                    set(expected.registers_mut(), 0x39);
                    let mut bus = RecordingBus::new();
                    bus.load(0xFFFE, &[prefix, opcode, d]);
                    bus.load(address, &[0x39]);
                    let memory = bus.memory.clone();
                    check(
                        &mut cpu,
                        &mut bus,
                        &expected,
                        &memory,
                        19,
                        &[
                            read(0xFFFE, OP, prefix),
                            read(0xFFFF, OP, opcode),
                            read(0, ARG, d),
                            read(address, DATA, 0x39),
                        ],
                    );
                }
            }
        }
    }
}

#[test]
fn indexed_register_stores_use_signed_displacements_and_real_h_l() {
    // UM pp. 81–84: DD/FD 01 110 r d, 19 states; no flags.
    for prefix in [0xDD, 0xFD] {
        for (base, d, address) in INDEX_ADDRESSES {
            for ((_, value, _), opcode) in REGS
                .into_iter()
                .zip([0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77])
            {
                for flags in [0, 0xD7] {
                    let mut cpu = index_cpu(prefix, base, flags);
                    let mut expected = after(&cpu, 1, 0x81);
                    expected.latches_mut().wz = address;
                    let mut bus = RecordingBus::new();
                    bus.load(0xFFFE, &[prefix, opcode, d]);
                    let mut memory = bus.memory.clone();
                    memory[usize::from(address)] = value;
                    check(
                        &mut cpu,
                        &mut bus,
                        &expected,
                        &memory,
                        19,
                        &[
                            read(0xFFFE, OP, prefix),
                            read(0xFFFF, OP, opcode),
                            read(0, ARG, d),
                            write(address, value),
                        ],
                    );
                }
            }
        }
    }
}

#[test]
fn indexed_immediate_fetches_d_before_n_and_does_not_increment_r_for_either() {
    // UM pp. 86–87: DD/FD 36 d n, 19 states, no flags. Both operands
    // cross PC zero here; signed address wraps are derived.
    for prefix in [0xDD, 0xFD] {
        for (base, d, address) in INDEX_ADDRESSES {
            for flags in [0, 0xD7] {
                let mut cpu = index_cpu(prefix, base, flags);
                let mut expected = after(&cpu, 2, 0x81);
                expected.latches_mut().wz = address;
                let mut bus = RecordingBus::new();
                bus.load(0xFFFE, &[prefix, 0x36, d, 0x5A]);
                let mut memory = bus.memory.clone();
                memory[usize::from(address)] = 0x5A;
                check(
                    &mut cpu,
                    &mut bus,
                    &expected,
                    &memory,
                    19,
                    &[
                        read(0xFFFE, OP, prefix),
                        read(0xFFFF, OP, 0x36),
                        read(0, ARG, d),
                        read(1, ARG, 0x5A),
                        write(address, 0x5A),
                    ],
                );
            }
        }
    }
}

#[test]
fn accumulator_indirect_loads_and_stores() {
    // UM pp. 88–89,91–92: BC and DE, 7 states; no flags.
    for (load, store, address, load_wz, store_wz) in [
        (0x0A, 0x02, 0xB2C3, 0xB2C4, 0xA1C4),
        (0x1A, 0x12, 0xD4E5, 0xD4E6, 0xA1E6),
    ] {
        for flags in [0, 0xD7] {
            for is_load in [false, true] {
                let opcode = if is_load { load } else { store };
                let mut cpu = cpu_at(0x1000, flags);
                let mut expected = after(&cpu, 0x1001, 0x80);
                expected.latches_mut().wz = if is_load { load_wz } else { store_wz };
                let mut bus = RecordingBus::new();
                bus.load(0x1000, &[opcode]);
                bus.load(address, &[0x12]);
                let mut memory = bus.memory.clone();
                let access = if is_load {
                    expected.registers_mut().a = 0x12;
                    read(address, DATA, 0x12)
                } else {
                    memory[usize::from(address)] = 0xA1;
                    write(address, 0xA1)
                };
                check(
                    &mut cpu,
                    &mut bus,
                    &expected,
                    &memory,
                    7,
                    &[read(0x1000, OP, opcode), access],
                );
            }
        }
    }
}

#[test]
fn accumulator_absolute_address_is_little_endian_with_operand_pc_wrap() {
    // UM pp. 90,93: first n is low, 13 states; no flags. Derived PC wrap.
    for (opcode, is_load) in [(0x3A, true), (0x32, false)] {
        for flags in [0, 0xD7] {
            let mut cpu = cpu_at(0xFFFE, flags);
            let mut expected = after(&cpu, 1, 0x80);
            expected.latches_mut().wz = if is_load { 0x8833 } else { 0xA133 };
            let mut bus = RecordingBus::new();
            bus.load(0xFFFE, &[opcode, 0x32, 0x88]);
            bus.load(0x8832, &[0x04]);
            let mut memory = bus.memory.clone();
            let access = if is_load {
                expected.registers_mut().a = 0x04;
                read(0x8832, DATA, 0x04)
            } else {
                memory[0x8832] = 0xA1;
                write(0x8832, 0xA1)
            };
            check(
                &mut cpu,
                &mut bus,
                &expected,
                &memory,
                13,
                &[
                    read(0xFFFE, OP, opcode),
                    read(0xFFFF, ARG, 0x32),
                    read(0, ARG, 0x88),
                    access,
                ],
            );
        }
    }
}

#[test]
fn ld_a_i_flags_take_iff2_instead_of_value_parity_or_iff1() {
    // UM p. 94; TM p. 44: S/Z from I, H/N reset, P/V=IFF2, C kept.
    // Literal outcomes include odd/even data parity and both S/Z directions.
    for (value, sz) in [
        (0, 0x40),
        (1, 0),
        (3, 0),
        (0x7F, 0),
        (0x80, 0x80),
        (0xFF, 0x80),
    ] {
        for (iff2, pv) in [(false, 0), (true, 4)] {
            for (initial, carry) in [(0xD6, 0), (0x13, 1)] {
                let mut cpu = cpu_at(0xFFFF, initial);
                cpu.registers_mut().i = value;
                cpu.latches_mut().iff1 = !iff2;
                cpu.latches_mut().iff2 = iff2;
                let mut expected = after(&cpu, 1, 0x81);
                expected.registers_mut().a = value;
                expected.registers_mut().flags = Flags {
                    sign: sz == 0x80,
                    zero: sz == 0x40,
                    y: value & 0x20 != 0,
                    x: value & 8 != 0,
                    half_carry: false,
                    parity_overflow: pv == 4,
                    subtract: false,
                    carry: carry == 1,
                };
                expected.latches_mut().parity_note_pending = true;
                expected.latches_mut().q = true;
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[0xED, 0x57]);
                let memory = bus.memory.clone();
                check(
                    &mut cpu,
                    &mut bus,
                    &expected,
                    &memory,
                    9,
                    &[read(0xFFFF, OP, 0xED), read(0, OP, 0x57)],
                );
            }
        }
    }
}

#[test]
fn ld_a_r_observes_own_two_fetch_increments_with_bit_seven_retained() {
    // UM p. 95, decision-15 rule 6: fetch-before-load ordering derived.
    // UM Registers p. 3; TM §2 p. 4 / §3 p. 7: seven-bit R increment.
    for (r, value, sz) in [
        (0x7E, 0, 0x40),
        (0x7F, 1, 0),
        (0xFE, 0x80, 0x80),
        (0xFF, 0x81, 0x80),
        (1, 3, 0),
    ] {
        for (iff2, pv) in [(false, 0), (true, 4)] {
            for (initial, carry) in [(0xD6, 0), (0x13, 1)] {
                let mut cpu = cpu_at(0xFFFF, initial);
                cpu.registers_mut().r = r;
                cpu.latches_mut().iff1 = !iff2;
                cpu.latches_mut().iff2 = iff2;
                let mut expected = after(&cpu, 1, value);
                expected.registers_mut().a = value;
                expected.registers_mut().flags = Flags {
                    sign: sz == 0x80,
                    zero: sz == 0x40,
                    y: value & 0x20 != 0,
                    x: value & 8 != 0,
                    half_carry: false,
                    parity_overflow: pv == 4,
                    subtract: false,
                    carry: carry == 1,
                };
                expected.latches_mut().parity_note_pending = true;
                expected.latches_mut().q = true;
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[0xED, 0x5F]);
                let memory = bus.memory.clone();
                check(
                    &mut cpu,
                    &mut bus,
                    &expected,
                    &memory,
                    9,
                    &[read(0xFFFF, OP, 0xED), read(0, OP, 0x5F)],
                );
            }
        }
    }
}

#[test]
fn ld_i_a_and_ld_r_a_preserve_flags_and_load_all_eight_bits_after_fetch() {
    // UM pp. 96–97; TM p. 44: 9 states, no flags. Decision-15 rule 6
    // orders LD R,A after fetch increments; test both programmable bit-7 values.
    for opcode in [0x47, 0x4F] {
        for value in [0, 0x7F, 0x80, 0xFF] {
            for flags in [0, 0xD7] {
                let mut cpu = cpu_at(0xFFFF, flags);
                cpu.registers_mut().a = value;
                let mut expected = after(&cpu, 1, 0x81);
                if opcode == 0x47 {
                    expected.registers_mut().i = value;
                } else {
                    expected.registers_mut().r = value;
                }
                let mut bus = RecordingBus::new();
                bus.load(0xFFFF, &[0xED, opcode]);
                let memory = bus.memory.clone();
                check(
                    &mut cpu,
                    &mut bus,
                    &expected,
                    &memory,
                    9,
                    &[read(0xFFFF, OP, 0xED), read(0, OP, opcode)],
                );
            }
        }
    }
}

#[test]
fn parity_note_is_set_after_step_and_cleared_before_next_ordinary_instruction() {
    // Z9 regression: decision-15 rule 10 (unverified boundary interpretation
    // of UM pp. 94–95's interrupt note). NOP: UM p. 180 / TM p. 48.
    for opcode in [0x57, 0x5F] {
        let mut cpu = cpu_at(0x1000, 0xD7);
        cpu.latches_mut().parity_note_pending = false;
        let mut expected = after(&cpu, 0x1002, 0x81);
        expected.registers_mut().a = if opcode == 0x57 { 0x38 } else { 0x81 };
        expected.registers_mut().flags = Flags {
            sign: opcode == 0x5F,
            zero: false,
            y: opcode == 0x57,
            x: opcode == 0x57,
            half_carry: false,
            parity_overflow: true,
            subtract: false,
            carry: true,
        };
        expected.latches_mut().parity_note_pending = true;
        expected.latches_mut().q = true;
        let mut bus = RecordingBus::new();
        bus.load(0x1000, &[0xED, opcode, 0]);
        let memory = bus.memory.clone();
        check(
            &mut cpu,
            &mut bus,
            &expected,
            &memory,
            9,
            &[read(0x1000, OP, 0xED), read(0x1001, OP, opcode)],
        );
        assert!(cpu.latches().parity_note_pending);
        let expected = after(&cpu, 0x1003, 0x82);
        check(
            &mut cpu,
            &mut bus,
            &expected,
            &memory,
            4,
            &[read(0x1002, OP, 0)],
        );
        assert!(!cpu.latches().parity_note_pending);
        assert!(cpu.registers().flags.parity_overflow);
    }
}
