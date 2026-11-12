//! The 8080 Data Transfer Group (98-153B pp. 4-4 and 4-5).
//!
//! Not implemented yet: every opcode of the group returns `Unsupported` with
//! nothing changed (decision-5 item 7). The opcodes are worked out by hand
//! from the entries; register codes exclude 110 (p. 4-3).

mod support;

use support::assert_unsupported;

#[test]
fn every_opcode_of_the_group_is_not_implemented_yet() {
    // MOV r1,r2, MOV r,M and MOV M,r: 01DDDSSS except 01110110 (HLT).
    let moves = (0x40..=0x7F).filter(|&opcode| opcode != 0x76);
    #[rustfmt::skip]
    let others = [
        // MVI r,data and MVI M,data: 00DDD110.
        0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E,
        // LXI B, D, H, SP: 00RP0001.
        0x01, 0x11, 0x21, 0x31,
        // LDA, STA, LHLD, SHLD.
        0x3A, 0x32, 0x2A, 0x22,
        // LDAX B, LDAX D, STAX B, STAX D.
        0x0A, 0x1A, 0x02, 0x12,
        // XCHG.
        0xEB,
    ];
    let opcodes: Vec<u8> = moves.chain(others).collect();
    assert_eq!(opcodes.len(), 84);
    for opcode in opcodes {
        assert_unsupported(opcode, 0x3456);
    }
}
