//! Literal register and F-image witnesses (decisions 13/14).
//!
//! Zilog UM008011-0816 (revision 11, August 2016), CPU Register / Registers
//! pp. 2–3 (PDF 16–17); 03-0029-01 (1977) §2.1 pp. 3–5, Fig. 2.0-2 p. 4
//! (PDF 9–11): main/alternate pairs and byte order. The UM Fig. 2 repeats B'
//! in C's alternate column; its p. 3 pair text and TM diagram identify BC'.
//! UM Tables 21/22 pp. 65–66 (PDF 79–80): S Z X H X P/V N C. Hardware
//! comparisons use D7h; zero filler bits are a library choice (decision-14
//! rule 5). All cited values checked on rendered page images.

use x80_core::z80::{Flags, Registers};

const NO_FLAGS: Flags = Flags {
    sign: false,
    zero: false,
    half_carry: false,
    parity_overflow: false,
    subtract: false,
    carry: false,
};
const ALL_FLAGS: Flags = Flags {
    sign: true,
    zero: true,
    half_carry: true,
    parity_overflow: true,
    subtract: true,
    carry: true,
};

const fn sample() -> Registers {
    Registers {
        a: 0xA1,
        flags: ALL_FLAGS,
        b: 0xB2,
        c: 0xC3,
        d: 0xD4,
        e: 0xE5,
        h: 0xF6,
        l: 0x07,
        a_alternate: 0x18,
        flags_alternate: NO_FLAGS,
        b_alternate: 0x29,
        c_alternate: 0x3A,
        d_alternate: 0x4B,
        e_alternate: 0x5C,
        h_alternate: 0x6D,
        l_alternate: 0x7E,
        ix: 0x8091,
        iy: 0x92A3,
        sp: 0xB4C5,
        pc: 0xD6E7,
        i: 0xF8,
        r: 0x89,
    }
}

// UM pp. 2–3 / TM §2.1: first register high, second low. Each setter
// comparison covers the entire file, including the untouched other set.
macro_rules! pair_witness {
    ($name:ident, $get:ident, $set:ident, $hi:ident, $lo:ident, $expected:expr) => {
        #[test]
        fn $name() {
            const REGISTERS: Registers = Registers {
                $hi: 0xAB,
                $lo: 0xCD,
                ..sample()
            };
            const PAIR: u16 = REGISTERS.$get();
            assert_eq!(PAIR, 0xABCD);
            let mut registers = sample();
            assert_eq!(registers.$get(), $expected);
            registers.$set(0x1234);
            assert_eq!(
                registers,
                Registers {
                    $hi: 0x12,
                    $lo: 0x34,
                    ..sample()
                }
            );
        }
    };
}

pair_witness!(bc_byte_order, bc, set_bc, b, c, 0xB2C3);
pair_witness!(de_byte_order, de, set_de, d, e, 0xD4E5);
pair_witness!(hl_byte_order, hl, set_hl, h, l, 0xF607);
pair_witness!(
    bc_alternate_byte_order,
    bc_alternate,
    set_bc_alternate,
    b_alternate,
    c_alternate,
    0x293A
);
pair_witness!(
    de_alternate_byte_order,
    de_alternate,
    set_de_alternate,
    d_alternate,
    e_alternate,
    0x4B5C
);
pair_witness!(
    hl_alternate_byte_order,
    hl_alternate,
    set_hl_alternate,
    h_alternate,
    l_alternate,
    0x6D7E
);

#[test]
fn af_byte_order_and_setter_isolation_under_d7h() {
    let mut registers = sample();
    assert_eq!(registers.af() & 0xFFD7, 0xA1D7);
    registers.set_af(0x12FF);
    assert_eq!(
        registers,
        Registers {
            a: 0x12,
            flags: ALL_FLAGS,
            ..sample()
        }
    );
    registers.set_af(0x3428);
    assert_eq!(
        registers,
        Registers {
            a: 0x34,
            flags: NO_FLAGS,
            ..sample()
        }
    );
}

#[test]
fn af_alternate_byte_order_and_setter_isolation_under_d7h() {
    let mut registers = sample();
    assert_eq!(registers.af_alternate() & 0xFFD7, 0x1800);
    registers.set_af_alternate(0x56FF);
    assert_eq!(
        registers,
        Registers {
            a_alternate: 0x56,
            flags_alternate: ALL_FLAGS,
            ..sample()
        }
    );
    registers.set_af_alternate(0x7828);
    assert_eq!(
        registers,
        Registers {
            a_alternate: 0x78,
            flags_alternate: NO_FLAGS,
            ..sample()
        }
    );
}

#[test]
fn each_flag_has_its_table21_bit_under_d7h() {
    for (flags, expected) in [
        (NO_FLAGS, 0x00),
        (
            Flags {
                sign: true,
                ..NO_FLAGS
            },
            0x80,
        ),
        (
            Flags {
                zero: true,
                ..NO_FLAGS
            },
            0x40,
        ),
        (
            Flags {
                half_carry: true,
                ..NO_FLAGS
            },
            0x10,
        ),
        (
            Flags {
                parity_overflow: true,
                ..NO_FLAGS
            },
            0x04,
        ),
        (
            Flags {
                subtract: true,
                ..NO_FLAGS
            },
            0x02,
        ),
        (
            Flags {
                carry: true,
                ..NO_FLAGS
            },
            0x01,
        ),
        (ALL_FLAGS, 0xD7),
    ] {
        assert_eq!(flags.to_byte() & 0xD7, expected);
    }
}

#[test]
fn from_byte_reads_the_six_table21_flags() {
    for (byte, expected) in [
        (0x00, NO_FLAGS),
        (
            0x80,
            Flags {
                sign: true,
                ..NO_FLAGS
            },
        ),
        (
            0x40,
            Flags {
                zero: true,
                ..NO_FLAGS
            },
        ),
        (
            0x10,
            Flags {
                half_carry: true,
                ..NO_FLAGS
            },
        ),
        (
            0x04,
            Flags {
                parity_overflow: true,
                ..NO_FLAGS
            },
        ),
        (
            0x02,
            Flags {
                subtract: true,
                ..NO_FLAGS
            },
        ),
        (
            0x01,
            Flags {
                carry: true,
                ..NO_FLAGS
            },
        ),
        (0xFF, ALL_FLAGS),
    ] {
        assert_eq!(Flags::from_byte(byte), expected);
    }
}

#[test]
fn library_choice_ignores_bits_5_3_and_reads_them_as_zero() {
    // UM p. 66 says unused, not zero: these full images pin only the API.
    const ALL_IMAGE: u8 = Flags::from_byte(0xFF).to_byte();
    assert_eq!(ALL_IMAGE, 0xD7);
    assert_eq!(Flags::from_byte(0x28), NO_FLAGS);
    assert_eq!(Flags::from_byte(0x28).to_byte(), 0x00);
    let mut registers = sample();
    registers.set_af(0xFFFF);
    registers.set_af_alternate(0xABFF);
    assert_eq!(registers.af(), 0xFFD7);
    assert_eq!(registers.af_alternate(), 0xABD7);
}
