//! Fact-free opcode-map machinery (decision-6, decision-10).
//!
//! Models supply patterns, field restrictions, lengths, groups and costs.
//! This module assigns no hardware meaning to those values.

/// Two model-supplied execution costs, selected by the executor's path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct States {
    pub(crate) untaken: u8,
    pub(crate) taken: u8,
}

impl States {
    /// Fills both slots with the same cost.
    pub(crate) const fn single(states: u8) -> Self {
        Self {
            untaken: states,
            taken: states,
        }
    }

    /// Selects the cost for the executor's reported path.
    pub(crate) const fn of(self, path: Path) -> u8 {
        match path {
            Path::Untaken => self.untaken,
            Path::Taken => self.taken,
        }
    }
}

/// Selects one of the model's two cost slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Path {
    Untaken,
    Taken,
}

/// A model-supplied excluded value of a letter field (decision-10).
#[derive(Clone, Copy, Debug)]
pub(crate) struct FieldRestriction {
    /// Repeated occurrences of this letter form a field, most significant first.
    pub(crate) field: u8,
    pub(crate) excluded: u8,
}

/// A byte pattern and the metadata to copy into matching map entries.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Encoding {
    /// Most significant bit first: `0`/`1` fix bits; ASCII letters are fields.
    bits: [u8; 8],
    length: u8,
    states: States,
}

impl Encoding {
    pub(crate) const fn new(bits: &[u8; 8], length: u8, states: States) -> Self {
        Self {
            bits: *bits,
            length,
            states,
        }
    }

    /// Matches fixed bits and applies only the supplied field restrictions.
    const fn matches(&self, byte: u8, restrictions: &[FieldRestriction]) -> bool {
        let mut index = 0;
        while index < self.bits.len() {
            let bit = (byte >> (7 - index)) & 1;
            let fits = match self.bits[index] {
                b'0' => bit == 0,
                b'1' => bit == 1,
                b'A'..=b'Z' | b'a'..=b'z' => true,
                _ => panic!("unknown symbol in an encoding"),
            };
            if !fits {
                return false;
            }
            index += 1;
        }
        let mut restriction_index = 0;
        while restriction_index < restrictions.len() {
            let restriction = restrictions[restriction_index];
            let mut present = false;
            let mut value = 0;
            index = 0;
            while index < self.bits.len() {
                if self.bits[index] == restriction.field {
                    present = true;
                    value = (value << 1) | ((byte >> (7 - index)) & 1);
                }
                index += 1;
            }
            if present && value == restriction.excluded {
                return false;
            }
            restriction_index += 1;
        }
        true
    }
}

/// A matching byte's model-supplied group, length and costs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Instruction<G> {
    pub(crate) group: G,
    pub(crate) length: u8,
    pub(crate) states: States,
}

/// Places each group's encodings into a byte-indexed map (decision-10).
/// Unmatched bytes remain `None`; overlapping encodings fail const evaluation.
pub(crate) const fn build<G: Copy>(
    groups: &[(G, &[Encoding])],
    restrictions: &[FieldRestriction],
) -> [Option<Instruction<G>>; 256] {
    let mut table = [None; 256];
    let mut group_index = 0;
    while group_index < groups.len() {
        let (group, encodings) = groups[group_index];
        let mut encoding_index = 0;
        while encoding_index < encodings.len() {
            let encoding = encodings[encoding_index];
            let mut byte = 0;
            while byte < table.len() {
                if encoding.matches(byte as u8, restrictions) {
                    assert!(table[byte].is_none(), "two encodings claim one byte");
                    table[byte] = Some(Instruction {
                        group,
                        length: encoding.length,
                        states: encoding.states,
                    });
                }
                byte += 1;
            }
            encoding_index += 1;
        }
        group_index += 1;
    }
    table
}

#[cfg(test)]
mod tests {
    use super::{build, Encoding, FieldRestriction, Instruction, States};

    #[test]
    fn unrestricted_letter_field_accepts_every_value() {
        const MAP: [Option<Instruction<u8>>; 256] = build(
            &[(9, &[Encoding::new(b"XXXXXXXX", 2, States::single(3))])],
            &[],
        );
        assert!(MAP.iter().all(|entry| *entry
            == Some(Instruction {
                group: 9,
                length: 2,
                states: States::single(3),
            })));
    }

    #[test]
    fn exclusions_apply_only_to_the_supplied_letter_and_value() {
        let encoding = Encoding::new(b"0QQQJJJJ", 1, States::single(1));
        let exclusions = [
            FieldRestriction {
                field: b'Q',
                excluded: 0b101,
            },
            // A restriction on a field absent from the pattern has no effect.
            FieldRestriction {
                field: b'Z',
                excluded: 0,
            },
        ];
        assert!(encoding.matches(0x50, &[]));
        assert!(!encoding.matches(0x50, &exclusions));
        assert!(encoding.matches(0x35, &exclusions));
        assert!(!encoding.matches(0xB5, &exclusions));
    }
}
