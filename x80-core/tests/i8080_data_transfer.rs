//! Independent Data Transfer Group witnesses: decision-5 items 3, 5, 7, 8;
//! decision-6 and plan §2.2/§2.6. Sources are pinned in PROVENANCE.md §1.
//! Intel 98-153B (September 1975), Ch. 4 pp. 4-3 to 4-5 supplies encodings,
//! effects, lengths, unchanged flags and literal state totals. Ch. 2 machine
//! cycles pp. 2-16/2-17 and notes 1, 6, 7 on p. 2-20 supply access order.
//! Intel 9800301-04 (May 1981), Ch. 3 entries cited below independently agree.
//! All cited values were checked against page images. No crate decoder,
//! register-pair or flag-byte helpers supply expected results.
//!
//! Wrap is derived from 16-bit PC/address logic (98-153B Ch. 2 p. 2-2) and
//! the address + 1 cycles (p. 2-17). Neither entry explicitly states the
//! 0xFFFF -> 0x0000 edge: that hardware edge remains unverified.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8080::{Flags, Latches, Registers, I8080};
use x80_core::{Exit, MemoryAccess, Report};

// Test-side register-code table, 98-153B Ch. 4 p. 4-3 (also Ch. 2 p. 2-20).
// The field names select a register independently of the implementation.
const REGISTERS: [(u8, &str); 7] = [
    (7, "a"),
    (0, "b"),
    (1, "c"),
    (2, "d"),
    (3, "e"),
    (4, "h"),
    (5, "l"),
];

fn register<'a>(r: &'a mut Registers, name: &str) -> &'a mut u8 {
    match name {
        "a" => &mut r.a,
        "b" => &mut r.b,
        "c" => &mut r.c,
        "d" => &mut r.d,
        "e" => &mut r.e,
        "h" => &mut r.h,
        "l" => &mut r.l,
        _ => panic!("invalid test register"),
    }
}

// Literal PC/operand-address/next-PC vectors, including both operand fetch
// edges. Ch. 4 p. 4-2: consecutive bytes, sequential execution; p. 2-16:
// PC increments during M1/M2, p. 2-17 during M3. Wrap caveat above applies.
const LOCATIONS: [(u16, u16, u16, u16); 3] = [
    (0x4000, 0x4001, 0x4002, 0x4003),
    (0xFFFE, 0xFFFF, 0x0000, 0x0001),
    (0xFFFF, 0x0000, 0x0001, 0x0002),
];

fn initial_states(pc: u16) -> impl Iterator<Item = Registers> {
    // Every combination of the five flags; Ch. 4 p. 4-4 says no instruction
    // in this group affects condition flags. This is a test enumeration,
    // not PSW packing.
    (0..32).map(move |bits| Registers {
        a: 0xE7,
        b: 0x12,
        c: 0x34,
        d: 0x9A,
        e: 0xBC,
        h: 0x56,
        l: 0x78,
        sp: 0xD234,
        pc,
        flags: Flags {
            sign: bits & 1 != 0,
            zero: bits & 2 != 0,
            auxiliary_carry: bits & 4 != 0,
            parity: bits & 8 != 0,
            carry: bits & 16 != 0,
        },
    })
}

fn read(address: u16, kind: MemoryAccess, value: u8) -> Event {
    Event::Read {
        address,
        kind,
        value,
    }
}

fn data_read(address: u16, value: u8) -> Event {
    read(address, MemoryAccess::Data, value)
}

fn data_write(address: u16, value: u8) -> Event {
    Event::Write {
        address,
        kind: MemoryAccess::Data,
        value,
    }
}

// decision-5 items 3/5; 98-153B Ch. 2 pp. 2-16/2-17 and p. 2-20 note 1:
// M1 opcode, M2 byte 2, M3 byte 3. OperandFetch is the API distinction.
fn fetches(pc: u16, bytes: &[u8]) -> Vec<Event> {
    let mut events = vec![Event::StopPoll(false), Event::InterruptPoll(false)];
    for (offset, &value) in bytes.iter().enumerate() {
        events.push(read(
            ((usize::from(pc) + offset) % 65536) as u16,
            if offset == 0 {
                MemoryAccess::OpcodeFetch
            } else {
                MemoryAccess::OperandFetch
            },
            value,
        ));
    }
    events
}

fn witness(
    initial: Registers,
    bytes: &[u8],
    seed: &[(u16, u8)],
    expected: Registers,
    states: u64,
    accesses: &[Event],
) {
    let mut bus = RecordingBus::new();
    for &(address, value) in seed {
        bus.memory[usize::from(address)] = value;
    }
    bus.load(initial.pc, bytes);
    let mut memory = bus.memory.clone();
    for event in accesses {
        if let Event::Write { address, value, .. } = *event {
            memory[usize::from(address)] = value;
        }
    }
    let latches = Latches {
        interrupt_enable: true,
        ei_delay: false,
        halted: false,
    };
    let mut cpu = I8080::new(initial, latches);
    let report = cpu.step(&mut bus);
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: states,
            exit: Exit::Budget
        },
        "{bytes:02X?} at {:04X}",
        initial.pc
    );
    assert_eq!(*cpu.registers(), expected, "{bytes:02X?}");
    assert_eq!(*cpu.latches(), latches);
    assert!(
        bus.memory == memory,
        "unexpected memory change: {bytes:02X?}"
    );
    let mut trace = fetches(initial.pc, bytes);
    trace.extend_from_slice(accesses);
    assert_eq!(bus.take_log(), trace, "{bytes:02X?}");
}

#[test]
fn mov_registers_every_encoding() {
    // 98-153B Ch. 4 p. 4-4: 01DDDSSS, r1 <- r2, 5 states, no flags.
    // Second witness: 9800301-04 Ch. 3 p. 3-36, including self moves.
    for (pc, next, _, _) in LOCATIONS {
        for mut initial in initial_states(pc) {
            for (destination, dst) in REGISTERS {
                for (source, src) in REGISTERS {
                    let value = *register(&mut initial, src);
                    let mut expected = initial;
                    *register(&mut expected, dst) = value;
                    expected.pc = next;
                    witness(
                        initial,
                        &[0x40 + destination * 8 + source],
                        &[],
                        expected,
                        5,
                        &[],
                    );
                }
            }
        }
    }
}

#[test]
fn mov_memory_every_encoding_and_ffff_address() {
    // 98-153B Ch. 4 p. 4-4; Ch. 2 p. 2-16: M2 HL read/write, 7 states.
    // 9800301-04 Ch. 3 pp. 3-36/3-37. Includes H/L destinations and sources:
    // use the original HL address, even when loading H or L changes it.
    for (pc, next, _, _) in LOCATIONS {
        for initial in initial_states(pc) {
            // Avoid overlapping the instruction at FFFF here; covered below.
            for (h, l, address) in [(0x56, 0x78, 0x5678), (0xFF, 0xFF, 0xFFFF)] {
                if address == pc {
                    continue;
                }
                let mut initial = Registers { h, l, ..initial };
                for (code, name) in REGISTERS {
                    let mut expected = initial;
                    *register(&mut expected, name) = 0xA5;
                    expected.pc = next;
                    witness(
                        initial,
                        &[0x46 + code * 8],
                        &[(address, 0xA5)],
                        expected,
                        7,
                        &[data_read(address, 0xA5)],
                    );
                    let value = *register(&mut initial, name);
                    witness(
                        initial,
                        &[0x70 + code],
                        &[],
                        Registers {
                            pc: next,
                            ..initial
                        },
                        7,
                        &[data_write(address, value)],
                    );
                }
            }
        }
    }
}

#[test]
fn mvi_every_encoding_and_operand_wrap() {
    // 98-153B Ch. 4 p. 4-4: 00DDD110; 7 states register, 10 memory.
    // Ch. 2 pp. 2-16/2-17: M2 byte 2 read, M3 HL write for memory.
    // 9800301-04 Ch. 3 pp. 3-37/3-38 agrees.
    for (pc, _, next, _) in LOCATIONS {
        for initial in initial_states(pc) {
            for value in [0x00, 0xA5, 0xFF] {
                for (code, name) in REGISTERS {
                    let mut expected = initial;
                    *register(&mut expected, name) = value;
                    expected.pc = next;
                    witness(initial, &[0x06 + code * 8, value], &[], expected, 7, &[]);
                }
                witness(
                    initial,
                    &[0x36, value],
                    &[],
                    Registers {
                        pc: next,
                        ..initial
                    },
                    10,
                    &[data_write(0x5678, value)],
                );
                let initial = Registers {
                    h: 0xFF,
                    l: 0xFF,
                    ..initial
                };
                witness(
                    initial,
                    &[0x36, value],
                    &[],
                    Registers {
                        pc: next,
                        ..initial
                    },
                    10,
                    &[data_write(0xFFFF, value)],
                );
            }
        }
    }
}

#[test]
fn lxi_every_pair_and_operand_wrap() {
    // 98-153B Ch. 4 pp. 4-3/4-4: RP table; low byte 2, high byte 3,
    // 10 states; Ch. 2 pp. 2-16/2-17 M2/M3. 9800301-04 Ch. 3 p. 3-35.
    for (pc, _, _, next) in LOCATIONS {
        for initial in initial_states(pc) {
            for (opcode, pair) in [(0x01, "bc"), (0x11, "de"), (0x21, "hl"), (0x31, "sp")] {
                let mut expected = Registers {
                    pc: next,
                    ..initial
                };
                match pair {
                    "bc" => {
                        expected.b = 0xA5;
                        expected.c = 0x39;
                    }
                    "de" => {
                        expected.d = 0xA5;
                        expected.e = 0x39;
                    }
                    "hl" => {
                        expected.h = 0xA5;
                        expected.l = 0x39;
                    }
                    "sp" => expected.sp = 0xA539,
                    _ => unreachable!(),
                }
                witness(initial, &[opcode, 0x39, 0xA5], &[], expected, 10, &[]);
            }
        }
    }
}

#[test]
fn lda_sta_direct_addresses_and_operand_wrap() {
    // 98-153B Ch. 4 p. 4-5: LDA/STA, 13 states; Ch. 2 pp. 2-16/2-17:
    // M2 low address, M3 high address, M4 data read/write.
    // 9800301-04 Ch. 3 pp. 3-32/3-33 (LDA), pp. 3-61/3-62 (STA).
    for (pc, _, _, next) in LOCATIONS {
        for initial in initial_states(pc) {
            for (lo, hi, address) in [(0xEF, 0x23, 0x23EF), (0xFF, 0xFF, 0xFFFF)] {
                // Separate opcode/operand overlap witnesses below.
                if address == 0xFFFF && pc != 0x4000 {
                    continue;
                }
                witness(
                    initial,
                    &[0x3A, lo, hi],
                    &[(address, 0xA5)],
                    Registers {
                        pc: next,
                        a: 0xA5,
                        ..initial
                    },
                    13,
                    &[data_read(address, 0xA5)],
                );
                witness(
                    initial,
                    &[0x32, lo, hi],
                    &[],
                    Registers {
                        pc: next,
                        ..initial
                    },
                    13,
                    &[data_write(address, 0xE7)],
                );
            }
        }
    }
}

#[test]
fn lhld_shld_byte_order_and_address_wrap() {
    // 98-153B Ch. 4 p. 4-5: 16 states, L at address then H at address+1;
    // Ch. 2 pp. 2-16/2-17: M2/M3 address, M4 L, increment WZ, M5 H.
    // 9800301-04 Ch. 3 p. 3-34 (LHLD), pp. 3-58/3-59 (SHLD).
    // The FFFF wrap is derived, explicitly unverified (module caveat).
    for (pc, _, _, next) in LOCATIONS {
        for initial in initial_states(pc) {
            for (lo, hi, address, second) in
                [(0xFF, 0x23, 0x23FF, 0x2400), (0xFF, 0xFF, 0xFFFF, 0x0000)]
            {
                if address == 0xFFFF && pc != 0x4000 {
                    continue;
                }
                witness(
                    initial,
                    &[0x2A, lo, hi],
                    &[(address, 0x39), (second, 0xA5)],
                    Registers {
                        pc: next,
                        l: 0x39,
                        h: 0xA5,
                        ..initial
                    },
                    16,
                    &[data_read(address, 0x39), data_read(second, 0xA5)],
                );
                witness(
                    initial,
                    &[0x22, lo, hi],
                    &[],
                    Registers {
                        pc: next,
                        ..initial
                    },
                    16,
                    &[data_write(address, 0x78), data_write(second, 0x56)],
                );
            }
        }
    }
}

#[test]
fn ldax_stax_both_pairs_and_ffff_address() {
    // 98-153B Ch. 4 p. 4-5: BC or DE only, A read/write, 7 states;
    // Ch. 2 p. 2-16: M2 pair-addressed read/write.
    // 9800301-04 Ch. 3 p. 3-33 (LDAX), p. 3-62 (STAX).
    for (pc, next, _, _) in LOCATIONS {
        for initial in initial_states(pc) {
            for (load, store, pair) in [(0x0A, 0x02, "bc"), (0x1A, 0x12, "de")] {
                for (hi, lo, address) in [(0x23, 0xEF, 0x23EF), (0xFF, 0xFF, 0xFFFF)] {
                    if address == pc {
                        continue;
                    }
                    let mut initial = initial;
                    if pair == "bc" {
                        initial.b = hi;
                        initial.c = lo;
                    } else {
                        initial.d = hi;
                        initial.e = lo;
                    }
                    witness(
                        initial,
                        &[load],
                        &[(address, 0xA5)],
                        Registers {
                            pc: next,
                            a: 0xA5,
                            ..initial
                        },
                        7,
                        &[data_read(address, 0xA5)],
                    );
                    witness(
                        initial,
                        &[store],
                        &[],
                        Registers {
                            pc: next,
                            ..initial
                        },
                        7,
                        &[data_write(address, 0xE7)],
                    );
                }
            }
        }
    }
}

#[test]
fn xchg_swaps_both_bytes_without_memory_access() {
    // 98-153B Ch. 4 p. 4-5, Ch. 2 p. 2-16: H <-> D, L <-> E,
    // 4 states, M1 only; 9800301-04 Ch. 3 p. 3-65.
    for (pc, next, _, _) in LOCATIONS {
        for initial in initial_states(pc) {
            witness(
                initial,
                &[0xEB],
                &[],
                Registers {
                    pc: next,
                    h: 0x9A,
                    l: 0xBC,
                    d: 0x56,
                    e: 0x78,
                    ..initial
                },
                4,
                &[],
            );
        }
    }
}

#[test]
fn data_accesses_can_alias_instruction_bytes() {
    // Same entries/cycle rows as above. No prefetch or stack classification:
    // each data access occurs after all instruction bytes have been fetched.
    for initial in initial_states(0x4000) {
        // SHLD overwrites both its own address operand bytes only after M3.
        witness(
            initial,
            &[0x22, 0x01, 0x40],
            &[],
            Registers {
                pc: 0x4003,
                ..initial
            },
            16,
            &[data_write(0x4001, 0x78), data_write(0x4002, 0x56)],
        );
        // LHLD uses a captured direct address even while it replaces HL.
        witness(
            initial,
            &[0x2A, 0x01, 0x40],
            &[],
            Registers {
                pc: 0x4003,
                l: 0x01,
                h: 0x40,
                ..initial
            },
            16,
            &[data_read(0x4001, 0x01), data_read(0x4002, 0x40)],
        );
        let initial = Registers {
            h: 0x40,
            l: 0x01,
            ..initial
        };
        witness(
            initial,
            &[0x36, 0xA5],
            &[],
            Registers {
                pc: 0x4002,
                ..initial
            },
            10,
            &[data_write(0x4001, 0xA5)],
        );
    }
    for initial in initial_states(0xFFFF) {
        let initial = Registers {
            h: 0xFF,
            l: 0xFF,
            ..initial
        };
        witness(
            initial,
            &[0x66],
            &[],
            Registers {
                pc: 0x0000,
                h: 0x66,
                ..initial
            },
            7,
            &[data_read(0xFFFF, 0x66)],
        );
        witness(
            initial,
            &[0x74],
            &[],
            Registers {
                pc: 0x0000,
                ..initial
            },
            7,
            &[data_write(0xFFFF, 0xFF)],
        );
    }
    // Both the instruction stream and the direct data pair cross FFFF.
    // Wrap remains derived/unverified as described at module level.
    for initial in initial_states(0xFFFE) {
        witness(
            initial,
            &[0x2A, 0xFF, 0xFF],
            &[],
            Registers {
                pc: 0x0001,
                l: 0xFF,
                h: 0xFF,
                ..initial
            },
            16,
            &[data_read(0xFFFF, 0xFF), data_read(0x0000, 0xFF)],
        );
        witness(
            initial,
            &[0x22, 0xFF, 0xFF],
            &[],
            Registers {
                pc: 0x0001,
                ..initial
            },
            16,
            &[data_write(0xFFFF, 0x78), data_write(0x0000, 0x56)],
        );
    }
}
