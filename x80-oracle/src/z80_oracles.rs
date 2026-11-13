//! Digest-selected Z80 oracle protocols; no guest CPU semantics live here.
//!
//! PROVENANCE §2 pins the offsets and independent descriptor correspondence
//! for both artifacts (decision-20 §7).
//! Only fingerprints and structural facts are stored here; GPL programs and
//! source stay fetch-only. The caller must identify the original digest first.

use std::io;

use crate::fingerprint;

const TABLE: usize = 0x3a; // File offset; guest address 013Ah.
const GROUPS: usize = 67;

pub(crate) fn audit_exerciser(image: &[u8], all_flags: bool) -> io::Result<()> {
    audit_image(
        image,
        if all_flags {
            &ALL_RECORDS
        } else {
            &DOC_RECORDS
        },
    )
}

pub(crate) fn audit_image(image: &[u8], records: &[u64; GROUPS]) -> io::Result<()> {
    let mismatch = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Z80 exerciser descriptor/table mismatch",
        )
    };
    let table = image
        .get(TABLE..TABLE + (GROUPS + 1) * 2)
        .ok_or_else(mismatch)?;
    // Decision-20 §7: verify all original pointers/records; never filter or patch.
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
    for (index, &expected) in records.iter().enumerate() {
        let start = 0xc2 + index * 0x60;
        let record = image.get(start..start + 0x60).ok_or_else(mismatch)?;
        if fingerprint(record) != (0x60, expected) {
            return Err(mismatch());
        }
    }
    Ok(())
}

/// Diagnostic output boundary only; verdict validation still consumes the full transcript.
pub(crate) fn completed_group(output: &[u8]) -> Option<usize> {
    if !output.ends_with(b"\r") {
        return None;
    }
    let line = output
        .split(|&b| b == b'\r' || b == b'\n')
        .rfind(|line| !line.is_empty())?;
    let descriptor = fingerprint(line.get(..30)?);
    DESCRIPTORS
        .iter()
        .position(|&id| id == descriptor)
        .map(|index| index + 1)
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

// zexdoc.z80 / zexall.z80:1205–1210; strip only the emitted CR/LF line separators.
const GRAMMAR: Grammar = Grammar {
    banner: (25, 0x431656fb9ea34c41),
    completion: (14, 0x948df6ea30f0dd7d),
    ok: (4, 0x37a17a74393dbe97),
    error: (26, 0xb3a627fe7ddd2ec7),
};

pub(crate) fn validate_exerciser(output: &[u8]) -> Result<(), String> {
    validate_groups(output, &DESCRIPTORS, GRAMMAR)
}

pub(crate) fn validate_groups(
    output: &[u8],
    descriptors: &[(usize, u64)],
    grammar: Grammar,
) -> Result<(), String> {
    // zexdoc.z80 / zexall.z80:746–810 prints the descriptor, then OK or expected/found
    // CRCs. OK is emitted only after cmpcrc (1215–1232) matches all four
    // expected bytes. Neither Z80 exerciser prints a CRC on success.
    let text = std::str::from_utf8(output).map_err(|_| "invalid UTF-8 in Z80 exerciser output")?;
    let mut lines = text.split(['\r', '\n']).filter(|line| !line.is_empty());
    if lines.next().map(|line| fingerprint(line.as_bytes())) != Some(grammar.banner) {
        return Err("missing Z80 exerciser banner".into());
    }
    for (index, expected) in descriptors.iter().enumerate() {
        let line = lines
            .next()
            .ok_or_else(|| format!("missing Z80 exerciser group {}", index + 1))?;
        let (descriptor, result) = line
            .split_at_checked(expected.0)
            .ok_or_else(|| format!("incomplete Z80 exerciser group {}", index + 1))?;
        if fingerprint(descriptor.as_bytes()) != *expected {
            return Err(format!(
                "unknown, missing, duplicated or reordered Z80 exerciser group {}",
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
                return Err(format!(
                    "judged Z80 exerciser group {} reports ERROR",
                    index + 1
                ));
            }
        }
        return Err(format!(
            "invalid Z80 exerciser group {} result (including trailing text)",
            index + 1
        ));
    }
    if lines.next().map(|line| fingerprint(line.as_bytes())) != Some(grammar.completion) {
        return Err("missing Z80 exerciser completion message after 67 judged groups".into());
    }
    if lines.next().is_some() {
        return Err("extra Z80 exerciser output after completion".into());
    }
    Ok(())
}

// Full 30-byte dot-padded descriptor fingerprints, in source table order.
// zexdoc.z80:170–193 (macros), 195–734 (67 descriptors), verified against the
// pinned binaries as recorded in PROVENANCE §2. Both independently audited
// sources emit these same messages, including the incyh/incyl DD naming error.
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

// Complete 96-byte record fingerprints, independently expanded from each source
// and matched to its own pinned COM. zexall.z80 / zexdoc.z80:195–731;
// PROVENANCE §2. These include masks, all vectors, CRCs and padded messages.
const DOC_RECORDS: [u64; GROUPS] = [
    0x8052062904be6d77, // adc16
    0x84727b5f78efea29, // add16
    0x0a6a8828d916b5c6, // add16x
    0x0cb7ba8a1b40983a, // add16y
    0xdc35b8f2f1bb55f8, // alu8i
    0xdc617b3be6c2e5af, // alu8r
    0xc6c61243cd2b0249, // alu8rx
    0x5ed02aff48b4cab2, // alu8x
    0xd5fa77b0c1d8042c, // bitx
    0xc0ac47196f42df2c, // bitz80
    0x15947f99d095f275, // cpd1
    0x6c35d122e3f58ae9, // cpi1
    0xbe4cb640af816890, // daa
    0x89323a36f5503f03, // inca
    0x50b986e6eabe1018, // incb
    0x3f5a30cb713b48ad, // incbc
    0x97849399d947b313, // incc
    0x2678d43ee993ae21, // incd
    0xa4de2188cc21c1ba, // incde
    0x4d208b270349b512, // ince
    0xc8235970aa6da31d, // inch
    0x3b791211c9bd6ec2, // inchl
    0x3dde2436f9347ae2, // incix
    0xa653ee0bcb5af551, // inciy
    0x59894194d831a1da, // incl
    0x477e5f3c5bef22c8, // incm
    0xf0c4da92f1fafce1, // incsp
    0x66ef3672bf42d7a2, // incx
    0x239381aceb4fb936, // incxh
    0xc4b7600c5370e809, // incxl
    0xdf934e1e03cd2e1d, // incyh
    0x40b1ffb2b67c8415, // incyl
    0xc7c22899ab6c951d, // ld161
    0x7908ad0dc0533544, // ld162
    0xab2bc4a884116189, // ld163
    0xa07d6a240ef8977d, // ld164
    0x768d5e52d45d4cac, // ld165
    0x7ab9650d90d47485, // ld166
    0x8d2c57bafe95e791, // ld167
    0x8d81b2346c12dc7f, // ld168
    0x3b70b9e0358b3764, // ld16im
    0x199153c1fa407c24, // ld16ix
    0xa6af7fe8e14de9fe, // ld8bd
    0x01a89b25b16eb9f0, // ld8im
    0xa9ff6e3fc7237e59, // ld8imx
    0x5a1d6da5501cde5b, // ld8ix1
    0x1a8b7b8d4bab83d9, // ld8ix2
    0x6d79c83532ffb6b9, // ld8ix3
    0x8ae523e2cc5957b6, // ld8ixy
    0xdcbb69690a19e893, // ld8rr
    0x1de50513958c43ad, // ld8rrx
    0xe17f960e5e5e5d72, // lda
    0x5e6139045ec4c0f6, // ldd1
    0xad7f1ce88c97fe29, // ldd2
    0xedc3bd4b2eb0497a, // ldi1
    0x2e3a47b8218694c4, // ldi2
    0xa5ca5b5f74f2ece9, // neg
    0xa854f75f0c16c8b7, // rld
    0x323268d9d0a34cf8, // rot8080
    0xbe77336c01c3b1f1, // rotxy
    0x85482a80b874724c, // rotz80
    0x25865f9a22316bf3, // srz80
    0xaf38e83fc224628c, // srzx
    0x6b038e851917d2ce, // st8ix1
    0x9dcaee7750d9c783, // st8ix2
    0x1f0b6f5a2e94f584, // st8ix3
    0x01fe2b3091dd1644, // stabd
];
const ALL_RECORDS: [u64; GROUPS] = [
    0xd496f8e27d0276fc, // adc16
    0xbd178095ba63a070, // add16
    0x20bc15470f6ca4a4, // add16x
    0x6d23392a57ea027c, // add16y
    0xed3a77c346e12380, // alu8i
    0x66cfe0c092673fcd, // alu8r
    0xadf5c5d3cbe8caf4, // alu8rx
    0xc26f02f06b9e6088, // alu8x
    0xcdab367777de3044, // bitx
    0xae8446cb38ddb738, // bitz80
    0x0dd07041bd2064b0, // cpd1
    0xd9a14fcd2fe1d11f, // cpi1
    0xa636f69e3e0c8c2e, // daa
    0x1d82fc8b60acbf6d, // inca
    0xfc9a53425a0558b8, // incb
    0x0dbb511bb4695bc5, // incbc
    0xc4f5971b8266a553, // incc
    0x1f6648aaa2975dd2, // incd
    0x71b6560676db3ff2, // incde
    0x0812c32f83675f83, // ince
    0xd0821be738a87845, // inch
    0xa05d82351a0eb63a, // inchl
    0x1fb8d7c5d13c229a, // incix
    0x4f44f73d408cf039, // inciy
    0x1a8b650712cc2bba, // incl
    0x9e2c7e816bb41210, // incm
    0x2222bd479dc87939, // incsp
    0xf229f4ee03ebc2f8, // incx
    0x7b917363d7bde32e, // incxh
    0x7acedd9b925c1b71, // incxl
    0x91e4db5b7b8cc855, // incyh
    0x8dd7615f3f124408, // incyl
    0x14a685675eade485, // ld161
    0x18361d2a2f81329c, // ld162
    0x4b3200abe274cb81, // ld163
    0xaeef27fe1daa7205, // ld164
    0x72178890b349b3b4, // ld165
    0x8edf3f8226b060ed, // ld166
    0x9a1f5da6967fb899, // ld167
    0x8c864a5bc7e6fb37, // ld168
    0xeec14accb062750c, // ld16im
    0xcb7c5ce2e215c94c, // ld16ix
    0x4eb5adce01230f96, // ld8bd
    0x187bc8b9c9b32c18, // ld8im
    0x194a60e633b04f11, // ld8imx
    0xf3903f2493a0e143, // ld8ix1
    0x63e311d05dfb7d91, // ld8ix2
    0x046abcd3083f2771, // ld8ix3
    0xec1af8707d715c5e, // ld8ixy
    0x8d54cdc26b5d1b1b, // ld8rr
    0xc9d0de6229ed16b5, // ld8rrx
    0xabdb899e0198ff2a, // lda
    0x265ae35cd88d0f9e, // ldd1
    0x2f7b11514f7d9b65, // ldd2
    0x59612d72b961a5b0, // ldi1
    0xcabfd16f58deeb3b, // ldi2
    0x9d1a0766bee966dc, // neg
    0xf714ef21744b8d86, // rld
    0x182b48fb37593ec4, // rot8080
    0x85a0d1c4682b1e86, // rotxy
    0x4e8ff7cab96d0d70, // rotz80
    0x6912c4167e274a6b, // srz80
    0x0c97457681159424, // srzx
    0x0dab049ec3c455c6, // st8ix1
    0x1cae92ee9fa042fb, // st8ix2
    0x7eb19cd1251829bc, // st8ix3
    0x9be5d73475857f6c, // stabd
];
