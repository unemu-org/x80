//! The 8080 Branch Group (98-153B pp. 4-11 and 4-12).
//!
//! Not implemented yet: every opcode of the group returns `Unsupported` with
//! nothing changed (decision-5 item 7). The opcodes are worked out by hand
//! from the entries; CCC runs over the eight conditions of p. 4-11.

mod support;

use support::assert_unsupported;

#[test]
fn every_opcode_of_the_group_is_not_implemented_yet() {
    #[rustfmt::skip]
    let opcodes = [
        // JMP, then Jcondition: 11CCC010.
        0xC3, 0xC2, 0xCA, 0xD2, 0xDA, 0xE2, 0xEA, 0xF2, 0xFA,
        // CALL, then Ccondition: 11CCC100.
        0xCD, 0xC4, 0xCC, 0xD4, 0xDC, 0xE4, 0xEC, 0xF4, 0xFC,
        // RET, then Rcondition: 11CCC000.
        0xC9, 0xC0, 0xC8, 0xD0, 0xD8, 0xE0, 0xE8, 0xF0, 0xF8,
        // RST 0 to 7: 11NNN111.
        0xC7, 0xCF, 0xD7, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF,
        // PCHL.
        0xE9,
    ];
    assert_eq!(opcodes.len(), 36);
    for opcode in opcodes {
        assert_unsupported(opcode, 0x6789);
    }
}
