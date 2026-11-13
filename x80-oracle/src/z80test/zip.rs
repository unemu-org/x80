//! Private, bounded ZIP/DEFLATE reader for the pinned fetch-only archive.
//! ZIP local/central/end records: PKWARE APPNOTE 6.3.10 §§4.3.7,12,16.
//! DEFLATE: RFC 1951 §§3.1.1, 3.2.3–7; CRC-32: RFC 1952 §8.

use super::Result;

const LIMIT: usize = 2 * 1024 * 1024;

fn bytes(input: &[u8], offset: usize, len: usize) -> Result<&[u8]> {
    input
        .get(offset..offset.checked_add(len).ok_or("ZIP offset overflow")?)
        .ok_or_else(|| "truncated ZIP record".into())
}

fn word(input: &[u8], offset: usize) -> Result<usize> {
    let b = bytes(input, offset, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]) as usize)
}

fn dword(input: &[u8], offset: usize) -> Result<u32> {
    let b = bytes(input, offset, 4)?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

struct Member {
    name: String,
    method: usize,
    flags: usize,
    crc: u32,
    size: usize,
    compressed: usize,
    offset: usize,
}

pub(super) struct Archive<'a> {
    input: &'a [u8],
    members: Vec<Member>,
    directory: usize,
}

impl<'a> Archive<'a> {
    /// Verify identity BEFORE interpreting any archive metadata or member bytes.
    pub(super) fn pinned(input: &'a [u8]) -> Result<Self> {
        if crate::sha256::digest(input) != super::ZIP_SHA256 {
            return Err("z80test ZIP SHA-256 mismatch".into());
        }
        Self::parse(input)
    }

    fn parse(input: &'a [u8]) -> Result<Self> {
        let end = (input.len().saturating_sub(65557)..input.len().saturating_sub(21))
            .rev()
            .find(|&p| {
                input.get(p..p + 4) == Some(b"PK\x05\x06")
                    && word(input, p + 20).is_ok_and(|n| p + 22 + n == input.len())
            })
            .ok_or("missing ZIP end record")?;
        let count = word(input, end + 10)?;
        if word(input, end + 4)? != 0
            || word(input, end + 6)? != 0
            || word(input, end + 8)? != count
            || count == 65535
        {
            return Err("multi-disk/ZIP64 archive is unsupported".into());
        }
        let directory = dword(input, end + 16)? as usize;
        let directory_size = dword(input, end + 12)? as usize;
        if directory.checked_add(directory_size) != Some(end) {
            return Err("ZIP directory bounds mismatch".into());
        }
        let mut pos = directory;
        let mut members: Vec<Member> = Vec::new();
        for _ in 0..count {
            if bytes(input, pos, 4)? != b"PK\x01\x02" {
                return Err("invalid ZIP directory signature".into());
            }
            let flags = word(input, pos + 8)?;
            let method = word(input, pos + 10)?;
            // Deflate hints, data descriptor and UTF-8 are the only allowed flags.
            if flags & !0x080e != 0 || ![0, 8].contains(&method) {
                return Err("unsupported ZIP flags or compression".into());
            }
            let size = dword(input, pos + 24)? as usize;
            let name_len = word(input, pos + 28)?;
            let name = std::str::from_utf8(bytes(input, pos + 46, name_len)?)
                .map_err(|_| "invalid ZIP member name")?
                .to_owned();
            if size > LIMIT || word(input, pos + 34)? != 0 {
                return Err("ZIP member exceeds limit or spans disks".into());
            }
            if members.iter().any(|m| m.name == name) {
                return Err("duplicate ZIP member".into());
            }
            members.push(Member {
                name,
                method,
                flags,
                size,
                crc: dword(input, pos + 16)?,
                compressed: dword(input, pos + 20)? as usize,
                offset: dword(input, pos + 42)? as usize,
            });
            pos += 46 + name_len + word(input, pos + 30)? + word(input, pos + 32)?;
            if pos > end {
                return Err("ZIP directory entry exceeds bounds".into());
            }
        }
        if pos != end {
            return Err("ZIP directory size/count mismatch".into());
        }
        Ok(Self {
            input,
            members,
            directory,
        })
    }

    pub(super) fn read(&self, name: &str) -> Result<Vec<u8>> {
        let name = format!("z80test-1.2a/{name}");
        let m = self
            .members
            .iter()
            .find(|m| m.name == name)
            .ok_or_else(|| format!("missing ZIP member {name}"))?;
        let p = m.offset;
        if bytes(self.input, p, 4)? != b"PK\x03\x04"
            || word(self.input, p + 6)? != m.flags
            || word(self.input, p + 8)? != m.method
        {
            return Err("ZIP local/central header mismatch".into());
        }
        let name_len = word(self.input, p + 26)?;
        if bytes(self.input, p + 30, name_len)? != name.as_bytes() {
            return Err("ZIP local/central name mismatch".into());
        }
        if m.flags & 8 == 0
            && (dword(self.input, p + 14)? != m.crc
                || dword(self.input, p + 18)? as usize != m.compressed
                || dword(self.input, p + 22)? as usize != m.size)
        {
            return Err("ZIP local/central size/CRC mismatch".into());
        }
        let start = p + 30 + name_len + word(self.input, p + 28)?;
        if start
            .checked_add(m.compressed)
            .is_none_or(|e| e > self.directory)
        {
            return Err("ZIP member overlaps directory".into());
        }
        let compressed = bytes(self.input, start, m.compressed)?;
        let output = if m.method == 0 {
            compressed.to_vec()
        } else {
            inflate(compressed, m.size)?
        };
        if output.len() != m.size || crc32(&output) != m.crc {
            return Err(format!("ZIP member size/CRC-32 mismatch: {name}"));
        }
        Ok(output)
    }
}

fn crc32(input: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for &b in input {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

struct Bits<'a> {
    input: &'a [u8],
    position: usize,
}

impl Bits<'_> {
    fn take(&mut self, count: usize) -> Result<usize> {
        let mut value = 0;
        for i in 0..count {
            let b = self
                .input
                .get(self.position / 8)
                .ok_or("truncated deflate")?;
            value |= usize::from((b >> (self.position % 8)) & 1) << i;
            self.position += 1;
        }
        Ok(value)
    }
}

struct Huffman {
    // Canonical codes indexed by bit length and code value. Decode consumes
    // Huffman bits most-significant first, unlike non-code DEFLATE fields.
    symbols: Vec<Vec<Option<usize>>>,
}

impl Huffman {
    fn new(lengths: &[usize]) -> Result<Self> {
        let mut counts = [0_usize; 16];
        for &len in lengths {
            if len > 15 {
                return Err("deflate code too long".into());
            }
            if len != 0 {
                counts[len] += 1;
            }
        }
        let mut codes = [0; 16];
        let mut available = 1_i32;
        for len in 1..=15 {
            available = available * 2 - counts[len] as i32;
            if available < 0 {
                return Err("oversubscribed deflate tree".into());
            }
            codes[len] = (codes[len - 1] + counts[len - 1]) << 1;
        }
        // Incomplete trees (including the no-distance case) are permitted;
        // an unused code can never decode successfully.
        let mut symbols: Vec<Vec<Option<usize>>> = (0..16).map(|_| Vec::new()).collect();
        for (symbol, &len) in lengths.iter().enumerate() {
            if len != 0 {
                symbols[len].resize(1 << len, None);
                symbols[len][codes[len]] = Some(symbol);
                codes[len] += 1;
            }
        }
        Ok(Self { symbols })
    }

    fn decode(&self, bits: &mut Bits<'_>) -> Result<usize> {
        let mut code = 0;
        for len in 1..=15 {
            code = (code << 1) | bits.take(1)?;
            if let Some(Some(symbol)) = self.symbols[len].get(code) {
                return Ok(*symbol);
            }
        }
        Err("invalid deflate Huffman code".into())
    }
}

fn trees(bits: &mut Bits<'_>, dynamic: bool) -> Result<(Huffman, Huffman)> {
    if !dynamic {
        let mut lengths = vec![8; 288];
        lengths[144..256].fill(9);
        lengths[256..280].fill(7);
        return Ok((Huffman::new(&lengths)?, Huffman::new(&[5; 32])?));
    }
    let literals = bits.take(5)? + 257;
    let distances = bits.take(5)? + 1;
    if literals > 286 || distances > 32 {
        return Err("invalid deflate alphabet size".into());
    }
    let count = bits.take(4)? + 4;
    let mut lengths = [0; 19];
    for symbol in [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ]
    .into_iter()
    .take(count)
    {
        lengths[symbol] = bits.take(3)?;
    }
    let alphabet = Huffman::new(&lengths)?;
    let mut lengths = Vec::new();
    while lengths.len() < literals + distances {
        let (value, count) = match alphabet.decode(bits)? {
            n @ 0..=15 => (n, 1),
            16 => (
                *lengths.last().ok_or("repeat without deflate predecessor")?,
                bits.take(2)? + 3,
            ),
            17 => (0, bits.take(3)? + 3),
            18 => (0, bits.take(7)? + 11),
            _ => return Err("invalid deflate length symbol".into()),
        };
        if lengths.len() + count > literals + distances {
            return Err("deflate length repeat overflow".into());
        }
        lengths.resize(lengths.len() + count, value);
    }
    if lengths[256] == 0 {
        return Err("deflate tree has no end code".into());
    }
    Ok((
        Huffman::new(&lengths[..literals])?,
        Huffman::new(&lengths[literals..])?,
    ))
}

fn inflate(input: &[u8], size: usize) -> Result<Vec<u8>> {
    if size > LIMIT {
        return Err("deflate output exceeds limit".into());
    }
    let mut bits = Bits { input, position: 0 };
    let mut output = Vec::with_capacity(size);
    loop {
        let last = bits.take(1)? != 0;
        match bits.take(2)? {
            0 => {
                bits.position = bits.position.div_ceil(8) * 8;
                let len = bits.take(16)?;
                if len ^ bits.take(16)? != 65535 || output.len() + len > size {
                    return Err("invalid stored deflate length".into());
                }
                for _ in 0..len {
                    output.push(bits.take(8)? as u8);
                }
            }
            kind @ (1 | 2) => {
                let (literal, distance) = trees(&mut bits, kind == 2)?;
                loop {
                    match literal.decode(&mut bits)? {
                        b @ 0..=255 if output.len() < size => output.push(b as u8),
                        256 => break,
                        n @ 257..=285 => {
                            const BASE: [usize; 29] = [
                                3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43,
                                51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258,
                            ];
                            const EXTRA: [usize; 29] = [
                                0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4,
                                4, 4, 5, 5, 5, 5, 0,
                            ];
                            let len = BASE[n - 257] + bits.take(EXTRA[n - 257])?;
                            let d = distance.decode(&mut bits)?;
                            if d >= 30 {
                                return Err("reserved deflate distance".into());
                            }
                            let extra = d.saturating_sub(2) / 2;
                            let base = if d < 4 {
                                d + 1
                            } else {
                                ((2 + (d & 1)) << extra) + 1
                            };
                            let back = base + bits.take(extra)?;
                            if back > output.len() || output.len() + len > size {
                                return Err("deflate back-reference outside output".into());
                            }
                            for _ in 0..len {
                                output.push(output[output.len() - back]);
                            }
                        }
                        _ => return Err("invalid deflate literal or output overflow".into()),
                    }
                }
            }
            _ => return Err("reserved deflate block type".into()),
        }
        if last {
            break;
        }
    }
    if output.len() != size || bits.position.div_ceil(8) != input.len() {
        return Err("deflate size mismatch or trailing compressed bytes".into());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A tiny hand-built bit writer; no compressor or artifact needed by tests.
    #[derive(Default)]
    struct Writer {
        bytes: Vec<u8>,
        position: usize,
    }
    impl Writer {
        fn put(&mut self, value: usize, count: usize) {
            for i in 0..count {
                if self.position.is_multiple_of(8) {
                    self.bytes.push(0);
                }
                self.bytes[self.position / 8] |= (((value >> i) & 1) as u8) << (self.position % 8);
                self.position += 1;
            }
        }
        fn code(&mut self, code: usize, count: usize) {
            for i in (0..count).rev() {
                self.put((code >> i) & 1, 1);
            }
        }
    }

    #[test]
    fn literal_stored_fixed_dynamic_and_overlapping_copy() {
        assert_eq!(
            inflate(&[1, 3, 0, 252, 255, b'A', b'B', b'C'], 3).unwrap(),
            b"ABC"
        );
        let mut fixed = Writer::default();
        fixed.put(3, 3); // final, fixed
        fixed.code(0x71, 8); // literal A (48 + 65)
        fixed.code(1, 7); // length 3, symbol 257
        fixed.code(0, 5); // distance 1: overlapping AAA
        fixed.code(0, 7); // end
        assert_eq!(inflate(&fixed.bytes, 4).unwrap(), b"AAAA");
        assert!(inflate(&fixed.bytes, 3).is_err());
        // Dynamic alphabet: only literal A and end, each length 1. Code
        // lengths use symbols 0/1, both length 1, without repeats.
        let mut dynamic = Writer::default();
        dynamic.put(5, 3); // final, dynamic
        dynamic.put(0, 5);
        dynamic.put(0, 5);
        dynamic.put(14, 4);
        for symbol in [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1] {
            dynamic.put(usize::from(symbol == 0 || symbol == 1), 3);
        }
        for symbol in 0..258 {
            dynamic.put(usize::from(symbol == 65 || symbol == 256), 1);
        }
        dynamic.code(0, 1);
        dynamic.code(1, 1);
        assert_eq!(inflate(&dynamic.bytes, 1).unwrap(), b"A");
        assert!(inflate(&dynamic.bytes[..dynamic.bytes.len() - 1], 1).is_err());
    }

    #[test]
    fn rejects_bad_blocks_distances_trees_and_pin() {
        for input in [&[7][..], &[1, 1, 0, 0, 0, 1], &[], &[3, 2, 0]] {
            assert!(inflate(input, 1).is_err());
        }
        assert!(Huffman::new(&[1, 1, 1]).is_err());
        assert!(Archive::pinned(b"not the artifact").is_err());
        assert_eq!(crc32(b"123456789"), 0xcbf43926);
    }

    #[test]
    fn dynamic_length_repeats_and_mixed_blocks() {
        let mut w = Writer::default();
        w.put(5, 3); // final dynamic
        w.put(0, 5);
        w.put(0, 5);
        w.put(14, 4);
        for symbol in [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1] {
            w.put(
                match symbol {
                    0 | 1 | 16 => 2,
                    17 | 18 => 3,
                    _ => 0,
                },
                3,
            );
        }
        // 65 zeros, A length 1, 190 zeros, end length 1, no distances.
        // Canonical alphabet: 0=00, 1=01, 16=10, 17=110, 18=111.
        w.code(7, 3);
        w.put(51, 7); // 62 zeroes
        w.code(2, 2);
        w.put(0, 2); // repeat zero three times (16)
        w.code(1, 2);
        w.code(7, 3);
        w.put(127, 7); // 138 zeroes
        w.code(7, 3);
        w.put(38, 7); // 49 zeroes
        w.code(6, 3);
        w.put(0, 3); // three zeroes (17)
        w.code(1, 2);
        w.code(0, 2);
        w.code(0, 1);
        w.code(1, 1); // A, end
        assert_eq!(inflate(&w.bytes, 1).unwrap(), b"A");
        // Non-final stored block followed by a final fixed empty block.
        assert_eq!(inflate(&[0, 1, 0, 254, 255, b'X', 3, 0], 1).unwrap(), b"X");
        assert!(inflate(&[1, 1, 0, 254, 255, b'X', 0], 1).is_err());
    }

    fn stored_zip() -> Vec<u8> {
        let name = b"z80test-1.2a/test";
        let data = b"123456789";
        let mut b = Vec::new();
        b.extend_from_slice(b"PK\x03\x04");
        for value in [20_u16, 0, 0, 0, 0] {
            b.extend_from_slice(&value.to_le_bytes());
        }
        b.extend_from_slice(&0xcbf43926_u32.to_le_bytes());
        for value in [9_u32, 9] {
            b.extend_from_slice(&value.to_le_bytes());
        }
        b.extend_from_slice(&(name.len() as u16).to_le_bytes());
        b.extend_from_slice(&[0, 0]);
        b.extend_from_slice(name);
        b.extend_from_slice(data);
        let directory = b.len();
        b.extend_from_slice(b"PK\x01\x02");
        for value in [20_u16, 20, 0, 0, 0, 0] {
            b.extend_from_slice(&value.to_le_bytes());
        }
        b.extend_from_slice(&0xcbf43926_u32.to_le_bytes());
        for value in [9_u32, 9] {
            b.extend_from_slice(&value.to_le_bytes());
        }
        b.extend_from_slice(&(name.len() as u16).to_le_bytes());
        b.extend_from_slice(&[0; 16]);
        b.extend_from_slice(name);
        let size = b.len() - directory;
        b.extend_from_slice(b"PK\x05\x06");
        b.extend_from_slice(&[0; 4]);
        b.extend_from_slice(&[1, 0, 1, 0]);
        b.extend_from_slice(&(size as u32).to_le_bytes());
        b.extend_from_slice(&(directory as u32).to_le_bytes());
        b.extend_from_slice(&[0; 2]);
        b
    }

    #[test]
    fn stored_zip_directory_crc_bounds_and_local_agreement() {
        let b = stored_zip();
        assert_eq!(
            Archive::parse(&b).unwrap().read("test").unwrap(),
            b"123456789"
        );
        for end in 0..b.len() {
            assert!(Archive::parse(&b[..end]).is_err());
        }
        let mut bad = b.clone();
        bad[14] ^= 1;
        assert!(Archive::parse(&bad).unwrap().read("test").is_err());
        let mut bad = b.clone();
        bad[47] ^= 1; // payload, unchanged directory CRC
        assert!(Archive::parse(&bad)
            .unwrap()
            .read("test")
            .unwrap_err()
            .contains("CRC-32"));
        let mut bad = b;
        let n = bad.len();
        bad[n - 10..n - 6].fill(255);
        assert!(Archive::parse(&bad).is_err());
    }
}
