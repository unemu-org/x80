//! Independent literal witnesses (decision-10, decision-11).
//!
//! Intel 205775-002 (January 1983), §2.2.1 p. 2-2: register set;
//! §5.4 p. 5-3: first register high, second low; §5.6.5 p. 5-15,
//! PUSH PSW / FLAG WORD: A above flags, S Z X AC X P X CY; p. 5-16,
//! POP PSW: only five flag destinations. Second witness: 9800301-04
//! (May 1981), Ch. 3 p. 3-45: PSW stack order and undefined 8085 filler
//! bits. Pages checked as rendered images. A-high u16 packing is derived.

use x80_core::i8085::{Flags, Registers};

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

fn sample() -> Registers {
    Registers {
        a: 0xA1,
        flags: ALL_FLAGS,
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
fn pair_getters_place_the_first_register_high() {
    let registers = sample();
    assert_eq!(registers.bc(), 0xB2C3);
    assert_eq!(registers.de(), 0xD4E5);
    assert_eq!(registers.hl(), 0xF607);
}

#[test]
fn pair_setters_change_only_the_named_pair() {
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
    registers = sample();
    registers.set_de(0xABCD);
    assert_eq!(
        registers,
        Registers {
            d: 0xAB,
            e: 0xCD,
            ..sample()
        }
    );
    registers = sample();
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

#[test]
fn psw_places_a_high_and_each_flag_at_its_documented_bit_under_d5h() {
    for (flags, expected) in [
        (NO_FLAGS, 0x5A00),
        (
            Flags {
                sign: true,
                ..NO_FLAGS
            },
            0x5A80,
        ),
        (
            Flags {
                zero: true,
                ..NO_FLAGS
            },
            0x5A40,
        ),
        (
            Flags {
                auxiliary_carry: true,
                ..NO_FLAGS
            },
            0x5A10,
        ),
        (
            Flags {
                parity: true,
                ..NO_FLAGS
            },
            0x5A04,
        ),
        (
            Flags {
                carry: true,
                ..NO_FLAGS
            },
            0x5A01,
        ),
        (ALL_FLAGS, 0x5AD5),
    ] {
        let registers = Registers {
            a: 0x5A,
            flags,
            ..sample()
        };
        assert_eq!(registers.psw() & 0xFFD5, expected);
    }
}

#[test]
fn pop_psw_style_setter_keeps_only_five_flags_and_a() {
    // Each expectation is a literal from p. 5-16's transfer list.
    for (image, flags) in [
        (
            0x1201,
            Flags {
                carry: true,
                ..NO_FLAGS
            },
        ),
        (
            0x1204,
            Flags {
                parity: true,
                ..NO_FLAGS
            },
        ),
        (
            0x1210,
            Flags {
                auxiliary_carry: true,
                ..NO_FLAGS
            },
        ),
        (
            0x1240,
            Flags {
                zero: true,
                ..NO_FLAGS
            },
        ),
        (
            0x1280,
            Flags {
                sign: true,
                ..NO_FLAGS
            },
        ),
        (0x12FF, ALL_FLAGS),
        (0x122A, NO_FLAGS),
        (0x1200, NO_FLAGS),
    ] {
        let mut registers = sample();
        registers.set_psw(image);
        assert_eq!(
            registers,
            Registers {
                a: 0x12,
                flags,
                ..sample()
            }
        );
    }
}

#[test]
fn psw_round_trips_under_d5h_with_literal_expectations() {
    for (image, expected) in [
        (0xFFFF, 0xFFD5),
        (0xA5D5, 0xA5D5),
        (0x002A, 0x0000),
        (0x0000, 0x0000),
        (0x3C93, 0x3C91),
        (0x5A44, 0x5A44),
    ] {
        let mut registers = sample();
        registers.set_psw(image);
        assert_eq!(registers.psw() & 0xFFD5, expected);
    }
}

#[test]
fn library_choice_pins_undefined_bits_5_3_1_to_the_8080_image() {
    // decision-11 library choice, NOT a hardware claim: p. 5-15 marks these
    // bits undefined. This is the sole witness comparing their full image.
    let mut registers = sample();
    registers.set_psw(0x002A);
    assert_eq!(registers.psw(), 0x0002);
    registers.set_psw(0xFFFF);
    assert_eq!(registers.psw(), 0xFFD7);
}
