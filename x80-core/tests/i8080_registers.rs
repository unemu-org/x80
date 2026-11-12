//! Witnesses for the Intel 8080 register pairs and the processor status word.
//!
//! Design: plan §2.2, decision-5 item 1. Every expected value below is a
//! literal derived by hand from the cited pages, never from a crate helper.
//!
//! Register pairs, Intel 98-153B (September 1975) p. 4-3, "rp": B is the
//! high-order and C the low-order register of pair B,C; likewise D,E and H,L.
//!
//! Processor status word, 98-153B p. 4-13, PUSH PSW / POP PSW and the "FLAG
//! WORD" figure; second witness 9800301-04 p. 3-45, PUSH PSW:
//!
//! - A is stored at SP - 1, the high-order position of PUSH rp, and the flag
//!   byte at SP - 2, so the 16-bit PSW is A in the high byte and the flag byte
//!   in the low byte.
//! - Flag byte bit positions: D7 S, D6 Z, D4 AC, D2 P, D0 CY.
//! - Fixed bits: D1 is always 1; D3 and D5 are always 0 (9800301-04 p. 3-45:
//!   "On the 8080, bits 3 and 5 are always zero; bit one is always set to
//!   one").
//! - The CPU holds only five flag bits (98-153B p. 2-3, "A 5-bit flag
//!   register"), and POP PSW loads only bits 0, 2, 4, 6 and 7 (p. 4-13), so
//!   setting the PSW keeps the five flags and the fixed bits read back as above.

use x80_core::i8080::{Flags, Registers};

const NO_FLAGS: Flags = Flags {
    sign: false,
    zero: false,
    auxiliary_carry: false,
    parity: false,
    carry: false,
};

const ALL_FLAGS: Flags = Flags {
    sign: true,
    zero: true,
    auxiliary_carry: true,
    parity: true,
    carry: true,
};

/// Distinct values in every register, so a stray write is visible.
fn sample() -> Registers {
    Registers {
        a: 0xA1,
        flags: NO_FLAGS,
        b: 0xB2,
        c: 0xC3,
        d: 0xD4,
        e: 0xE5,
        h: 0xF6,
        l: 0x07,
        sp: 0x1829,
        pc: 0x3A4B,
    }
}

#[test]
fn pair_getters_put_the_first_register_in_the_high_byte() {
    let registers = sample();
    assert_eq!(registers.bc(), 0xB2C3);
    assert_eq!(registers.de(), 0xD4E5);
    assert_eq!(registers.hl(), 0xF607);
}

#[test]
fn set_bc_writes_b_high_and_c_low_only() {
    let mut registers = sample();
    registers.set_bc(0x1234);
    assert_eq!(
        registers,
        Registers {
            b: 0x12,
            c: 0x34,
            ..sample()
        }
    );
}

#[test]
fn set_de_writes_d_high_and_e_low_only() {
    let mut registers = sample();
    registers.set_de(0xABCD);
    assert_eq!(
        registers,
        Registers {
            d: 0xAB,
            e: 0xCD,
            ..sample()
        }
    );
}

#[test]
fn set_hl_writes_h_high_and_l_low_only() {
    let mut registers = sample();
    registers.set_hl(0xFF00);
    assert_eq!(
        registers,
        Registers {
            h: 0xFF,
            l: 0x00,
            ..sample()
        }
    );
}

fn psw_of(a: u8, flags: Flags) -> u16 {
    Registers {
        a,
        flags,
        ..Registers::default()
    }
    .psw()
}

#[test]
fn psw_image_has_bit1_set_and_bits_3_and_5_clear() {
    // No flags: only D1 = 1 (0b0000_0010).
    assert_eq!(psw_of(0x00, NO_FLAGS), 0x0002);
    // All five flags: D7 + D6 + D4 + D2 + D1 + D0 = 0x80 + 0x40 + 0x10 + 0x04 +
    // 0x02 + 0x01 = 0xD7; D5 and D3 stay 0.
    assert_eq!(psw_of(0xFF, ALL_FLAGS), 0xFFD7);
}

#[test]
fn psw_image_places_each_flag_at_its_bit() {
    let only = |set: fn(&mut Flags)| {
        let mut flags = NO_FLAGS;
        set(&mut flags);
        psw_of(0x00, flags)
    };
    assert_eq!(only(|f| f.carry = true), 0x0003); // D0 CY, D1
    assert_eq!(only(|f| f.parity = true), 0x0006); // D2 P, D1
    assert_eq!(only(|f| f.auxiliary_carry = true), 0x0012); // D4 AC, D1
    assert_eq!(only(|f| f.zero = true), 0x0042); // D6 Z, D1
    assert_eq!(only(|f| f.sign = true), 0x0082); // D7 S, D1
}

#[test]
fn psw_image_puts_a_in_the_high_byte() {
    let flags = Flags {
        sign: true,
        carry: true,
        ..NO_FLAGS
    };
    // A = 0x5A; flag byte D7 + D1 + D0 = 0x83.
    assert_eq!(psw_of(0x5A, flags), 0x5A83);
}

#[test]
fn set_psw_loads_a_and_the_five_flags_only() {
    let mut registers = sample();
    registers.set_psw(0x1201);
    assert_eq!(
        registers,
        Registers {
            a: 0x12,
            flags: Flags {
                carry: true,
                ..NO_FLAGS
            },
            ..sample()
        }
    );
}

#[test]
fn set_psw_reads_each_flag_from_its_bit() {
    let flags_from = |low: u16| {
        let mut registers = Registers::default();
        registers.set_psw(low);
        registers.flags
    };
    let with = |set: fn(&mut Flags)| {
        let mut flags = NO_FLAGS;
        set(&mut flags);
        flags
    };
    assert_eq!(flags_from(0x0001), with(|f| f.carry = true));
    assert_eq!(flags_from(0x0004), with(|f| f.parity = true));
    assert_eq!(flags_from(0x0010), with(|f| f.auxiliary_carry = true));
    assert_eq!(flags_from(0x0040), with(|f| f.zero = true));
    assert_eq!(flags_from(0x0080), with(|f| f.sign = true));
    // D1, D3 and D5 (0x02 + 0x08 + 0x20) are not flags.
    assert_eq!(flags_from(0x002A), NO_FLAGS);
}

#[test]
fn set_psw_then_psw_restores_the_fixed_bits() {
    let mut registers = Registers::default();

    registers.set_psw(0xFFFF);
    assert_eq!(registers.a, 0xFF);
    assert_eq!(registers.flags, ALL_FLAGS);
    assert_eq!(registers.psw(), 0xFFD7);

    // 0xD5: D7 D6 D4 D2 D0 set, D1 clear; it reads back with D1 set.
    registers.set_psw(0xA5D5);
    assert_eq!(registers.a, 0xA5);
    assert_eq!(registers.flags, ALL_FLAGS);
    assert_eq!(registers.psw(), 0xA5D7);

    // 0x2A: only D5, D3 and D1 set; no flag, and only D1 reads back.
    registers.set_psw(0x002A);
    assert_eq!(registers.a, 0x00);
    assert_eq!(registers.flags, NO_FLAGS);
    assert_eq!(registers.psw(), 0x0002);

    registers.set_psw(0x0000);
    assert_eq!(registers.psw(), 0x0002);
}
