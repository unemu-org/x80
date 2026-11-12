//! Independent public-API T-state witnesses (decision-2; decision-5 items 5, 8).
//!
//! Primary transcription: Intel 8080 Microcomputer Systems User's Manual,
//! 98-153B (September 1975), Ch. 2 machine-cycle/state table, printed pp. 2-16
//! through 2-20 (PDF pp. 30-34), read from 200-dpi page images. Transfer,
//! arithmetic and ANA rows occupy pp. 2-16/17; the remaining logical, branch,
//! stack, I/O and control rows occupy pp. 2-18/19. Apply p. 2-20 notes:
//! 3 counts internal X states; 5 skips shaded states; 8 counts DAD's internal
//! cycles; 9 and 11 assign overlapping work to the NEXT instruction fetch;
//! 13 skips Ccc M4/M5 when untaken; 14 skips Rcc M2/M3 when untaken.
//! Thus Jcc is 10/10, Ccc is 11/17 and Rcc is 5/11 (untaken/taken).
//! No READY waits or time spent remaining halted are included (notes 2, 20).
//!
//! Every state total was cross-read against the per-instruction Cycles/States
//! lines in Intel 8080/8085 Assembly Language Programming Manual, 9800301-04
//! (May 1981), Ch. 3. Each row below cites its printed 3-n page (PDF 54+n).
//! Only the 8080 figure is used, excluding parenthesized 8085 counts. All state
//! totals agree. One machine-cycle-count discrepancy: HLT has M1 and M2 in
//! 98-153B p. 2-18, whereas 9800301-04 p. 3-24 prints Cycles: 1; both give
//! seven states. `Report.cycles` counts states, not those machine cycles.
//!
//! Source revisions/hashes are pinned in PROVENANCE.md section 1 and were
//! verified before transcription. Neither the implementation's timing values
//! nor the Ch. 4 summary supplies the expectations here. The tests need no PDFs.

mod support;

use support::{assert_unsupported, Event, RecordingBus};
use x80_core::i8080::{Flags, Latches, Registers, I8080};
use x80_core::{Budget, Exit, Report};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Timing {
    Undocumented,
    Fixed(u8),
    Conditional(u8, u8), // Untaken, taken (including equal Jcc totals).
}

use Timing::{Conditional as C, Fixed as F, Undocumented as U};

// One independently selectable test per byte allows a discovered discrepancy
// to be ignored on that opcode alone, retaining the other witnesses.
macro_rules! timing_witnesses {
    ($( $(#[$attr:meta])* $name:ident: $opcode:literal => $timing:expr; )*) => {
        const TIMINGS: [(u8, Timing); 256] = [$(($opcode, $timing)),*];

        $(
            $(#[$attr])*
            #[test]
            fn $name() {
                witness($opcode, TIMINGS[$opcode as usize].1);
            }
        )*
    };
}

// Literal 256-byte transcription in opcode order; U marks undocumented bytes.
// Trailing citations are the independent 9800301-04 Ch. 3 cross-read pages.
timing_witnesses! {
    op_00: 0x00 => F(4);       // NOP, 3-38.
    op_01: 0x01 => F(10);      // LXI B, 3-35.
    op_02: 0x02 => F(7);       // STAX B, 3-62.
    op_03: 0x03 => F(5);       // INX B, 3-26.
    op_04: 0x04 => F(5);       // INR B, 3-25.
    op_05: 0x05 => F(5);       // DCR B, 3-21.
    op_06: 0x06 => F(7);       // MVI B, 3-37.
    op_07: 0x07 => F(4);       // RLC, 3-49.
    op_08: 0x08 => U;
    op_09: 0x09 => F(10);      // DAD B, 3-20.
    op_0a: 0x0A => F(7);       // LDAX B, 3-33.
    op_0b: 0x0B => F(5);       // DCX B, 3-22.
    op_0c: 0x0C => F(5);       // INR C, 3-25.
    op_0d: 0x0D => F(5);       // DCR C, 3-21.
    op_0e: 0x0E => F(7);       // MVI C, 3-37.
    op_0f: 0x0F => F(4);       // RRC, 3-54.

    op_10: 0x10 => U;
    op_11: 0x11 => F(10);      // LXI D, 3-35.
    op_12: 0x12 => F(7);       // STAX D, 3-62.
    op_13: 0x13 => F(5);       // INX D, 3-26.
    op_14: 0x14 => F(5);       // INR D, 3-25.
    op_15: 0x15 => F(5);       // DCR D, 3-21.
    op_16: 0x16 => F(7);       // MVI D, 3-37.
    op_17: 0x17 => F(4);       // RAL, 3-46.
    op_18: 0x18 => U;
    op_19: 0x19 => F(10);      // DAD D, 3-20.
    op_1a: 0x1A => F(7);       // LDAX D, 3-33.
    op_1b: 0x1B => F(5);       // DCX D, 3-22.
    op_1c: 0x1C => F(5);       // INR E, 3-25.
    op_1d: 0x1D => F(5);       // DCR E, 3-21.
    op_1e: 0x1E => F(7);       // MVI E, 3-37.
    op_1f: 0x1F => F(4);       // RAR, 3-46.

    op_20: 0x20 => U;
    op_21: 0x21 => F(10);      // LXI H, 3-35.
    op_22: 0x22 => F(16);      // SHLD, 3-59.
    op_23: 0x23 => F(5);       // INX H, 3-26.
    op_24: 0x24 => F(5);       // INR H, 3-25.
    op_25: 0x25 => F(5);       // DCR H, 3-21.
    op_26: 0x26 => F(7);       // MVI H, 3-37.
    op_27: 0x27 => F(4);       // DAA, 3-19.
    op_28: 0x28 => U;
    op_29: 0x29 => F(10);      // DAD H, 3-20.
    op_2a: 0x2A => F(16);      // LHLD, 3-34.
    op_2b: 0x2B => F(5);       // DCX H, 3-22.
    op_2c: 0x2C => F(5);       // INR L, 3-25.
    op_2d: 0x2D => F(5);       // DCR L, 3-21.
    op_2e: 0x2E => F(7);       // MVI L, 3-37.
    op_2f: 0x2F => F(4);       // CMA, 3-11.

    op_30: 0x30 => U;
    op_31: 0x31 => F(10);      // LXI SP, 3-35.
    op_32: 0x32 => F(13);      // STA, 3-62.
    op_33: 0x33 => F(5);       // INX SP, 3-26.
    op_34: 0x34 => F(10);      // INR M, 3-25.
    op_35: 0x35 => F(10);      // DCR M, 3-21.
    op_36: 0x36 => F(10);      // MVI M, 3-38.
    op_37: 0x37 => F(4);       // STC, 3-63.
    op_38: 0x38 => U;
    op_39: 0x39 => F(10);      // DAD SP, 3-20.
    op_3a: 0x3A => F(13);      // LDA, 3-33.
    op_3b: 0x3B => F(5);       // DCX SP, 3-22.
    op_3c: 0x3C => F(5);       // INR A, 3-25.
    op_3d: 0x3D => F(5);       // DCR A, 3-21.
    op_3e: 0x3E => F(7);       // MVI A, 3-37.
    op_3f: 0x3F => F(4);       // CMC, 3-12.

    op_40: 0x40 => F(5);       // MOV B,B, 3-36.
    op_41: 0x41 => F(5);       // MOV B,C, 3-36.
    op_42: 0x42 => F(5);       // MOV B,D, 3-36.
    op_43: 0x43 => F(5);       // MOV B,E, 3-36.
    op_44: 0x44 => F(5);       // MOV B,H, 3-36.
    op_45: 0x45 => F(5);       // MOV B,L, 3-36.
    op_46: 0x46 => F(7);       // MOV B,M, 3-37.
    op_47: 0x47 => F(5);       // MOV B,A, 3-36.
    op_48: 0x48 => F(5);       // MOV C,B, 3-36.
    op_49: 0x49 => F(5);       // MOV C,C, 3-36.
    op_4a: 0x4A => F(5);       // MOV C,D, 3-36.
    op_4b: 0x4B => F(5);       // MOV C,E, 3-36.
    op_4c: 0x4C => F(5);       // MOV C,H, 3-36.
    op_4d: 0x4D => F(5);       // MOV C,L, 3-36.
    op_4e: 0x4E => F(7);       // MOV C,M, 3-37.
    op_4f: 0x4F => F(5);       // MOV C,A, 3-36.

    op_50: 0x50 => F(5);       // MOV D,B, 3-36.
    op_51: 0x51 => F(5);       // MOV D,C, 3-36.
    op_52: 0x52 => F(5);       // MOV D,D, 3-36.
    op_53: 0x53 => F(5);       // MOV D,E, 3-36.
    op_54: 0x54 => F(5);       // MOV D,H, 3-36.
    op_55: 0x55 => F(5);       // MOV D,L, 3-36.
    op_56: 0x56 => F(7);       // MOV D,M, 3-37.
    op_57: 0x57 => F(5);       // MOV D,A, 3-36.
    op_58: 0x58 => F(5);       // MOV E,B, 3-36.
    op_59: 0x59 => F(5);       // MOV E,C, 3-36.
    op_5a: 0x5A => F(5);       // MOV E,D, 3-36.
    op_5b: 0x5B => F(5);       // MOV E,E, 3-36.
    op_5c: 0x5C => F(5);       // MOV E,H, 3-36.
    op_5d: 0x5D => F(5);       // MOV E,L, 3-36.
    op_5e: 0x5E => F(7);       // MOV E,M, 3-37.
    op_5f: 0x5F => F(5);       // MOV E,A, 3-36.

    op_60: 0x60 => F(5);       // MOV H,B, 3-36.
    op_61: 0x61 => F(5);       // MOV H,C, 3-36.
    op_62: 0x62 => F(5);       // MOV H,D, 3-36.
    op_63: 0x63 => F(5);       // MOV H,E, 3-36.
    op_64: 0x64 => F(5);       // MOV H,H, 3-36.
    op_65: 0x65 => F(5);       // MOV H,L, 3-36.
    op_66: 0x66 => F(7);       // MOV H,M, 3-37.
    op_67: 0x67 => F(5);       // MOV H,A, 3-36.
    op_68: 0x68 => F(5);       // MOV L,B, 3-36.
    op_69: 0x69 => F(5);       // MOV L,C, 3-36.
    op_6a: 0x6A => F(5);       // MOV L,D, 3-36.
    op_6b: 0x6B => F(5);       // MOV L,E, 3-36.
    op_6c: 0x6C => F(5);       // MOV L,H, 3-36.
    op_6d: 0x6D => F(5);       // MOV L,L, 3-36.
    op_6e: 0x6E => F(7);       // MOV L,M, 3-37.
    op_6f: 0x6F => F(5);       // MOV L,A, 3-36.

    op_70: 0x70 => F(7);       // MOV M,B, 3-36.
    op_71: 0x71 => F(7);       // MOV M,C, 3-36.
    op_72: 0x72 => F(7);       // MOV M,D, 3-36.
    op_73: 0x73 => F(7);       // MOV M,E, 3-36.
    op_74: 0x74 => F(7);       // MOV M,H, 3-36.
    op_75: 0x75 => F(7);       // MOV M,L, 3-36.
    op_76: 0x76 => F(7);       // HLT, 3-24.
    op_77: 0x77 => F(7);       // MOV M,A, 3-36.
    op_78: 0x78 => F(5);       // MOV A,B, 3-36.
    op_79: 0x79 => F(5);       // MOV A,C, 3-36.
    op_7a: 0x7A => F(5);       // MOV A,D, 3-36.
    op_7b: 0x7B => F(5);       // MOV A,E, 3-36.
    op_7c: 0x7C => F(5);       // MOV A,H, 3-36.
    op_7d: 0x7D => F(5);       // MOV A,L, 3-36.
    op_7e: 0x7E => F(7);       // MOV A,M, 3-37.
    op_7f: 0x7F => F(5);       // MOV A,A, 3-36.

    op_80: 0x80 => F(4);       // ADD B, 3-4.
    op_81: 0x81 => F(4);       // ADD C, 3-4.
    op_82: 0x82 => F(4);       // ADD D, 3-4.
    op_83: 0x83 => F(4);       // ADD E, 3-4.
    op_84: 0x84 => F(4);       // ADD H, 3-4.
    op_85: 0x85 => F(4);       // ADD L, 3-4.
    op_86: 0x86 => F(7);       // ADD M, 3-4.
    op_87: 0x87 => F(4);       // ADD A, 3-4.
    op_88: 0x88 => F(4);       // ADC B, 3-3.
    op_89: 0x89 => F(4);       // ADC C, 3-3.
    op_8a: 0x8A => F(4);       // ADC D, 3-3.
    op_8b: 0x8B => F(4);       // ADC E, 3-3.
    op_8c: 0x8C => F(4);       // ADC H, 3-3.
    op_8d: 0x8D => F(4);       // ADC L, 3-3.
    op_8e: 0x8E => F(7);       // ADC M, 3-3.
    op_8f: 0x8F => F(4);       // ADC A, 3-3.

    op_90: 0x90 => F(4);       // SUB B, 3-63.
    op_91: 0x91 => F(4);       // SUB C, 3-63.
    op_92: 0x92 => F(4);       // SUB D, 3-63.
    op_93: 0x93 => F(4);       // SUB E, 3-63.
    op_94: 0x94 => F(4);       // SUB H, 3-63.
    op_95: 0x95 => F(4);       // SUB L, 3-63.
    op_96: 0x96 => F(7);       // SUB M, 3-64.
    op_97: 0x97 => F(4);       // SUB A, 3-63.
    op_98: 0x98 => F(4);       // SBB B, 3-56.
    op_99: 0x99 => F(4);       // SBB C, 3-56.
    op_9a: 0x9A => F(4);       // SBB D, 3-56.
    op_9b: 0x9B => F(4);       // SBB E, 3-56.
    op_9c: 0x9C => F(4);       // SBB H, 3-56.
    op_9d: 0x9D => F(4);       // SBB L, 3-56.
    op_9e: 0x9E => F(7);       // SBB M, 3-57.
    op_9f: 0x9F => F(4);       // SBB A, 3-56.

    op_a0: 0xA0 => F(4);       // ANA B, 3-6.
    op_a1: 0xA1 => F(4);       // ANA C, 3-6.
    op_a2: 0xA2 => F(4);       // ANA D, 3-6.
    op_a3: 0xA3 => F(4);       // ANA E, 3-6.
    op_a4: 0xA4 => F(4);       // ANA H, 3-6.
    op_a5: 0xA5 => F(4);       // ANA L, 3-6.
    op_a6: 0xA6 => F(7);       // ANA M, 3-7.
    op_a7: 0xA7 => F(4);       // ANA A, 3-6.
    op_a8: 0xA8 => F(4);       // XRA B, 3-66.
    op_a9: 0xA9 => F(4);       // XRA C, 3-66.
    op_aa: 0xAA => F(4);       // XRA D, 3-66.
    op_ab: 0xAB => F(4);       // XRA E, 3-66.
    op_ac: 0xAC => F(4);       // XRA H, 3-66.
    op_ad: 0xAD => F(4);       // XRA L, 3-66.
    op_ae: 0xAE => F(7);       // XRA M, 3-67.
    op_af: 0xAF => F(4);       // XRA A, 3-66.

    op_b0: 0xB0 => F(4);       // ORA B, 3-39.
    op_b1: 0xB1 => F(4);       // ORA C, 3-39.
    op_b2: 0xB2 => F(4);       // ORA D, 3-39.
    op_b3: 0xB3 => F(4);       // ORA E, 3-39.
    op_b4: 0xB4 => F(4);       // ORA H, 3-39.
    op_b5: 0xB5 => F(4);       // ORA L, 3-39.
    op_b6: 0xB6 => F(7);       // ORA M, 3-39.
    op_b7: 0xB7 => F(4);       // ORA A, 3-39.
    op_b8: 0xB8 => F(4);       // CMP B, 3-13.
    op_b9: 0xB9 => F(4);       // CMP C, 3-13.
    op_ba: 0xBA => F(4);       // CMP D, 3-13.
    op_bb: 0xBB => F(4);       // CMP E, 3-13.
    op_bc: 0xBC => F(4);       // CMP H, 3-13.
    op_bd: 0xBD => F(4);       // CMP L, 3-13.
    op_be: 0xBE => F(7);       // CMP M, 3-13.
    op_bf: 0xBF => F(4);       // CMP A, 3-13.

    op_c0: 0xC0 => C(5, 11);   // RNZ, 3-51.
    op_c1: 0xC1 => F(10);      // POP B, 3-43.
    op_c2: 0xC2 => C(10, 10);  // JNZ, 3-29.
    op_c3: 0xC3 => F(10);      // JMP, 3-28.
    op_c4: 0xC4 => C(11, 17);  // CNZ, 3-15.
    op_c5: 0xC5 => F(11);      // PUSH B, 3-44.
    op_c6: 0xC6 => F(7);       // ADI, 3-5.
    op_c7: 0xC7 => F(11);      // RST 0, 3-55.
    op_c8: 0xC8 => C(5, 11);   // RZ, 3-56.
    op_c9: 0xC9 => F(10);      // RET, 3-48.
    op_ca: 0xCA => C(10, 10);  // JZ, 3-32.
    op_cb: 0xCB => U;
    op_cc: 0xCC => C(11, 17);  // CZ, 3-18.
    op_cd: 0xCD => F(17);      // CALL, 3-9.
    op_ce: 0xCE => F(7);       // ACI, 3-2.
    op_cf: 0xCF => F(11);      // RST 1, 3-55.

    op_d0: 0xD0 => C(5, 11);   // RNC, 3-51.
    op_d1: 0xD1 => F(10);      // POP D, 3-43.
    op_d2: 0xD2 => C(10, 10);  // JNC, 3-28.
    op_d3: 0xD3 => F(10);      // OUT, 3-41.
    op_d4: 0xD4 => C(11, 17);  // CNC, 3-14.
    op_d5: 0xD5 => F(11);      // PUSH D, 3-44.
    op_d6: 0xD6 => F(7);       // SUI, 3-65.
    op_d7: 0xD7 => F(11);      // RST 2, 3-55.
    op_d8: 0xD8 => C(5, 11);   // RC, 3-47.
    op_d9: 0xD9 => U;
    op_da: 0xDA => C(10, 10);  // JC, 3-27.
    op_db: 0xDB => F(10);      // IN, 3-25.
    op_dc: 0xDC => C(11, 17);  // CC, 3-10.
    op_dd: 0xDD => U;
    op_de: 0xDE => F(7);       // SBI, 3-58.
    op_df: 0xDF => F(11);      // RST 3, 3-55.

    op_e0: 0xE0 => C(5, 11);   // RPO, 3-53.
    op_e1: 0xE1 => F(10);      // POP H, 3-43.
    op_e2: 0xE2 => C(10, 10);  // JPO, 3-31.
    op_e3: 0xE3 => F(18);      // XTHL, 3-69.
    op_e4: 0xE4 => C(11, 17);  // CPO, 3-17.
    op_e5: 0xE5 => F(11);      // PUSH H, 3-44.
    op_e6: 0xE6 => F(7);       // ANI, 3-8.
    op_e7: 0xE7 => F(11);      // RST 4, 3-55.
    op_e8: 0xE8 => C(5, 11);   // RPE, 3-52.
    op_e9: 0xE9 => F(5);       // PCHL, 3-42.
    op_ea: 0xEA => C(10, 10);  // JPE, 3-30.
    op_eb: 0xEB => F(4);       // XCHG, 3-65.
    op_ec: 0xEC => C(11, 17);  // CPE, 3-16.
    op_ed: 0xED => U;
    op_ee: 0xEE => F(7);       // XRI, 3-68.
    op_ef: 0xEF => F(11);      // RST 5, 3-55.

    op_f0: 0xF0 => C(5, 11);   // RP, 3-52.
    op_f1: 0xF1 => F(10);      // POP PSW, 3-43.
    op_f2: 0xF2 => C(10, 10);  // JP, 3-30.
    op_f3: 0xF3 => F(4);       // DI, 3-23.
    op_f4: 0xF4 => C(11, 17);  // CP, 3-15.
    op_f5: 0xF5 => F(11);      // PUSH PSW, 3-45.
    op_f6: 0xF6 => F(7);       // ORI, 3-40.
    op_f7: 0xF7 => F(11);      // RST 6, 3-55.
    op_f8: 0xF8 => C(5, 11);   // RM, 3-50.
    op_f9: 0xF9 => F(5);       // SPHL, 3-61.
    op_fa: 0xFA => C(10, 10);  // JM, 3-27.
    op_fb: 0xFB => F(4);       // EI, 3-23.
    op_fc: 0xFC => C(11, 17);  // CM, 3-11.
    op_fd: 0xFD => U;
    op_fe: 0xFE => F(7);       // CPI, 3-17.
    op_ff: 0xFF => F(11);      // RST 7, 3-55.
}

#[test]
fn table_covers_exactly_the_documented_set_and_all_conditional_paths() {
    let mut undocumented = Vec::new();
    let mut conditional = Vec::new();
    for (index, &(opcode, timing)) in TIMINGS.iter().enumerate() {
        assert_eq!(usize::from(opcode), index, "table must be in byte order");
        match timing {
            U => undocumented.push(opcode),
            C(_, _) => conditional.push(opcode),
            F(_) => {}
        }
    }
    assert_eq!(
        undocumented,
        [0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38, 0xCB, 0xD9, 0xDD, 0xED, 0xFD]
    );
    assert_eq!(256 - undocumented.len(), 244);
    assert_eq!(
        conditional,
        [
            0xC0, 0xC2, 0xC4, 0xC8, 0xCA, 0xCC, 0xD0, 0xD2, 0xD4, 0xD8, 0xDA, 0xDC, 0xE0, 0xE2,
            0xE4, 0xE8, 0xEA, 0xEC, 0xF0, 0xF2, 0xF4, 0xF8, 0xFA, 0xFC,
        ]
    );
}

fn fixture(opcode: u8, flags: Flags) -> (I8080, RecordingBus) {
    let cpu = I8080::new(
        Registers {
            a: 0x21,
            b: 0x20,
            c: 0x00,
            d: 0x21,
            e: 0x00,
            h: 0x30,
            l: 0x00,
            flags,
            pc: 0x4000,
            sp: 0x8000,
        },
        Latches::default(),
    );
    let mut bus = RecordingBus::new();
    // Instruction stream, register-indirect data, direct data and stack are
    // disjoint. Address operands name 0x6020; stack returns name 0x7040.
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

// CCC and flag polarity: 98-153B p. 2-20 note 17. The other flags remain
// unrelated to the selected condition. No implementation helper is used.
fn flags_for_path(opcode: u8, taken: bool) -> Flags {
    let mut flags = Flags::default();
    match (opcode >> 3) & 7 {
        0 => flags.zero = !taken,
        1 => flags.zero = taken,
        2 => flags.carry = !taken,
        3 => flags.carry = taken,
        4 => flags.parity = !taken,
        5 => flags.parity = taken,
        6 => flags.sign = !taken,
        7 => flags.sign = taken,
        _ => unreachable!(),
    }
    flags
}

fn witness(opcode: u8, timing: Timing) {
    match timing {
        U => assert_unsupported(opcode, 0x4000),
        F(states) => witness_path(opcode, states, None),
        C(untaken, taken) => {
            witness_path(opcode, untaken, Some(false));
            witness_path(opcode, taken, Some(true));
        }
    }
}

fn witness_path(opcode: u8, states: u8, taken: Option<bool>) {
    let flags = taken.map_or_else(Flags::default, |path| flags_for_path(opcode, path));
    let (mut cpu, mut bus) = fixture(opcode, flags);
    let report = cpu.step(&mut bus);
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: u64::from(states),
            exit: Exit::Budget,
        },
        "opcode {opcode:#04x}, taken={taken:?}"
    );

    // Check that the flags actually selected the requested path, especially
    // for Jcc where both paths cost ten states. Destinations/stack effects:
    // 98-153B pp. 2-18/19 and p. 2-20 notes 12-14, 17.
    if let Some(taken) = taken {
        let (pc, sp) = match (opcode & 7, taken) {
            (0, false) => (0x4001, 0x8000), // Rcc untaken.
            (0, true) => (0x7040, 0x8002),  // Rcc taken.
            (2 | 4, false) => (0x4003, 0x8000),
            (2, true) => (0x6020, 0x8000), // Jcc taken.
            (4, true) => (0x6020, 0x7FFE), // Ccc taken.
            _ => panic!("not a conditional opcode: {opcode:#04x}"),
        };
        assert_eq!(
            (cpu.registers().pc, cpu.registers().sp),
            (pc, sp),
            "opcode {opcode:#04x}, taken={taken}"
        );
    }

    if opcode == 0xDB {
        assert!(bus.inputs.is_empty());
        assert!(bus.log.contains(&Event::Input {
            port: 0x20,
            value: 0x5A,
        }));
    } else if opcode == 0xD3 {
        assert!(bus.log.contains(&Event::Output {
            port: 0x20,
            value: 0x21,
        }));
    } else if opcode == 0x76 {
        assert!(cpu.latches().halted);
        bus.take_log();
        assert_eq!(
            cpu.run(
                &mut bus,
                Budget {
                    steps: u64::MAX,
                    cycles: u64::MAX,
                },
            ),
            Report {
                steps: 0,
                cycles: 0,
                exit: Exit::Halt,
            }
        );
        assert_eq!(cpu.registers().pc, 0x4001);
        assert_eq!(
            bus.take_log(),
            [Event::StopPoll(false), Event::InterruptPoll(false)]
        );
    }
}

#[test]
fn xthl_completes_all_eighteen_states_under_a_smaller_cycle_budget() {
    // 98-153B pp. 2-18/19: 4+3+3+3+5; 9800301-04 p. 3-69: 18.
    // decision-2: every started instruction completes, including its writes.
    let (mut cpu, mut bus) = fixture(0xE3, Flags::default());
    let report = cpu.run(
        &mut bus,
        Budget {
            steps: u64::MAX,
            cycles: 1,
        },
    );
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: 18,
            exit: Exit::Budget
        }
    );
    assert_eq!((cpu.registers().h, cpu.registers().l), (0x70, 0x40));
    assert_eq!(&bus.memory[0x8000..0x8002], &[0x00, 0x30]);
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x4001, 0x8000));
}

#[test]
fn taken_call_overshoots_by_its_full_taken_total() {
    // CZ: 98-153B pp. 2-18/19, note 13 p. 2-20; 9800301-04 p. 3-18.
    // A budget equal to the UNTAKEN total must still allow the taken 17.
    let (mut cpu, mut bus) = fixture(0xCC, flags_for_path(0xCC, true));
    let report = cpu.run(
        &mut bus,
        Budget {
            steps: u64::MAX,
            cycles: 11,
        },
    );
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: 17,
            exit: Exit::Budget
        }
    );
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x6020, 0x7FFE));
    assert_eq!(&bus.memory[0x7FFE..0x8000], &[0x03, 0x40]);
}

#[test]
fn taken_return_overshoots_by_its_full_taken_total() {
    // RZ: 98-153B pp. 2-18/19, note 14 p. 2-20; 9800301-04 p. 3-56.
    let (mut cpu, mut bus) = fixture(0xC8, flags_for_path(0xC8, true));
    let report = cpu.run(
        &mut bus,
        Budget {
            steps: u64::MAX,
            cycles: 5,
        },
    );
    assert_eq!(
        report,
        Report {
            steps: 1,
            cycles: 11,
            exit: Exit::Budget
        }
    );
    assert_eq!((cpu.registers().pc, cpu.registers().sp), (0x7040, 0x8002));
}

#[test]
fn mixed_run_reports_the_literal_sum_including_both_call_paths_and_halt() {
    // 98-153B pp. 2-16/17: MVI r=4+3, INR r=5;
    // pp. 2-18/19 and notes 13/14: CZ=11/17, RZ=11, XTHL=18, HLT=7.
    // Second witness: 9800301-04 pp. 3-37, 3-25, 3-18, 3-56, 3-69, 3-24.
    let (mut cpu, mut bus) = fixture(0x3E, Flags::default());
    bus.load(
        0x4000,
        &[
            0x3E, 0xFF, // MVI A,FF: 7.
            0xCC, 0x00, 0x50, // CZ 5000 untaken (initial Z=0): 11.
            0x3C, // INR A: 5, sets Z.
            0xCC, 0x00, 0x50, // CZ 5000 taken: 17.
            0xE3, // XTHL: 18.
            0x76, // HLT: 7.
        ],
    );
    bus.load(0x5000, &[0xC8]); // RZ taken: 11, returning to XTHL.
    let report = cpu.run(
        &mut bus,
        Budget {
            steps: u64::MAX,
            cycles: u64::MAX,
        },
    );
    assert_eq!(
        report,
        Report {
            steps: 7,
            cycles: 7 + 11 + 5 + 17 + 11 + 18 + 7,
            exit: Exit::Halt,
        }
    );
    assert_eq!(cpu.registers().pc, 0x400B);
    assert_eq!(cpu.registers().sp, 0x8000);
    assert!(cpu.latches().halted);
}
