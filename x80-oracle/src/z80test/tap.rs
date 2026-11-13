//! TAP framing and a source/binary layout audit, decision-20 §7.
//! Locators refer to the SHA-pinned v1.2a archive (PROVENANCE §2).

use super::{zip::Archive, Result, Variant};

pub(super) struct Tap<'a> {
    pub(super) basic: &'a [u8],
    pub(super) code: &'a [u8],
    pub(super) load: u16,
    names: [&'a [u8]; 2],
}

fn u16_at(bytes: &[u8], pos: usize) -> Result<u16> {
    let pair = bytes.get(pos..pos + 2).ok_or("truncated TAP word")?;
    Ok(u16::from_le_bytes([pair[0], pair[1]]))
}

pub(super) fn parse(bytes: &[u8]) -> Result<Tap<'_>> {
    let mut blocks = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let len = usize::from(u16_at(bytes, pos)?);
        pos += 2;
        let block = bytes.get(pos..pos + len).ok_or("truncated TAP block")?;
        if len < 2 || block.iter().fold(0, |a, b| a ^ b) != 0 {
            return Err("invalid TAP length/checksum".into());
        }
        blocks.push(block);
        pos += len;
    }
    if blocks.len() != 4 {
        return Err("expected BASIC and CODE TAP header/data pairs".into());
    }
    for (pair, kind) in blocks.as_chunks::<2>().0.iter().zip([0, 3]) {
        if pair[0].len() != 19
            || pair[0][0] != 0
            || pair[0][1] != kind
            || pair[1][0] != 255
            || usize::from(u16_at(pair[0], 12)?) + 2 != pair[1].len()
        {
            return Err("TAP header/data type or length mismatch".into());
        }
    }
    if u16_at(blocks[0], 14)? != 10
        || u16_at(blocks[0], 16)? != u16_at(blocks[0], 12)?
        || u16_at(blocks[2], 16)? != 0
    {
        return Err("TAP BASIC autostart/variables or CODE parameter mismatch".into());
    }
    Ok(Tap {
        basic: &blocks[1][1..blocks[1].len() - 1],
        code: &blocks[3][1..blocks[3].len() - 1],
        load: u16_at(blocks[2], 14)?,
        names: [&blocks[0][2..12], &blocks[2][2..12]],
    })
}

#[derive(Debug, Clone)]
pub(super) struct Group {
    pub(super) name: String,
    pub(super) expected: u32,
    pub(super) precheck: u8,
}

pub(super) struct Image {
    pub(super) code: Vec<u8>,
    pub(super) sites: super::host::Sites,
    pub(super) groups: Vec<Group>,
    pub(super) tap_sha256: String,
    pub(super) table: u16,
}

fn source(archive: &Archive<'_>, file: &str) -> Result<String> {
    String::from_utf8(archive.read(&format!("src/{file}"))?).map_err(|e| e.to_string())
}

fn require(source: &str, fragments: &[&str]) -> Result<()> {
    for fragment in fragments {
        if !source.contains(fragment) {
            return Err(format!("source correspondence mismatch: {fragment}"));
        }
    }
    Ok(())
}

pub(super) fn load(archive: &Archive<'_>, variant: Variant) -> Result<Image> {
    // Each source/member read also checks the directory CRC-32. These checks
    // establish layout correspondence, not an assembler rebuild or CPU facts.
    require(
        &source(archive, "Makefile")?,
        &[
            "mktap -b $(basename $(word 2,$^)) 10",
            "mktap    $(basename $(word 2,$^)) 32768",
        ],
    )?;
    require(
        &source(archive, "loader.bas")?,
        &["CLEAR 32767:LOAD \"\" CODE:CLS", "RANDOMIZE USR 32768"],
    )?;
    require(
        &source(archive, "main.asm")?,
        &[
            "org     0x8000",
            "ld      hl,testtable",
            "ld      hl,data+3",
            "ei\n            ret",
        ],
    )?;
    require(
        &source(archive, "idea.asm")?,
        &[
            "If this moves from 0x8800",
            "data\n.regs",
            "include crctab.asm",
        ],
    )?;
    require(
        &source(archive, "print.asm")?,
        &["jp      0x1601", "ld      iy,0x5c3a", "rst     0x10"],
    )?;
    let tap_bytes = archive.read(&format!("{}.tap", variant.name()))?;
    let tap = parse(&tap_bytes)?;
    let mut name = format!("{:<10}", variant.name()).into_bytes();
    name.truncate(10);
    if tap.names.iter().any(|&n| n != name) || tap.load != 0x8000 {
        return Err("TAP name/load does not correspond to Makefile/main.asm".into());
    }
    // Tokenised BASIC has two lines, CLEAR's binary number is 32767 and
    // USR's is 32768. Its payload length is checked against the TAP header.
    if tap.basic.len() != 42
        || !tap.basic.windows(5).any(|w| w == [0x0e, 0, 0, 0xff, 0x7f])
        || !tap.basic.windows(5).any(|w| w == [0x0e, 0, 0, 0, 0x80])
    {
        return Err("TAP BASIC loader correspondence mismatch".into());
    }
    let variant_source = source(archive, &format!("{}.asm", variant.name()))?;
    let ccf = variant == Variant::Ccf;
    let masked = matches!(variant, Variant::Doc | Variant::DocFlags);
    for (key, enabled) in [
        ("maskflags", masked),
        ("postccf", ccf),
        ("memptr", variant == Variant::Memptr),
        (
            "onlyflags",
            !matches!(variant, Variant::Full | Variant::Doc),
        ),
    ] {
        require(
            &variant_source,
            &[&format!(
                "{key}{}equ         {}",
                " ".repeat(12 - key.len()),
                u8::from(enabled)
            )],
        )?;
    }
    let code = tap.code;
    let sites = super::host::Sites::audit(code, variant)?;
    if code.get(..5) != Some(&[0xf3, 0xfd, 0xe5, 0xd9, 0xe5]) {
        return Err("TAP entry does not match main DI/PUSH IY/EXX/PUSH HL".into());
    }
    let table = if ccf { 0x887f } else { 0x887a };
    let table_bytes = (table as u16).to_le_bytes();
    if !code[..128.min(code.len())]
        .windows(6)
        .any(|w| w == [1, 0, 0, 0x21, table_bytes[0], table_bytes[1]])
        || code.get(0x810..0x812) != Some(&[if ccf { 0x3f } else { 3 }, 0xc3])
        || code.get(0x800..0x810) != Some(&[0; 16])
    {
        return Err("TAP 8800h workspace/jump/table layout mismatch".into());
    }
    let tests = source(archive, "tests.asm")?;
    let table_source = tests
        .split_once("dw      .selftest")
        .ok_or("missing source table")?
        .1
        .split_once("dw      0")
        .ok_or("unterminated source table")?
        .0;
    let mut labels = vec![".selftest"];
    labels.extend(
        table_source
            .lines()
            .filter_map(|l| l.trim().strip_prefix("dw      ")),
    );
    if labels.len() != 160 {
        return Err("source group table count mismatch".into());
    }
    let mut groups = Vec::new();
    let vecsize = if ccf { 21 } else { 20 };
    let mut next_descriptor = table + (labels.len() + 1) * 2;
    for (index, label) in labels.iter().enumerate() {
        let ptr = usize::from(u16_at(code, table - 0x8000 + index * 2)?);
        if ptr != next_descriptor {
            return Err(format!("noncontiguous descriptor {index}"));
        }
        let marker = format!("{label} ");
        let entry = tests
            .lines()
            .position(|line| line.starts_with(&marker))
            .ok_or("missing source descriptor")?;
        let lines: Vec<_> = tests.lines().skip(entry).take(9).collect();
        let flag_args: Vec<_> = lines[0]
            .split_once("flags")
            .ok_or("missing flags")?
            .1
            .trim()
            .split(',')
            .collect();
        if flag_args.len() != 16 {
            return Err("invalid source flags".into());
        }
        let flag = if masked {
            flag_args
                .iter()
                .skip(1)
                .step_by(2)
                .try_fold(0_u8, |mask, bit| {
                    Ok::<_, String>((mask << 1) | bit.parse::<u8>().map_err(|e| e.to_string())?)
                })?
        } else {
            255
        };
        let offset = ptr - 0x8000;
        if code.get(offset) != Some(&flag) {
            return Err(format!("flag mask mismatch {label}"));
        }
        let crc_line = lines
            .iter()
            .find_map(|l| l.trim().strip_prefix("crcs    "))
            .ok_or("missing source CRCs")?;
        let crc_args: Vec<_> = crc_line.split(',').collect();
        let key = match variant {
            Variant::Full => "all",
            Variant::Doc => "doc",
            Variant::Flags => "allflags",
            Variant::DocFlags => "docflags",
            Variant::Ccf => "ccf",
            Variant::Memptr => "mptr",
        };
        let crc = crc_args
            .as_chunks::<2>()
            .0
            .iter()
            .find(|p| p[0] == key)
            .ok_or("missing variant CRC")?[1];
        let expected =
            u32::from_str_radix(crc.trim_start_matches("0x"), 16).map_err(|e| e.to_string())?;
        let crc_offset = offset + 1 + 3 * vecsize;
        if code.get(crc_offset..crc_offset + 4) != Some(expected.to_be_bytes().as_slice()) {
            return Err(format!("descriptor CRC correspondence mismatch {label}"));
        }
        let name = lines
            .iter()
            .find_map(|l| l.trim().strip_prefix("name    \""))
            .ok_or("missing source name")?
            .trim_end_matches('"');
        let start = crc_offset + 4;
        if code.get(start..start + name.len()) != Some(name.as_bytes())
            || code.get(start + name.len()) != Some(&0)
        {
            return Err(format!("descriptor name mismatch {label}"));
        }
        let end = start + name.len() + 1;
        let explicit = lines
            .iter()
            .skip(1)
            .skip_while(|l| !l.trim().starts_with("name "))
            .nth(1)
            .and_then(|l| l.trim().strip_prefix("db      "));
        let precheck = *code.get(end).ok_or("missing precheck")?;
        if let Some(check) = explicit {
            let expected = match check {
                "failcheck" => 0,
                "incheck" => 1,
                "nocheck" => 255,
                _ => return Err("unknown precheck".into()),
            };
            if precheck != expected {
                return Err(format!("precheck mismatch {label}"));
            }
        } else if precheck < 2 {
            return Err(format!("unexpected implicit precheck {label}"));
        }
        next_descriptor = 0x8000 + end + usize::from(explicit.is_some());
        groups.push(Group {
            name: name.to_owned(),
            expected,
            precheck,
        });
    }
    if u16_at(code, table - 0x8000 + groups.len() * 2)? != 0
        || next_descriptor != 0x8000 + code.len()
    {
        return Err(
            "TAP code length/table terminator does not match final source descriptor".into(),
        );
    }
    Ok(Image {
        code: code.to_vec(),
        sites,
        groups,
        table: table as u16,
        tap_sha256: crate::sha256::digest(&tap_bytes),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(out: &mut Vec<u8>, data: &[u8]) {
        out.extend_from_slice(&((data.len() + 1) as u16).to_le_bytes());
        out.extend_from_slice(data);
        out.push(data.iter().fold(0, |a, b| a ^ b));
    }
    fn fixture() -> Vec<u8> {
        let mut out = Vec::new();
        for (kind, payload) in [(0, &[13][..]), (3, &[0xc9][..])] {
            let mut h = vec![0, kind];
            h.extend_from_slice(b"synthetic ");
            h.extend_from_slice(&(payload.len() as u16).to_le_bytes());
            h.extend_from_slice(&(if kind == 0 { 10_u16 } else { 0x8000 }).to_le_bytes());
            h.extend_from_slice(&(if kind == 0 { 1_u16 } else { 0 }).to_le_bytes());
            block(&mut out, &h);
            let mut d = vec![255];
            d.extend_from_slice(payload);
            block(&mut out, &d);
        }
        out
    }
    #[test]
    fn tap_pairs_checksum_truncation_and_header_length() {
        let b = fixture();
        let tap = parse(&b).unwrap();
        assert_eq!(tap.load, 0x8000);
        assert_eq!(tap.code, [0xc9]);
        for end in 0..b.len() {
            assert!(parse(&b[..end]).is_err());
        }
        let mut bad = b.clone();
        bad[3] ^= 1;
        assert!(parse(&bad).is_err());
        let mut bad = b.clone();
        bad[14] = 2;
        bad[20] ^= 3;
        assert!(parse(&bad).is_err());
        let mut bad = b;
        bad.push(0);
        assert!(parse(&bad).is_err());
    }
}
