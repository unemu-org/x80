//! Consume the exact main.asm:16–168 output protocol, including partial runs.
//! A captured OK is a group observation, never a licence or conformance claim.

use super::{tap::Group, Variant};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Status {
    Ok,
    Failed { found: u32, expected: u32 },
    Incheck { found: u8 },
    Skipped,
}

#[derive(Debug)]
pub(super) struct Line {
    pub(super) number: usize,
    pub(super) name: String,
    pub(super) diagnostic: bool,
    pub(super) status: Status,
}

#[derive(Debug, Default)]
pub(super) struct Parsed {
    pub(super) lines: Vec<Line>,
    pub(super) final_counts: Option<(usize, usize)>, // failures, total
    pub(super) pending: Option<String>,
    pub(super) error: Option<String>,
}

impl Parsed {
    pub(super) fn judged(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| {
                !l.diagnostic && !matches!(l.status, Status::Skipped | Status::Incheck { .. })
            })
            .count()
    }
    pub(super) fn ok(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| !l.diagnostic && l.status == Status::Ok)
            .count()
    }
    pub(super) fn failed(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| !l.diagnostic && matches!(l.status, Status::Failed { .. }))
            .count()
    }
    pub(super) fn incheck(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| matches!(l.status, Status::Incheck { .. }))
            .count()
    }
    pub(super) fn skipped(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| l.status == Status::Skipped)
            .count()
    }
    pub(super) fn diagnostics(&self) -> usize {
        self.lines.iter().filter(|l| l.diagnostic).count()
    }
    pub(super) fn complete(&self, count: usize) -> bool {
        self.error.is_none()
            && self.pending.is_none()
            && self.lines.len() == count
            && self.final_counts == Some((0, count))
            && self.incheck() == 0
    }
}

/// Screen controls are part of the protocol, not arbitrary whitespace: TAB
/// takes exactly two bytes (including 1); copyright 127 is rendered as (c).
pub(super) fn screen_text(raw: &[u8]) -> Result<String, String> {
    decode(raw).map(|(text, _)| text)
}

fn decode(raw: &[u8]) -> Result<(String, bool), String> {
    let mut text = String::new();
    let mut pos = 0;
    while pos < raw.len() {
        match raw[pos] {
            23 => {
                let Some(args) = raw.get(pos + 1..pos + 3) else {
                    // A step/time cap can cut a multi-character ROM protocol
                    // at any byte. Preserve all earlier complete group lines.
                    return Ok((text, true));
                };
                if ![19, 25, 26, 30, 21].contains(&args[0]) || args[1] != 1 {
                    return Err("unexpected Spectrum TAB arguments".into());
                }
                text.push('\t');
                pos += 2;
            }
            13 => text.push('\n'),
            127 => text.push_str("(c)"),
            b @ 32..=126 => text.push(char::from(b)),
            _ => return Err("unexpected Spectrum output byte".into()),
        }
        pos += 1;
    }
    Ok((text, false))
}

pub(super) fn parse(raw: &[u8], variant: Variant, groups: &[Group]) -> Parsed {
    let mut parsed = Parsed::default();
    let result = parse_into(raw, variant, groups, &mut parsed);
    if let Err(error) = result {
        parsed.error = Some(error);
    }
    parsed
}

fn hex(value: &str, digits: usize) -> Result<u32, String> {
    if value.len() != digits
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'A'..=b'F').contains(&b))
    {
        return Err("invalid or incomplete z80test CRC/input byte".into());
    }
    u32::from_str_radix(value, 16).map_err(|e| e.to_string())
}

fn decimal(value: &str) -> Result<usize, String> {
    if value.len() != 3 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid z80test decimal count".into());
    }
    value
        .parse()
        .map_err(|e: std::num::ParseIntError| e.to_string())
}

fn parse_into(
    raw: &[u8],
    variant: Variant,
    groups: &[Group],
    parsed: &mut Parsed,
) -> Result<(), String> {
    let (text, partial_control) = decode(raw)?;
    if partial_control {
        parsed.pending = Some("partial Spectrum TAB control".into());
    }
    let banner = format!("Z80 {} test\t(c) 2012 RAXOFT\n\n", variant.label());
    let Some(mut rest) = text.strip_prefix(&banner) else {
        if banner.starts_with(&text) {
            parsed.pending = Some("banner".into());
            return Ok(());
        }
        return Err("missing/invalid z80test banner".into());
    };
    let mut failures = 0;
    for (number, group) in groups.iter().enumerate() {
        let prefix = format!("{number:03} {}\t", group.name);
        let Some(tail) = rest.strip_prefix(&prefix) else {
            if prefix.starts_with(rest) {
                parsed.pending = Some(format!("{number:03} {}", group.name));
                return Ok(());
            }
            return Err(format!(
                "unknown/missing/duplicated/reordered group {number}"
            ));
        };
        let Some((result, tail)) = tail.split_once('\n') else {
            parsed.pending = Some(format!("{number:03} {} (partial result)", group.name));
            return Ok(());
        };
        let (status, tail) = match result {
            "OK" => (Status::Ok, tail),
            "Skipped" if group.precheck == 0 && failures == 0 => (Status::Skipped, tail),
            "FAILED" => {
                let Some((detail, tail)) = tail.split_once('\n') else {
                    parsed.pending = Some(format!(
                        "{number:03} {} (missing CRC/input detail)",
                        group.name
                    ));
                    return Ok(());
                };
                if let Some(pair) = detail.strip_prefix("CRC:") {
                    let (found, expected) =
                        pair.split_once("   Expected:").ok_or("missing CRC pair")?;
                    let (found, expected) = (hex(found, 8)?, hex(expected, 8)?);
                    if expected != group.expected || found == expected {
                        return Err("CRC report disagrees with descriptor or FAILED status".into());
                    }
                    (Status::Failed { found, expected }, tail)
                } else if let Some(pair) = detail.strip_prefix("IN FE:") {
                    let (found, expected) = pair
                        .split_once("\tExpected:")
                        .ok_or("missing IN precheck expectation")?;
                    let found = hex(found, 2)? as u8;
                    if expected != "BF" || found == 0xbf || group.precheck != 1 {
                        return Err("invalid .incheck failure report".into());
                    }
                    (Status::Incheck { found }, tail)
                } else {
                    return Err("unknown FAILED detail".into());
                }
            }
            _ => return Err("invalid z80test group result".into()),
        };
        if group.precheck == 0 && failures == 0 && status != Status::Skipped {
            return Err("diagnostic executed without prior failure".into());
        }
        failures += usize::from(matches!(
            status,
            Status::Failed { .. } | Status::Incheck { .. }
        ));
        parsed.lines.push(Line {
            number,
            name: group.name.clone(),
            diagnostic: group.precheck == 0,
            status,
        });
        rest = tail;
    }
    let Some((summary, trailing)) = rest.strip_prefix('\n').and_then(|s| s.split_once('\n')) else {
        parsed.pending = Some("final result".into());
        return Ok(());
    };
    let summary = summary
        .strip_prefix("Result: ")
        .ok_or("missing final Result")?;
    let counts = if summary == "all tests passed." {
        (0, groups.len())
    } else {
        let (failed, total) = summary
            .strip_suffix(" tests failed.")
            .and_then(|s| s.split_once(" of "))
            .ok_or("invalid final counts")?;
        (decimal(failed)?, decimal(total)?)
    };
    if counts != (failures, groups.len()) || !trailing.is_empty() {
        return Err("final counts disagree with complete groups or trailing output".into());
    }
    parsed.final_counts = Some(counts);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn groups() -> Vec<Group> {
        vec![
            Group {
                name: "ALPHA".into(),
                expected: 0x12345678,
                precheck: 255,
            },
            Group {
                name: "INPUT".into(),
                expected: 9,
                precheck: 1,
            },
            Group {
                name: "DIAG".into(),
                expected: 1,
                precheck: 0,
            },
        ]
    }
    fn raw(body: &str) -> Vec<u8> {
        format!("Z80 full test\t(c) 2012 RAXOFT\n\n{body}")
            .replace("\t(c)", "\x17\x13\x01\x7f")
            .replace('\t', "\x17\x1e\x01")
            .replace('\n', "\r")
            .into_bytes()
    }
    #[test]
    fn cap_in_tab_preserves_completed_groups_and_cannot_pass() {
        let prefix = raw("000 ALPHA\tOK\n001 INPUT");
        for tail in [&[23][..], &[23, 30]] {
            let mut truncated = prefix.clone();
            truncated.extend_from_slice(tail);
            let p = parse(&truncated, Variant::Full, &groups());
            assert_eq!(p.ok(), 1);
            assert!(p.pending.is_some());
            assert!(!p.complete(3));
        }
        let mut trailing =
            raw("000 ALPHA\tOK\n001 INPUT\tOK\n002 DIAG\tSkipped\n\nResult: all tests passed.\n");
        trailing.push(23);
        let p = parse(&trailing, Variant::Full, &groups());
        assert_eq!(p.ok(), 2);
        assert!(!p.complete(3));
    }

    #[test]
    fn complete_partial_crc_incheck_and_diagnostics() {
        let good =
            raw("000 ALPHA\tOK\n001 INPUT\tOK\n002 DIAG\tSkipped\n\nResult: all tests passed.\n");
        let p = parse(&good, Variant::Full, &groups());
        assert!(p.complete(3));
        assert_eq!(
            (p.judged(), p.ok(), p.skipped(), p.diagnostics()),
            (2, 2, 1, 1)
        );
        let fail = raw("000 ALPHA\tFAILED\nCRC:00000001   Expected:12345678\n001 INPUT\tFAILED\nIN FE:FF\tExpected:BF\n002 DIAG\tOK\n\nResult: 002 of 003 tests failed.\n");
        let p = parse(&fail, Variant::Full, &groups());
        assert_eq!(p.error, None);
        assert_eq!(p.final_counts, Some((2, 3)));
        assert_eq!((p.failed(), p.incheck()), (1, 1));
        assert!(!p.complete(3));
        for body in ["000 ALPHA", "000 ALPHA\tFAILED\nCRC:123", ""] {
            let p = parse(&raw(body), Variant::Full, &groups());
            assert!(p.pending.is_some());
            assert!(!p.complete(3));
        }
        for body in [
            "001 INPUT\tOK\n",
            "000 ALPHA\tOK trailing\n",
            "000 ALPHA\tFAILED\nCRC:bad   Expected:12345678\n",
            "000 ALPHA\tFAILED\nCRC:12345678   Expected:12345678\n",
            "000 ALPHA\tSkipped\n",
        ] {
            assert!(
                parse(&raw(body), Variant::Full, &groups()).error.is_some(),
                "{body}"
            );
        }
        let mut extra = good;
        extra.extend_from_slice(b"junk");
        assert!(parse(&extra, Variant::Full, &groups()).error.is_some());
    }
}
