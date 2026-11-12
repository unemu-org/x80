//! Independent 8085A T-state witnesses through the public API (decision-2,
//! decision-5 items 5-8, decision-11 and decision-12 rules 3-5).
//!
//! Transcribed from rendered Intel 205775-002 (January 1983) Table 5-1,
//! p. 5-19 (PDF 104), 8085A column, before reading the encoding/executor
//! tables or their group tests. Register/pair fields are expanded here.
//! Each row cites the primary instruction entry as U5-n and the independent
//! Intel 9800301-04 (May 1981) Ch. 3 entry as L3-n and appendix as A-n.
//! All totals were cross-read on page images: U5-4..18 (PDF 89..103),
//! L3-2..69 (PDF 56..123), A-1..5 (PDF 189..193). Source hashes were checked
//! against PROVENANCE.md section 1. The tests require no external artifacts.
//!
//! Known disagreements (PROVENANCE.md section 1): CPE is 9/18 in U5-14
//! and Table 5-1, and L3-16, but A-1 prints "9/17". PUSH rp is
//! "12 (8085)" in U5-15 and 12 in Table 5-1/A-2, whereas L3-44 prints
//! "11 (13 on 8085)". Follow the agreeing primary entry/table in both cases.
//!
//! Bus sequences come from Table 5-1's Machine Cycles column, with addresses
//! and stack classification from the cited instruction entries. Table 5-2,
//! p. 5-20 (PDF 105), is the opcode-order list, not a cycle chart. The compact
//! trace alphabet below is an API projection: f=opcode fetch (F or S),
//! o=operand read, r/w=data read/write, s/t=stack read/write, i/u=port input/
//! output. R in the source becomes o, r or s according to the entry;
//! W becomes w or t. B (bus idle) makes no Bus call: DAD is F B B and HLT
//! is F B; six-state register operations are S. Conditional strings use
//! untaken/taken order. Polls and serial callbacks are outside this alphabet.
//! No READY waits or elapsed halted time are counted.
//!
//! Interrupt totals are DERIVED, not quoted: 205775-002 section 2.3.4,
//! Figs 2-17/18 pp. 2-13/14 (PDF 36/37) show CALL as 6+3+3+3+3=18;
//! section 2.3.5 Fig. 2-19 p. 2-15 (PDF 38) shows a six-state internal
//! restart followed by normal stack writes, hence 6+3+3=12. INTR RST
//! substitutes INA for its six-state fetch and has the same derived 12.
//! Second witnesses: L3-9 (CALL 18), L3-55 (RST 12), and 231718-001
//! (September 1987) Table 3, handbook p. 16-20 (PDF 967): INA and RST/TRAP
//! bus idle have no RD; ordinary machine cycles have three states, fetch
//! four or six. Treating 8085A and 8085AH as one model is an assumption
//! (PROVENANCE.md section 1), not an extra per-instruction timing source.

mod support;

use support::{Event, RecordingBus};
use x80_core::i8085::{Flags, Latches, Registers, I8085};
use x80_core::{Budget, Exit, MemoryAccess, Report};

#[derive(Clone, Copy, Debug)]
enum Timing {
    Undocumented,
    Fixed(u8),
    Conditional(u8, u8),
}

use Timing::{Conditional as C, Fixed as F, Undocumented as U};

// Separate tests let a confirmed defect be ignored for its byte alone.
macro_rules! timing_witnesses {
    ($( $(#[$attr:meta])* $name:ident: $opcode:literal => $timing:expr, $trace:literal; )*) => {
        const TIMINGS: [(u8, Timing); 256] = [$(($opcode, $timing)),*];
        $(
            $(#[$attr])*
            #[test]
            fn $name() {
                witness($opcode, $timing, $trace);
            }
        )*
    };
}

// Literal expected totals; no crate decode, timing, flag-byte or pair helpers.
timing_witnesses! {
    op_00: 0x00 => F(4), "f"; // NOP; U5-17, L3-38, A-4.
    op_01: 0x01 => F(10), "foo"; // LXI B; U5-5, L3-35, A-2.
    op_02: 0x02 => F(7), "fw"; // STAX B; U5-6, L3-62, A-2.
    op_03: 0x03 => F(6), "f"; // INX B; U5-9, L3-26, A-2.
    op_04: 0x04 => F(4), "f"; // INR B; U5-8, L3-25, A-2.
    op_05: 0x05 => F(4), "f"; // DCR B; U5-8, L3-21, A-2.
    op_06: 0x06 => F(7), "fo"; // MVI B; U5-4, L3-37, A-2.
    op_07: 0x07 => F(4), "f"; // RLC; U5-11, L3-49, A-3.
    op_08: 0x08 => U, ""; // Table 5-2; A-5.
    op_09: 0x09 => F(10), "f"; // DAD B; U5-9, L3-20, A-2.
    op_0a: 0x0A => F(7), "fr"; // LDAX B; U5-5, L3-33, A-2.
    op_0b: 0x0B => F(6), "f"; // DCX B; U5-9, L3-22, A-3.
    op_0c: 0x0C => F(4), "f"; // INR C; U5-8, L3-25, A-2.
    op_0d: 0x0D => F(4), "f"; // DCR C; U5-8, L3-21, A-2.
    op_0e: 0x0E => F(7), "fo"; // MVI C; U5-4, L3-37, A-2.
    op_0f: 0x0F => F(4), "f"; // RRC; U5-12, L3-54, A-3.

    op_10: 0x10 => U, ""; // Table 5-2; A-5.
    op_11: 0x11 => F(10), "foo"; // LXI D; U5-5, L3-35, A-2.
    op_12: 0x12 => F(7), "fw"; // STAX D; U5-6, L3-62, A-2.
    op_13: 0x13 => F(6), "f"; // INX D; U5-9, L3-26, A-2.
    op_14: 0x14 => F(4), "f"; // INR D; U5-8, L3-25, A-2.
    op_15: 0x15 => F(4), "f"; // DCR D; U5-8, L3-21, A-2.
    op_16: 0x16 => F(7), "fo"; // MVI D; U5-4, L3-37, A-2.
    op_17: 0x17 => F(4), "f"; // RAL; U5-12, L3-46, A-3.
    op_18: 0x18 => U, ""; // Table 5-2; A-5.
    op_19: 0x19 => F(10), "f"; // DAD D; U5-9, L3-20, A-2.
    op_1a: 0x1A => F(7), "fr"; // LDAX D; U5-5, L3-33, A-2.
    op_1b: 0x1B => F(6), "f"; // DCX D; U5-9, L3-22, A-3.
    op_1c: 0x1C => F(4), "f"; // INR E; U5-8, L3-25, A-2.
    op_1d: 0x1D => F(4), "f"; // DCR E; U5-8, L3-21, A-2.
    op_1e: 0x1E => F(7), "fo"; // MVI E; U5-4, L3-37, A-2.
    op_1f: 0x1F => F(4), "f"; // RAR; U5-12, L3-46, A-3.

    op_20: 0x20 => F(4), "f"; // RIM; U5-17, L3-49, A-4.
    op_21: 0x21 => F(10), "foo"; // LXI H; U5-5, L3-35, A-2.
    op_22: 0x22 => F(16), "fooww"; // SHLD; U5-5, L3-59, A-4.
    op_23: 0x23 => F(6), "f"; // INX H; U5-9, L3-26, A-2.
    op_24: 0x24 => F(4), "f"; // INR H; U5-8, L3-25, A-2.
    op_25: 0x25 => F(4), "f"; // DCR H; U5-8, L3-21, A-2.
    op_26: 0x26 => F(7), "fo"; // MVI H; U5-4, L3-37, A-2.
    op_27: 0x27 => F(4), "f"; // DAA; U5-9, L3-19, A-4.
    op_28: 0x28 => U, ""; // Table 5-2; A-5.
    op_29: 0x29 => F(10), "f"; // DAD H; U5-9, L3-20, A-2.
    op_2a: 0x2A => F(16), "foorr"; // LHLD; U5-5, L3-34, A-4.
    op_2b: 0x2B => F(6), "f"; // DCX H; U5-9, L3-22, A-3.
    op_2c: 0x2C => F(4), "f"; // INR L; U5-8, L3-25, A-2.
    op_2d: 0x2D => F(4), "f"; // DCR L; U5-8, L3-21, A-2.
    op_2e: 0x2E => F(7), "fo"; // MVI L; U5-4, L3-37, A-2.
    op_2f: 0x2F => F(4), "f"; // CMA; U5-12, L3-11, A-4.

    op_30: 0x30 => F(4), "f"; // SIM; U5-18, L3-60, A-4.
    op_31: 0x31 => F(10), "foo"; // LXI SP; U5-5, L3-35, A-2.
    op_32: 0x32 => F(13), "foow"; // STA; U5-5, L3-62, A-2.
    op_33: 0x33 => F(6), "f"; // INX SP; U5-9, L3-26, A-2.
    op_34: 0x34 => F(10), "frw"; // INR M; U5-8, L3-25, A-3.
    op_35: 0x35 => F(10), "frw"; // DCR M; U5-8, L3-21, A-3.
    op_36: 0x36 => F(10), "fow"; // MVI M; U5-4, L3-38, A-2.
    op_37: 0x37 => F(4), "f"; // STC; U5-12, L3-63, A-4.
    op_38: 0x38 => U, ""; // Table 5-2; A-5.
    op_39: 0x39 => F(10), "f"; // DAD SP; U5-9, L3-20, A-2.
    op_3a: 0x3A => F(13), "foor"; // LDA; U5-5, L3-33, A-2.
    op_3b: 0x3B => F(6), "f"; // DCX SP; U5-9, L3-22, A-3.
    op_3c: 0x3C => F(4), "f"; // INR A; U5-8, L3-25, A-3.
    op_3d: 0x3D => F(4), "f"; // DCR A; U5-8, L3-21, A-3.
    op_3e: 0x3E => F(7), "fo"; // MVI A; U5-4, L3-37, A-2.
    op_3f: 0x3F => F(4), "f"; // CMC; U5-12, L3-12, A-4.

    op_40: 0x40 => F(4), "f"; // MOV B,B; U5-4, L3-36, A-2.
    op_41: 0x41 => F(4), "f"; // MOV B,C; U5-4, L3-36, A-2.
    op_42: 0x42 => F(4), "f"; // MOV B,D; U5-4, L3-36, A-2.
    op_43: 0x43 => F(4), "f"; // MOV B,E; U5-4, L3-36, A-2.
    op_44: 0x44 => F(4), "f"; // MOV B,H; U5-4, L3-36, A-2.
    op_45: 0x45 => F(4), "f"; // MOV B,L; U5-4, L3-36, A-2.
    op_46: 0x46 => F(7), "fr"; // MOV B,M; U5-4, L3-37, A-2.
    op_47: 0x47 => F(4), "f"; // MOV B,A; U5-4, L3-36, A-2.
    op_48: 0x48 => F(4), "f"; // MOV C,B; U5-4, L3-36, A-2.
    op_49: 0x49 => F(4), "f"; // MOV C,C; U5-4, L3-36, A-2.
    op_4a: 0x4A => F(4), "f"; // MOV C,D; U5-4, L3-36, A-2.
    op_4b: 0x4B => F(4), "f"; // MOV C,E; U5-4, L3-36, A-2.
    op_4c: 0x4C => F(4), "f"; // MOV C,H; U5-4, L3-36, A-2.
    op_4d: 0x4D => F(4), "f"; // MOV C,L; U5-4, L3-36, A-2.
    op_4e: 0x4E => F(7), "fr"; // MOV C,M; U5-4, L3-37, A-2.
    op_4f: 0x4F => F(4), "f"; // MOV C,A; U5-4, L3-36, A-2.

    op_50: 0x50 => F(4), "f"; // MOV D,B; U5-4, L3-36, A-2.
    op_51: 0x51 => F(4), "f"; // MOV D,C; U5-4, L3-36, A-2.
    op_52: 0x52 => F(4), "f"; // MOV D,D; U5-4, L3-36, A-2.
    op_53: 0x53 => F(4), "f"; // MOV D,E; U5-4, L3-36, A-2.
    op_54: 0x54 => F(4), "f"; // MOV D,H; U5-4, L3-36, A-2.
    op_55: 0x55 => F(4), "f"; // MOV D,L; U5-4, L3-36, A-2.
    op_56: 0x56 => F(7), "fr"; // MOV D,M; U5-4, L3-37, A-2.
    op_57: 0x57 => F(4), "f"; // MOV D,A; U5-4, L3-36, A-2.
    op_58: 0x58 => F(4), "f"; // MOV E,B; U5-4, L3-36, A-2.
    op_59: 0x59 => F(4), "f"; // MOV E,C; U5-4, L3-36, A-2.
    op_5a: 0x5A => F(4), "f"; // MOV E,D; U5-4, L3-36, A-2.
    op_5b: 0x5B => F(4), "f"; // MOV E,E; U5-4, L3-36, A-2.
    op_5c: 0x5C => F(4), "f"; // MOV E,H; U5-4, L3-36, A-2.
    op_5d: 0x5D => F(4), "f"; // MOV E,L; U5-4, L3-36, A-2.
    op_5e: 0x5E => F(7), "fr"; // MOV E,M; U5-4, L3-37, A-2.
    op_5f: 0x5F => F(4), "f"; // MOV E,A; U5-4, L3-36, A-2.

    op_60: 0x60 => F(4), "f"; // MOV H,B; U5-4, L3-36, A-2.
    op_61: 0x61 => F(4), "f"; // MOV H,C; U5-4, L3-36, A-2.
    op_62: 0x62 => F(4), "f"; // MOV H,D; U5-4, L3-36, A-2.
    op_63: 0x63 => F(4), "f"; // MOV H,E; U5-4, L3-36, A-2.
    op_64: 0x64 => F(4), "f"; // MOV H,H; U5-4, L3-36, A-2.
    op_65: 0x65 => F(4), "f"; // MOV H,L; U5-4, L3-36, A-2.
    op_66: 0x66 => F(7), "fr"; // MOV H,M; U5-4, L3-37, A-2.
    op_67: 0x67 => F(4), "f"; // MOV H,A; U5-4, L3-36, A-2.
    op_68: 0x68 => F(4), "f"; // MOV L,B; U5-4, L3-36, A-2.
    op_69: 0x69 => F(4), "f"; // MOV L,C; U5-4, L3-36, A-2.
    op_6a: 0x6A => F(4), "f"; // MOV L,D; U5-4, L3-36, A-2.
    op_6b: 0x6B => F(4), "f"; // MOV L,E; U5-4, L3-36, A-2.
    op_6c: 0x6C => F(4), "f"; // MOV L,H; U5-4, L3-36, A-2.
    op_6d: 0x6D => F(4), "f"; // MOV L,L; U5-4, L3-36, A-2.
    op_6e: 0x6E => F(7), "fr"; // MOV L,M; U5-4, L3-37, A-2.
    op_6f: 0x6F => F(4), "f"; // MOV L,A; U5-4, L3-36, A-2.

    op_70: 0x70 => F(7), "fw"; // MOV M,B; U5-4, L3-36, A-2.
    op_71: 0x71 => F(7), "fw"; // MOV M,C; U5-4, L3-36, A-2.
    op_72: 0x72 => F(7), "fw"; // MOV M,D; U5-4, L3-36, A-2.
    op_73: 0x73 => F(7), "fw"; // MOV M,E; U5-4, L3-36, A-2.
    op_74: 0x74 => F(7), "fw"; // MOV M,H; U5-4, L3-36, A-2.
    op_75: 0x75 => F(7), "fw"; // MOV M,L; U5-4, L3-36, A-2.
    op_76: 0x76 => F(5), "f"; // HLT; U5-17, L3-24, A-2.
    op_77: 0x77 => F(7), "fw"; // MOV M,A; U5-4, L3-36, A-2.
    op_78: 0x78 => F(4), "f"; // MOV A,B; U5-4, L3-36, A-2.
    op_79: 0x79 => F(4), "f"; // MOV A,C; U5-4, L3-36, A-2.
    op_7a: 0x7A => F(4), "f"; // MOV A,D; U5-4, L3-36, A-2.
    op_7b: 0x7B => F(4), "f"; // MOV A,E; U5-4, L3-36, A-2.
    op_7c: 0x7C => F(4), "f"; // MOV A,H; U5-4, L3-36, A-2.
    op_7d: 0x7D => F(4), "f"; // MOV A,L; U5-4, L3-36, A-2.
    op_7e: 0x7E => F(7), "fr"; // MOV A,M; U5-4, L3-37, A-2.
    op_7f: 0x7F => F(4), "f"; // MOV A,A; U5-4, L3-36, A-2.

    op_80: 0x80 => F(4), "f"; // ADD B; U5-6, L3-4, A-3.
    op_81: 0x81 => F(4), "f"; // ADD C; U5-6, L3-4, A-3.
    op_82: 0x82 => F(4), "f"; // ADD D; U5-6, L3-4, A-3.
    op_83: 0x83 => F(4), "f"; // ADD E; U5-6, L3-4, A-3.
    op_84: 0x84 => F(4), "f"; // ADD H; U5-6, L3-4, A-3.
    op_85: 0x85 => F(4), "f"; // ADD L; U5-6, L3-4, A-3.
    op_86: 0x86 => F(7), "fr"; // ADD M; U5-6, L3-4, A-3.
    op_87: 0x87 => F(4), "f"; // ADD A; U5-6, L3-4, A-3.
    op_88: 0x88 => F(4), "f"; // ADC B; U5-6, L3-3, A-3.
    op_89: 0x89 => F(4), "f"; // ADC C; U5-6, L3-3, A-3.
    op_8a: 0x8A => F(4), "f"; // ADC D; U5-6, L3-3, A-3.
    op_8b: 0x8B => F(4), "f"; // ADC E; U5-6, L3-3, A-3.
    op_8c: 0x8C => F(4), "f"; // ADC H; U5-6, L3-3, A-3.
    op_8d: 0x8D => F(4), "f"; // ADC L; U5-6, L3-3, A-3.
    op_8e: 0x8E => F(7), "fr"; // ADC M; U5-7, L3-3, A-3.
    op_8f: 0x8F => F(4), "f"; // ADC A; U5-6, L3-3, A-3.

    op_90: 0x90 => F(4), "f"; // SUB B; U5-7, L3-63, A-3.
    op_91: 0x91 => F(4), "f"; // SUB C; U5-7, L3-63, A-3.
    op_92: 0x92 => F(4), "f"; // SUB D; U5-7, L3-63, A-3.
    op_93: 0x93 => F(4), "f"; // SUB E; U5-7, L3-63, A-3.
    op_94: 0x94 => F(4), "f"; // SUB H; U5-7, L3-63, A-3.
    op_95: 0x95 => F(4), "f"; // SUB L; U5-7, L3-63, A-3.
    op_96: 0x96 => F(7), "fr"; // SUB M; U5-7, L3-64, A-3.
    op_97: 0x97 => F(4), "f"; // SUB A; U5-7, L3-63, A-3.
    op_98: 0x98 => F(4), "f"; // SBB B; U5-7, L3-56, A-3.
    op_99: 0x99 => F(4), "f"; // SBB C; U5-7, L3-56, A-3.
    op_9a: 0x9A => F(4), "f"; // SBB D; U5-7, L3-56, A-3.
    op_9b: 0x9B => F(4), "f"; // SBB E; U5-7, L3-56, A-3.
    op_9c: 0x9C => F(4), "f"; // SBB H; U5-7, L3-56, A-3.
    op_9d: 0x9D => F(4), "f"; // SBB L; U5-7, L3-56, A-3.
    op_9e: 0x9E => F(7), "fr"; // SBB M; U5-8, L3-57, A-3.
    op_9f: 0x9F => F(4), "f"; // SBB A; U5-7, L3-56, A-3.

    op_a0: 0xA0 => F(4), "f"; // ANA B; U5-9, L3-6, A-3.
    op_a1: 0xA1 => F(4), "f"; // ANA C; U5-9, L3-6, A-3.
    op_a2: 0xA2 => F(4), "f"; // ANA D; U5-9, L3-6, A-3.
    op_a3: 0xA3 => F(4), "f"; // ANA E; U5-9, L3-6, A-3.
    op_a4: 0xA4 => F(4), "f"; // ANA H; U5-9, L3-6, A-3.
    op_a5: 0xA5 => F(4), "f"; // ANA L; U5-9, L3-6, A-3.
    op_a6: 0xA6 => F(7), "fr"; // ANA M; U5-10, L3-7, A-3.
    op_a7: 0xA7 => F(4), "f"; // ANA A; U5-9, L3-6, A-3.
    op_a8: 0xA8 => F(4), "f"; // XRA B; U5-10, L3-66, A-3.
    op_a9: 0xA9 => F(4), "f"; // XRA C; U5-10, L3-66, A-3.
    op_aa: 0xAA => F(4), "f"; // XRA D; U5-10, L3-66, A-3.
    op_ab: 0xAB => F(4), "f"; // XRA E; U5-10, L3-66, A-3.
    op_ac: 0xAC => F(4), "f"; // XRA H; U5-10, L3-66, A-3.
    op_ad: 0xAD => F(4), "f"; // XRA L; U5-10, L3-66, A-3.
    op_ae: 0xAE => F(7), "fr"; // XRA M; U5-10, L3-67, A-3.
    op_af: 0xAF => F(4), "f"; // XRA A; U5-10, L3-66, A-3.

    op_b0: 0xB0 => F(4), "f"; // ORA B; U5-10, L3-39, A-3.
    op_b1: 0xB1 => F(4), "f"; // ORA C; U5-10, L3-39, A-3.
    op_b2: 0xB2 => F(4), "f"; // ORA D; U5-10, L3-39, A-3.
    op_b3: 0xB3 => F(4), "f"; // ORA E; U5-10, L3-39, A-3.
    op_b4: 0xB4 => F(4), "f"; // ORA H; U5-10, L3-39, A-3.
    op_b5: 0xB5 => F(4), "f"; // ORA L; U5-10, L3-39, A-3.
    op_b6: 0xB6 => F(7), "fr"; // ORA M; U5-11, L3-39, A-3.
    op_b7: 0xB7 => F(4), "f"; // ORA A; U5-10, L3-39, A-3.
    op_b8: 0xB8 => F(4), "f"; // CMP B; U5-11, L3-13, A-3.
    op_b9: 0xB9 => F(4), "f"; // CMP C; U5-11, L3-13, A-3.
    op_ba: 0xBA => F(4), "f"; // CMP D; U5-11, L3-13, A-3.
    op_bb: 0xBB => F(4), "f"; // CMP E; U5-11, L3-13, A-3.
    op_bc: 0xBC => F(4), "f"; // CMP H; U5-11, L3-13, A-3.
    op_bd: 0xBD => F(4), "f"; // CMP L; U5-11, L3-13, A-3.
    op_be: 0xBE => F(7), "fr"; // CMP M; U5-11, L3-13, A-3.
    op_bf: 0xBF => F(4), "f"; // CMP A; U5-11, L3-13, A-3.

    op_c0: 0xC0 => C(6, 12), "f/fss"; // RNZ; U5-14, L3-51, A-2.
    op_c1: 0xC1 => F(10), "fss"; // POP B; U5-15, L3-43, A-2.
    op_c2: 0xC2 => C(7, 10), "fo/foo"; // JNZ; U5-13, L3-29, A-3.
    op_c3: 0xC3 => F(10), "foo"; // JMP; U5-13, L3-28, A-3.
    op_c4: 0xC4 => C(9, 18), "fo/foott"; // CNZ; U5-14, L3-15, A-1.
    op_c5: 0xC5 => F(12), "ftt"; // PUSH B; U5-15, L3-44, A-2.
    op_c6: 0xC6 => F(7), "fo"; // ADI; U5-6, L3-5, A-3.
    op_c7: 0xC7 => F(12), "ftt"; // RST 0; U5-14, L3-55, A-2.
    op_c8: 0xC8 => C(6, 12), "f/fss"; // RZ; U5-14, L3-56, A-1.
    op_c9: 0xC9 => F(10), "fss"; // RET; U5-14, L3-48, A-1.
    op_ca: 0xCA => C(7, 10), "fo/foo"; // JZ; U5-13, L3-32, A-3.
    op_cb: 0xCB => U, ""; // Table 5-2; A-5.
    op_cc: 0xCC => C(9, 18), "fo/foott"; // CZ; U5-14, L3-18, A-1.
    op_cd: 0xCD => F(18), "foott"; // CALL; U5-13, L3-9, A-1.
    op_ce: 0xCE => F(7), "fo"; // ACI; U5-7, L3-2, A-3.
    op_cf: 0xCF => F(12), "ftt"; // RST 1; U5-14, L3-55, A-2.

    op_d0: 0xD0 => C(6, 12), "f/fss"; // RNC; U5-14, L3-51, A-1.
    op_d1: 0xD1 => F(10), "fss"; // POP D; U5-15, L3-43, A-2.
    op_d2: 0xD2 => C(7, 10), "fo/foo"; // JNC; U5-13, L3-28, A-3.
    op_d3: 0xD3 => F(10), "fou"; // OUT; U5-16, L3-41, A-2.
    op_d4: 0xD4 => C(9, 18), "fo/foott"; // CNC; U5-14, L3-14, A-1.
    op_d5: 0xD5 => F(12), "ftt"; // PUSH D; U5-15, L3-44, A-2.
    op_d6: 0xD6 => F(7), "fo"; // SUI; U5-7, L3-65, A-3.
    op_d7: 0xD7 => F(12), "ftt"; // RST 2; U5-14, L3-55, A-2.
    op_d8: 0xD8 => C(6, 12), "f/fss"; // RC; U5-14, L3-47, A-1.
    op_d9: 0xD9 => U, ""; // Table 5-2; A-5.
    op_da: 0xDA => C(7, 10), "fo/foo"; // JC; U5-13, L3-27, A-3.
    op_db: 0xDB => F(10), "foi"; // IN; U5-16, L3-25, A-2.
    op_dc: 0xDC => C(9, 18), "fo/foott"; // CC; U5-14, L3-10, A-1.
    op_dd: 0xDD => U, ""; // Table 5-2; A-5.
    op_de: 0xDE => F(7), "fo"; // SBI; U5-8, L3-58, A-3.
    op_df: 0xDF => F(12), "ftt"; // RST 3; U5-14, L3-55, A-2.

    op_e0: 0xE0 => C(6, 12), "f/fss"; // RPO; U5-14, L3-53, A-2.
    op_e1: 0xE1 => F(10), "fss"; // POP H; U5-15, L3-43, A-2.
    op_e2: 0xE2 => C(7, 10), "fo/foo"; // JPO; U5-13, L3-31, A-3.
    op_e3: 0xE3 => F(16), "fsstt"; // XTHL; U5-16, L3-69, A-2.
    op_e4: 0xE4 => C(9, 18), "fo/foott"; // CPO; U5-14, L3-17, A-1.
    op_e5: 0xE5 => F(12), "ftt"; // PUSH H; U5-15, L3-44, A-2.
    op_e6: 0xE6 => F(7), "fo"; // ANI; U5-10, L3-8, A-3.
    op_e7: 0xE7 => F(12), "ftt"; // RST 4; U5-14, L3-55, A-2.
    op_e8: 0xE8 => C(6, 12), "f/fss"; // RPE; U5-14, L3-52, A-2.
    op_e9: 0xE9 => F(6), "f"; // PCHL; U5-15, L3-42, A-2.
    op_ea: 0xEA => C(7, 10), "fo/foo"; // JPE; U5-13, L3-30, A-3.
    op_eb: 0xEB => F(4), "f"; // XCHG; U5-6, L3-65, A-2.
    op_ec: 0xEC => C(9, 18), "fo/foott"; // CPE; U5-14, L3-16, A-1.
    op_ed: 0xED => U, ""; // Table 5-2; A-5.
    op_ee: 0xEE => F(7), "fo"; // XRI; U5-10, L3-68, A-3.
    op_ef: 0xEF => F(12), "ftt"; // RST 5; U5-14, L3-55, A-2.

    op_f0: 0xF0 => C(6, 12), "f/fss"; // RP; U5-14, L3-52, A-2.
    op_f1: 0xF1 => F(10), "fss"; // POP PSW; U5-16, L3-43, A-2.
    op_f2: 0xF2 => C(7, 10), "fo/foo"; // JP; U5-13, L3-30, A-3.
    op_f3: 0xF3 => F(4), "f"; // DI; U5-17, L3-23, A-4.
    op_f4: 0xF4 => C(9, 18), "fo/foott"; // CP; U5-14, L3-15, A-1.
    op_f5: 0xF5 => F(12), "ftt"; // PUSH PSW; U5-15, L3-45, A-2.
    op_f6: 0xF6 => F(7), "fo"; // ORI; U5-11, L3-40, A-3.
    op_f7: 0xF7 => F(12), "ftt"; // RST 6; U5-14, L3-55, A-2.
    op_f8: 0xF8 => C(6, 12), "f/fss"; // RM; U5-14, L3-50, A-2.
    op_f9: 0xF9 => F(6), "f"; // SPHL; U5-16, L3-61, A-2.
    op_fa: 0xFA => C(7, 10), "fo/foo"; // JM; U5-13, L3-27, A-3.
    op_fb: 0xFB => F(4), "f"; // EI; U5-17, L3-23, A-4.
    op_fc: 0xFC => C(9, 18), "fo/foott"; // CM; U5-14, L3-11, A-1.
    op_fd: 0xFD => U, ""; // Table 5-2; A-5.
    op_fe: 0xFE => F(7), "fo"; // CPI; U5-11, L3-17, A-3.
    op_ff: 0xFF => F(12), "ftt"; // RST 7; U5-14, L3-55, A-2.
}

#[test]
fn table_covers_246_documented_bytes_and_24_double_path_conditionals() {
    let mut undocumented = Vec::new();
    let mut conditional = Vec::new();
    let mut paths = 0;
    for (index, &(opcode, timing)) in TIMINGS.iter().enumerate() {
        assert_eq!(usize::from(opcode), index);
        match timing {
            U => undocumented.push(opcode),
            F(_) => paths += 1,
            C(_, _) => {
                conditional.push(opcode);
                paths += 2;
            }
        }
    }
    // 205775-002 Table 5-2 p. 5-20; 9800301-04 p. A-5.
    assert_eq!(
        undocumented,
        [0x08, 0x10, 0x18, 0x28, 0x38, 0xCB, 0xD9, 0xDD, 0xED, 0xFD]
    );
    assert_eq!(256 - undocumented.len(), 246);
    assert_eq!(paths, 270);
    // CCC=000..111, J/C/R entries: 205775-002 pp. 5-13/5-14.
    assert_eq!(
        conditional,
        [
            0xC0, 0xC2, 0xC4, 0xC8, 0xCA, 0xCC, 0xD0, 0xD2, 0xD4, 0xD8, 0xDA, 0xDC, 0xE0, 0xE2,
            0xE4, 0xE8, 0xEA, 0xEC, 0xF0, 0xF2, 0xF4, 0xF8, 0xFA, 0xFC,
        ]
    );
}

fn fixture(opcode: u8, flags: Flags) -> (I8085, RecordingBus) {
    let cpu = I8085::new(
        Registers {
            a: 0x21,
            b: 0x20,
            c: 0,
            d: 0x21,
            e: 0,
            h: 0x30,
            l: 0,
            flags,
            pc: 0x4000,
            sp: 0x8000,
        },
        Latches::default(),
    );
    let mut bus = RecordingBus::new();
    // No address/register wrap is needed. Indirect data, instruction stream,
    // direct data and stack are disjoint. The page entries define byte 2 low,
    // byte 3 high (205775-002 pp. 5-5/5-13); RET reads low at SP (p. 5-14).
    bus.load(0x4000, &[opcode, 0x20, 0x60]);
    bus.load(0x2000, &[0x42]);
    bus.load(0x2100, &[0x53]);
    bus.load(0x3000, &[0x64]);
    bus.load(0x6020, &[0x75, 0x86]);
    bus.load(0x8000, &[0x40, 0x70]);
    if opcode == 0xDB {
        bus.inputs.push_back(0x5A);
    }
    (cpu, bus)
}

// CCC mapping and polarity: 205775-002 section 5.6.4 p. 5-13 condition
// table; independently, 9800301-04 pp. A-1..3 and the cited Ch. 3 entries.
fn flags_for_path(opcode: u8, taken: bool) -> Flags {
    let mut flags = Flags::default();
    let polarity = if opcode & 8 == 0 { !taken } else { taken };
    match (opcode >> 4) & 3 {
        0 => flags.zero = polarity,
        1 => flags.carry = polarity,
        2 => flags.parity = polarity,
        3 => flags.sign = polarity,
        _ => unreachable!(),
    }
    flags
}

fn witness(opcode: u8, timing: Timing, trace: &str) {
    match timing {
        U => {
            let (mut cpu, mut bus) = fixture(opcode, Flags::default());
            let before = cpu.clone();
            let report = cpu.step(&mut bus);
            assert_eq!((report.steps, report.cycles), (0, 0));
            let Exit::Unsupported(unsupported) = report.exit else {
                panic!("opcode {opcode:#04x}: {report:?}");
            };
            assert_eq!((unsupported.pc, unsupported.opcode), (0x4000, opcode));
            assert!(!unsupported.acknowledge);
            assert_eq!(cpu, before);
            assert_eq!(bus_trace(&bus), "f");
        }
        F(states) => witness_path(opcode, states, None, trace),
        C(untaken, taken) => {
            let (untaken_trace, taken_trace) = trace.split_once('/').unwrap();
            witness_path(opcode, untaken, Some(false), untaken_trace);
            witness_path(opcode, taken, Some(true), taken_trace);
        }
    }
}

fn witness_path(opcode: u8, states: u8, taken: Option<bool>, trace: &str) {
    let flags = taken.map_or_else(Flags::default, |path| flags_for_path(opcode, path));
    let (mut cpu, mut bus) = fixture(opcode, flags);
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: u64::from(states),
            exit: Exit::Budget,
        },
        "opcode {opcode:#04x}, taken={taken:?}"
    );
    assert_eq!(
        bus_trace(&bus),
        trace,
        "opcode {opcode:#04x}, taken={taken:?}"
    );

    // Check that each condition really selected its requested path, rather
    // than merely selecting a reported total. PC/SP effects: 205775-002
    // pp. 5-13/5-14; 9800301-04 Ch. 3 entries cited per byte above.
    if let Some(taken) = taken {
        let (pc, sp) = match (opcode & 7, taken) {
            (0, false) => (0x4001, 0x8000),
            (0, true) => (0x7040, 0x8002),
            (2 | 4, false) => (0x4003, 0x8000),
            (2, true) => (0x6020, 0x8000),
            (4, true) => (0x6020, 0x7FFE),
            _ => unreachable!(),
        };
        assert_eq!(
            (cpu.registers().pc, cpu.registers().sp),
            (pc, sp),
            "opcode {opcode:#04x}, taken={taken}"
        );
    }
    if opcode == 0x76 {
        assert!(cpu.latches().halted);
        bus.take_log();
        // The five states cover executing HLT, not time remaining halted.
        assert_eq!(
            cpu.step(&mut bus),
            Report {
                steps: 0,
                cycles: 0,
                exit: Exit::Halt
            }
        );
        assert_eq!(bus_trace(&bus), "");
    }
}

// Projection of recorded accesses, not a state-count calculation. Internal
// states cannot be reconstructed from bus calls (e.g. F vs S and bus idle).
fn bus_trace(bus: &RecordingBus) -> String {
    bus.log
        .iter()
        .filter_map(|event| match event {
            Event::Read {
                kind: MemoryAccess::OpcodeFetch,
                ..
            } => Some('f'),
            Event::Read {
                kind: MemoryAccess::OperandFetch,
                ..
            } => Some('o'),
            Event::Read {
                kind: MemoryAccess::Data,
                ..
            } => Some('r'),
            Event::Read {
                kind: MemoryAccess::Stack,
                ..
            } => Some('s'),
            Event::Read {
                kind: MemoryAccess::InterruptAcknowledge,
                ..
            } => Some('a'),
            Event::Write {
                kind: MemoryAccess::Data,
                ..
            } => Some('w'),
            Event::Write {
                kind: MemoryAccess::Stack,
                ..
            } => Some('t'),
            Event::Input { .. } => Some('i'),
            Event::Output { .. } => Some('u'),
            Event::Read { .. } | Event::Write { .. } => panic!("unexpected access: {event:?}"),
            _ => None,
        })
        .collect()
}

fn interrupt_fixture() -> (I8085, RecordingBus) {
    let (mut cpu, bus) = fixture(0x08, Flags::default());
    *cpu.latches_mut() = Latches {
        interrupt_enable: true,
        mask7_5: false,
        mask6_5: false,
        mask5_5: false,
        ..Latches::default()
    };
    (cpu, bus)
}

fn acceptance_report(cpu: &mut I8085, bus: &mut RecordingBus, run: bool) -> Report {
    if run {
        // decision-2: acceptance finishes even though its first state
        // exhausts this budget. decision-12 counts it as one step.
        cpu.run(
            bus,
            Budget {
                steps: u64::MAX,
                cycles: 1,
            },
        )
    } else {
        cpu.step(bus)
    }
}

// DERIVED 12 states: Fig. 2-19 p. 2-15 plus RST p. 5-14; second
// witnesses L3-55 and DS Table 3 p. 16-20. No opcode/acknowledge read.
fn vectored_acceptance(source: u8, vector: u16) {
    for run in [false, true] {
        let (mut cpu, mut bus) = interrupt_fixture();
        // Vectors/line types: 205775-002 p. 2-5; 231718-001 Table 2,
        // handbook p. 16-13. Each test raises exactly one source.
        match source {
            0 => bus.trap = true,
            1 => bus.rst7_5 = true,
            2 => bus.rst6_5 = true,
            3 => bus.rst5_5 = true,
            _ => unreachable!(),
        }
        assert_eq!(
            acceptance_report(&mut cpu, &mut bus, run),
            Report {
                steps: 1,
                cycles: 12,
                exit: Exit::Budget
            }
        );
        assert_eq!((cpu.registers().pc, cpu.registers().sp), (vector, 0x7FFE));
        assert!(!cpu.latches().interrupt_enable);
        assert_eq!(bus_trace(&bus), "tt");
        assert_eq!(
            bus.log
                .iter()
                .filter(|event| matches!(event, Event::Write { .. }))
                .copied()
                .collect::<Vec<_>>(),
            [
                Event::Write {
                    address: 0x7FFF,
                    value: 0x40,
                    kind: MemoryAccess::Stack
                },
                Event::Write {
                    address: 0x7FFE,
                    value: 0x00,
                    kind: MemoryAccess::Stack
                },
            ]
        );
    }
}

#[test]
fn trap_acceptance_is_derived_twelve_states() {
    vectored_acceptance(0, 0x0024);
}

#[test]
fn rst7_5_acceptance_is_derived_twelve_states() {
    vectored_acceptance(1, 0x003C);
}

#[test]
fn rst6_5_acceptance_is_derived_twelve_states() {
    vectored_acceptance(2, 0x0034);
}

#[test]
fn rst5_5_acceptance_is_derived_twelve_states() {
    vectored_acceptance(3, 0x002C);
}

fn intr_acceptance(bytes: &[u8], target: u16, states: u64) {
    for run in [false, true] {
        let (mut cpu, mut bus) = interrupt_fixture();
        bus.intr = true;
        bus.acknowledges.extend(bytes.iter().copied());
        assert_eq!(
            acceptance_report(&mut cpu, &mut bus, run),
            Report {
                steps: 1,
                cycles: states,
                exit: Exit::Budget
            }
        );
        assert_eq!((cpu.registers().pc, cpu.registers().sp), (target, 0x7FFE));
        assert!(!cpu.latches().interrupt_enable);
        assert!(bus.acknowledges.is_empty());
        // Figs 2-17/18 pp. 2-13/14: every INA uses the unchanged PC,
        // then MW writes high followed by low. No operand fetch follows.
        let mut expected = bytes
            .iter()
            .map(|&value| Event::Read {
                address: 0x4000,
                kind: MemoryAccess::InterruptAcknowledge,
                value,
            })
            .collect::<Vec<_>>();
        expected.extend([
            Event::Write {
                address: 0x7FFF,
                kind: MemoryAccess::Stack,
                value: 0x40,
            },
            Event::Write {
                address: 0x7FFE,
                kind: MemoryAccess::Stack,
                value: 0x00,
            },
        ]);
        assert_eq!(
            bus.log
                .iter()
                .filter(|event| matches!(event, Event::Read { .. } | Event::Write { .. }))
                .copied()
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn all_eight_intr_restarts_accept_in_derived_twelve_states() {
    // RST encodings/vectors: U5-14, L3-55; INA replacement: Figs 2-17/18.
    for (opcode, target) in [
        (0xC7, 0x0000),
        (0xCF, 0x0008),
        (0xD7, 0x0010),
        (0xDF, 0x0018),
        (0xE7, 0x0020),
        (0xEF, 0x0028),
        (0xF7, 0x0030),
        (0xFF, 0x0038),
    ] {
        intr_acceptance(&[opcode], target, 12);
    }
}

#[test]
fn intr_call_accepts_in_derived_eighteen_states() {
    // Figs 2-17/18: INA(6), INA(3), INA(3), MW(3), MW(3).
    // Independently: normal CALL is 18 in U5-13 and L3-9.
    intr_acceptance(&[0xCD, 0x20, 0x60], 0x6020, 18);
}

#[test]
fn cycle_budget_ends_mid_sequence_and_completes_the_next_boundary() {
    // U5-4 MVI=7, U5-9 INX=6, U5-16 XTHL=16, U5-17 HLT=5;
    // second witnesses L3-37/3-26/3-69/3-24 (8085 figures).
    // Table 5-1: FR, S, FRRWW, FB. decision-2 allows overshoot.
    let (mut cpu, mut bus) = fixture(0x3E, Flags::default());
    bus.load(0x4000, &[0x3E, 0x44, 0x23, 0xE3, 0x76]);
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: u64::MAX,
                cycles: 14
            }
        ),
        Report {
            steps: 3,
            cycles: 29,
            exit: Exit::Budget
        }
    );
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x4004, 0x8000));
    assert_eq!((cpu.registers().h, cpu.registers().l), (0x70, 0x40));
    assert_eq!(&bus.memory[0x8000..0x8002], &[0x01, 0x30]);
    assert_eq!(bus_trace(&bus), "foffsstt");
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 1,
            cycles: 5,
            exit: Exit::Budget
        }
    );
    assert!(cpu.latches().halted);
}

#[test]
fn taken_conditionals_overshoot_their_untaken_cycle_budget() {
    // U5-13/14 and Table 5-1: Jcc 7/10, Ccc 9/18, Rcc 6/12;
    // independently L3-32 (JZ), L3-18 (CZ), L3-56 (RZ).
    for (opcode, budget, total, pc, sp) in [
        (0xCA, 7, 10, 0x6020, 0x8000),
        (0xCC, 9, 18, 0x6020, 0x7FFE),
        (0xC8, 6, 12, 0x7040, 0x8002),
    ] {
        let (mut cpu, mut bus) = fixture(opcode, flags_for_path(opcode, true));
        assert_eq!(
            cpu.run(
                &mut bus,
                Budget {
                    steps: u64::MAX,
                    cycles: budget
                }
            ),
            Report {
                steps: 1,
                cycles: total,
                exit: Exit::Budget
            }
        );
        assert_eq!((cpu.registers().pc, cpu.registers().sp), (pc, sp));
    }
}

#[test]
fn step_budget_counts_interrupt_acceptance_then_the_vector_instruction() {
    // decision-5 item 8 / decision-12 rule 4: acceptance is exactly one
    // step. Derived acceptance 12 (Fig. 2-19); NOP 4 (U5-17, L3-38).
    let (mut cpu, mut bus) = interrupt_fixture();
    bus.trap = true;
    bus.load(0x0024, &[0x00, 0x08]);
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: 2,
                cycles: u64::MAX
            }
        ),
        Report {
            steps: 2,
            cycles: 16,
            exit: Exit::Budget
        }
    );
    assert_eq!(cpu.registers().pc, 0x0025);
    assert_eq!(bus_trace(&bus), "ttf");
    assert_eq!(bus.memory[0x7FFE..0x8000], [0x00, 0x40]);
}

#[test]
fn unsupported_exit_preserves_earlier_real_instruction_progress() {
    // decision-5 item 6; decision-11: 08 is undocumented (Table 5-2/A-5).
    // MVI 7 (U5-4/L3-37), INR 4 (U5-8/L3-25),
    // LXI 10 (U5-5/L3-35); Table 5-1 sequences FR, F, FRR.
    let (mut cpu, mut bus) = fixture(0x3E, Flags::default());
    bus.load(0x4000, &[0x3E, 0x44, 0x3C, 0x21, 0x00, 0x30, 0x08]);
    let report = cpu.run(
        &mut bus,
        Budget {
            steps: u64::MAX,
            cycles: u64::MAX,
        },
    );
    assert_eq!((report.steps, report.cycles), (3, 21));
    let Exit::Unsupported(unsupported) = report.exit else {
        panic!("{report:?}");
    };
    assert_eq!(
        (unsupported.pc, unsupported.opcode, unsupported.acknowledge),
        (0x4006, 0x08, false)
    );
    assert_eq!(cpu.registers().a, 0x45);
    assert_eq!((cpu.registers().h, cpu.registers().l), (0x30, 0x00));
    assert_eq!(cpu.registers().pc, 0x4006);
    assert_eq!(bus_trace(&bus), "foffoof");
    let before = cpu.clone();
    assert_eq!(
        cpu.step(&mut bus),
        Report {
            steps: 0,
            cycles: 0,
            exit: report.exit
        }
    );
    assert_eq!(cpu, before);
}
