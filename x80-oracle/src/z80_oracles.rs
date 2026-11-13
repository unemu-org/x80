//! Digest-selected Z80 oracle protocols; no guest CPU semantics live here.
//!
//! PROVENANCE §2, Z80 binary/source audit, pins the offsets and exclusions.
//! Only fingerprints and structural facts are stored here; GPL programs and
//! source stay fetch-only. The caller must identify the original digest first.

use std::io;

use crate::fingerprint;

pub(crate) const NOT_RUN: [&str; 9] = [
    "alu8rx", "incxh", "incxl", "incyh", "incyl", "ld8ixy", "ld8rrx", "rotxy", "rotz80",
];
// Zero-based table positions, in source order (zexdoc.z80:100–168).
const EXCLUDED: [usize; 9] = [6, 28, 29, 30, 31, 48, 50, 59, 60];
const TABLE: usize = 0x3a; // File offset; guest address 013Ah.
const GROUPS: usize = 67;
const JUDGED: usize = GROUPS - EXCLUDED.len();

pub(crate) fn patch_table(image: &mut [u8]) -> io::Result<()> {
    let mismatch = || io::Error::new(io::ErrorKind::InvalidData, "ZEXDOC pointer table mismatch");
    let table = image
        .get_mut(TABLE..TABLE + (GROUPS + 1) * 2)
        .ok_or_else(mismatch)?;
    // Verify the ENTIRE original table and terminator before the first write.
    // Descriptor guest addresses: 01C2h + n*0060h (PROVENANCE §2 audit).
    for index in 0..=GROUPS {
        let expected = if index == GROUPS {
            0
        } else {
            0x1c2 + index as u16 * 0x60
        };
        if table[index * 2..index * 2 + 2] != expected.to_le_bytes() {
            return Err(mismatch());
        }
    }
    let mut retained = Vec::with_capacity(JUDGED * 2);
    for (index, pointer) in table.as_chunks::<2>().0.iter().take(GROUPS).enumerate() {
        if !EXCLUDED.contains(&index) {
            retained.extend_from_slice(pointer);
        }
    }
    table.fill(0);
    table[..retained.len()].copy_from_slice(&retained);
    Ok(())
}

pub(crate) fn validate_preliminary(output: &[u8]) -> Result<(), String> {
    // prelim.z80:269–274 prints exactly one completion string, with no CR/LF.
    // Errors at 278–312 print a four-digit address and CR/LF; early failures
    // may be silent. A success substring or extra line must never pass.
    if fingerprint(output) == (26, 0x12e9649063313a51) {
        Ok(())
    } else {
        Err("invalid or missing Z80 preliminary completion message".into())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Grammar {
    pub(crate) banner: (usize, u64),
    pub(crate) completion: (usize, u64),
    pub(crate) ok: (usize, u64),
    pub(crate) error: (usize, u64),
}

// zexdoc.z80:1205–1210; strip only the emitted CR/LF line separators.
const GRAMMAR: Grammar = Grammar {
    banner: (25, 0x431656fb9ea34c41),
    completion: (14, 0x948df6ea30f0dd7d),
    ok: (4, 0x37a17a74393dbe97),
    error: (26, 0xb3a627fe7ddd2ec7),
};

pub(crate) fn validate_exerciser(output: &[u8]) -> Result<(), String> {
    let descriptors: Vec<_> = DESCRIPTORS
        .iter()
        .enumerate()
        .filter(|(index, _)| !EXCLUDED.contains(index))
        .map(|(_, &id)| id)
        .collect();
    validate_groups(output, &descriptors, GRAMMAR)
}

pub(crate) fn validate_groups(
    output: &[u8],
    descriptors: &[(usize, u64)],
    grammar: Grammar,
) -> Result<(), String> {
    // zexdoc.z80:746–810 prints the descriptor, then OK or expected/found
    // CRCs. OK is emitted only after cmpcrc (1215–1232) matches all four
    // expected bytes. Unlike 8080EXM, ZEXDOC does not print a CRC on success.
    let text = std::str::from_utf8(output).map_err(|_| "invalid UTF-8 in ZEXDOC output")?;
    let mut lines = text.split(['\r', '\n']).filter(|line| !line.is_empty());
    if lines.next().map(|line| fingerprint(line.as_bytes())) != Some(grammar.banner) {
        return Err("missing ZEXDOC banner".into());
    }
    for (index, expected) in descriptors.iter().enumerate() {
        let line = lines
            .next()
            .ok_or_else(|| format!("missing ZEXDOC group {}", index + 1))?;
        let (descriptor, result) = line
            .split_at_checked(expected.0)
            .ok_or_else(|| format!("incomplete ZEXDOC group {}", index + 1))?;
        if fingerprint(descriptor.as_bytes()) != *expected {
            return Err(format!(
                "unknown, missing, duplicated or reordered ZEXDOC group {}",
                index + 1
            ));
        }
        if fingerprint(result.as_bytes()) == grammar.ok {
            continue;
        }
        let crc = |s: &str| {
            s.len() == 8
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };
        if let Some((label, rest)) = result.split_at_checked(grammar.error.0) {
            if fingerprint(label.as_bytes()) == grammar.error
                && rest
                    .split_once(" found:")
                    .is_some_and(|(expected, found)| crc(expected) && crc(found))
            {
                return Err(format!("judged ZEXDOC group {} reports ERROR", index + 1));
            }
        }
        return Err(format!(
            "invalid ZEXDOC group {} result (including trailing text)",
            index + 1
        ));
    }
    if lines.next().map(|line| fingerprint(line.as_bytes())) != Some(grammar.completion) {
        return Err("missing ZEXDOC completion message after 58 judged groups".into());
    }
    if lines.next().is_some() {
        return Err("extra ZEXDOC output after completion".into());
    }
    Ok(())
}

// Full 30-byte dot-padded descriptor fingerprints, in source table order.
// zexdoc.z80:170–193 (macros), 195–734 (67 descriptors), verified against the
// pinned binary as recorded in PROVENANCE §2. Exclusions retain their slots
// here so patching and output validation share the same audited positions.
const DESCRIPTORS: [(usize, u64); GROUPS] = [
    (30, 0x18e3175dacd488a6), // 1
    (30, 0x78120c28eeb0eed5), // 2
    (30, 0xefef7047b89cb905), // 3
    (30, 0x861a3000079c0e49), // 4
    (30, 0x22cb78927c1c4f75), // 5
    (30, 0x42609dedb9f3644b), // 6
    (30, 0x5f6f4213c97402c9), // 7
    (30, 0x74a30ef818f9fbd3), // 8
    (30, 0xdf89b10e0ed33fe4), // 9
    (30, 0x9616cce26d79c2d8), // 10
    (30, 0x0ced992949557f8e), // 11
    (30, 0xabee18bd69eff2b7), // 12
    (30, 0x1af1346fe6e353b6), // 13
    (30, 0x29847d8909e9cba4), // 14
    (30, 0x5b2c8660a8ac4eff), // 15
    (30, 0xddc67b6f1043dfdc), // 16
    (30, 0x7376bd7c3b2ddb66), // 17
    (30, 0x7849d3d1a1ebc019), // 18
    (30, 0x8fd87dee101598ac), // 19
    (30, 0xb361c292b021d540), // 20
    (30, 0xb7046ba2fc351fb5), // 21
    (30, 0x85174a965620c4bf), // 22
    (30, 0x2e426a0168ad9d6e), // 23
    (30, 0xf3ba554b1de7c0ad), // 24
    (30, 0xb2c8a7a24f594391), // 25
    (30, 0x2f00d5fb9dc213d4), // 26
    (30, 0xd2cf5ac6077fedec), // 27
    (30, 0x7670179018283d75), // 28
    (30, 0x61311d027160b1a0), // 29
    (30, 0x8a5898c665422b5c), // 30
    (30, 0x66d137071bfd9a9b), // 31
    (30, 0xe211dfe0fe1817df), // 32
    (30, 0x3e55dff36eaf8172), // 33
    (30, 0x73d1a39c9591c29e), // 34
    (30, 0x3299843493501e0b), // 35
    (30, 0xcdc0a08e033cc5af), // 36
    (30, 0x828be5ce2b678a5e), // 37
    (30, 0x90f0b82e67988e48), // 38
    (30, 0x85de9e178e9ea0f3), // 39
    (30, 0xabebfbe7b430439d), // 40
    (30, 0x13360a8cd501b6fe), // 41
    (30, 0xa09c6ca9d96a45f6), // 42
    (30, 0x112d0168f962e44c), // 43
    (30, 0x86de52f84261aedf), // 44
    (30, 0x940ea029da94dfaf), // 45
    (30, 0x2695f625eba1f637), // 46
    (30, 0x3c0336b57868739b), // 47
    (30, 0x151d7f269bac4326), // 48
    (30, 0x034d2df777fb14e5), // 49
    (30, 0xec6abeb3776f7e49), // 50
    (30, 0x3c72d8d143820911), // 51
    (30, 0xb0344b4b548200fe), // 52
    (30, 0xd22fca9c2d1a2c9b), // 53
    (30, 0x1af0e1fa3e13e568), // 54
    (30, 0x2082aec8e34934b6), // 55
    (30, 0xbfdca68583d7e85d), // 56
    (30, 0x7711ac9784bd85e5), // 57
    (30, 0xe7eeff7ff9ab0ce3), // 58
    (30, 0xdd627bdddb994f8d), // 59
    (30, 0x221663d3d78c45d0), // 60
    (30, 0x41fe4b78997624c4), // 61
    (30, 0x1a4f2765726a6041), // 62
    (30, 0xd95cdb6719e49a65), // 63
    (30, 0xe3e7b4acb85eda0f), // 64
    (30, 0x9a2bbfb5c5d0365b), // 65
    (30, 0x8d88335229acb1da), // 66
    (30, 0x903c432921557d65), // 67
];
