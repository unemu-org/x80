//! Independent literal map transcription, read on rendered source pages.
//!
//! Primary: UM008011-0816, rev. 11 (August 2016), compact op-code tables
//! pp. 42–64, reconciled with the instruction entries pp. 71–316.
//! Cross-read: 03-0029-01 (1977), Tables 7.0-1–7.0-11, pp. 44–54.
//! PDF pages are respectively printed + 14 and printed + 6.
//! The sources share Zilog lineage; agreement is no hardware measurement.
//!
//! Audit checklist (each key, group, complete byte length and both totals):
//! MAIN 252/252; CB 256/256; ED 256/256; INDEX 85/85; INDEX_CB 256/256.
//! The last two maps each serve DD and FD: 1105 stored entries represent
//! 1446 effective logical entries; 334 ignored MAIN fallbacks bring terminal
//! coverage to 1780 (decision-20 §§1–3; dispatch slots excluded).
//! These literal lists neither use the encoding slices nor expand patterns.
//! The tests below pin the PROVENANCE.md §1 resolutions separately.

use super::{Group, Instruction, CB, ED, INDEX, INDEX_CB, MAIN};

const L8: Group = Group::Load8;
const L16: Group = Group::Load16;
const EX: Group = Group::ExchangeBlockSearch;
const A8: Group = Group::Arithmetic8;
const GC: Group = Group::GeneralArithmeticCpuControl;
const A16: Group = Group::Arithmetic16;
const RS: Group = Group::RotateShift;
const BT: Group = Group::BitSetResetTest;
const JP: Group = Group::Jump;
const CR: Group = Group::CallReturn;
const IO: Group = Group::InputOutput;

// key byte, group, complete length, terminating/untaken states, repeating/taken states.
type Row = (u8, Group, u8, u8, u8);

// UM Tables 6–13 pp. 42,45,47–52, Fig. 39 p. 54, Tables 15/17–20
// pp. 59,61–64; TM Tables 7.0-1–7.0-11 pp. 44–54. Costs cross-read with
// UM entries (per-group page ranges in decision-13 rule 2). In particular,
// JP cc is always 10 (UM pp. 263–264; TM p. 52), CALL cc is 10/17
// (UM pp. 283–284; TM p. 53), RET cc is 5/11 (UM pp. 286–287; TM p. 53).
const MAIN_ROWS: &[Row] = &[
    (0x00, GC, 1, 4, 4),
    (0x01, L16, 3, 10, 10),
    (0x02, L8, 1, 7, 7),
    (0x03, A16, 1, 6, 6),
    (0x04, A8, 1, 4, 4),
    (0x05, A8, 1, 4, 4),
    (0x06, L8, 2, 7, 7),
    (0x07, RS, 1, 4, 4),
    (0x08, EX, 1, 4, 4),
    (0x09, A16, 1, 11, 11),
    (0x0A, L8, 1, 7, 7),
    (0x0B, A16, 1, 6, 6),
    (0x0C, A8, 1, 4, 4),
    (0x0D, A8, 1, 4, 4),
    (0x0E, L8, 2, 7, 7),
    (0x0F, RS, 1, 4, 4),
    (0x10, JP, 2, 8, 13),
    (0x11, L16, 3, 10, 10),
    (0x12, L8, 1, 7, 7),
    (0x13, A16, 1, 6, 6),
    (0x14, A8, 1, 4, 4),
    (0x15, A8, 1, 4, 4),
    (0x16, L8, 2, 7, 7),
    (0x17, RS, 1, 4, 4),
    (0x18, JP, 2, 12, 12),
    (0x19, A16, 1, 11, 11),
    (0x1A, L8, 1, 7, 7),
    (0x1B, A16, 1, 6, 6),
    (0x1C, A8, 1, 4, 4),
    (0x1D, A8, 1, 4, 4),
    (0x1E, L8, 2, 7, 7),
    (0x1F, RS, 1, 4, 4),
    (0x20, JP, 2, 7, 12),
    (0x21, L16, 3, 10, 10),
    (0x22, L16, 3, 16, 16),
    (0x23, A16, 1, 6, 6),
    (0x24, A8, 1, 4, 4),
    (0x25, A8, 1, 4, 4),
    (0x26, L8, 2, 7, 7),
    (0x27, GC, 1, 4, 4),
    (0x28, JP, 2, 7, 12),
    (0x29, A16, 1, 11, 11),
    (0x2A, L16, 3, 16, 16),
    (0x2B, A16, 1, 6, 6),
    (0x2C, A8, 1, 4, 4),
    (0x2D, A8, 1, 4, 4),
    (0x2E, L8, 2, 7, 7),
    (0x2F, GC, 1, 4, 4),
    (0x30, JP, 2, 7, 12),
    (0x31, L16, 3, 10, 10),
    (0x32, L8, 3, 13, 13),
    (0x33, A16, 1, 6, 6),
    (0x34, A8, 1, 11, 11),
    (0x35, A8, 1, 11, 11),
    (0x36, L8, 2, 10, 10),
    (0x37, GC, 1, 4, 4),
    (0x38, JP, 2, 7, 12),
    (0x39, A16, 1, 11, 11),
    (0x3A, L8, 3, 13, 13),
    (0x3B, A16, 1, 6, 6),
    (0x3C, A8, 1, 4, 4),
    (0x3D, A8, 1, 4, 4),
    (0x3E, L8, 2, 7, 7),
    (0x3F, GC, 1, 4, 4),
    (0x40, L8, 1, 4, 4),
    (0x41, L8, 1, 4, 4),
    (0x42, L8, 1, 4, 4),
    (0x43, L8, 1, 4, 4),
    (0x44, L8, 1, 4, 4),
    (0x45, L8, 1, 4, 4),
    (0x46, L8, 1, 7, 7),
    (0x47, L8, 1, 4, 4),
    (0x48, L8, 1, 4, 4),
    (0x49, L8, 1, 4, 4),
    (0x4A, L8, 1, 4, 4),
    (0x4B, L8, 1, 4, 4),
    (0x4C, L8, 1, 4, 4),
    (0x4D, L8, 1, 4, 4),
    (0x4E, L8, 1, 7, 7),
    (0x4F, L8, 1, 4, 4),
    (0x50, L8, 1, 4, 4),
    (0x51, L8, 1, 4, 4),
    (0x52, L8, 1, 4, 4),
    (0x53, L8, 1, 4, 4),
    (0x54, L8, 1, 4, 4),
    (0x55, L8, 1, 4, 4),
    (0x56, L8, 1, 7, 7),
    (0x57, L8, 1, 4, 4),
    (0x58, L8, 1, 4, 4),
    (0x59, L8, 1, 4, 4),
    (0x5A, L8, 1, 4, 4),
    (0x5B, L8, 1, 4, 4),
    (0x5C, L8, 1, 4, 4),
    (0x5D, L8, 1, 4, 4),
    (0x5E, L8, 1, 7, 7),
    (0x5F, L8, 1, 4, 4),
    (0x60, L8, 1, 4, 4),
    (0x61, L8, 1, 4, 4),
    (0x62, L8, 1, 4, 4),
    (0x63, L8, 1, 4, 4),
    (0x64, L8, 1, 4, 4),
    (0x65, L8, 1, 4, 4),
    (0x66, L8, 1, 7, 7),
    (0x67, L8, 1, 4, 4),
    (0x68, L8, 1, 4, 4),
    (0x69, L8, 1, 4, 4),
    (0x6A, L8, 1, 4, 4),
    (0x6B, L8, 1, 4, 4),
    (0x6C, L8, 1, 4, 4),
    (0x6D, L8, 1, 4, 4),
    (0x6E, L8, 1, 7, 7),
    (0x6F, L8, 1, 4, 4),
    (0x70, L8, 1, 7, 7),
    (0x71, L8, 1, 7, 7),
    (0x72, L8, 1, 7, 7),
    (0x73, L8, 1, 7, 7),
    (0x74, L8, 1, 7, 7),
    (0x75, L8, 1, 7, 7),
    (0x76, GC, 1, 4, 4),
    (0x77, L8, 1, 7, 7),
    (0x78, L8, 1, 4, 4),
    (0x79, L8, 1, 4, 4),
    (0x7A, L8, 1, 4, 4),
    (0x7B, L8, 1, 4, 4),
    (0x7C, L8, 1, 4, 4),
    (0x7D, L8, 1, 4, 4),
    (0x7E, L8, 1, 7, 7),
    (0x7F, L8, 1, 4, 4),
    (0x80, A8, 1, 4, 4),
    (0x81, A8, 1, 4, 4),
    (0x82, A8, 1, 4, 4),
    (0x83, A8, 1, 4, 4),
    (0x84, A8, 1, 4, 4),
    (0x85, A8, 1, 4, 4),
    (0x86, A8, 1, 7, 7),
    (0x87, A8, 1, 4, 4),
    (0x88, A8, 1, 4, 4),
    (0x89, A8, 1, 4, 4),
    (0x8A, A8, 1, 4, 4),
    (0x8B, A8, 1, 4, 4),
    (0x8C, A8, 1, 4, 4),
    (0x8D, A8, 1, 4, 4),
    (0x8E, A8, 1, 7, 7),
    (0x8F, A8, 1, 4, 4),
    (0x90, A8, 1, 4, 4),
    (0x91, A8, 1, 4, 4),
    (0x92, A8, 1, 4, 4),
    (0x93, A8, 1, 4, 4),
    (0x94, A8, 1, 4, 4),
    (0x95, A8, 1, 4, 4),
    (0x96, A8, 1, 7, 7),
    (0x97, A8, 1, 4, 4),
    (0x98, A8, 1, 4, 4),
    (0x99, A8, 1, 4, 4),
    (0x9A, A8, 1, 4, 4),
    (0x9B, A8, 1, 4, 4),
    (0x9C, A8, 1, 4, 4),
    (0x9D, A8, 1, 4, 4),
    (0x9E, A8, 1, 7, 7),
    (0x9F, A8, 1, 4, 4),
    (0xA0, A8, 1, 4, 4),
    (0xA1, A8, 1, 4, 4),
    (0xA2, A8, 1, 4, 4),
    (0xA3, A8, 1, 4, 4),
    (0xA4, A8, 1, 4, 4),
    (0xA5, A8, 1, 4, 4),
    (0xA6, A8, 1, 7, 7),
    (0xA7, A8, 1, 4, 4),
    (0xA8, A8, 1, 4, 4),
    (0xA9, A8, 1, 4, 4),
    (0xAA, A8, 1, 4, 4),
    (0xAB, A8, 1, 4, 4),
    (0xAC, A8, 1, 4, 4),
    (0xAD, A8, 1, 4, 4),
    (0xAE, A8, 1, 7, 7),
    (0xAF, A8, 1, 4, 4),
    (0xB0, A8, 1, 4, 4),
    (0xB1, A8, 1, 4, 4),
    (0xB2, A8, 1, 4, 4),
    (0xB3, A8, 1, 4, 4),
    (0xB4, A8, 1, 4, 4),
    (0xB5, A8, 1, 4, 4),
    (0xB6, A8, 1, 7, 7),
    (0xB7, A8, 1, 4, 4),
    (0xB8, A8, 1, 4, 4),
    (0xB9, A8, 1, 4, 4),
    (0xBA, A8, 1, 4, 4),
    (0xBB, A8, 1, 4, 4),
    (0xBC, A8, 1, 4, 4),
    (0xBD, A8, 1, 4, 4),
    (0xBE, A8, 1, 7, 7),
    (0xBF, A8, 1, 4, 4),
    (0xC0, CR, 1, 5, 11),
    (0xC1, L16, 1, 10, 10),
    (0xC2, JP, 3, 10, 10),
    (0xC3, JP, 3, 10, 10),
    (0xC4, CR, 3, 10, 17),
    (0xC5, L16, 1, 11, 11),
    (0xC6, A8, 2, 7, 7),
    (0xC7, CR, 1, 11, 11),
    (0xC8, CR, 1, 5, 11),
    (0xC9, CR, 1, 10, 10),
    (0xCA, JP, 3, 10, 10),
    (0xCC, CR, 3, 10, 17),
    (0xCD, CR, 3, 17, 17),
    (0xCE, A8, 2, 7, 7),
    (0xCF, CR, 1, 11, 11),
    (0xD0, CR, 1, 5, 11),
    (0xD1, L16, 1, 10, 10),
    (0xD2, JP, 3, 10, 10),
    (0xD3, IO, 2, 11, 11),
    (0xD4, CR, 3, 10, 17),
    (0xD5, L16, 1, 11, 11),
    (0xD6, A8, 2, 7, 7),
    (0xD7, CR, 1, 11, 11),
    (0xD8, CR, 1, 5, 11),
    (0xD9, EX, 1, 4, 4),
    (0xDA, JP, 3, 10, 10),
    (0xDB, IO, 2, 11, 11),
    (0xDC, CR, 3, 10, 17),
    (0xDE, A8, 2, 7, 7),
    (0xDF, CR, 1, 11, 11),
    (0xE0, CR, 1, 5, 11),
    (0xE1, L16, 1, 10, 10),
    (0xE2, JP, 3, 10, 10),
    (0xE3, EX, 1, 19, 19),
    (0xE4, CR, 3, 10, 17),
    (0xE5, L16, 1, 11, 11),
    (0xE6, A8, 2, 7, 7),
    (0xE7, CR, 1, 11, 11),
    (0xE8, CR, 1, 5, 11),
    (0xE9, JP, 1, 4, 4),
    (0xEA, JP, 3, 10, 10),
    (0xEB, EX, 1, 4, 4),
    (0xEC, CR, 3, 10, 17),
    (0xEE, A8, 2, 7, 7),
    (0xEF, CR, 1, 11, 11),
    (0xF0, CR, 1, 5, 11),
    (0xF1, L16, 1, 10, 10),
    (0xF2, JP, 3, 10, 10),
    (0xF3, GC, 1, 4, 4),
    (0xF4, CR, 3, 10, 17),
    (0xF5, L16, 1, 11, 11),
    (0xF6, A8, 2, 7, 7),
    (0xF7, CR, 1, 11, 11),
    (0xF8, CR, 1, 5, 11),
    (0xF9, L16, 1, 6, 6),
    (0xFA, JP, 3, 10, 10),
    (0xFB, GC, 1, 4, 4),
    (0xFC, CR, 3, 10, 17),
    (0xFE, A8, 2, 7, 7),
    (0xFF, CR, 1, 11, 11),
];

// UM Fig. 39 p. 54 / Table 14 pp. 55–57, corrected by entries pp. 213–237,
// 243–260; TM Tables 7.0-7/8 pp. 50–51. The register rows take 8 states;
// memory rotates/RES/SET take 15, BIT memory takes 12. CB 30–37 are absent.
const CB_ROWS: &[Row] = &[
    (0x00, RS, 2, 8, 8),
    (0x01, RS, 2, 8, 8),
    (0x02, RS, 2, 8, 8),
    (0x03, RS, 2, 8, 8),
    (0x04, RS, 2, 8, 8),
    (0x05, RS, 2, 8, 8),
    (0x06, RS, 2, 15, 15),
    (0x07, RS, 2, 8, 8),
    (0x08, RS, 2, 8, 8),
    (0x09, RS, 2, 8, 8),
    (0x0A, RS, 2, 8, 8),
    (0x0B, RS, 2, 8, 8),
    (0x0C, RS, 2, 8, 8),
    (0x0D, RS, 2, 8, 8),
    (0x0E, RS, 2, 15, 15),
    (0x0F, RS, 2, 8, 8),
    (0x10, RS, 2, 8, 8),
    (0x11, RS, 2, 8, 8),
    (0x12, RS, 2, 8, 8),
    (0x13, RS, 2, 8, 8),
    (0x14, RS, 2, 8, 8),
    (0x15, RS, 2, 8, 8),
    (0x16, RS, 2, 15, 15),
    (0x17, RS, 2, 8, 8),
    (0x18, RS, 2, 8, 8),
    (0x19, RS, 2, 8, 8),
    (0x1A, RS, 2, 8, 8),
    (0x1B, RS, 2, 8, 8),
    (0x1C, RS, 2, 8, 8),
    (0x1D, RS, 2, 8, 8),
    (0x1E, RS, 2, 15, 15),
    (0x1F, RS, 2, 8, 8),
    (0x20, RS, 2, 8, 8),
    (0x21, RS, 2, 8, 8),
    (0x22, RS, 2, 8, 8),
    (0x23, RS, 2, 8, 8),
    (0x24, RS, 2, 8, 8),
    (0x25, RS, 2, 8, 8),
    (0x26, RS, 2, 15, 15),
    (0x27, RS, 2, 8, 8),
    (0x28, RS, 2, 8, 8),
    (0x29, RS, 2, 8, 8),
    (0x2A, RS, 2, 8, 8),
    (0x2B, RS, 2, 8, 8),
    (0x2C, RS, 2, 8, 8),
    (0x2D, RS, 2, 8, 8),
    (0x2E, RS, 2, 15, 15),
    (0x2F, RS, 2, 8, 8),
    (0x38, RS, 2, 8, 8),
    (0x39, RS, 2, 8, 8),
    (0x3A, RS, 2, 8, 8),
    (0x3B, RS, 2, 8, 8),
    (0x3C, RS, 2, 8, 8),
    (0x3D, RS, 2, 8, 8),
    (0x3E, RS, 2, 15, 15),
    (0x3F, RS, 2, 8, 8),
    (0x40, BT, 2, 8, 8),
    (0x41, BT, 2, 8, 8),
    (0x42, BT, 2, 8, 8),
    (0x43, BT, 2, 8, 8),
    (0x44, BT, 2, 8, 8),
    (0x45, BT, 2, 8, 8),
    (0x46, BT, 2, 12, 12),
    (0x47, BT, 2, 8, 8),
    (0x48, BT, 2, 8, 8),
    (0x49, BT, 2, 8, 8),
    (0x4A, BT, 2, 8, 8),
    (0x4B, BT, 2, 8, 8),
    (0x4C, BT, 2, 8, 8),
    (0x4D, BT, 2, 8, 8),
    (0x4E, BT, 2, 12, 12),
    (0x4F, BT, 2, 8, 8),
    (0x50, BT, 2, 8, 8),
    (0x51, BT, 2, 8, 8),
    (0x52, BT, 2, 8, 8),
    (0x53, BT, 2, 8, 8),
    (0x54, BT, 2, 8, 8),
    (0x55, BT, 2, 8, 8),
    (0x56, BT, 2, 12, 12),
    (0x57, BT, 2, 8, 8),
    (0x58, BT, 2, 8, 8),
    (0x59, BT, 2, 8, 8),
    (0x5A, BT, 2, 8, 8),
    (0x5B, BT, 2, 8, 8),
    (0x5C, BT, 2, 8, 8),
    (0x5D, BT, 2, 8, 8),
    (0x5E, BT, 2, 12, 12),
    (0x5F, BT, 2, 8, 8),
    (0x60, BT, 2, 8, 8),
    (0x61, BT, 2, 8, 8),
    (0x62, BT, 2, 8, 8),
    (0x63, BT, 2, 8, 8),
    (0x64, BT, 2, 8, 8),
    (0x65, BT, 2, 8, 8),
    (0x66, BT, 2, 12, 12),
    (0x67, BT, 2, 8, 8),
    (0x68, BT, 2, 8, 8),
    (0x69, BT, 2, 8, 8),
    (0x6A, BT, 2, 8, 8),
    (0x6B, BT, 2, 8, 8),
    (0x6C, BT, 2, 8, 8),
    (0x6D, BT, 2, 8, 8),
    (0x6E, BT, 2, 12, 12),
    (0x6F, BT, 2, 8, 8),
    (0x70, BT, 2, 8, 8),
    (0x71, BT, 2, 8, 8),
    (0x72, BT, 2, 8, 8),
    (0x73, BT, 2, 8, 8),
    (0x74, BT, 2, 8, 8),
    (0x75, BT, 2, 8, 8),
    (0x76, BT, 2, 12, 12),
    (0x77, BT, 2, 8, 8),
    (0x78, BT, 2, 8, 8),
    (0x79, BT, 2, 8, 8),
    (0x7A, BT, 2, 8, 8),
    (0x7B, BT, 2, 8, 8),
    (0x7C, BT, 2, 8, 8),
    (0x7D, BT, 2, 8, 8),
    (0x7E, BT, 2, 12, 12),
    (0x7F, BT, 2, 8, 8),
    (0x80, BT, 2, 8, 8),
    (0x81, BT, 2, 8, 8),
    (0x82, BT, 2, 8, 8),
    (0x83, BT, 2, 8, 8),
    (0x84, BT, 2, 8, 8),
    (0x85, BT, 2, 8, 8),
    (0x86, BT, 2, 15, 15),
    (0x87, BT, 2, 8, 8),
    (0x88, BT, 2, 8, 8),
    (0x89, BT, 2, 8, 8),
    (0x8A, BT, 2, 8, 8),
    (0x8B, BT, 2, 8, 8),
    (0x8C, BT, 2, 8, 8),
    (0x8D, BT, 2, 8, 8),
    (0x8E, BT, 2, 15, 15),
    (0x8F, BT, 2, 8, 8),
    (0x90, BT, 2, 8, 8),
    (0x91, BT, 2, 8, 8),
    (0x92, BT, 2, 8, 8),
    (0x93, BT, 2, 8, 8),
    (0x94, BT, 2, 8, 8),
    (0x95, BT, 2, 8, 8),
    (0x96, BT, 2, 15, 15),
    (0x97, BT, 2, 8, 8),
    (0x98, BT, 2, 8, 8),
    (0x99, BT, 2, 8, 8),
    (0x9A, BT, 2, 8, 8),
    (0x9B, BT, 2, 8, 8),
    (0x9C, BT, 2, 8, 8),
    (0x9D, BT, 2, 8, 8),
    (0x9E, BT, 2, 15, 15),
    (0x9F, BT, 2, 8, 8),
    (0xA0, BT, 2, 8, 8),
    (0xA1, BT, 2, 8, 8),
    (0xA2, BT, 2, 8, 8),
    (0xA3, BT, 2, 8, 8),
    (0xA4, BT, 2, 8, 8),
    (0xA5, BT, 2, 8, 8),
    (0xA6, BT, 2, 15, 15),
    (0xA7, BT, 2, 8, 8),
    (0xA8, BT, 2, 8, 8),
    (0xA9, BT, 2, 8, 8),
    (0xAA, BT, 2, 8, 8),
    (0xAB, BT, 2, 8, 8),
    (0xAC, BT, 2, 8, 8),
    (0xAD, BT, 2, 8, 8),
    (0xAE, BT, 2, 15, 15),
    (0xAF, BT, 2, 8, 8),
    (0xB0, BT, 2, 8, 8),
    (0xB1, BT, 2, 8, 8),
    (0xB2, BT, 2, 8, 8),
    (0xB3, BT, 2, 8, 8),
    (0xB4, BT, 2, 8, 8),
    (0xB5, BT, 2, 8, 8),
    (0xB6, BT, 2, 15, 15),
    (0xB7, BT, 2, 8, 8),
    (0xB8, BT, 2, 8, 8),
    (0xB9, BT, 2, 8, 8),
    (0xBA, BT, 2, 8, 8),
    (0xBB, BT, 2, 8, 8),
    (0xBC, BT, 2, 8, 8),
    (0xBD, BT, 2, 8, 8),
    (0xBE, BT, 2, 15, 15),
    (0xBF, BT, 2, 8, 8),
    (0xC0, BT, 2, 8, 8),
    (0xC1, BT, 2, 8, 8),
    (0xC2, BT, 2, 8, 8),
    (0xC3, BT, 2, 8, 8),
    (0xC4, BT, 2, 8, 8),
    (0xC5, BT, 2, 8, 8),
    (0xC6, BT, 2, 15, 15),
    (0xC7, BT, 2, 8, 8),
    (0xC8, BT, 2, 8, 8),
    (0xC9, BT, 2, 8, 8),
    (0xCA, BT, 2, 8, 8),
    (0xCB, BT, 2, 8, 8),
    (0xCC, BT, 2, 8, 8),
    (0xCD, BT, 2, 8, 8),
    (0xCE, BT, 2, 15, 15),
    (0xCF, BT, 2, 8, 8),
    (0xD0, BT, 2, 8, 8),
    (0xD1, BT, 2, 8, 8),
    (0xD2, BT, 2, 8, 8),
    (0xD3, BT, 2, 8, 8),
    (0xD4, BT, 2, 8, 8),
    (0xD5, BT, 2, 8, 8),
    (0xD6, BT, 2, 15, 15),
    (0xD7, BT, 2, 8, 8),
    (0xD8, BT, 2, 8, 8),
    (0xD9, BT, 2, 8, 8),
    (0xDA, BT, 2, 8, 8),
    (0xDB, BT, 2, 8, 8),
    (0xDC, BT, 2, 8, 8),
    (0xDD, BT, 2, 8, 8),
    (0xDE, BT, 2, 15, 15),
    (0xDF, BT, 2, 8, 8),
    (0xE0, BT, 2, 8, 8),
    (0xE1, BT, 2, 8, 8),
    (0xE2, BT, 2, 8, 8),
    (0xE3, BT, 2, 8, 8),
    (0xE4, BT, 2, 8, 8),
    (0xE5, BT, 2, 8, 8),
    (0xE6, BT, 2, 15, 15),
    (0xE7, BT, 2, 8, 8),
    (0xE8, BT, 2, 8, 8),
    (0xE9, BT, 2, 8, 8),
    (0xEA, BT, 2, 8, 8),
    (0xEB, BT, 2, 8, 8),
    (0xEC, BT, 2, 8, 8),
    (0xED, BT, 2, 8, 8),
    (0xEE, BT, 2, 15, 15),
    (0xEF, BT, 2, 8, 8),
    (0xF0, BT, 2, 8, 8),
    (0xF1, BT, 2, 8, 8),
    (0xF2, BT, 2, 8, 8),
    (0xF3, BT, 2, 8, 8),
    (0xF4, BT, 2, 8, 8),
    (0xF5, BT, 2, 8, 8),
    (0xF6, BT, 2, 15, 15),
    (0xF7, BT, 2, 8, 8),
    (0xF8, BT, 2, 8, 8),
    (0xF9, BT, 2, 8, 8),
    (0xFA, BT, 2, 8, 8),
    (0xFB, BT, 2, 8, 8),
    (0xFC, BT, 2, 8, 8),
    (0xFD, BT, 2, 8, 8),
    (0xFE, BT, 2, 15, 15),
    (0xFF, BT, 2, 8, 8),
    // H SLL / L costs: decision-20 §3; Young v0.91 §3.1 p. 11,
    // §8.7 p. 32; Rak v1.2a .slia_a/.slia_r (testtable 81–100).
    (0x30, RS, 2, 8, 8),
    (0x31, RS, 2, 8, 8),
    (0x32, RS, 2, 8, 8),
    (0x33, RS, 2, 8, 8),
    (0x34, RS, 2, 8, 8),
    (0x35, RS, 2, 8, 8),
    (0x36, RS, 2, 15, 15),
    (0x37, RS, 2, 8, 8),
];

// UM Tables 6/7/9/10/12/13/15/18–20 pp. 42–64, Fig. 39 p. 54;
// TM pp. 44–54. UM entries pp. 103,108 and TM p. 45 explicitly admit
// ED 63/6B. Decision-20 §1 adds A ED70 (TM Table 7.0-11 p. 54),
// H NEG/returns (Young v0.91 §3.4 pp. 12–13; Rak .neg_/.reti_retn),
// L IM/ED71/cost extensions and 178 NOPs (Young §§3.4/8.11 pp. 12–13/35).
const ED_ROWS: &[Row] = &[
    (0x00, GC, 2, 8, 8),
    (0x01, GC, 2, 8, 8),
    (0x02, GC, 2, 8, 8),
    (0x03, GC, 2, 8, 8),
    (0x04, GC, 2, 8, 8),
    (0x05, GC, 2, 8, 8),
    (0x06, GC, 2, 8, 8),
    (0x07, GC, 2, 8, 8),
    (0x08, GC, 2, 8, 8),
    (0x09, GC, 2, 8, 8),
    (0x0A, GC, 2, 8, 8),
    (0x0B, GC, 2, 8, 8),
    (0x0C, GC, 2, 8, 8),
    (0x0D, GC, 2, 8, 8),
    (0x0E, GC, 2, 8, 8),
    (0x0F, GC, 2, 8, 8),
    (0x10, GC, 2, 8, 8),
    (0x11, GC, 2, 8, 8),
    (0x12, GC, 2, 8, 8),
    (0x13, GC, 2, 8, 8),
    (0x14, GC, 2, 8, 8),
    (0x15, GC, 2, 8, 8),
    (0x16, GC, 2, 8, 8),
    (0x17, GC, 2, 8, 8),
    (0x18, GC, 2, 8, 8),
    (0x19, GC, 2, 8, 8),
    (0x1A, GC, 2, 8, 8),
    (0x1B, GC, 2, 8, 8),
    (0x1C, GC, 2, 8, 8),
    (0x1D, GC, 2, 8, 8),
    (0x1E, GC, 2, 8, 8),
    (0x1F, GC, 2, 8, 8),
    (0x20, GC, 2, 8, 8),
    (0x21, GC, 2, 8, 8),
    (0x22, GC, 2, 8, 8),
    (0x23, GC, 2, 8, 8),
    (0x24, GC, 2, 8, 8),
    (0x25, GC, 2, 8, 8),
    (0x26, GC, 2, 8, 8),
    (0x27, GC, 2, 8, 8),
    (0x28, GC, 2, 8, 8),
    (0x29, GC, 2, 8, 8),
    (0x2A, GC, 2, 8, 8),
    (0x2B, GC, 2, 8, 8),
    (0x2C, GC, 2, 8, 8),
    (0x2D, GC, 2, 8, 8),
    (0x2E, GC, 2, 8, 8),
    (0x2F, GC, 2, 8, 8),
    (0x30, GC, 2, 8, 8),
    (0x31, GC, 2, 8, 8),
    (0x32, GC, 2, 8, 8),
    (0x33, GC, 2, 8, 8),
    (0x34, GC, 2, 8, 8),
    (0x35, GC, 2, 8, 8),
    (0x36, GC, 2, 8, 8),
    (0x37, GC, 2, 8, 8),
    (0x38, GC, 2, 8, 8),
    (0x39, GC, 2, 8, 8),
    (0x3A, GC, 2, 8, 8),
    (0x3B, GC, 2, 8, 8),
    (0x3C, GC, 2, 8, 8),
    (0x3D, GC, 2, 8, 8),
    (0x3E, GC, 2, 8, 8),
    (0x3F, GC, 2, 8, 8),
    (0x40, IO, 2, 12, 12),
    (0x41, IO, 2, 12, 12),
    (0x42, A16, 2, 15, 15),
    (0x43, L16, 4, 20, 20),
    (0x44, GC, 2, 8, 8),
    (0x45, CR, 2, 14, 14),
    (0x46, GC, 2, 8, 8),
    (0x47, L8, 2, 9, 9),
    (0x48, IO, 2, 12, 12),
    (0x49, IO, 2, 12, 12),
    (0x4A, A16, 2, 15, 15),
    (0x4B, L16, 4, 20, 20),
    (0x4C, GC, 2, 8, 8),
    (0x4D, CR, 2, 14, 14),
    (0x4E, GC, 2, 8, 8),
    (0x4F, L8, 2, 9, 9),
    (0x50, IO, 2, 12, 12),
    (0x51, IO, 2, 12, 12),
    (0x52, A16, 2, 15, 15),
    (0x53, L16, 4, 20, 20),
    (0x54, GC, 2, 8, 8),
    (0x55, CR, 2, 14, 14),
    (0x56, GC, 2, 8, 8),
    (0x57, L8, 2, 9, 9),
    (0x58, IO, 2, 12, 12),
    (0x59, IO, 2, 12, 12),
    (0x5A, A16, 2, 15, 15),
    (0x5B, L16, 4, 20, 20),
    (0x5C, GC, 2, 8, 8),
    (0x5D, CR, 2, 14, 14),
    (0x5E, GC, 2, 8, 8),
    (0x5F, L8, 2, 9, 9),
    (0x60, IO, 2, 12, 12),
    (0x61, IO, 2, 12, 12),
    (0x62, A16, 2, 15, 15),
    (0x63, L16, 4, 20, 20),
    (0x64, GC, 2, 8, 8),
    (0x65, CR, 2, 14, 14),
    (0x66, GC, 2, 8, 8),
    (0x67, RS, 2, 18, 18),
    (0x68, IO, 2, 12, 12),
    (0x69, IO, 2, 12, 12),
    (0x6A, A16, 2, 15, 15),
    (0x6B, L16, 4, 20, 20),
    (0x6C, GC, 2, 8, 8),
    (0x6D, CR, 2, 14, 14),
    (0x6E, GC, 2, 8, 8),
    (0x6F, RS, 2, 18, 18),
    (0x70, IO, 2, 12, 12),
    (0x71, IO, 2, 12, 12),
    (0x72, A16, 2, 15, 15),
    (0x73, L16, 4, 20, 20),
    (0x74, GC, 2, 8, 8),
    (0x75, CR, 2, 14, 14),
    (0x76, GC, 2, 8, 8),
    (0x77, GC, 2, 8, 8),
    (0x78, IO, 2, 12, 12),
    (0x79, IO, 2, 12, 12),
    (0x7A, A16, 2, 15, 15),
    (0x7B, L16, 4, 20, 20),
    (0x7C, GC, 2, 8, 8),
    (0x7D, CR, 2, 14, 14),
    (0x7E, GC, 2, 8, 8),
    (0x7F, GC, 2, 8, 8),
    (0x80, GC, 2, 8, 8),
    (0x81, GC, 2, 8, 8),
    (0x82, GC, 2, 8, 8),
    (0x83, GC, 2, 8, 8),
    (0x84, GC, 2, 8, 8),
    (0x85, GC, 2, 8, 8),
    (0x86, GC, 2, 8, 8),
    (0x87, GC, 2, 8, 8),
    (0x88, GC, 2, 8, 8),
    (0x89, GC, 2, 8, 8),
    (0x8A, GC, 2, 8, 8),
    (0x8B, GC, 2, 8, 8),
    (0x8C, GC, 2, 8, 8),
    (0x8D, GC, 2, 8, 8),
    (0x8E, GC, 2, 8, 8),
    (0x8F, GC, 2, 8, 8),
    (0x90, GC, 2, 8, 8),
    (0x91, GC, 2, 8, 8),
    (0x92, GC, 2, 8, 8),
    (0x93, GC, 2, 8, 8),
    (0x94, GC, 2, 8, 8),
    (0x95, GC, 2, 8, 8),
    (0x96, GC, 2, 8, 8),
    (0x97, GC, 2, 8, 8),
    (0x98, GC, 2, 8, 8),
    (0x99, GC, 2, 8, 8),
    (0x9A, GC, 2, 8, 8),
    (0x9B, GC, 2, 8, 8),
    (0x9C, GC, 2, 8, 8),
    (0x9D, GC, 2, 8, 8),
    (0x9E, GC, 2, 8, 8),
    (0x9F, GC, 2, 8, 8),
    (0xA0, EX, 2, 16, 16),
    (0xA1, EX, 2, 16, 16),
    (0xA2, IO, 2, 16, 16),
    (0xA3, IO, 2, 16, 16),
    (0xA4, GC, 2, 8, 8),
    (0xA5, GC, 2, 8, 8),
    (0xA6, GC, 2, 8, 8),
    (0xA7, GC, 2, 8, 8),
    (0xA8, EX, 2, 16, 16),
    (0xA9, EX, 2, 16, 16),
    (0xAA, IO, 2, 16, 16),
    (0xAB, IO, 2, 16, 16),
    (0xAC, GC, 2, 8, 8),
    (0xAD, GC, 2, 8, 8),
    (0xAE, GC, 2, 8, 8),
    (0xAF, GC, 2, 8, 8),
    (0xB0, EX, 2, 16, 21),
    (0xB1, EX, 2, 16, 21),
    (0xB2, IO, 2, 16, 21),
    (0xB3, IO, 2, 16, 21),
    (0xB4, GC, 2, 8, 8),
    (0xB5, GC, 2, 8, 8),
    (0xB6, GC, 2, 8, 8),
    (0xB7, GC, 2, 8, 8),
    (0xB8, EX, 2, 16, 21),
    (0xB9, EX, 2, 16, 21),
    (0xBA, IO, 2, 16, 21),
    (0xBB, IO, 2, 16, 21),
    (0xBC, GC, 2, 8, 8),
    (0xBD, GC, 2, 8, 8),
    (0xBE, GC, 2, 8, 8),
    (0xBF, GC, 2, 8, 8),
    (0xC0, GC, 2, 8, 8),
    (0xC1, GC, 2, 8, 8),
    (0xC2, GC, 2, 8, 8),
    (0xC3, GC, 2, 8, 8),
    (0xC4, GC, 2, 8, 8),
    (0xC5, GC, 2, 8, 8),
    (0xC6, GC, 2, 8, 8),
    (0xC7, GC, 2, 8, 8),
    (0xC8, GC, 2, 8, 8),
    (0xC9, GC, 2, 8, 8),
    (0xCA, GC, 2, 8, 8),
    (0xCB, GC, 2, 8, 8),
    (0xCC, GC, 2, 8, 8),
    (0xCD, GC, 2, 8, 8),
    (0xCE, GC, 2, 8, 8),
    (0xCF, GC, 2, 8, 8),
    (0xD0, GC, 2, 8, 8),
    (0xD1, GC, 2, 8, 8),
    (0xD2, GC, 2, 8, 8),
    (0xD3, GC, 2, 8, 8),
    (0xD4, GC, 2, 8, 8),
    (0xD5, GC, 2, 8, 8),
    (0xD6, GC, 2, 8, 8),
    (0xD7, GC, 2, 8, 8),
    (0xD8, GC, 2, 8, 8),
    (0xD9, GC, 2, 8, 8),
    (0xDA, GC, 2, 8, 8),
    (0xDB, GC, 2, 8, 8),
    (0xDC, GC, 2, 8, 8),
    (0xDD, GC, 2, 8, 8),
    (0xDE, GC, 2, 8, 8),
    (0xDF, GC, 2, 8, 8),
    (0xE0, GC, 2, 8, 8),
    (0xE1, GC, 2, 8, 8),
    (0xE2, GC, 2, 8, 8),
    (0xE3, GC, 2, 8, 8),
    (0xE4, GC, 2, 8, 8),
    (0xE5, GC, 2, 8, 8),
    (0xE6, GC, 2, 8, 8),
    (0xE7, GC, 2, 8, 8),
    (0xE8, GC, 2, 8, 8),
    (0xE9, GC, 2, 8, 8),
    (0xEA, GC, 2, 8, 8),
    (0xEB, GC, 2, 8, 8),
    (0xEC, GC, 2, 8, 8),
    (0xED, GC, 2, 8, 8),
    (0xEE, GC, 2, 8, 8),
    (0xEF, GC, 2, 8, 8),
    (0xF0, GC, 2, 8, 8),
    (0xF1, GC, 2, 8, 8),
    (0xF2, GC, 2, 8, 8),
    (0xF3, GC, 2, 8, 8),
    (0xF4, GC, 2, 8, 8),
    (0xF5, GC, 2, 8, 8),
    (0xF6, GC, 2, 8, 8),
    (0xF7, GC, 2, 8, 8),
    (0xF8, GC, 2, 8, 8),
    (0xF9, GC, 2, 8, 8),
    (0xFA, GC, 2, 8, 8),
    (0xFB, GC, 2, 8, 8),
    (0xFC, GC, 2, 8, 8),
    (0xFD, GC, 2, 8, 8),
    (0xFE, GC, 2, 8, 8),
    (0xFF, GC, 2, 8, 8),
];

// UM Tables 6–8/11/13/15 pp. 42,45,47,50,52,59; TM pp. 44–47,49,52.
// Full lengths include DD/FD and d/nn: UM pp. 75–87,100–122,128–129,
// 149–171,194–203,276–277. Shared keys represent both IX and IY.
const INDEX_ROWS: &[Row] = &[
    (0x09, A16, 2, 15, 15),
    (0x19, A16, 2, 15, 15),
    (0x21, L16, 4, 14, 14),
    (0x22, L16, 4, 20, 20),
    (0x23, A16, 2, 10, 10),
    (0x29, A16, 2, 15, 15),
    (0x2A, L16, 4, 20, 20),
    (0x2B, A16, 2, 10, 10),
    (0x34, A8, 3, 23, 23),
    (0x35, A8, 3, 23, 23),
    (0x36, L8, 4, 19, 19),
    (0x39, A16, 2, 15, 15),
    (0x46, L8, 3, 19, 19),
    (0x4E, L8, 3, 19, 19),
    (0x56, L8, 3, 19, 19),
    (0x5E, L8, 3, 19, 19),
    (0x66, L8, 3, 19, 19),
    (0x6E, L8, 3, 19, 19),
    (0x70, L8, 3, 19, 19),
    (0x71, L8, 3, 19, 19),
    (0x72, L8, 3, 19, 19),
    (0x73, L8, 3, 19, 19),
    (0x74, L8, 3, 19, 19),
    (0x75, L8, 3, 19, 19),
    (0x77, L8, 3, 19, 19),
    (0x7E, L8, 3, 19, 19),
    (0x86, A8, 3, 19, 19),
    (0x8E, A8, 3, 19, 19),
    (0x96, A8, 3, 19, 19),
    (0x9E, A8, 3, 19, 19),
    (0xA6, A8, 3, 19, 19),
    (0xAE, A8, 3, 19, 19),
    (0xB6, A8, 3, 19, 19),
    (0xBE, A8, 3, 19, 19),
    (0xE1, L16, 2, 14, 14),
    (0xE3, EX, 2, 23, 23),
    (0xE5, L16, 2, 15, 15),
    (0xE9, JP, 2, 8, 8),
    (0xF9, L16, 2, 10, 10),
    // Decision-20 §2: H operation (Young v0.91 §§3.2/8.1/8.4 pp. 11–12/26/30;
    // Rak .inc_x/.dec_x/.ld_x_n/.ld_x_x/.alo_a_x/.alo_a_y); L costs.
    (0x24, A8, 2, 8, 8),
    (0x25, A8, 2, 8, 8),
    (0x26, L8, 3, 11, 11),
    (0x2C, A8, 2, 8, 8),
    (0x2D, A8, 2, 8, 8),
    (0x2E, L8, 3, 11, 11),
    (0x44, L8, 2, 8, 8),
    (0x45, L8, 2, 8, 8),
    (0x4C, L8, 2, 8, 8),
    (0x4D, L8, 2, 8, 8),
    (0x54, L8, 2, 8, 8),
    (0x55, L8, 2, 8, 8),
    (0x5C, L8, 2, 8, 8),
    (0x5D, L8, 2, 8, 8),
    (0x60, L8, 2, 8, 8),
    (0x61, L8, 2, 8, 8),
    (0x62, L8, 2, 8, 8),
    (0x63, L8, 2, 8, 8),
    (0x64, L8, 2, 8, 8),
    (0x65, L8, 2, 8, 8),
    (0x67, L8, 2, 8, 8),
    (0x68, L8, 2, 8, 8),
    (0x69, L8, 2, 8, 8),
    (0x6A, L8, 2, 8, 8),
    (0x6B, L8, 2, 8, 8),
    (0x6C, L8, 2, 8, 8),
    (0x6D, L8, 2, 8, 8),
    (0x6F, L8, 2, 8, 8),
    (0x7C, L8, 2, 8, 8),
    (0x7D, L8, 2, 8, 8),
    (0x84, A8, 2, 8, 8),
    (0x85, A8, 2, 8, 8),
    (0x8C, A8, 2, 8, 8),
    (0x8D, A8, 2, 8, 8),
    (0x94, A8, 2, 8, 8),
    (0x95, A8, 2, 8, 8),
    (0x9C, A8, 2, 8, 8),
    (0x9D, A8, 2, 8, 8),
    (0xA4, A8, 2, 8, 8),
    (0xA5, A8, 2, 8, 8),
    (0xAC, A8, 2, 8, 8),
    (0xAD, A8, 2, 8, 8),
    (0xB4, A8, 2, 8, 8),
    (0xB5, A8, 2, 8, 8),
    (0xBC, A8, 2, 8, 8),
    (0xBD, A8, 2, 8, 8),
];

// UM Fig. 39 p. 54 / Table 14 pp. 55–57 reconciled with entries
// pp. 217–237,247–260; TM Tables 7.0-7/8 pp. 50–51. Fourth-byte keys;
// lengths include DD/FD CB d. BIT costs 20; rotate/shift/SET/RES cost 23.
const INDEX_CB_ROWS: &[Row] = &[
    (0x06, RS, 4, 23, 23),
    (0x0E, RS, 4, 23, 23),
    (0x16, RS, 4, 23, 23),
    (0x1E, RS, 4, 23, 23),
    (0x26, RS, 4, 23, 23),
    (0x2E, RS, 4, 23, 23),
    (0x3E, RS, 4, 23, 23),
    (0x46, BT, 4, 20, 20),
    (0x4E, BT, 4, 20, 20),
    (0x56, BT, 4, 20, 20),
    (0x5E, BT, 4, 20, 20),
    (0x66, BT, 4, 20, 20),
    (0x6E, BT, 4, 20, 20),
    (0x76, BT, 4, 20, 20),
    (0x7E, BT, 4, 20, 20),
    (0x86, BT, 4, 23, 23),
    (0x8E, BT, 4, 23, 23),
    (0x96, BT, 4, 23, 23),
    (0x9E, BT, 4, 23, 23),
    (0xA6, BT, 4, 23, 23),
    (0xAE, BT, 4, 23, 23),
    (0xB6, BT, 4, 23, 23),
    (0xBE, BT, 4, 23, 23),
    (0xC6, BT, 4, 23, 23),
    (0xCE, BT, 4, 23, 23),
    (0xD6, BT, 4, 23, 23),
    (0xDE, BT, 4, 23, 23),
    (0xE6, BT, 4, 23, 23),
    (0xEE, BT, 4, 23, 23),
    (0xF6, BT, 4, 23, 23),
    (0xFE, BT, 4, 23, 23),
    // H copies/BIT aliases, L costs: decision-20 §3; Young v0.91
    // §§3.5–3.6 pp. 13–14 / §§8.7–8.8 pp. 32–33; Rak v1.2a
    // .sro_xyd_r/.bit_n_xyd_/.set_n_xyd_/.res_n_xyd_.
    (0x00, RS, 4, 23, 23),
    (0x01, RS, 4, 23, 23),
    (0x02, RS, 4, 23, 23),
    (0x03, RS, 4, 23, 23),
    (0x04, RS, 4, 23, 23),
    (0x05, RS, 4, 23, 23),
    (0x07, RS, 4, 23, 23),
    (0x08, RS, 4, 23, 23),
    (0x09, RS, 4, 23, 23),
    (0x0A, RS, 4, 23, 23),
    (0x0B, RS, 4, 23, 23),
    (0x0C, RS, 4, 23, 23),
    (0x0D, RS, 4, 23, 23),
    (0x0F, RS, 4, 23, 23),
    (0x10, RS, 4, 23, 23),
    (0x11, RS, 4, 23, 23),
    (0x12, RS, 4, 23, 23),
    (0x13, RS, 4, 23, 23),
    (0x14, RS, 4, 23, 23),
    (0x15, RS, 4, 23, 23),
    (0x17, RS, 4, 23, 23),
    (0x18, RS, 4, 23, 23),
    (0x19, RS, 4, 23, 23),
    (0x1A, RS, 4, 23, 23),
    (0x1B, RS, 4, 23, 23),
    (0x1C, RS, 4, 23, 23),
    (0x1D, RS, 4, 23, 23),
    (0x1F, RS, 4, 23, 23),
    (0x20, RS, 4, 23, 23),
    (0x21, RS, 4, 23, 23),
    (0x22, RS, 4, 23, 23),
    (0x23, RS, 4, 23, 23),
    (0x24, RS, 4, 23, 23),
    (0x25, RS, 4, 23, 23),
    (0x27, RS, 4, 23, 23),
    (0x28, RS, 4, 23, 23),
    (0x29, RS, 4, 23, 23),
    (0x2A, RS, 4, 23, 23),
    (0x2B, RS, 4, 23, 23),
    (0x2C, RS, 4, 23, 23),
    (0x2D, RS, 4, 23, 23),
    (0x2F, RS, 4, 23, 23),
    (0x30, RS, 4, 23, 23),
    (0x31, RS, 4, 23, 23),
    (0x32, RS, 4, 23, 23),
    (0x33, RS, 4, 23, 23),
    (0x34, RS, 4, 23, 23),
    (0x35, RS, 4, 23, 23),
    (0x36, RS, 4, 23, 23),
    (0x37, RS, 4, 23, 23),
    (0x38, RS, 4, 23, 23),
    (0x39, RS, 4, 23, 23),
    (0x3A, RS, 4, 23, 23),
    (0x3B, RS, 4, 23, 23),
    (0x3C, RS, 4, 23, 23),
    (0x3D, RS, 4, 23, 23),
    (0x3F, RS, 4, 23, 23),
    (0x40, BT, 4, 20, 20),
    (0x41, BT, 4, 20, 20),
    (0x42, BT, 4, 20, 20),
    (0x43, BT, 4, 20, 20),
    (0x44, BT, 4, 20, 20),
    (0x45, BT, 4, 20, 20),
    (0x47, BT, 4, 20, 20),
    (0x48, BT, 4, 20, 20),
    (0x49, BT, 4, 20, 20),
    (0x4A, BT, 4, 20, 20),
    (0x4B, BT, 4, 20, 20),
    (0x4C, BT, 4, 20, 20),
    (0x4D, BT, 4, 20, 20),
    (0x4F, BT, 4, 20, 20),
    (0x50, BT, 4, 20, 20),
    (0x51, BT, 4, 20, 20),
    (0x52, BT, 4, 20, 20),
    (0x53, BT, 4, 20, 20),
    (0x54, BT, 4, 20, 20),
    (0x55, BT, 4, 20, 20),
    (0x57, BT, 4, 20, 20),
    (0x58, BT, 4, 20, 20),
    (0x59, BT, 4, 20, 20),
    (0x5A, BT, 4, 20, 20),
    (0x5B, BT, 4, 20, 20),
    (0x5C, BT, 4, 20, 20),
    (0x5D, BT, 4, 20, 20),
    (0x5F, BT, 4, 20, 20),
    (0x60, BT, 4, 20, 20),
    (0x61, BT, 4, 20, 20),
    (0x62, BT, 4, 20, 20),
    (0x63, BT, 4, 20, 20),
    (0x64, BT, 4, 20, 20),
    (0x65, BT, 4, 20, 20),
    (0x67, BT, 4, 20, 20),
    (0x68, BT, 4, 20, 20),
    (0x69, BT, 4, 20, 20),
    (0x6A, BT, 4, 20, 20),
    (0x6B, BT, 4, 20, 20),
    (0x6C, BT, 4, 20, 20),
    (0x6D, BT, 4, 20, 20),
    (0x6F, BT, 4, 20, 20),
    (0x70, BT, 4, 20, 20),
    (0x71, BT, 4, 20, 20),
    (0x72, BT, 4, 20, 20),
    (0x73, BT, 4, 20, 20),
    (0x74, BT, 4, 20, 20),
    (0x75, BT, 4, 20, 20),
    (0x77, BT, 4, 20, 20),
    (0x78, BT, 4, 20, 20),
    (0x79, BT, 4, 20, 20),
    (0x7A, BT, 4, 20, 20),
    (0x7B, BT, 4, 20, 20),
    (0x7C, BT, 4, 20, 20),
    (0x7D, BT, 4, 20, 20),
    (0x7F, BT, 4, 20, 20),
    (0x80, BT, 4, 23, 23),
    (0x81, BT, 4, 23, 23),
    (0x82, BT, 4, 23, 23),
    (0x83, BT, 4, 23, 23),
    (0x84, BT, 4, 23, 23),
    (0x85, BT, 4, 23, 23),
    (0x87, BT, 4, 23, 23),
    (0x88, BT, 4, 23, 23),
    (0x89, BT, 4, 23, 23),
    (0x8A, BT, 4, 23, 23),
    (0x8B, BT, 4, 23, 23),
    (0x8C, BT, 4, 23, 23),
    (0x8D, BT, 4, 23, 23),
    (0x8F, BT, 4, 23, 23),
    (0x90, BT, 4, 23, 23),
    (0x91, BT, 4, 23, 23),
    (0x92, BT, 4, 23, 23),
    (0x93, BT, 4, 23, 23),
    (0x94, BT, 4, 23, 23),
    (0x95, BT, 4, 23, 23),
    (0x97, BT, 4, 23, 23),
    (0x98, BT, 4, 23, 23),
    (0x99, BT, 4, 23, 23),
    (0x9A, BT, 4, 23, 23),
    (0x9B, BT, 4, 23, 23),
    (0x9C, BT, 4, 23, 23),
    (0x9D, BT, 4, 23, 23),
    (0x9F, BT, 4, 23, 23),
    (0xA0, BT, 4, 23, 23),
    (0xA1, BT, 4, 23, 23),
    (0xA2, BT, 4, 23, 23),
    (0xA3, BT, 4, 23, 23),
    (0xA4, BT, 4, 23, 23),
    (0xA5, BT, 4, 23, 23),
    (0xA7, BT, 4, 23, 23),
    (0xA8, BT, 4, 23, 23),
    (0xA9, BT, 4, 23, 23),
    (0xAA, BT, 4, 23, 23),
    (0xAB, BT, 4, 23, 23),
    (0xAC, BT, 4, 23, 23),
    (0xAD, BT, 4, 23, 23),
    (0xAF, BT, 4, 23, 23),
    (0xB0, BT, 4, 23, 23),
    (0xB1, BT, 4, 23, 23),
    (0xB2, BT, 4, 23, 23),
    (0xB3, BT, 4, 23, 23),
    (0xB4, BT, 4, 23, 23),
    (0xB5, BT, 4, 23, 23),
    (0xB7, BT, 4, 23, 23),
    (0xB8, BT, 4, 23, 23),
    (0xB9, BT, 4, 23, 23),
    (0xBA, BT, 4, 23, 23),
    (0xBB, BT, 4, 23, 23),
    (0xBC, BT, 4, 23, 23),
    (0xBD, BT, 4, 23, 23),
    (0xBF, BT, 4, 23, 23),
    (0xC0, BT, 4, 23, 23),
    (0xC1, BT, 4, 23, 23),
    (0xC2, BT, 4, 23, 23),
    (0xC3, BT, 4, 23, 23),
    (0xC4, BT, 4, 23, 23),
    (0xC5, BT, 4, 23, 23),
    (0xC7, BT, 4, 23, 23),
    (0xC8, BT, 4, 23, 23),
    (0xC9, BT, 4, 23, 23),
    (0xCA, BT, 4, 23, 23),
    (0xCB, BT, 4, 23, 23),
    (0xCC, BT, 4, 23, 23),
    (0xCD, BT, 4, 23, 23),
    (0xCF, BT, 4, 23, 23),
    (0xD0, BT, 4, 23, 23),
    (0xD1, BT, 4, 23, 23),
    (0xD2, BT, 4, 23, 23),
    (0xD3, BT, 4, 23, 23),
    (0xD4, BT, 4, 23, 23),
    (0xD5, BT, 4, 23, 23),
    (0xD7, BT, 4, 23, 23),
    (0xD8, BT, 4, 23, 23),
    (0xD9, BT, 4, 23, 23),
    (0xDA, BT, 4, 23, 23),
    (0xDB, BT, 4, 23, 23),
    (0xDC, BT, 4, 23, 23),
    (0xDD, BT, 4, 23, 23),
    (0xDF, BT, 4, 23, 23),
    (0xE0, BT, 4, 23, 23),
    (0xE1, BT, 4, 23, 23),
    (0xE2, BT, 4, 23, 23),
    (0xE3, BT, 4, 23, 23),
    (0xE4, BT, 4, 23, 23),
    (0xE5, BT, 4, 23, 23),
    (0xE7, BT, 4, 23, 23),
    (0xE8, BT, 4, 23, 23),
    (0xE9, BT, 4, 23, 23),
    (0xEA, BT, 4, 23, 23),
    (0xEB, BT, 4, 23, 23),
    (0xEC, BT, 4, 23, 23),
    (0xED, BT, 4, 23, 23),
    (0xEF, BT, 4, 23, 23),
    (0xF0, BT, 4, 23, 23),
    (0xF1, BT, 4, 23, 23),
    (0xF2, BT, 4, 23, 23),
    (0xF3, BT, 4, 23, 23),
    (0xF4, BT, 4, 23, 23),
    (0xF5, BT, 4, 23, 23),
    (0xF7, BT, 4, 23, 23),
    (0xF8, BT, 4, 23, 23),
    (0xF9, BT, 4, 23, 23),
    (0xFA, BT, 4, 23, 23),
    (0xFB, BT, 4, 23, 23),
    (0xFC, BT, 4, 23, 23),
    (0xFD, BT, 4, 23, 23),
    (0xFF, BT, 4, 23, 23),
];

fn assert_row(table: &[Option<Instruction>; 256], row: Row) {
    let (key, group, length, untaken, taken) = row;
    let actual = table[usize::from(key)].map(|entry| {
        (
            entry.group,
            entry.length,
            entry.states.untaken,
            entry.states.taken,
        )
    });
    assert_eq!(
        actual,
        Some((group, length, untaken, taken)),
        "key {key:02X}"
    );
}

fn assert_map(table: &[Option<Instruction>; 256], rows: &[Row], count: usize) {
    assert_eq!(rows.len(), count, "literal transcription count");
    assert_eq!(
        table.iter().flatten().count(),
        count,
        "documented map count"
    );
    let mut seen = [false; 256];
    for &row in rows {
        let key = usize::from(row.0);
        assert!(!seen[key], "duplicate literal key {key:02X}");
        seen[key] = true;
        assert_row(table, row);
    }
    for (key, entry) in table.iter().enumerate() {
        if !seen[key] {
            assert!(entry.is_none(), "undocumented key {key:02X}");
        }
    }
}

#[test]
fn main_map_matches_all_252_literal_rows_and_its_complement() {
    assert_map(&MAIN, MAIN_ROWS, 252);
}

#[test]
fn cb_map_matches_all_256_literal_rows() {
    // Decision-20 §3 admits SLL.
    assert_map(&CB, CB_ROWS, 256);
}

#[test]
fn ed_map_matches_all_256_literal_rows() {
    // Decision-20 §1: no refused ED terminal, including prefix-valued keys.
    assert_map(&ED, ED_ROWS, 256);
}

#[test]
fn index_map_matches_all_85_literal_rows_and_its_complement() {
    assert_map(&INDEX, INDEX_ROWS, 85);
}

#[test]
fn index_cb_map_matches_all_256_literal_rows() {
    // Decision-20 §3 admits copies and BIT aliases.
    assert_map(&INDEX_CB, INDEX_CB_ROWS, 256);
}

#[test]
fn shared_index_maps_represent_1446_effective_logical_entries() {
    // decision-14 rule 1 counts DD and FD separately, while decision-13
    // rule 3 stores each shared table once.
    let count = MAIN.iter().flatten().count()
        + CB.iter().flatten().count()
        + ED.iter().flatten().count()
        + 2 * INDEX.iter().flatten().count()
        + 2 * INDEX_CB.iter().flatten().count();
    // Decision-20 §2 adds 46 effective half entries per selector; ignored
    // MAIN fallbacks are dispatcher-owned and not duplicated in INDEX.
    // Decision-20 §3 adds 8 CB and 225 terminals per indexed-CB selector.
    assert_eq!(count, 1446);
}

#[test]
fn prefix_dispatch_slots_have_no_terminal_entry() {
    for key in [0xCB, 0xDD, 0xED, 0xFD] {
        assert!(MAIN[key].is_none());
    }
    assert!(INDEX[0xCB].is_none());
}

#[test]
fn remaining_exclusions_and_main_fallbacks_are_explicit() {
    // decision-14 rule 1; UM Fig. 39 p. 54, IN p. 296, OUT p. 307,
    // indexed entries pp. 75–87,217–237,247–260; TM pp. 44,50–51,54.
    // Decision-20 §3 removes all CB and indexed-CB exclusions.
    assert!(CB.iter().all(Option::is_some));
    assert!(INDEX_CB.iter().all(Option::is_some));
    // Decision-20 §§1–2 remove ED and DD/FD main refusals.
    assert!(ED.iter().all(Option::is_some));
    for key in [0x00, 0x76, 0xCB, 0xDD, 0xED, 0xFD] {
        assert!(INDEX[key].is_none());
    }
    // Decision-20 §2: these now substitute index halves.
    assert_row(&INDEX, (0x44, L8, 2, 8, 8));
    assert_row(&INDEX, (0x65, L8, 2, 8, 8));
    for key in [0x00, 0x36, 0x47, 0x7F, 0xC0] {
        assert!(INDEX_CB[key].is_some());
    }
}

#[test]
fn ed_hl_pair_loads_omitted_by_compact_table_7_are_documented() {
    // PROVENANCE "Z80 ED 63 / ED 6B omitted from compact loads": UM
    // Table 7 p. 45 omits these; entries pp. 103/108 and TM p. 45 include HL.
    assert_row(&ED, (0x63, L16, 4, 20, 20));
    assert_row(&ED, (0x6B, L16, 4, 20, 20));
    assert_row(&MAIN, (0x22, L16, 3, 16, 16));
    assert_row(&MAIN, (0x2A, L16, 3, 16, 16));
}

#[test]
fn h_register_column_does_not_introduce_an_f_operand() {
    // PROVENANCE "Z80 compact H register headings": UM pp. 42/50/54
    // label H as F; entries pp. 71/145/213 and TM pp. 44/47/50 give H=100.
    assert_row(&MAIN, (0x44, L8, 1, 4, 4)); // LD B,H
    assert_row(&MAIN, (0x84, A8, 1, 4, 4)); // ADD A,H
    assert_row(&CB, (0x04, RS, 2, 8, 8)); // RLC H
}

#[test]
fn compact_push_pop_misprints_do_not_claim_alu_bytes() {
    // PROVENANCE "Z80 compact PUSH/POP encodings": UM Table 7 p. 45
    // prints PUSH F6/C6/D6/E6, DD/FD E6 and POP AF P1; entries
    // pp. 115/117–119 and TM p. 45 give the bytes below.
    for key in [0xC5, 0xD5, 0xE5, 0xF5] {
        assert_row(&MAIN, (key, L16, 1, 11, 11));
    }
    for key in [0xC6, 0xD6, 0xE6, 0xF6] {
        assert_row(&MAIN, (key, A8, 2, 7, 7));
    }
    assert_row(&INDEX, (0xE5, L16, 2, 15, 15));
    assert!(INDEX[0xE6].is_none());
    assert_row(&MAIN, (0xF1, L16, 1, 10, 10));
}

#[test]
fn compact_add_hl_and_dec_bc_cells_do_not_displace_other_instructions() {
    // PROVENANCE "Z80 compact ADD A,(HL) and DEC BC encodings": UM
    // pp. 50/52 print 88/DB; entries pp. 148/201 and TM pp. 47/49 give 86/0B.
    assert_row(&MAIN, (0x86, A8, 1, 7, 7));
    assert_row(&MAIN, (0x88, A8, 1, 4, 4));
    assert_row(&MAIN, (0x0B, A16, 1, 6, 6));
    assert_row(&MAIN, (0xDB, IO, 2, 11, 11));
}

#[test]
fn compact_rotate_cells_follow_entry_patterns() {
    // PROVENANCE "Z80 compact rotate encodings": UM Fig. 39 p. 54
    // prints CB06 for RLC L, CB0E for RLC (HL), CB06 for RRC E.
    // Entries pp. 213/215/225 and TM p. 50 give CB05/CB06/CB0B.
    assert_row(&CB, (0x05, RS, 2, 8, 8));
    assert_row(&CB, (0x06, RS, 2, 15, 15));
    assert_row(&CB, (0x0B, RS, 2, 8, 8));
    assert_row(&CB, (0x0E, RS, 2, 15, 15));
    assert_row(&INDEX_CB, (0x06, RS, 4, 23, 23));
}

#[test]
fn corrupt_compact_bit_cells_cannot_add_encodings() {
    // PROVENANCE "Z80 compact BIT/SET/RES table corruption": UM
    // Table 14 pp. 55–57 includes C8 prefixes, repeated terminals and DD
    // in IY cells. Entry patterns pp. 243–260 and TM p. 51 prevail.
    // Indexed BIT 3/7 have terminals 5E/7E, rather than the printed 46.
    assert_row(&INDEX_CB, (0x46, BT, 4, 20, 20));
    assert_row(&INDEX_CB, (0x5E, BT, 4, 20, 20));
    assert_row(&INDEX_CB, (0x7E, BT, 4, 20, 20));
    assert_row(&CB, (0x4B, BT, 2, 8, 8)); // BIT 1,E: compact 48
    assert_row(&CB, (0x6B, BT, 2, 8, 8)); // BIT 5,E: compact 68
    assert_row(&CB, (0x9C, BT, 2, 8, 8)); // RES 3,H: compact 90
    assert_row(&CB, (0xBE, BT, 2, 15, 15)); // RES 7,(HL): compact 9E
    assert_row(&CB, (0xD9, BT, 2, 8, 8)); // SET 3,C: compact 09
}

#[test]
fn compact_jump_and_input_cells_follow_entry_patterns() {
    // PROVENANCE "Z80 compact JP and IN A,(C) encodings": UM
    // pp. 59/62 print D8/EB/ED7B; entries pp. 263/275/296 and TM
    // pp. 52/54 give DA/E9/ED78. ED7B remains a pair load.
    assert_row(&MAIN, (0xDA, JP, 3, 10, 10));
    assert_row(&MAIN, (0xD8, CR, 1, 5, 11));
    assert_row(&MAIN, (0xE9, JP, 1, 4, 4));
    assert_row(&MAIN, (0xEB, EX, 1, 4, 4));
    assert_row(&ED, (0x78, IO, 2, 12, 12));
    assert_row(&ED, (0x7B, L16, 4, 20, 20));
}

#[test]
fn arithmetic_entry_bit_diagrams_use_the_agreeing_hex_and_table_readings() {
    // PROVENANCE "Z80 arithmetic entry bit diagrams": UM p. 151 prints
    // ADC (IX+d)'s prefix bits as DE beside hex DD; p. 159 prints OR
    // (IY+d)'s key bits as AE beside hex B6. UM Table 11 p. 50 and
    // TM p. 47 agree with those hex annotations: DD 8E d / FD B6 d.
    assert_row(&INDEX, (0x8E, A8, 3, 19, 19));
    assert_row(&INDEX, (0xB6, A8, 3, 19, 19));
    assert_row(&INDEX, (0xAE, A8, 3, 19, 19)); // XOR remains documented too
    assert!(INDEX[0xDE].is_none());
}

#[test]
fn rotate_entry_bit_diagrams_use_the_agreeing_hex_and_table_readings() {
    // PROVENANCE "Z80 rotate entry bit diagrams": UM p. 225 gives
    // RRC (IY+d)'s key bits as 1E beside hex 0E; p. 228 prints RR r's
    // bits as 00001rrr (RRC). UM Fig. 39 p. 54 and TM p. 50 give
    // FD CB d 0E and CB 00011rrr respectively. These are metadata pins;
    // instruction semantics are left to the executor bead's witnesses.
    assert_row(&INDEX_CB, (0x0E, RS, 4, 23, 23));
    assert_row(&INDEX_CB, (0x1E, RS, 4, 23, 23));
    for key in [0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1F] {
        assert_row(&CB, (key, RS, 2, 8, 8));
    }
}

#[test]
fn rrca_and_rra_encodings_agree_between_sources() {
    // 03-0029-01 Table 7.0-7 p. 50 (PDF 56) prints RRCA as 00 001 111
    // (0F) and RRA as 00 011 111 (1F). UM008011-0816 Fig. 39 p. 54
    // and entries pp. 209/211 agree: each is one byte, 4 states.
    assert_row(&MAIN, (0x0F, RS, 1, 4, 4));
    assert_row(&MAIN, (0x1F, RS, 1, 4, 4));
}

#[test]
fn jr_nz_timing_paths_agree_between_sources() {
    // 03-0029-01 Table 7.0-9 p. 52 (PDF 58) labels 7 "If condition not
    // met" and 12 "If condition met". UM008011-0816 JR NZ p. 273
    // (PDF 287) agrees: untaken/taken = 7/12.
    assert_row(&MAIN, (0x20, JP, 2, 7, 12));
}

#[test]
fn repeating_blocks_put_termination_in_untaken_and_repetition_in_taken() {
    // PROVENANCE "Z80 block iteration retirement and two States slots";
    // "Z80 CPIR/CPDR terminating timing condition": UM pp. 132/136/139/142,
    // 300/304/311–312/315–316; TM pp. 46/54. CPIR/CPDR timing headings
    // say count zero AND match; prose and TM say OR. Only totals are
    // implemented here; execution witnesses must exercise both termination causes.
    for key in [0xB0, 0xB1, 0xB8, 0xB9] {
        assert_row(&ED, (key, EX, 2, 16, 21));
    }
    for key in [0xB2, 0xB3, 0xBA, 0xBB] {
        assert_row(&ED, (key, IO, 2, 16, 21));
    }
}
