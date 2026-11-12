//! The 8080 Logical Group (98-153B pp. 4-8 to 4-11).
//!
//! Not implemented yet: every opcode of the group returns `Unsupported` with
//! nothing changed (decision-5 item 7). The opcodes are worked out by hand
//! from the entries.

mod support;

use support::assert_unsupported;

#[test]
fn every_opcode_of_the_group_is_not_implemented_yet() {
    // ANA, XRA, ORA and CMP with a register or M: 101xxSSS.
    let register_or_memory = 0xA0..=0xBF;
    #[rustfmt::skip]
    let others = [
        // ANI, XRI, ORI, CPI.
        0xE6, 0xEE, 0xF6, 0xFE,
        // RLC, RRC, RAL, RAR.
        0x07, 0x0F, 0x17, 0x1F,
        // CMA, CMC, STC.
        0x2F, 0x3F, 0x37,
    ];
    let opcodes: Vec<u8> = register_or_memory.chain(others).collect();
    assert_eq!(opcodes.len(), 43);
    for opcode in opcodes {
        assert_unsupported(opcode, 0x5678);
    }
}
