//! Console-text identifiers shared by the oracle protocols and their tests.

pub(crate) fn fingerprint(bytes: &[u8]) -> (usize, u64) {
    (
        bytes.len(),
        bytes.iter().fold(0xcbf29ce484222325_u64, |hash, &byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        }),
    )
}
