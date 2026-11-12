//! Independent map witness: Intel 205775-002 (January 1983) Table 5-1
//! p. 5-19 (8085A column) and Table 5-2 p. 5-20, read on page images.
//! Cross-read: Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! No expectations use the encoding lists or the shared builder.
//!
//! Disagreements retained explicitly: both sequence tables put Adr at C9
//! instead of CA. 9800301-04 p. A-1 gives CPE 9/17; 205775-002 p. 5-14
//! and Table 5-1 give 9/18. All other lengths/state totals agree with the
//! per-instruction pages; the tests below pin these exceptions.

use super::{Group, OPCODES};

const DT: Group = Group::DataTransfer;
const AR: Group = Group::Arithmetic;
const LO: Group = Group::Logical;
const BR: Group = Group::Branch;
const SC: Group = Group::StackIoControl;
const RS: Group = Group::RimSim;

// group, Table 5-1 row, code, byte count, states (untaken, taken).
// Lowercase d/s are register-only fields (110 excluded); p and x take all
// values. Table 5-1's memory rows and HLT are separate. PUSH/POP RP include
// PSW in the last slot (per-instruction pp. 5-15/5-16, App. A p. A-2).
type Row = (Group, &'static str, &'static [u8; 8], u8, u8, u8);

const TABLE_5_1: &[Row] = &[
    (AR, "ACI", b"11001110", 2, 7, 7),
    (AR, "ADC REG", b"10001sss", 1, 4, 4),
    (AR, "ADC M", b"10001110", 1, 7, 7),
    (AR, "ADD REG", b"10000sss", 1, 4, 4),
    (AR, "ADD M", b"10000110", 1, 7, 7),
    (AR, "ADI", b"11000110", 2, 7, 7),
    (LO, "ANA REG", b"10100sss", 1, 4, 4),
    (LO, "ANA M", b"10100110", 1, 7, 7),
    (LO, "ANI", b"11100110", 2, 7, 7),
    (BR, "CALL", b"11001101", 3, 18, 18),
    (BR, "CC", b"11011100", 3, 9, 18),
    (BR, "CM", b"11111100", 3, 9, 18),
    (LO, "CMA", b"00101111", 1, 4, 4),
    (LO, "CMC", b"00111111", 1, 4, 4),
    (LO, "CMP REG", b"10111sss", 1, 4, 4),
    (LO, "CMP M", b"10111110", 1, 7, 7),
    (BR, "CNC", b"11010100", 3, 9, 18),
    (BR, "CNZ", b"11000100", 3, 9, 18),
    (BR, "CP", b"11110100", 3, 9, 18),
    (BR, "CPE", b"11101100", 3, 9, 18),
    (LO, "CPI", b"11111110", 2, 7, 7),
    (BR, "CPO", b"11100100", 3, 9, 18),
    (BR, "CZ", b"11001100", 3, 9, 18),
    (AR, "DAA", b"00100111", 1, 4, 4),
    (AR, "DAD", b"00pp1001", 1, 10, 10),
    (AR, "DCR REG", b"00ddd101", 1, 4, 4),
    (AR, "DCR M", b"00110101", 1, 10, 10),
    (AR, "DCX", b"00pp1011", 1, 6, 6),
    (SC, "DI", b"11110011", 1, 4, 4),
    (SC, "EI", b"11111011", 1, 4, 4),
    (SC, "HLT", b"01110110", 1, 5, 5),
    (SC, "IN", b"11011011", 2, 10, 10),
    (AR, "INR REG", b"00sss100", 1, 4, 4),
    (AR, "INR M", b"00110100", 1, 10, 10),
    (AR, "INX", b"00pp0011", 1, 6, 6),
    (BR, "JC", b"11011010", 3, 7, 10),
    (BR, "JM", b"11111010", 3, 7, 10),
    (BR, "JMP", b"11000011", 3, 10, 10),
    (BR, "JNC", b"11010010", 3, 7, 10),
    (BR, "JNZ", b"11000010", 3, 7, 10),
    (BR, "JP", b"11110010", 3, 7, 10),
    (BR, "JPE", b"11101010", 3, 7, 10),
    (BR, "JPO", b"11100010", 3, 7, 10),
    (BR, "JZ", b"11001010", 3, 7, 10),
    (DT, "LDA", b"00111010", 3, 13, 13),
    (DT, "LDAX", b"000x1010", 1, 7, 7),
    (DT, "LHLD", b"00101010", 3, 16, 16),
    (DT, "LXI", b"00pp0001", 3, 10, 10),
    (DT, "MOV REG,REG", b"01dddsss", 1, 4, 4),
    (DT, "MOV M,REG", b"01110sss", 1, 7, 7),
    (DT, "MOV REG,M", b"01ddd110", 1, 7, 7),
    (DT, "MVI REG", b"00ddd110", 2, 7, 7),
    (DT, "MVI M", b"00110110", 2, 10, 10),
    (SC, "NOP", b"00000000", 1, 4, 4),
    (LO, "ORA REG", b"10110sss", 1, 4, 4),
    (LO, "ORA M", b"10110110", 1, 7, 7),
    (LO, "ORI", b"11110110", 2, 7, 7),
    (SC, "OUT", b"11010011", 2, 10, 10),
    (BR, "PCHL", b"11101001", 1, 6, 6),
    (SC, "POP", b"11pp0001", 1, 10, 10),
    (SC, "PUSH", b"11pp0101", 1, 12, 12),
    (LO, "RAL", b"00010111", 1, 4, 4),
    (LO, "RAR", b"00011111", 1, 4, 4),
    (BR, "RC", b"11011000", 1, 6, 12),
    (BR, "RET", b"11001001", 1, 10, 10),
    (RS, "RIM", b"00100000", 1, 4, 4),
    (LO, "RLC", b"00000111", 1, 4, 4),
    (BR, "RM", b"11111000", 1, 6, 12),
    (BR, "RNC", b"11010000", 1, 6, 12),
    (BR, "RNZ", b"11000000", 1, 6, 12),
    (BR, "RP", b"11110000", 1, 6, 12),
    (BR, "RPE", b"11101000", 1, 6, 12),
    (BR, "RPO", b"11100000", 1, 6, 12),
    (LO, "RRC", b"00001111", 1, 4, 4),
    (BR, "RST", b"11xxx111", 1, 12, 12),
    (BR, "RZ", b"11001000", 1, 6, 12),
    (AR, "SBB REG", b"10011sss", 1, 4, 4),
    (AR, "SBB M", b"10011110", 1, 7, 7),
    (AR, "SBI", b"11011110", 2, 7, 7),
    (DT, "SHLD", b"00100010", 3, 16, 16),
    (RS, "SIM", b"00110000", 1, 4, 4),
    (SC, "SPHL", b"11111001", 1, 6, 6),
    (DT, "STA", b"00110010", 3, 13, 13),
    (DT, "STAX", b"000x0010", 1, 7, 7),
    (LO, "STC", b"00110111", 1, 4, 4),
    (AR, "SUB REG", b"10010sss", 1, 4, 4),
    (AR, "SUB M", b"10010110", 1, 7, 7),
    (AR, "SUI", b"11010110", 2, 7, 7),
    (DT, "XCHG", b"11101011", 1, 4, 4),
    (LO, "XRA REG", b"10101sss", 1, 4, 4),
    (LO, "XRA M", b"10101110", 1, 7, 7),
    (LO, "XRI", b"11101110", 2, 7, 7),
    (SC, "XTHL", b"11100011", 1, 16, 16),
];

/// Table 5-2 p. 5-20, opcode order, preserving the C9/CA marker slip.
/// Cross-read with 9800301-04 p. A-5: same instruction/operand entries;
/// A-5 prints DRC at 1D, SPD16 at 31 and 8G for opcode 90.
#[rustfmt::skip]
const TABLE_5_2: [&str; 256] = [
    // 00-0F
    "NOP", "LXI B,D16", "STAX B", "INX B", "INR B", "DCR B", "MVI B,D8", "RLC",
    "-", "DAD B", "LDAX B", "DCX B", "INR C", "DCR C", "MVI C,D8", "RRC",
    // 10-1F
    "-", "LXI D,D16", "STAX D", "INX D", "INR D", "DCR D", "MVI D,D8", "RAL",
    "-", "DAD D", "LDAX D", "DCX D", "INR E", "DCR E", "MVI E,D8", "RAR",
    // 20-2F
    "RIM", "LXI H,D16", "SHLD Adr", "INX H", "INR H", "DCR H", "MVI H,D8", "DAA",
    "-", "DAD H", "LHLD Adr", "DCX H", "INR L", "DCR L", "MVI L,D8", "CMA",
    // 30-3F
    "SIM", "LXI SP,D16", "STA Adr", "INX SP", "INR M", "DCR M", "MVI M,D8", "STC",
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

// Literal dashes in Table 5-2, corroborated by 9800301-04 p. A-5.
const UNDOCUMENTED: [u8; 10] = [0x08, 0x10, 0x18, 0x28, 0x38, 0xCB, 0xD9, 0xDD, 0xED, 0xFD];

fn row_matches(code: &[u8; 8], byte: u8) -> bool {
    let mut register = 0;
    let mut source = 0;
    for (position, symbol) in code.iter().enumerate() {
        let bit = (byte >> (7 - position)) & 1;
        match symbol {
            b'0' if bit != 0 => return false,
            b'1' if bit != 1 => return false,
            b'd' => register = register * 2 + bit,
            b's' => source = source * 2 + bit,
            b'0' | b'1' | b'p' | b'x' => {}
            _ => panic!("unknown reference-table symbol"),
        }
    }
    register != 6 && source != 6
}

#[test]
fn table_5_1_independently_witnesses_every_group_length_and_state_total() {
    let mut count = 0;
    for byte in 0..=u8::MAX {
        let mut rows = TABLE_5_1.iter().filter(|row| row_matches(row.2, byte));
        let row = rows.next();
        assert!(
            rows.next().is_none(),
            "overlapping witness rows for {byte:02X}"
        );
        let actual = OPCODES[usize::from(byte)];
        assert_eq!(actual.is_some(), row.is_some(), "{byte:02X}");
        if let Some(row) = row {
            let instruction = actual.unwrap();
            assert_eq!(
                (
                    instruction.group,
                    instruction.length,
                    instruction.states.untaken,
                    instruction.states.taken
                ),
                (row.0, row.3, row.4, row.5),
                "{byte:02X}: {}",
                row.1
            );
            count += 1;
        }
    }
    assert_eq!(count, 246);
}

#[test]
fn table_5_2_has_246_documented_and_ten_undocumented_bytes() {
    for byte in 0..=u8::MAX {
        let listed = TABLE_5_2[usize::from(byte)] != "-";
        assert_eq!(listed, !UNDOCUMENTED.contains(&byte), "{byte:02X}");
        assert_eq!(OPCODES[usize::from(byte)].is_some(), listed, "{byte:02X}");
    }
    let mut lengths = [0; 4];
    for instruction in OPCODES.iter().flatten() {
        lengths[usize::from(instruction.length)] += 1;
    }
    assert_eq!(lengths, [0, 202, 18, 26]);
}

#[test]
fn c9_ca_are_the_only_table_5_2_operand_marker_disagreements() {
    // decision-11: per-instruction pp. 5-13/5-14 win. Both Table 5-2
    // and 9800301-04 p. A-5 print RET Adr at C9 and JZ alone at CA.
    // Second witness: 9800301-04 pp. 3-32/3-48, JZ/RET.
    assert_eq!((TABLE_5_2[0xC9], TABLE_5_2[0xCA]), ("RET Adr", "JZ"));
    let mut mismatches = [None; 256];
    for (byte, entry) in TABLE_5_2.iter().enumerate() {
        if let Some(instruction) = OPCODES[byte] {
            let implied = if entry.contains("Adr") || entry.contains("D16") {
                3
            } else if entry.contains("D8") {
                2
            } else {
                1
            };
            if implied != instruction.length {
                mismatches[byte] = Some((implied, instruction.length));
            }
        }
    }
    let mut expected = [None; 256];
    expected[0xC9] = Some((3, 1));
    expected[0xCA] = Some((1, 3));
    assert_eq!(mismatches, expected);
}

#[test]
fn cpe_appendix_a1_timing_disagreement_is_explicit() {
    // 9800301-04 p. A-1's 8085 column, read as printed, has CPE 9/17.
    // 205775-002 p. 5-14 Ccondition and Table 5-1 p. 5-19 say 9/18;
    // this model follows that primary source. No silent normalization.
    let appendix_a1_cpe = (9, 17);
    let table_5_1_cpe = (9, 18);
    assert_ne!(appendix_a1_cpe, table_5_1_cpe);
    let instruction = OPCODES[0xEC].unwrap();
    assert_eq!(
        (instruction.states.untaken, instruction.states.taken),
        table_5_1_cpe
    );
}
