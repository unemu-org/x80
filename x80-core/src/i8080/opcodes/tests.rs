//! Witness for the opcode map (decision-3, PROVENANCE.md §1).
//!
//! The map is built from the per-instruction entries of 98-153B Ch. 4. The
//! tables below are transcribed separately, by hand, from two summaries, each
//! checked against the page image:
//!
//! - [`SUMMARY_4_15`]: 98-153B p. 4-15, "Summary of Processor Instructions",
//!   with the instruction code bits and the clock cycles of every row;
//! - [`SEQUENCE_A5`]: 9800301-04 p. A-5, "8080/85 CPU Instructions in
//!   Operation Code Sequence".
//!
//! Neither summary names the Ch. 4 groups; the group of each p. 4-15 row is
//! taken from the group heading over its entry (pp. 4-4, 4-6, 4-8, 4-11,
//! 4-13). Nothing here calls the map's builder.

use super::{Group, Instruction, States, OPCODES};

const DT: Group = Group::DataTransfer;
const AR: Group = Group::Arithmetic;
const LO: Group = Group::Logical;
const BR: Group = Group::Branch;
const SC: Group = Group::StackIoControl;

/// One row of p. 4-15: group, mnemonic, instruction code D7 to D0 as printed,
/// and the clock cycles (the smaller and larger total, equal for one total).
type Row = (Group, &'static str, &'static [u8; 8], u8, u8);

/// 98-153B p. 4-15, left column then right column, in printed order.
///
/// Fields (note 1): DDD or SSS is 000 B, 001 C, 010 D, 011 E, 100 H, 101 L,
/// 110 Memory, 111 A. The rows marked `r` take the seven register codes; the
/// memory forms are rows of their own, and 01110110 is HLT. `AAA` in the RST
/// row takes every value.
///
/// Legibility: the D1 digit of the RNZ row is printed as a broken glyph that
/// resembles "C". It is read as 0: the Rcondition entry on p. 4-12 is
/// 11CCC000, NZ is CCC 000 (p. 4-11), and 9800301-04 prints RNZ as 11000000
/// (p. A-2) and C0 (p. A-5).
const SUMMARY_4_15: &[Row] = &[
    (DT, "MOV r1,r2", b"01DDDSSS", 5, 5),
    (DT, "MOV M,r", b"01110SSS", 7, 7),
    (DT, "MOV r,M", b"01DDD110", 7, 7),
    (SC, "HLT", b"01110110", 7, 7),
    (DT, "MVI r", b"00DDD110", 7, 7),
    (DT, "MVI M", b"00110110", 10, 10),
    (AR, "INR r", b"00DDD100", 5, 5),
    (AR, "DCR r", b"00DDD101", 5, 5),
    (AR, "INR M", b"00110100", 10, 10),
    (AR, "DCR M", b"00110101", 10, 10),
    (AR, "ADD r", b"10000SSS", 4, 4),
    (AR, "ADC r", b"10001SSS", 4, 4),
    (AR, "SUB r", b"10010SSS", 4, 4),
    (AR, "SBB r", b"10011SSS", 4, 4),
    (LO, "ANA r", b"10100SSS", 4, 4),
    (LO, "XRA r", b"10101SSS", 4, 4),
    (LO, "ORA r", b"10110SSS", 4, 4),
    (LO, "CMP r", b"10111SSS", 4, 4),
    (AR, "ADD M", b"10000110", 7, 7),
    (AR, "ADC M", b"10001110", 7, 7),
    (AR, "SUB M", b"10010110", 7, 7),
    (AR, "SBB M", b"10011110", 7, 7),
    (LO, "ANA M", b"10100110", 7, 7),
    (LO, "XRA M", b"10101110", 7, 7),
    (LO, "ORA M", b"10110110", 7, 7),
    (LO, "CMP M", b"10111110", 7, 7),
    (AR, "ADI", b"11000110", 7, 7),
    (AR, "ACI", b"11001110", 7, 7),
    (AR, "SUI", b"11010110", 7, 7),
    (AR, "SBI", b"11011110", 7, 7),
    (LO, "ANI", b"11100110", 7, 7),
    (LO, "XRI", b"11101110", 7, 7),
    (LO, "ORI", b"11110110", 7, 7),
    (LO, "CPI", b"11111110", 7, 7),
    (LO, "RLC", b"00000111", 4, 4),
    (LO, "RRC", b"00001111", 4, 4),
    (LO, "RAL", b"00010111", 4, 4),
    (LO, "RAR", b"00011111", 4, 4),
    (BR, "JMP", b"11000011", 10, 10),
    (BR, "JC", b"11011010", 10, 10),
    (BR, "JNC", b"11010010", 10, 10),
    (BR, "JZ", b"11001010", 10, 10),
    (BR, "JNZ", b"11000010", 10, 10),
    (BR, "JP", b"11110010", 10, 10),
    (BR, "JM", b"11111010", 10, 10),
    (BR, "JPE", b"11101010", 10, 10),
    (BR, "JPO", b"11100010", 10, 10),
    (BR, "CALL", b"11001101", 17, 17),
    (BR, "CC", b"11011100", 11, 17),
    (BR, "CNC", b"11010100", 11, 17),
    (BR, "CZ", b"11001100", 11, 17),
    (BR, "CNZ", b"11000100", 11, 17),
    (BR, "CP", b"11110100", 11, 17),
    (BR, "CM", b"11111100", 11, 17),
    (BR, "CPE", b"11101100", 11, 17),
    (BR, "CPO", b"11100100", 11, 17),
    (BR, "RET", b"11001001", 10, 10),
    (BR, "RC", b"11011000", 5, 11),
    (BR, "RNC", b"11010000", 5, 11),
    // Right column.
    (BR, "RZ", b"11001000", 5, 11),
    (BR, "RNZ", b"11000000", 5, 11),
    (BR, "RP", b"11110000", 5, 11),
    (BR, "RM", b"11111000", 5, 11),
    (BR, "RPE", b"11101000", 5, 11),
    (BR, "RPO", b"11100000", 5, 11),
    (BR, "RST", b"11AAA111", 11, 11),
    (SC, "IN", b"11011011", 10, 10),
    (SC, "OUT", b"11010011", 10, 10),
    (DT, "LXI B", b"00000001", 10, 10),
    (DT, "LXI D", b"00010001", 10, 10),
    (DT, "LXI H", b"00100001", 10, 10),
    (DT, "LXI SP", b"00110001", 10, 10),
    (SC, "PUSH B", b"11000101", 11, 11),
    (SC, "PUSH D", b"11010101", 11, 11),
    (SC, "PUSH H", b"11100101", 11, 11),
    (SC, "PUSH PSW", b"11110101", 11, 11),
    (SC, "POP B", b"11000001", 10, 10),
    (SC, "POP D", b"11010001", 10, 10),
    (SC, "POP H", b"11100001", 10, 10),
    (SC, "POP PSW", b"11110001", 10, 10),
    (DT, "STA", b"00110010", 13, 13),
    (DT, "LDA", b"00111010", 13, 13),
    (DT, "XCHG", b"11101011", 4, 4),
    (SC, "XTHL", b"11100011", 18, 18),
    (SC, "SPHL", b"11111001", 5, 5),
    (BR, "PCHL", b"11101001", 5, 5),
    (AR, "DAD B", b"00001001", 10, 10),
    (AR, "DAD D", b"00011001", 10, 10),
    (AR, "DAD H", b"00101001", 10, 10),
    (AR, "DAD SP", b"00111001", 10, 10),
    (DT, "STAX B", b"00000010", 7, 7),
    (DT, "STAX D", b"00010010", 7, 7),
    (DT, "LDAX B", b"00001010", 7, 7),
    (DT, "LDAX D", b"00011010", 7, 7),
    (AR, "INX B", b"00000011", 5, 5),
    (AR, "INX D", b"00010011", 5, 5),
    (AR, "INX H", b"00100011", 5, 5),
    (AR, "INX SP", b"00110011", 5, 5),
    (AR, "DCX B", b"00001011", 5, 5),
    (AR, "DCX D", b"00011011", 5, 5),
    (AR, "DCX H", b"00101011", 5, 5),
    (AR, "DCX SP", b"00111011", 5, 5),
    (LO, "CMA", b"00101111", 4, 4),
    (LO, "STC", b"00110111", 4, 4),
    (LO, "CMC", b"00111111", 4, 4),
    (AR, "DAA", b"00100111", 4, 4),
    (DT, "SHLD", b"00100010", 16, 16),
    (DT, "LHLD", b"00101010", 16, 16),
    (SC, "EI", b"11111011", 4, 4),
    (SC, "DI", b"11110011", 4, 4),
    (SC, "NOP", b"00000000", 4, 4),
];

/// 9800301-04 p. A-5, by opcode, mnemonic and operand as printed; `-` is the
/// dash the table prints for an opcode with no instruction.
///
/// Operand symbols (p. A-5 footnote): D8 is 8-bit data, D16 is 16-bit data and
/// Adr is a 16-bit address. Printing faults kept as printed: 1D reads
/// "DRC E"; the opcode column labels 90 as "8G"; C9 reads "RET Adr" and CA
/// reads "JZ" without "Adr" (see `lengths_agree_with_the_operands_on_a5`).
#[rustfmt::skip]
const SEQUENCE_A5: [&str; 256] = [
    // 00-0F
    "NOP", "LXI B,D16", "STAX B", "INX B", "INR B", "DCR B", "MVI B,D8", "RLC",
    "-", "DAD B", "LDAX B", "DCX B", "INR C", "DCR C", "MVI C,D8", "RRC",
    // 10-1F
    "-", "LXI D,D16", "STAX D", "INX D", "INR D", "DCR D", "MVI D,D8", "RAL",
    "-", "DAD D", "LDAX D", "DCX D", "INR E", "DRC E", "MVI E,D8", "RAR",
    // 20-2F
    "RIM", "LXI H,D16", "SHLD Adr", "INX H", "INR H", "DCR H", "MVI H,D8", "DAA",
    "-", "DAD H", "LHLD Adr", "DCX H", "INR L", "DCR L", "MVI L,D8", "CMA",
    // 30-3F
    "SIM", "LXI SPD16", "STA Adr", "INX SP", "INR M", "DCR M", "MVI M,D8", "STC",
    "-", "DAD SP", "LDA Adr", "DCX SP", "INR A", "DCR A", "MVI A,D8", "CMC",
    // 40-4F
    "MOV B,B", "MOV B,C", "MOV B,D", "MOV B,E", "MOV B,H", "MOV B,L", "MOV B,M", "MOV B,A",
    "MOV C,B", "MOV C,C", "MOV C,D", "MOV C,E", "MOV C,H", "MOV C,L", "MOV C,M", "MOV C,A",
    // 50-5F
    "MOV D,B", "MOV D,C", "MOV D,D", "MOV D,E", "MOV D,H", "MOV D,L", "MOV D,M", "MOV D,A",
    "MOV E,B", "MOV E,C", "MOV E,D", "MOV E,E", "MOV E,H", "MOV E,L", "MOV E,M", "MOV E,A",
    // 60-6F
    "MOV H,B", "MOV H,C", "MOV H,D", "MOV H,E", "MOV H,H", "MOV H,L", "MOV H,M", "MOV H,A",
    "MOV L,B", "MOV L,C", "MOV L,D", "MOV L,E", "MOV L,H", "MOV L,L", "MOV L,M", "MOV L,A",
    // 70-7F
    "MOV M,B", "MOV M,C", "MOV M,D", "MOV M,E", "MOV M,H", "MOV M,L", "HLT", "MOV M,A",
    "MOV A,B", "MOV A,C", "MOV A,D", "MOV A,E", "MOV A,H", "MOV A,L", "MOV A,M", "MOV A,A",
    // 80-8F
    "ADD B", "ADD C", "ADD D", "ADD E", "ADD H", "ADD L", "ADD M", "ADD A",
    "ADC B", "ADC C", "ADC D", "ADC E", "ADC H", "ADC L", "ADC M", "ADC A",
    // 90-9F
    "SUB B", "SUB C", "SUB D", "SUB E", "SUB H", "SUB L", "SUB M", "SUB A",
    "SBB B", "SBB C", "SBB D", "SBB E", "SBB H", "SBB L", "SBB M", "SBB A",
    // A0-AF
    "ANA B", "ANA C", "ANA D", "ANA E", "ANA H", "ANA L", "ANA M", "ANA A",
    "XRA B", "XRA C", "XRA D", "XRA E", "XRA H", "XRA L", "XRA M", "XRA A",
    // B0-BF
    "ORA B", "ORA C", "ORA D", "ORA E", "ORA H", "ORA L", "ORA M", "ORA A",
    "CMP B", "CMP C", "CMP D", "CMP E", "CMP H", "CMP L", "CMP M", "CMP A",
    // C0-CF
    "RNZ", "POP B", "JNZ Adr", "JMP Adr", "CNZ Adr", "PUSH B", "ADI D8", "RST 0",
    "RZ", "RET Adr", "JZ", "-", "CZ Adr", "CALL Adr", "ACI D8", "RST 1",
    // D0-DF
    "RNC", "POP D", "JNC Adr", "OUT D8", "CNC Adr", "PUSH D", "SUI D8", "RST 2",
    "RC", "-", "JC Adr", "IN D8", "CC Adr", "-", "SBI D8", "RST 3",
    // E0-EF
    "RPO", "POP H", "JPO Adr", "XTHL", "CPO Adr", "PUSH H", "ANI D8", "RST 4",
    "RPE", "PCHL", "JPE Adr", "XCHG", "CPE Adr", "-", "XRI D8", "RST 5",
    // F0-FF
    "RP", "POP PSW", "JP Adr", "DI", "CP Adr", "PUSH PSW", "ORI D8", "RST 6",
    "RM", "SPHL", "JM Adr", "EI", "CM Adr", "-", "CPI D8", "RST 7",
];

/// The opcodes that 98-153B Ch. 4 does not document, worked out by hand as
/// the complement of its entries.
const UNDOCUMENTED: [u8; 12] = [
    0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0xCB, 0xD9, 0xDD, 0xED, 0xFD,
];

/// Whether `byte` has the instruction code of a p. 4-15 row.
fn row_matches(bits: &[u8; 8], byte: u8) -> bool {
    for (index, &symbol) in bits.iter().enumerate() {
        let bit = (byte >> (7 - index)) & 1;
        match symbol {
            b'0' if bit != 0 => return false,
            b'1' if bit != 1 => return false,
            b'0' | b'1' | b'D' | b'S' | b'A' => {}
            other => panic!("unexpected symbol {:?} in a p. 4-15 row", other as char),
        }
    }
    // Rows marked `r` exclude 110, Memory (note 1).
    let ddd = bits[2..5] == *b"DDD" && (byte >> 3) & 0b111 == 0b110;
    let sss = bits[5..8] == *b"SSS" && byte & 0b111 == 0b110;
    !ddd && !sss
}

/// The p. 4-15 row of each opcode byte; panics if two rows claim one byte.
fn summary_by_opcode() -> [Option<Row>; 256] {
    let mut by_opcode = [None; 256];
    for row in SUMMARY_4_15 {
        for byte in 0..=u8::MAX {
            if row_matches(row.2, byte) {
                let slot = &mut by_opcode[usize::from(byte)];
                assert!(
                    slot.is_none(),
                    "rows {:?} and {:?} claim {byte:#04x}",
                    slot,
                    row.1
                );
                *slot = Some(*row);
            }
        }
    }
    by_opcode
}

/// Whether p. A-5 lists an 8080 instruction at `opcode`. RIM and SIM are
/// 8085 instructions: p. A-4 prints "—" in their 8080 column, p. A-6 marks
/// them "8085 ONLY" and p. 3-48 heads RIM "(8085 PROCESSOR ONLY)".
fn sequence_lists_8080(opcode: u8) -> bool {
    !matches!(SEQUENCE_A5[usize::from(opcode)], "-" | "RIM" | "SIM")
}

/// The length that the operand symbol of a p. A-5 entry implies.
fn sequence_length(opcode: u8) -> u8 {
    let entry = SEQUENCE_A5[usize::from(opcode)];
    if entry.contains("D16") || entry.contains("Adr") {
        3
    } else if entry.contains("D8") {
        2
    } else {
        1
    }
}

#[test]
fn summary_rows_give_the_documented_set_and_the_groups() {
    let summary = summary_by_opcode();
    let mut documented = 0;
    for byte in 0..=u8::MAX {
        let in_map = OPCODES[usize::from(byte)];
        let row = summary[usize::from(byte)];
        assert_eq!(
            in_map.map(|instruction| instruction.group),
            row.map(|row| row.0),
            "opcode {byte:#04x}: map {in_map:?}, p. 4-15 row {row:?}"
        );
        documented += usize::from(row.is_some());
    }
    assert_eq!(documented, 244);
}

#[test]
fn the_undocumented_set_is_the_complement() {
    for byte in 0..=u8::MAX {
        assert_eq!(
            OPCODES[usize::from(byte)].is_none(),
            UNDOCUMENTED.contains(&byte),
            "opcode {byte:#04x}"
        );
    }
}

#[test]
fn the_opcode_sequence_on_a5_lists_the_same_set() {
    for byte in 0..=u8::MAX {
        assert_eq!(
            OPCODES[usize::from(byte)].is_some(),
            sequence_lists_8080(byte),
            "opcode {byte:#04x}: p. A-5 lists {:?}",
            SEQUENCE_A5[usize::from(byte)]
        );
    }
}

#[test]
fn lengths_agree_with_the_operands_on_a5() {
    // p. A-5 prints "RET Adr" at C9 and "JZ" without "Adr" at CA, as if the
    // operand were shifted up one row. RET is one byte: 98-153B p. 4-12 shows
    // one instruction byte, and 9800301-04 p. 3-48 says "Operands are not
    // permitted with the RET instruction". JZ is three bytes: the
    // Jcondition entry on 98-153B p. 4-11 and JZ on 9800301-04 p. 3-32 show
    // the low and high address bytes. These two rows are the only
    // disagreement; the map follows the per-instruction pages.
    let mut disagreements = [None; 256];
    for byte in 0..=u8::MAX {
        if let Some(instruction) = OPCODES[usize::from(byte)] {
            if instruction.length != sequence_length(byte) {
                disagreements[usize::from(byte)] = Some(instruction.length);
            }
        }
    }
    let mut expected = [None; 256];
    expected[0xC9] = Some(1);
    expected[0xCA] = Some(3);
    assert_eq!(disagreements, expected);
}

#[test]
fn length_counts_by_size() {
    // From the Ch. 4 entries: 26 three-byte instructions (LXI x4, LDA, STA,
    // LHLD, SHLD, JMP, Jcondition x8, CALL, Ccondition x8), 18 two-byte
    // instructions (MVI x8, ADI, ACI, SUI, SBI, ANI, XRI, ORI, CPI, IN, OUT)
    // and 200 one-byte instructions.
    let mut counts = [0; 4];
    for instruction in OPCODES.iter().flatten() {
        counts[usize::from(instruction.length)] += 1;
    }
    assert_eq!(counts, [0, 200, 18, 26]);
}

#[test]
fn cited_state_totals_match_the_summary() {
    let summary = summary_by_opcode();
    for byte in 0..=u8::MAX {
        let Some(Instruction { states, .. }) = OPCODES[usize::from(byte)] else {
            continue;
        };
        let (.., untaken, taken) = summary[usize::from(byte)].expect("documented");
        assert_eq!(states, States { untaken, taken }, "opcode {byte:#04x}");
    }
}

#[test]
fn nop_is_one_byte_of_four_states() {
    // 98-153B p. 4-14 (NOP: 00000000, Cycles 1, States 4) and p. 4-15;
    // 9800301-04 pp. A-4 and A-5.
    assert_eq!(
        OPCODES[0x00],
        Some(Instruction {
            group: Group::StackIoControl,
            length: 1,
            states: States {
                untaken: 4,
                taken: 4,
            },
        })
    );
}
