//! Independent 8085 data-transfer witnesses (decision-10; decision-12
//! rules 1/10; decision-5 items 3/7/8; plan §2.2/§2.6).
//!
//! Intel 205775-002 (January 1983), §5.2 pp. 5-1/5-2 and §5.6.1
//! pp. 5-4 to 5-6: fields, effects, lengths, states and unchanged flags.
//! Table 5-1 p. 5-19 supplies machine-cycle sequences; Table 5-2 p. 5-20
//! supplies opcode order (it is not the machine-cycle table). Ch. 2
//! §§2.3.1-2.3.3 pp. 2-8 to 2-12 distinguishes OF, MR and MW.
//! Intel 9800301-04 (May 1981), Ch. 3 entries cited below independently
//! agree, using the parenthesized 8085 total for MOV r,r. Every cited
//! value was checked on the page image; hashes match PROVENANCE.md §1.
//! Expected values use literal registers/bytes, never crate pair/flag helpers.
//! All forms have a single state total: `opcodes/tests.rs` independently
//! pins both path slots against Table 5-1; these tests measure `step` reports.
//!
//! Wrap witnesses are DERIVED: 205775-002 §2.2.1 p. 2-2 gives 16-bit PC,
//! §5.3 p. 5-2 gives 16-bit addresses and consecutive instruction bytes,
//! and p. 5-5 gives address+1 for LHLD/SHLD. These pages do not explicitly
//! state FFFFh -> 0000h; that edge is not claimed as a verified hardware fact.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Exit, MemoryAccess, Report};

fn initial(flags_set: bool) -> Registers {
    Registers {
        a: 0x91,
        b: 0x12,
        c: 0x34,
        d: 0x56,
        e: 0x78,
        h: 0x9A,
        l: 0xBC,
        sp: 0xDEF0,
        pc: 0x2000,
        flags: Flags {
            sign: flags_set,
            zero: flags_set,
            auxiliary_carry: flags_set,
            parity: flags_set,
            carry: flags_set,
        },
    }
}

// Literal register values from initial(), in §5.2 p. 5-1 field order.
const REGISTERS: [(&str, u8); 7] = [
    ("b", 0x12),
    ("c", 0x34),
    ("d", 0x56),
    ("e", 0x78),
    ("h", 0x9A),
    ("l", 0xBC),
    ("a", 0x91),
];

fn set_register(r: &mut Registers, name: &str, value: u8) {
    match name {
        "a" => r.a = value,
        "b" => r.b = value,
        "c" => r.c = value,
        "d" => r.d = value,
        "e" => r.e = value,
        "h" => r.h = value,
        "l" => r.l = value,
        _ => panic!("invalid test register"),
    }
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

fn witness(
    before: Registers,
    bytes: &[u8],
    seed: &[(u16, u8)],
    after: Registers,
    states: u64,
    accesses: &[Event],
) {
    let mut bus = RecordingBus::new();
    for &(address, value) in seed {
        bus.memory[usize::from(address)] = value;
    }
    bus.load(before.pc, bytes);
    let mut expected_memory = bus.memory.clone();
    for &event in accesses {
        if let Event::Write { address, value, .. } = event {
            expected_memory[usize::from(address)] = value;
        }
    }
    let latches = Latches {
        interrupt_enable: true,
        ..Latches::default()
    };
    let mut cpu = I8085::new(before, latches);
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: states,
            exit: Exit::Budget
        },
        "{bytes:02X?} at {:04X}",
        before.pc
    );
    assert_eq!(*cpu.registers(), after, "{bytes:02X?}");
    assert_eq!(*cpu.latches(), latches, "{bytes:02X?}");
    assert!(
        bus.memory == expected_memory,
        "unexpected memory change: {bytes:02X?}"
    );

    // Boundary polls: decision-12 rule 1. Instruction bytes: §5.3 p. 5-2;
    // OF at PC then MR operands: Ch. 2 pp. 2-9/2-12, Table 5-1 p. 5-19.
    // OperandFetch/Data split is the decision-5 item 3 API distinction.
    // Literal address vectors include both DERIVED operand-wrap positions.
    let addresses = match before.pc {
        0x2000 => [0x2000, 0x2001, 0x2002],
        0xFFFE => [0xFFFE, 0xFFFF, 0x0000],
        0xFFFF => [0xFFFF, 0x0000, 0x0001],
        _ => panic!("unspecified witness PC"),
    };
    let mut trace = vec![
        Event::StopPoll(false),
        Event::TrapPoll(false),
        Event::Rst7_5Poll(false),
        Event::Rst6_5Poll(false),
        Event::Rst5_5Poll(false),
        Event::IntrPoll(false),
    ];
    for (index, &value) in bytes.iter().enumerate() {
        trace.push(read(
            addresses[index],
            if index == 0 {
                MemoryAccess::OpcodeFetch
            } else {
                MemoryAccess::OperandFetch
            },
            value,
        ));
    }
    trace.extend_from_slice(accesses);
    assert_eq!(bus.take_log(), trace, "{bytes:02X?}");
}

#[test]
fn mov_registers_all_49_pairs() {
    // 205775-002 §5.6.1 p. 5-4: copy source, 4 states (8085), no flags.
    // 9800301-04 Ch. 3 p. 3-36: 5 (4 on 8085), including self moves.
    // Literal opcode matrix independently read from Table 5-2 p. 5-20;
    // rows/columns B,C,D,E,H,L,A. Table 5-1 p. 5-19: F only.
    let opcodes = [
        [0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x47],
        [0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4F],
        [0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x57],
        [0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5F],
        [0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x67],
        [0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6F],
        [0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7F],
    ];
    for flags in [false, true] {
        let before = initial(flags);
        for ((destination, _), row) in REGISTERS.into_iter().zip(opcodes) {
            for ((_, value), opcode) in REGISTERS.into_iter().zip(row) {
                let mut after = Registers {
                    pc: 0x2001,
                    ..before
                };
                set_register(&mut after, destination, value);
                witness(before, &[opcode], &[], after, 4, &[]);
            }
        }
    }
}

#[test]
fn mov_memory_all_14_forms_use_original_hl() {
    // 205775-002 §5.6.1 p. 5-4; 9800301-04 Ch. 3 pp. 3-36/3-37:
    // HL memory, no flags, 7 states. Table 5-1 p. 5-19: FR / FW.
    // H/L destinations must not change the address of the read.
    for flags in [false, true] {
        let before = initial(flags);
        for (((name, value), load), store) in REGISTERS
            .into_iter()
            .zip([0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x7E])
            .zip([0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77])
        {
            let mut after = Registers {
                pc: 0x2001,
                ..before
            };
            set_register(&mut after, name, 0xE3);
            witness(
                before,
                &[load],
                &[(0x9ABC, 0xE3)],
                after,
                7,
                &[data_read(0x9ABC, 0xE3)],
            );
            witness(
                before,
                &[store],
                &[(0x9ABC, 0xE3)],
                Registers {
                    pc: 0x2001,
                    ..before
                },
                7,
                &[data_write(0x9ABC, value)],
            );
        }
    }
}

#[test]
fn mvi_registers_all_seven_encodings() {
    // 205775-002 §5.6.1 p. 5-4; 9800301-04 Ch. 3 p. 3-37:
    // byte 2 to register, 7 states, no flags. Table 5-1 p. 5-19: FR.
    for flags in [false, true] {
        let before = initial(flags);
        for ((name, _), opcode) in REGISTERS
            .into_iter()
            .zip([0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x3E])
        {
            for value in [0x00, 0xE3, 0xFF] {
                let mut after = Registers {
                    pc: 0x2002,
                    ..before
                };
                set_register(&mut after, name, value);
                witness(before, &[opcode, value], &[], after, 7, &[]);
            }
        }
    }
}

#[test]
fn mvi_memory_fetches_immediate_then_writes_hl() {
    // 205775-002 §5.6.1 p. 5-4; 9800301-04 Ch. 3 p. 3-38:
    // byte 2 to (HL), 10 states, no flags. Table 5-1 p. 5-19: FRW.
    for flags in [false, true] {
        let before = initial(flags);
        for value in [0x00, 0xE3, 0xFF] {
            witness(
                before,
                &[0x36, value],
                &[(0x9ABC, 0xA5)],
                Registers {
                    pc: 0x2002,
                    ..before
                },
                10,
                &[data_write(0x9ABC, value)],
            );
        }
    }
}

#[test]
fn lxi_all_four_pairs_low_byte_first() {
    // 205775-002 §5.2 p. 5-1, §5.6.1 p. 5-5;
    // 9800301-04 Ch. 3 p. 3-35: B/D/H/SP, low then high, 10 states,
    // no flags. Table 5-1 p. 5-19: FRR.
    for flags in [false, true] {
        let before = initial(flags);
        let base = Registers {
            pc: 0x2003,
            ..before
        };
        for (opcode, after) in [
            (
                0x01,
                Registers {
                    b: 0x8D,
                    c: 0x27,
                    ..base
                },
            ),
            (
                0x11,
                Registers {
                    d: 0x8D,
                    e: 0x27,
                    ..base
                },
            ),
            (
                0x21,
                Registers {
                    h: 0x8D,
                    l: 0x27,
                    ..base
                },
            ),
            (0x31, Registers { sp: 0x8D27, ..base }),
        ] {
            witness(before, &[opcode, 0x27, 0x8D], &[], after, 10, &[]);
        }
    }
}

#[test]
fn lda_reads_direct_address_after_both_operands() {
    // 205775-002 §5.6.1 p. 5-5; 9800301-04 Ch. 3 pp. 3-32/3-33:
    // direct byte to A, 13 states, no flags. Table 5-1 p. 5-19: FRRR.
    for flags in [false, true] {
        let before = initial(flags);
        witness(
            before,
            &[0x3A, 0x27, 0x8D],
            &[(0x8D27, 0xE3)],
            Registers {
                a: 0xE3,
                pc: 0x2003,
                ..before
            },
            13,
            &[data_read(0x8D27, 0xE3)],
        );
    }
}

#[test]
fn sta_writes_direct_address_after_both_operands() {
    // 205775-002 §5.6.1 p. 5-5; 9800301-04 Ch. 3 pp. 3-61/3-62:
    // A to direct byte, 13 states, no flags. Table 5-1 p. 5-19: FRRW.
    for flags in [false, true] {
        let before = initial(flags);
        witness(
            before,
            &[0x32, 0x27, 0x8D],
            &[(0x8D27, 0xE3)],
            Registers {
                pc: 0x2003,
                ..before
            },
            13,
            &[data_write(0x8D27, 0x91)],
        );
    }
}

#[test]
fn lhld_reads_low_then_high_from_successive_addresses() {
    // 205775-002 §5.6.1 p. 5-5; 9800301-04 Ch. 3 p. 3-34:
    // (addr) to L, (addr+1) to H, 16 states, no flags.
    // Table 5-1 p. 5-19: FRRRR, operands before data.
    for flags in [false, true] {
        let before = initial(flags);
        witness(
            before,
            &[0x2A, 0x27, 0x8D],
            &[(0x8D27, 0xE3), (0x8D28, 0x45)],
            Registers {
                l: 0xE3,
                h: 0x45,
                pc: 0x2003,
                ..before
            },
            16,
            &[data_read(0x8D27, 0xE3), data_read(0x8D28, 0x45)],
        );
    }
}

#[test]
fn shld_writes_low_then_high_to_successive_addresses() {
    // 205775-002 §5.6.1 p. 5-5; 9800301-04 Ch. 3 pp. 3-58/3-59:
    // L to (addr), H to (addr+1), 16 states, no flags.
    // Table 5-1 p. 5-19: FRRWW, operands before data.
    for flags in [false, true] {
        let before = initial(flags);
        witness(
            before,
            &[0x22, 0x27, 0x8D],
            &[(0x8D27, 0xE3), (0x8D28, 0x45)],
            Registers {
                pc: 0x2003,
                ..before
            },
            16,
            &[data_write(0x8D27, 0xBC), data_write(0x8D28, 0x9A)],
        );
    }
}

#[test]
fn ldax_b_and_d_read_the_selected_pair() {
    // 205775-002 §5.6.1 p. 5-5; 9800301-04 Ch. 3 p. 3-33:
    // B/D only, indirect byte to A, 7 states, no flags.
    // Table 5-1 p. 5-19: FR.
    // 231718-001 Table 6 p. 16-28 prints LDAX B as 00101010 (2Ah).
    // Follow 205775-002 p. 5-5 "00RP1010" (RP=00 for B, p. 5-1),
    // Table 5-2 p. 5-20 "0A LDAX B", and 9800301-04 p. 3-33
    // "000r1010", r=0 for B. 2Ah is LHLD, witnessed separately above.
    for flags in [false, true] {
        let before = initial(flags);
        for (opcode, address, value) in [(0x0A, 0x1234, 0xE3), (0x1A, 0x5678, 0x45)] {
            witness(
                before,
                &[opcode],
                &[(0x1234, 0xE3), (0x5678, 0x45)],
                Registers {
                    a: value,
                    pc: 0x2001,
                    ..before
                },
                7,
                &[data_read(address, value)],
            );
        }
    }
}

#[test]
fn stax_b_and_d_write_the_selected_pair() {
    // 205775-002 §5.6.1 p. 5-6; 9800301-04 Ch. 3 p. 3-62:
    // B/D only, A to indirect byte, 7 states, no flags.
    // Table 5-1 p. 5-19: FW.
    for flags in [false, true] {
        let before = initial(flags);
        for (opcode, address) in [(0x02, 0x1234), (0x12, 0x5678)] {
            witness(
                before,
                &[opcode],
                &[(0x1234, 0xE3), (0x5678, 0x45)],
                Registers {
                    pc: 0x2001,
                    ..before
                },
                7,
                &[data_write(address, 0x91)],
            );
        }
    }
}

#[test]
fn xchg_swaps_h_with_d_and_l_with_e() {
    // 205775-002 §5.6.1 p. 5-6; 9800301-04 Ch. 3 p. 3-65:
    // H<->D and L<->E, 4 states, no flags. Table 5-1 p. 5-19: F only.
    for flags in [false, true] {
        let before = initial(flags);
        witness(
            before,
            &[0xEB],
            &[],
            Registers {
                h: 0x56,
                l: 0x78,
                d: 0x9A,
                e: 0xBC,
                pc: 0x2001,
                ..before
            },
            4,
            &[],
        );
    }
}

#[test]
fn derived_lhld_shld_wrap_ffff_to_zero() {
    // DERIVED wrap; see module caveat. Otherwise p. 5-5 / 9800301-04
    // pp. 3-34, 3-58/3-59 and Table 5-1 FRRRR/FRRWW, 16 states.
    for flags in [false, true] {
        let before = initial(flags);
        witness(
            before,
            &[0x2A, 0xFF, 0xFF],
            &[(0xFFFF, 0xE3), (0x0000, 0x45)],
            Registers {
                l: 0xE3,
                h: 0x45,
                pc: 0x2003,
                ..before
            },
            16,
            &[data_read(0xFFFF, 0xE3), data_read(0x0000, 0x45)],
        );
        witness(
            before,
            &[0x22, 0xFF, 0xFF],
            &[(0xFFFF, 0xE3), (0x0000, 0x45)],
            Registers {
                pc: 0x2003,
                ..before
            },
            16,
            &[data_write(0xFFFF, 0xBC), data_write(0x0000, 0x9A)],
        );
    }
}

#[test]
fn derived_pc_wrap_and_operand_fetch_wrap() {
    // DERIVED wrap at each of the three instruction-byte positions.
    // Normal effects/states: pp. 5-4/5-5; 9800301-04 pp. 3-35 to 3-37.
    for flags in [false, true] {
        let before = Registers {
            pc: 0xFFFF,
            ..initial(flags)
        };
        witness(
            before,
            &[0x47],
            &[],
            Registers {
                b: 0x91,
                pc: 0x0000,
                ..before
            },
            4,
            &[],
        );
        for (pc, next) in [(0xFFFE, 0x0000), (0xFFFF, 0x0001)] {
            let before = Registers {
                pc,
                ..initial(flags)
            };
            witness(
                before,
                &[0x3E, 0xE3],
                &[],
                Registers {
                    a: 0xE3,
                    pc: next,
                    ..before
                },
                7,
                &[],
            );
        }
        for (pc, next) in [(0xFFFE, 0x0001), (0xFFFF, 0x0002)] {
            let before = Registers {
                pc,
                ..initial(flags)
            };
            witness(
                before,
                &[0x31, 0x27, 0x8D],
                &[],
                Registers {
                    sp: 0x8D27,
                    pc: next,
                    ..before
                },
                10,
                &[],
            );
        }
    }
}

#[test]
fn data_accesses_can_overlap_instruction_bytes() {
    // §5.6.1 pp. 5-4/5-5 and 9800301-04 pp. 3-34, 3-38,
    // 3-58/3-59. Table 5-1 p. 5-19 orders all operand reads before writes.
    for flags in [false, true] {
        let before = Registers {
            h: 0x20,
            l: 0x01,
            ..initial(flags)
        };
        witness(
            before,
            &[0x36, 0xE3],
            &[],
            Registers {
                pc: 0x2002,
                ..before
            },
            10,
            &[data_write(0x2001, 0xE3)],
        );
        let before = initial(flags);
        witness(
            before,
            &[0x22, 0x01, 0x20],
            &[],
            Registers {
                pc: 0x2003,
                ..before
            },
            16,
            &[data_write(0x2001, 0xBC), data_write(0x2002, 0x9A)],
        );
        witness(
            before,
            &[0x2A, 0x01, 0x20],
            &[],
            Registers {
                l: 0x01,
                h: 0x20,
                pc: 0x2003,
                ..before
            },
            16,
            &[data_read(0x2001, 0x01), data_read(0x2002, 0x20)],
        );
    }
}
