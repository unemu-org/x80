//! The Intel 8080 model type: construction, ownership and `Default`
//! (decision-5 items 1 and 2). Expected values are literals.
//!
//! `Default` is a library choice, not a hardware claim: 98-153B p. 2-13 says
//! the register contents are indeterminate at power-up.

use x80_core::i8080::{Flags, Latches, Registers, I8080};

const ZERO_REGISTERS: Registers = Registers {
    a: 0,
    flags: Flags {
        sign: false,
        zero: false,
        auxiliary_carry: false,
        parity: false,
        carry: false,
    },
    b: 0,
    c: 0,
    d: 0,
    e: 0,
    h: 0,
    l: 0,
    sp: 0,
    pc: 0,
};

const CLEAR_LATCHES: Latches = Latches {
    interrupt_enable: false,
    ei_delay: false,
    halted: false,
};

#[test]
fn default_is_zeroed_with_interrupts_disabled_and_not_halted() {
    let cpu = I8080::default();
    assert_eq!(*cpu.registers(), ZERO_REGISTERS);
    assert_eq!(*cpu.latches(), CLEAR_LATCHES);
    assert_eq!(Registers::default(), ZERO_REGISTERS);
    assert_eq!(Latches::default(), CLEAR_LATCHES);
}

#[test]
fn new_keeps_the_given_registers_and_latches() {
    let registers = Registers {
        a: 0x01,
        flags: Flags {
            zero: true,
            ..ZERO_REGISTERS.flags
        },
        b: 0x02,
        c: 0x03,
        d: 0x04,
        e: 0x05,
        h: 0x06,
        l: 0x07,
        sp: 0x0809,
        pc: 0x0A0B,
    };
    let latches = Latches {
        interrupt_enable: true,
        ei_delay: true,
        halted: true,
    };
    let cpu = I8080::new(registers, latches);
    assert_eq!(*cpu.registers(), registers);
    assert_eq!(*cpu.latches(), latches);
}

#[test]
fn mutable_accessors_change_the_owned_state() {
    let mut cpu = I8080::default();
    cpu.registers_mut().pc = 0x0100;
    cpu.registers_mut().set_hl(0x2345);
    cpu.latches_mut().halted = true;
    assert_eq!(
        *cpu.registers(),
        Registers {
            h: 0x23,
            l: 0x45,
            pc: 0x0100,
            ..ZERO_REGISTERS
        }
    );
    assert_eq!(
        *cpu.latches(),
        Latches {
            halted: true,
            ..CLEAR_LATCHES
        }
    );
}
