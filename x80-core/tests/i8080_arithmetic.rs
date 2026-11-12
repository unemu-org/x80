//! The 8080 Arithmetic Group (98-153B pp. 4-6 to 4-8).
//!
//! Not implemented yet: every opcode of the group returns `Unsupported` with
//! nothing changed (decision-5 item 7). The opcodes are worked out by hand
//! from the entries.

mod support;

use support::assert_unsupported;

#[test]
fn every_opcode_of_the_group_is_not_implemented_yet() {
    // ADD, ADC, SUB and SBB with a register or M: 100xxSSS.
    let register_or_memory = 0x80..=0x9F;
    #[rustfmt::skip]
    let others = [
        // ADI, ACI, SUI, SBI.
        0xC6, 0xCE, 0xD6, 0xDE,
        // INR r and INR M: 00DDD100.
        0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x34, 0x3C,
        // DCR r and DCR M: 00DDD101.
        0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x35, 0x3D,
        // INX B, D, H, SP: 00RP0011.
        0x03, 0x13, 0x23, 0x33,
        // DCX B, D, H, SP: 00RP1011.
        0x0B, 0x1B, 0x2B, 0x3B,
        // DAD B, D, H, SP: 00RP1001.
        0x09, 0x19, 0x29, 0x39,
        // DAA.
        0x27,
    ];
    let opcodes: Vec<u8> = register_or_memory.chain(others).collect();
    assert_eq!(opcodes.len(), 65);
    for opcode in opcodes {
        assert_unsupported(opcode, 0x4567);
    }
}
