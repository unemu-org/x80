//! Independent, public-API T-state census (decisions 13–15; x80-bi2.24).
//!
//! Transcription order: Mostek MK78515 V2.0, October 1978 (MK below), then
//! Zilog Product Specification 03-0027-02, March 1978 (PS), then primary
//! UM008011-0816 rev. 11, August 2016 (UM). MK 2-n is PDF n+14; UM p. n is
//! PDF n+14. Each fixture cites the instruction's timing line, not a CPU
//! table. Register/bit fields are expanded here, independently of src/z80's
//! group modules, ENCODINGS, dispatcher tables and opcodes/tests.rs; none of
//! those were read for this transcription. Only the public state/API and
//! cpu.rs's seam contract were consulted.
//!
//! READ: MK timing lines on rendered 150 dpi images, PDF pages
//! 36–37,39–40,42,44–45,47,49,51,54–55,57,59,61,64,66,68,70–71,73,75,
//! 77,79–80,82–83,85–88,90–102,104–105,107,109–111,113–114,116,118,
//! 120,123–129,131,133,135,137–144,146–154,156–159,161–162,164–169,
//! 171,173–178,180,182,184,186,188,190–191,193,195,197–198,200,
//! 202–204,206–208,210–211,213–214,216,219,221,223,225,227,229,231,
//! 233,236,238,240,243,245,247,250–251,253–255,257,259,261,264,267,
//! 270,272. MK Appendix A's opcode listing (PDF 275–289) is an encoding
//! locator, not a timing table; PDF 309–310 contain conversion tables.
//! PS pp. 4–5's instruction summary has no T-state totals. Its summary
//! confirms the instruction families, rather than independently timing them.
//! UM per-instruction M Cycles / T States lines were then read on rendered
//! 150 dpi images at every fixture's cited page (including continuations).
//! OCR located the lines; it supplied no expected values. MK and Zilog share
//! documentation lineage: agreement here is not a hardware measurement.
//!
//! Disagreements: MK LD (nn),HL 2-150 prints 17 despite (4,3,3,3,3);
//! UM p. 107 prints 16. Both re-read on enlarged 200 dpi crops; the primary
//! total is used and pinned separately below (PROVENANCE §1). MK CPIR/CPDR
//! 2-63/2-59 says OR for termination, UM pp. 139/142's timing headings say
//! AND; decision-15 rule 11 and the existing PROVENANCE row resolve this as
//! OR. Both independent termination causes are exercised below.
//!
//! These witnesses isolate timing: they assert successful one-step retirement
//! and cycles, plus PC/counter/stack evidence that each variable path occurred.
//! Full instruction effects and bus traces remain the instruction witnesses'
//! responsibility. No unknown flag is asserted as hardware behaviour here.

#![forbid(unsafe_code)]

mod support;

use std::collections::BTreeSet;

use support::{Event, RecordingBus};
use x80_core::z80::{Flags, InterruptMode, Latches, Registers, Z80};
use x80_core::{Budget, Exit, MemoryAccess, Report};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Table {
    Main,
    Cb,
    Ed,
    Dd,
    Fd,
    DdCb,
    FdCb,
}

#[derive(Clone, Copy, Debug)]
enum Flow {
    Jr,
    Jp,
    Call,
    Ret,
    Djnz,
    Transfer,
    Search,
    Io,
}

#[derive(Debug)]
struct Case {
    table: Table,
    opcode: u8,
    cycles: [u64; 2], // fixed: equal; variable: untaken/terminating, taken/repeating
    flow: Option<Flow>,
    source: &'static str,
}

fn fixed(cases: &mut Vec<Case>, table: Table, keys: &[u8], cycles: u64, source: &'static str) {
    for &opcode in keys {
        cases.push(Case {
            table,
            opcode,
            cycles: [cycles; 2],
            flow: None,
            source,
        });
    }
}

fn variable(
    cases: &mut Vec<Case>,
    keys: &[u8],
    cycles: [u64; 2],
    flow: Flow,
    source: &'static str,
) {
    let table = match flow {
        Flow::Transfer | Flow::Search | Flow::Io => Table::Ed,
        _ => Table::Main,
    };
    for &opcode in keys {
        cases.push(Case {
            table,
            opcode,
            cycles,
            flow: Some(flow),
            source,
        });
    }
}

fn transcription() -> Vec<Case> {
    use Table::{Cb, Dd, DdCb, Ed, Fd, FdCb, Main};
    let mut c = Vec::new();

    // LD r,r': seven printed register fields, excluding memory field 110.
    for dst in [0, 1, 2, 3, 4, 5, 7] {
        for src in [0, 1, 2, 3, 4, 5, 7] {
            fixed(&mut c, Main, &[0x40 + 8 * dst + src], 4, "MK 2-160; UM 71");
        }
    }
    fixed(
        &mut c,
        Main,
        &[0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x7E],
        7,
        "MK 2-154; UM 74",
    );
    fixed(
        &mut c,
        Main,
        &[0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77],
        7,
        "MK 2-135; UM 79",
    );
    fixed(
        &mut c,
        Main,
        &[0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x3E],
        7,
        "MK 2-159; UM 72",
    );
    fixed(&mut c, Main, &[0x36], 10, "MK 2-133; UM 85");
    for (op, states, source) in [
        (0x0A, 7, "MK 2-123; UM 88"),
        (0x1A, 7, "MK 2-124; UM 89"),
        (0x3A, 13, "MK 2-125; UM 90"),
        (0x02, 7, "MK 2-128; UM 91"),
        (0x12, 7, "MK 2-132; UM 92"),
        (0x32, 13, "MK 2-147; UM 93"),
    ] {
        fixed(&mut c, Main, &[op], states, source);
    }
    fixed(
        &mut c,
        Main,
        &[0x01, 0x11, 0x21, 0x31],
        10,
        "MK 2-129; UM 99",
    );
    fixed(&mut c, Main, &[0x2A], 16, "MK 2-134; UM 102");
    fixed(
        &mut c,
        Main,
        &[0x22],
        16,
        "MK 2-150 prints 17 (resolved); UM 107 prints 16",
    );
    fixed(&mut c, Main, &[0xF9], 6, "MK 2-161; UM 112");
    fixed(
        &mut c,
        Main,
        &[0xC5, 0xD5, 0xE5, 0xF5],
        11,
        "MK 2-194; UM 115",
    );
    fixed(
        &mut c,
        Main,
        &[0xC1, 0xD1, 0xE1, 0xF1],
        10,
        "MK 2-190; UM 119",
    );
    for (op, states, source) in [
        (0xEB, 4, "MK 2-78; UM 124"),
        (0x08, 4, "MK 2-77; UM 125"),
        (0xD9, 4, "MK 2-83; UM 126"),
        (0xE3, 19, "MK 2-79; UM 127"),
    ] {
        fixed(&mut c, Main, &[op], states, source);
    }

    // Each source explicitly lists r, n, (HL), (IX+d), (IY+d) separately.
    for (base, immediate, source) in [
        (0x80, 0xC6, "MK 2-25/26/28/30/31; UM 145/147/148/149/150"),
        (0x88, 0xCE, "MK 2-22; UM 152"),
        (0x90, 0xD6, "MK 2-256; UM 154"),
        (0x98, 0xDE, "MK 2-236; UM 156"),
        (0xA0, 0xE6, "MK 2-40; UM 158"),
        (0xA8, 0xEE, "MK 2-258; UM 162"),
        (0xB0, 0xF6, "MK 2-176; UM 160"),
        (0xB8, 0xFE, "MK 2-56; UM 164"),
    ] {
        for r in [0, 1, 2, 3, 4, 5, 7] {
            fixed(&mut c, Main, &[base + r], 4, source);
        }
        fixed(&mut c, Main, &[base + 6, immediate], 7, source);
        fixed(&mut c, Dd, &[base + 6], 19, source);
        fixed(&mut c, Fd, &[base + 6], 19, source);
    }
    fixed(
        &mut c,
        Main,
        &[0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x3C],
        4,
        "MK 2-97; UM 165",
    );
    fixed(&mut c, Main, &[0x34], 11, "MK 2-90; UM 167");
    fixed(
        &mut c,
        Main,
        &[0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x3D],
        4,
        "MK 2-71; UM 171",
    );
    fixed(&mut c, Main, &[0x35], 11, "MK 2-71; UM 171");
    for (op, source) in [
        (0x27, "MK 2-66; UM 174"),
        (0x2F, "MK 2-65; UM 175"),
        (0x3F, "MK 2-54; UM 178"),
        (0x37, "MK 2-239; UM 179"),
        (0x00, "MK 2-174; UM 180"),
        (0x76, "MK 2-82; UM 181"),
        (0xF3, "MK 2-73; UM 182"),
        (0xFB, "MK 2-76; UM 183"),
    ] {
        fixed(&mut c, Main, &[op], 4, source);
    }
    fixed(
        &mut c,
        Main,
        &[0x09, 0x19, 0x29, 0x39],
        11,
        "MK 2-33; UM 188",
    );
    fixed(
        &mut c,
        Main,
        &[0x03, 0x13, 0x23, 0x33],
        6,
        "MK 2-99; UM 198",
    );
    fixed(
        &mut c,
        Main,
        &[0x0B, 0x1B, 0x2B, 0x3B],
        6,
        "MK 2-72; UM 201",
    );
    for (op, source) in [
        (0x07, "MK 2-209; UM 205"),
        (0x17, "MK 2-207; UM 207"),
        (0x0F, "MK 2-226; UM 209"),
        (0x1F, "MK 2-224; UM 211"),
    ] {
        fixed(&mut c, Main, &[op], 4, source);
    }
    fixed(&mut c, Main, &[0xC3], 10, "MK 2-113; UM 262");
    variable(
        &mut c,
        &[0xC2, 0xCA, 0xD2, 0xDA, 0xE2, 0xEA, 0xF2, 0xFA],
        [10, 10],
        Flow::Jp,
        "MK 2-109; UM 264",
    );
    fixed(&mut c, Main, &[0x18], 12, "MK 2-114; UM 265");
    for (op, source) in [
        (0x38, "MK 2-115; UM 267"),
        (0x30, "MK 2-117; UM 269"),
        (0x20, "MK 2-119; UM 271"),
        (0x28, "MK 2-121; UM 273"),
    ] {
        variable(&mut c, &[op], [7, 12], Flow::Jr, source);
    }
    fixed(&mut c, Main, &[0xE9], 4, "MK 2-110; UM 275");
    variable(&mut c, &[0x10], [8, 13], Flow::Djnz, "MK 2-74; UM 278");
    fixed(&mut c, Main, &[0xCD], 17, "MK 2-52; UM 281");
    variable(
        &mut c,
        &[0xC4, 0xCC, 0xD4, 0xDC, 0xE4, 0xEC, 0xF4, 0xFC],
        [10, 17],
        Flow::Call,
        "MK 2-50; UM 284",
    );
    fixed(&mut c, Main, &[0xC9], 10, "MK 2-197; UM 285");
    variable(
        &mut c,
        &[0xC0, 0xC8, 0xD0, 0xD8, 0xE0, 0xE8, 0xF0, 0xF8],
        [5, 11],
        Flow::Ret,
        "MK 2-199; UM 287",
    );
    fixed(
        &mut c,
        Main,
        &[0xC7, 0xCF, 0xD7, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF],
        11,
        "MK 2-233; UM 293",
    );
    fixed(&mut c, Main, &[0xDB], 11, "MK 2-87; UM 295");
    fixed(&mut c, Main, &[0xD3], 11, "MK 2-183; UM 306");

    for (op, source) in [
        (0x57, "MK 2-126; UM 94"),
        (0x5F, "MK 2-127; UM 95"),
        (0x47, "MK 2-136; UM 96"),
        (0x4F, "MK 2-153; UM 97"),
    ] {
        fixed(&mut c, Ed, &[op], 9, source);
    }
    fixed(
        &mut c,
        Ed,
        &[0x4B, 0x5B, 0x6B, 0x7B],
        20,
        "MK 2-130; UM 103",
    );
    fixed(
        &mut c,
        Ed,
        &[0x43, 0x53, 0x63, 0x73],
        20,
        "MK 2-148; UM 108",
    );
    fixed(&mut c, Ed, &[0xA0], 16, "MK 2-168; UM 130");
    fixed(&mut c, Ed, &[0xA8], 16, "MK 2-164; UM 134");
    fixed(&mut c, Ed, &[0xA1], 16, "MK 2-61; UM 138");
    fixed(&mut c, Ed, &[0xA9], 16, "MK 2-57; UM 141");
    variable(
        &mut c,
        &[0xB0],
        [16, 21],
        Flow::Transfer,
        "MK 2-170; UM 132",
    );
    variable(
        &mut c,
        &[0xB8],
        [16, 21],
        Flow::Transfer,
        "MK 2-166; UM 136",
    );
    variable(
        &mut c,
        &[0xB1],
        [16, 21],
        Flow::Search,
        "MK 2-63; UM 139 (OR resolution)",
    );
    variable(
        &mut c,
        &[0xB9],
        [16, 21],
        Flow::Search,
        "MK 2-59; UM 142 (OR resolution)",
    );
    fixed(&mut c, Ed, &[0x44], 8, "MK 2-172; UM 176");
    fixed(
        &mut c,
        Ed,
        &[0x46, 0x56, 0x5E],
        8,
        "MK 2-84/85/86; UM 184/185/186",
    );
    fixed(&mut c, Ed, &[0x4A, 0x5A, 0x6A, 0x7A], 15, "MK 2-23; UM 190");
    fixed(
        &mut c,
        Ed,
        &[0x42, 0x52, 0x62, 0x72],
        15,
        "MK 2-237; UM 192",
    );
    fixed(&mut c, Ed, &[0x6F], 18, "MK 2-219; UM 238");
    fixed(&mut c, Ed, &[0x67], 18, "MK 2-231; UM 240");
    fixed(&mut c, Ed, &[0x4D], 14, "MK 2-200; UM 288");
    fixed(&mut c, Ed, &[0x45], 14, "MK 2-202; UM 290");
    fixed(
        &mut c,
        Ed,
        &[0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x78],
        12,
        "MK 2-88; UM 296",
    );
    fixed(
        &mut c,
        Ed,
        &[0x41, 0x49, 0x51, 0x59, 0x61, 0x69, 0x79],
        12,
        "MK 2-181; UM 307",
    );
    // Decision-20 §1: A ED70 total (TM Table 7.0-11 p. 54);
    // L ED71/alias/NOP totals (Young v0.91 §3.4 pp. 12–13,
    // §§8.5/8.10/8.11 pp. 30/34/35; UM pp. 176/184–186/288/290).
    fixed(
        &mut c,
        Ed,
        &[0x70, 0x71],
        12,
        "decision-20 §1; TM 54 / Young 35",
    );
    fixed(
        &mut c,
        Ed,
        &[
            0x4C, 0x54, 0x5C, 0x64, 0x6C, 0x74, 0x7C, 0x4E, 0x66, 0x6E, 0x76, 0x7E,
        ],
        8,
        "L decision-20 §1; Young 12–13",
    );
    fixed(
        &mut c,
        Ed,
        &[0x55, 0x5D, 0x65, 0x6D, 0x75, 0x7D],
        14,
        "L decision-20 §1; Young 34",
    );
    let holes: Vec<_> = (0x00..=0x3F)
        .chain([0x77, 0x7F])
        .chain(0x80..=0x9F)
        .chain(0xA4..=0xA7)
        .chain(0xAC..=0xAF)
        .chain(0xB4..=0xB7)
        .chain(0xBC..=0xFF)
        .collect();
    fixed(&mut c, Ed, &holes, 8, "L decision-20 §1; Young 12–13");
    for (op, source) in [
        (0xA2, "MK 2-104; UM 298"),
        (0xAA, "MK 2-100; UM 302"),
        (0xA3, "MK 2-186; UM 309"),
        (0xAB, "MK 2-184; UM 313"),
    ] {
        fixed(&mut c, Ed, &[op], 16, source);
    }
    for (op, source) in [
        (0xB2, "MK 2-106; UM 300"),
        (0xBA, "MK 2-102; UM 304"),
        (0xB3, "MK 2-179; UM 311/312"),
        (0xBB, "MK 2-177; UM 315/316"),
    ] {
        variable(&mut c, &[op], [16, 21], Flow::Io, source);
    }

    // IX and IY are separate census entries, with their own source pages.
    for (
        table,
        load,
        store,
        immediate,
        load_mem,
        store_mem,
        mem_n,
        inc,
        dec,
        inc_mem,
        add,
        exchange,
        pop,
        push,
        jp,
        sp,
    ) in [
        (
            Dd,
            "MK 2-137; UM 105",
            "MK 2-151; UM 110",
            "MK 2-138; UM 100",
            "MK 2-155; UM 75",
            "MK 2-140; UM 81",
            "MK 2-139; UM 86",
            "MK 2-95; UM 199",
            "MK 2-68; UM 202",
            "MK 2-91; UM 168",
            "MK 2-35; UM 194",
            "MK 2-80; UM 128",
            "MK 2-188; UM 121",
            "MK 2-192; UM 117",
            "MK 2-111; UM 276",
            "MK 2-162; UM 113",
        ),
        (
            Fd,
            "MK 2-142; UM 106",
            "MK 2-152; UM 111",
            "MK 2-143; UM 101",
            "MK 2-157; UM 77",
            "MK 2-145; UM 83",
            "MK 2-144; UM 87",
            "MK 2-96; UM 200",
            "MK 2-69; UM 203",
            "MK 2-93; UM 169",
            "MK 2-37; UM 196",
            "MK 2-81; UM 129",
            "MK 2-189; UM 122",
            "MK 2-193; UM 118",
            "MK 2-112; UM 277",
            "MK 2-163; UM 114",
        ),
    ] {
        fixed(&mut c, table, &[0x2A], 20, load);
        fixed(&mut c, table, &[0x22], 20, store);
        fixed(&mut c, table, &[0x21], 14, immediate);
        fixed(
            &mut c,
            table,
            &[0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x7E],
            19,
            load_mem,
        );
        fixed(
            &mut c,
            table,
            &[0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77],
            19,
            store_mem,
        );
        fixed(&mut c, table, &[0x36], 19, mem_n);
        fixed(&mut c, table, &[0x23], 10, inc);
        fixed(&mut c, table, &[0x2B], 10, dec);
        fixed(&mut c, table, &[0x34], 23, inc_mem);
        fixed(&mut c, table, &[0x35], 23, "MK 2-71; UM 171");
        fixed(&mut c, table, &[0x09, 0x19, 0x29, 0x39], 15, add);
        fixed(&mut c, table, &[0xE3], 23, exchange);
        fixed(&mut c, table, &[0xE1], 14, pop);
        fixed(&mut c, table, &[0xE5], 15, push);
        fixed(&mut c, table, &[0xE9], 8, jp);
        fixed(&mut c, table, &[0xF9], 10, sp);
    }

    // All seven documented rotate/shift families; no CB 30–37 (decision-14).
    for (base, reg, mem, ix, iy) in [
        (
            0x00,
            "MK 2-217; UM 213",
            "MK 2-211; UM 215",
            "MK 2-213; UM 217",
            "MK 2-215; UM 219",
        ),
        (
            0x08,
            "MK 2-229; UM 226",
            "MK 2-229; UM 226",
            "MK 2-229; UM 226",
            "MK 2-229; UM 226",
        ),
        (
            0x10,
            "MK 2-205; UM 222",
            "MK 2-205; UM 222",
            "MK 2-205; UM 222",
            "MK 2-205; UM 222",
        ),
        (
            0x18,
            "MK 2-222; UM 229",
            "MK 2-222; UM 229",
            "MK 2-222; UM 229",
            "MK 2-222; UM 229",
        ),
        (
            0x20,
            "MK 2-247; UM 231",
            "MK 2-247; UM 231",
            "MK 2-247; UM 231",
            "MK 2-247; UM 231",
        ),
        (
            0x28,
            "MK 2-250; UM 234",
            "MK 2-250; UM 234",
            "MK 2-250; UM 234",
            "MK 2-250; UM 234",
        ),
        (
            0x38,
            "MK 2-253; UM 237",
            "MK 2-253; UM 237",
            "MK 2-253; UM 237",
            "MK 2-253; UM 237",
        ),
    ] {
        for r in [0, 1, 2, 3, 4, 5, 7] {
            fixed(&mut c, Cb, &[base + r], 8, reg);
        }
        fixed(&mut c, Cb, &[base + 6], 15, mem);
        fixed(&mut c, DdCb, &[base + 6], 23, ix);
        fixed(&mut c, FdCb, &[base + 6], 23, iy);
    }
    for bit in 0..8 {
        for r in [0, 1, 2, 3, 4, 5, 7] {
            fixed(&mut c, Cb, &[0x40 + 8 * bit + r], 8, "MK 2-47; UM 243");
            fixed(&mut c, Cb, &[0x80 + 8 * bit + r], 8, "MK 2-196; UM 260");
            fixed(&mut c, Cb, &[0xC0 + 8 * bit + r], 8, "MK 2-245; UM 251");
        }
        fixed(&mut c, Cb, &[0x46 + 8 * bit], 12, "MK 2-41; UM 245");
        fixed(&mut c, Cb, &[0x86 + 8 * bit], 15, "MK 2-196; UM 260");
        fixed(&mut c, Cb, &[0xC6 + 8 * bit], 15, "MK 2-240; UM 253");
        fixed(&mut c, DdCb, &[0x46 + 8 * bit], 20, "MK 2-43; UM 247");
        fixed(&mut c, FdCb, &[0x46 + 8 * bit], 20, "MK 2-45; UM 249");
        fixed(&mut c, DdCb, &[0x86 + 8 * bit], 23, "MK 2-196; UM 260");
        fixed(&mut c, FdCb, &[0x86 + 8 * bit], 23, "MK 2-196; UM 260");
        fixed(&mut c, DdCb, &[0xC6 + 8 * bit], 23, "MK 2-241; UM 255");
        fixed(&mut c, FdCb, &[0xC6 + 8 * bit], 23, "MK 2-243; UM 257");
    }
    c
}

fn fixture() -> (Z80, RecordingBus) {
    let cpu = Z80::new(
        Registers {
            a: 0x55,
            b: 2,
            c: 1,
            d: 0x60,
            e: 0,
            h: 0x40,
            l: 0,
            ix: 0x7000,
            iy: 0x8000,
            sp: 0x9000,
            pc: 0x1000,
            ..Registers::default()
        },
        Latches::default(),
    );
    let mut bus = RecordingBus::new();
    bus.load(0x4000, &[0x11, 0x22, 0x33]);
    bus.load(0x9000, &[0x00, 0x50]);
    bus.inputs.push_back(0x33);
    (cpu, bus)
}

fn bytes(table: Table, opcode: u8) -> Vec<u8> {
    match table {
        Table::Main => vec![opcode, 0x08, 0x40],
        Table::Cb => vec![0xCB, opcode],
        Table::Ed => vec![0xED, opcode, 0x08, 0x40],
        Table::Dd => vec![0xDD, opcode, 0x08, 0x40],
        Table::Fd => vec![0xFD, opcode, 0x08, 0x40],
        Table::DdCb => vec![0xDD, 0xCB, 0x08, opcode],
        Table::FdCb => vec![0xFD, 0xCB, 0x08, opcode],
    }
}

// Literal condition meanings: MK 2-50 and PS p. 4; UM JP cc p. 263,
// RET cc p. 286. In particular CALL C uses C, not p. 284's mislabeled Z.
fn condition(flags: &mut Flags, cc: u8, taken: bool) {
    let bit = if cc & 1 == 0 { !taken } else { taken };
    match cc / 2 {
        0 => flags.zero = bit,
        1 => flags.carry = bit,
        2 => flags.parity_overflow = bit,
        3 => flags.sign = bit,
        _ => unreachable!("three-bit condition field"),
    }
}

fn expected_report(cycles: u64) -> Report {
    Report {
        steps: 1,
        cycles,
        exit: Exit::Budget,
    }
}

fn witness(case: &Case, taken: bool, early_match: bool) -> Vec<String> {
    let (mut cpu, mut bus) = fixture();
    bus.load(0x1000, &bytes(case.table, case.opcode));
    let mut expected_pc = None;
    let mut expected_sp = None;
    let mut expected_bc = None;
    let mut expected_b = None;
    match case.flow {
        Some(Flow::Jr) => {
            condition(
                &mut cpu.registers_mut().flags,
                (case.opcode >> 3) & 3,
                taken,
            );
            expected_pc = Some(if taken { 0x100A } else { 0x1002 });
        }
        Some(flow @ (Flow::Jp | Flow::Call | Flow::Ret)) => {
            condition(
                &mut cpu.registers_mut().flags,
                (case.opcode >> 3) & 7,
                taken,
            );
            expected_pc = Some(match (flow, taken) {
                (Flow::Ret, true) => 0x5000,
                (Flow::Ret, false) => 0x1001,
                (_, true) => 0x4008,
                (_, false) => 0x1003,
            });
            expected_sp = Some(match (flow, taken) {
                (Flow::Call, true) => 0x8FFE,
                (Flow::Ret, true) => 0x9002,
                _ => 0x9000,
            });
        }
        Some(Flow::Djnz) => {
            cpu.registers_mut().b = if taken { 2 } else { 1 };
            expected_b = Some(if taken { 1 } else { 0 });
            expected_pc = Some(if taken { 0x100A } else { 0x1002 });
        }
        Some(Flow::Transfer | Flow::Search) => {
            // Count exhaustion without match, or count remaining without match.
            // The separate early_match call terminates with count remaining.
            cpu.registers_mut()
                .set_bc(if taken || early_match { 2 } else { 1 });
            if early_match {
                cpu.registers_mut().a = 0x11;
            }
            expected_bc = Some(if taken || early_match { 1 } else { 0 });
            expected_pc = Some(if taken { 0x1000 } else { 0x1002 });
        }
        Some(Flow::Io) => {
            cpu.registers_mut().b = if taken { 2 } else { 1 };
            expected_b = Some(if taken { 1 } else { 0 });
            expected_pc = Some(if taken { 0x1000 } else { 0x1002 });
        }
        None => {}
    }
    let report = cpu.step(&mut bus);
    let mut errors = Vec::new();
    let expected = expected_report(case.cycles[usize::from(taken)]);
    if report != expected {
        errors.push(format!("{:?} {:02X}, taken/repeat={taken}, early_match={early_match}: expected {expected:?}, got {report:?}; {}", case.table, case.opcode, case.source));
    }
    for (label, actual, wanted) in [
        ("PC", cpu.registers().pc, expected_pc),
        ("SP", cpu.registers().sp, expected_sp),
        ("BC", cpu.registers().bc(), expected_bc),
        ("B", u16::from(cpu.registers().b), expected_b),
    ] {
        if let Some(wanted) = wanted {
            if actual != wanted {
                errors.push(format!("{:?} {:02X}, taken/repeat={taken}, early_match={early_match}: path evidence {label}: expected {wanted:04X}, got {actual:04X}; {}", case.table, case.opcode, case.source));
            }
        }
    }
    errors
}

fn check_table(table: Table) {
    let mut errors = Vec::new();
    for case in transcription().iter().filter(|case| case.table == table) {
        errors.extend(witness(case, false, false));
        if case.flow.is_some() {
            errors.extend(witness(case, true, false));
        }
        if matches!(case.flow, Some(Flow::Search)) {
            errors.extend(witness(case, false, true));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn independent_census_has_all_896_unique_admitted_encodings() {
    let cases = transcription();
    let keys: BTreeSet<_> = cases.iter().map(|c| (c.table, c.opcode)).collect();
    // Decision-20 §1 admits 198 additional ED terminals.
    assert_eq!(cases.len(), 896);
    assert_eq!(keys.len(), 896, "duplicate transcription entry");
    for (table, count) in [
        (Table::Main, 252),
        (Table::Cb, 248),
        (Table::Ed, 256),
        (Table::Dd, 39),
        (Table::Fd, 39),
        (Table::DdCb, 31),
        (Table::FdCb, 31),
    ] {
        assert_eq!(
            keys.iter().filter(|(t, _)| *t == table).count(),
            count,
            "{table:?}"
        );
    }
    // A literal complement check guards the full main/CB sets, independently
    // of the decoder and of the family-expansion loops above (decision-14).
    for op in 0..=255 {
        assert_eq!(
            keys.contains(&(Table::Main, op)),
            ![0xCB, 0xDD, 0xED, 0xFD].contains(&op)
        );
        assert_eq!(
            keys.contains(&(Table::Cb, op)),
            !(0x30..=0x37).contains(&op)
        );
    }
    for table in [Table::Dd, Table::Fd] {
        let expected: BTreeSet<_> = [
            0x09, 0x19, 0x21, 0x22, 0x23, 0x29, 0x2A, 0x2B, 0x34, 0x35, 0x36, 0x39, 0x46, 0x4E,
            0x56, 0x5E, 0x66, 0x6E, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x77, 0x7E, 0x86, 0x8E,
            0x96, 0x9E, 0xA6, 0xAE, 0xB6, 0xBE, 0xE1, 0xE3, 0xE5, 0xE9, 0xF9,
        ]
        .into_iter()
        .collect();
        assert_eq!(
            keys.iter()
                .filter_map(|&(t, op)| (t == table).then_some(op))
                .collect::<BTreeSet<_>>(),
            expected
        );
    }
    // Decision-20 §1: the full ED terminal set, including CB/DD/ED/FD.
    assert_eq!(
        keys.iter()
            .filter_map(|&(t, op)| (t == Table::Ed).then_some(op))
            .collect::<BTreeSet<_>>(),
        (0..=255).collect()
    );
    for table in [Table::DdCb, Table::FdCb] {
        let expected: BTreeSet<_> = [
            0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x3E, 0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x76,
            0x7E, 0x86, 0x8E, 0x96, 0x9E, 0xA6, 0xAE, 0xB6, 0xBE, 0xC6, 0xCE, 0xD6, 0xDE, 0xE6,
            0xEE, 0xF6, 0xFE,
        ]
        .into_iter()
        .collect();
        assert_eq!(
            keys.iter()
                .filter_map(|&(t, op)| (t == table).then_some(op))
                .collect::<BTreeSet<_>>(),
            expected
        );
    }
}

#[test]
fn main_all_252_encodings_and_both_conditional_paths() {
    check_table(Table::Main);
}
#[test]
fn cb_all_248_encodings() {
    check_table(Table::Cb);
}
#[test]
fn ed_all_256_encodings_and_all_repeat_termination_paths() {
    check_table(Table::Ed);
}
#[test]
fn dd_all_39_encodings() {
    check_table(Table::Dd);
}
#[test]
fn fd_all_39_encodings() {
    check_table(Table::Fd);
}
#[test]
fn dd_cb_all_31_encodings() {
    check_table(Table::DdCb);
}
#[test]
fn fd_cb_all_31_encodings() {
    check_table(Table::FdCb);
}

#[test]
fn ld_nn_hl_resolves_mostek_17_against_primary_16() {
    // MK V2.0 2-150/PDF 164: 17 (4,3,3,3,3).
    // UM rev. 11 p. 107/PDF 121: 16 (4,3,3,3,3).
    // READ: both enlarged 200 dpi crops. Primary precedence (PROVENANCE §1).
    let mostek_printed_total = 17;
    let primary_printed_total = 16;
    let mostek_printed_components = [4, 3, 3, 3, 3];
    assert_ne!(mostek_printed_total, primary_printed_total);
    assert_eq!(
        mostek_printed_components.iter().sum::<u64>(),
        primary_printed_total
    );
    let (mut cpu, mut bus) = fixture();
    bus.load(0x1000, &[0x22, 0x08, 0x40]);
    assert_eq!(cpu.step(&mut bus), expected_report(16));
    assert_eq!(cpu.registers().pc, 0x1003);
    assert_eq!(&bus.memory[0x4008..0x400A], &[0x00, 0x40]);
}

fn interrupt_trace(nmi: bool, acknowledge: u8, vector: Option<u16>) -> Vec<Event> {
    let mut events = vec![
        Event::StopPoll(false),
        Event::NmiPoll(nmi),
        Event::IntPoll(!nmi),
        Event::Read {
            address: 0x1000,
            kind: if nmi {
                MemoryAccess::OpcodeFetch
            } else {
                MemoryAccess::InterruptAcknowledge
            },
            value: if nmi { 0 } else { acknowledge },
        },
        Event::Write {
            address: 0x8FFF,
            kind: MemoryAccess::Stack,
            value: 0x10,
        },
        Event::Write {
            address: 0x8FFE,
            kind: MemoryAccess::Stack,
            value: 0x00,
        },
    ];
    if let Some(address) = vector {
        events.extend([
            Event::Read {
                address,
                kind: MemoryAccess::Data,
                value: 0x34,
            },
            Event::Read {
                address: address + 1,
                kind: MemoryAccess::Data,
                value: 0x12,
            },
        ]);
    }
    events
}

#[test]
fn nmi_acceptance_is_derived_eleven_states() {
    // decision-15 rule 8; UM CPU Response p. 19 (PDF 33) restart equivalence,
    // RST p. 293 (PDF 307) 11; independently MK RST 2-233/PDF 247 gives 11.
    let (mut cpu, mut bus) = fixture();
    bus.nmi = true; // through Inputs::nmi(), not an executor call
    assert_eq!(cpu.step(&mut bus), expected_report(11));
    assert_eq!(cpu.registers().pc, 0x0066);
    assert_eq!(cpu.registers().sp, 0x8FFE);
    assert_eq!(bus.take_log(), interrupt_trace(true, 0, None));
}

#[test]
fn im0_all_eight_rst_acceptances_are_derived_thirteen_states() {
    // decision-15 rule 9; UM p. 19: normal restart plus two clocks;
    // UM RST p. 293 and MK 2-233: 11. Derived acceptance total = 13.
    for (acknowledge, target) in [
        (0xC7, 0x00),
        (0xCF, 0x08),
        (0xD7, 0x10),
        (0xDF, 0x18),
        (0xE7, 0x20),
        (0xEF, 0x28),
        (0xF7, 0x30),
        (0xFF, 0x38),
    ] {
        let (mut cpu, mut bus) = fixture();
        cpu.latches_mut().iff1 = true;
        cpu.latches_mut().interrupt_mode = InterruptMode::Mode0;
        bus.int = true;
        bus.acknowledges.push_back(acknowledge);
        assert_eq!(cpu.step(&mut bus), expected_report(13), "RST {target:02X}");
        assert_eq!(cpu.registers().pc, target);
        assert_eq!(bus.take_log(), interrupt_trace(false, acknowledge, None));
    }
}

#[test]
fn im1_acceptance_is_derived_thirteen_states() {
    // Same restart-plus-two derivation; UM p. 19 explicitly describes IM 1.
    let (mut cpu, mut bus) = fixture();
    cpu.latches_mut().iff1 = true;
    cpu.latches_mut().interrupt_mode = InterruptMode::Mode1;
    bus.int = true;
    bus.acknowledges.push_back(0x00);
    assert_eq!(cpu.step(&mut bus), expected_report(13));
    assert_eq!(cpu.registers().pc, 0x0038);
    assert_eq!(bus.take_log(), interrupt_trace(false, 0x00, None));
}

#[test]
fn im2_acceptance_is_stated_nineteen_states() {
    // decision-15 rule 9; UM CPU Response/Mode 2 p. 20/PDF 34 explicitly
    // states 19 clocks: 7 acknowledge + 6 save PC + 6 table lookup.
    let (mut cpu, mut bus) = fixture();
    cpu.registers_mut().i = 0xA0;
    cpu.latches_mut().iff1 = true;
    cpu.latches_mut().interrupt_mode = InterruptMode::Mode2;
    bus.int = true;
    bus.acknowledges.push_back(0x10);
    bus.load(0xA010, &[0x34, 0x12]);
    assert_eq!(cpu.step(&mut bus), expected_report(19));
    assert_eq!(cpu.registers().pc, 0x1234);
    assert_eq!(bus.take_log(), interrupt_trace(false, 0x10, Some(0xA010)));
}

#[test]
fn cycle_budget_splits_a_repeating_block_and_resumes_with_literal_sum() {
    // decision-15 rule 11 / decision-2: one iteration per step; whole-step
    // overshoot. MK LDIR 2-170/PDF 184 and UM p. 132/PDF 146: 21 repeat,
    // 16 final. Three transfers split after two: 42 + 16 = 58, not 63.
    let (mut cpu, mut bus) = fixture();
    cpu.registers_mut().set_bc(3);
    bus.load(0x1000, &[0xED, 0xB0]);
    let first = cpu.run(
        &mut bus,
        Budget {
            steps: 99,
            cycles: 22,
        },
    );
    assert_eq!(
        first,
        Report {
            steps: 2,
            cycles: 42,
            exit: Exit::Budget
        }
    );
    assert_eq!((cpu.registers().pc, cpu.registers().bc()), (0x1000, 1));
    assert_eq!(
        (cpu.registers().hl(), cpu.registers().de()),
        (0x4002, 0x6002)
    );
    assert_eq!(
        bus.log
            .iter()
            .filter(|event| matches!(event, Event::StopPoll(_)))
            .count(),
        2
    );
    let second = cpu.run(
        &mut bus,
        Budget {
            steps: 99,
            cycles: 1,
        },
    );
    assert_eq!(second, expected_report(16));
    assert_eq!(first.cycles + second.cycles, 58);
    assert_eq!((cpu.registers().pc, cpu.registers().bc()), (0x1002, 0));
    assert_eq!(&bus.memory[0x6000..0x6003], &[0x11, 0x22, 0x33]);
    assert_eq!(cpu.registers().r, 6);
    assert_eq!(
        bus.log
            .iter()
            .filter(|event| matches!(event, Event::StopPoll(_)))
            .count(),
        3
    );
}

#[test]
fn cycle_budget_overshoots_by_a_complete_indexed_instruction() {
    // decision-2: an instruction started below budget completes. MK EX
    // (SP),IX 2-80/PDF 94; UM p. 128/PDF 142: 23; next NOP is not started.
    let (mut cpu, mut bus) = fixture();
    bus.load(0x1000, &[0xDD, 0xE3, 0x00]);
    assert_eq!(
        cpu.run(
            &mut bus,
            Budget {
                steps: 99,
                cycles: 1
            }
        ),
        expected_report(23)
    );
    assert_eq!(cpu.registers().pc, 0x1002);
    assert_eq!(cpu.registers().ix, 0x5000);
    assert_eq!(
        bus.log
            .iter()
            .filter(|event| matches!(event, Event::StopPoll(_)))
            .count(),
        1
    );
    assert_eq!(cpu.step(&mut bus), expected_report(4));
    assert_eq!(cpu.registers().pc, 0x1003);
}
