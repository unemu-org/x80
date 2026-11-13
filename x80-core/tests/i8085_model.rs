//! State ownership and reset witnesses (decision-12 rule 9), literal values.
//!
//! Intel 205775-002 (January 1983), §2.3.7 p. 2-18 RESET IN table:
//! PC, INTE, RST 7.5 FF, TRAP FF, SOD FF reset; all three masks set.
//! §2.3.6 p. 2-16: RESET exits halt. Second source: 231718-001 (September
//! 1987), handbook Table 1 p. 16-13: PC zero, interrupt enable reset,
//! internal registers/flags unpredictable. Both checked as page images.
//!
//! Preserving A/B/C/D/E/H/L, flags and SP is a library choice, as are Default
//! and reset's sampler bookkeeping. Clearing a pending EI delay on reset is
//! unverified, not stated by those pages (decision-12 rule 9).

use x80_core::i8085::{Flags, Latches, Registers, I8085};

const DEFAULT_REGISTERS: Registers = Registers {
    a: 0,
    flags: Flags {
        overflow: false,
        k: false,
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

const RESET_LATCHES: Latches = Latches {
    interrupt_enable: false,
    ei_delay: false,
    halted: false,
    mask5_5: true,
    mask6_5: true,
    mask7_5: true,
    rst7_5_pending: false,
    trap_pending: false,
    trap_armed: true,
    trap_saved_interrupt_enable: false,
    trap_rim_pending: false,
    sod: false,
    last_trap: false,
    last_rst7_5: false,
};

// Every latch differs from its reset value, including sampler bookkeeping.
const DIRTY_LATCHES: Latches = Latches {
    interrupt_enable: true,
    ei_delay: true,
    halted: true,
    mask5_5: false,
    mask6_5: false,
    mask7_5: false,
    rst7_5_pending: true,
    trap_pending: true,
    trap_armed: false,
    trap_saved_interrupt_enable: true,
    trap_rim_pending: true,
    sod: true,
    last_trap: true,
    last_rst7_5: true,
};

const REGISTERS: Registers = Registers {
    a: 0xA1,
    flags: Flags {
        overflow: false,
        k: false,
        sign: true,
        zero: false,
        auxiliary_carry: true,
        parity: false,
        carry: true,
    },
    b: 0xB2,
    c: 0xC3,
    d: 0xD4,
    e: 0xE5,
    h: 0xF6,
    l: 0x07,
    sp: 0x8F90,
    pc: 0x1234,
};

#[test]
fn default_is_a_library_choice_without_a_power_up_claim() {
    let cpu = I8085::default();
    assert_eq!(*cpu.registers(), DEFAULT_REGISTERS);
    assert_eq!(*cpu.latches(), RESET_LATCHES);
    assert_eq!(Registers::default(), DEFAULT_REGISTERS);
    assert_eq!(Flags::default(), DEFAULT_REGISTERS.flags);
    assert_eq!(Latches::default(), RESET_LATCHES);
}

#[test]
fn new_owns_all_supplied_state_unchanged() {
    // Construction and read accessors are const, as for the 8080 API.
    const CPU: I8085 = I8085::new(REGISTERS, DIRTY_LATCHES);
    const PC: u16 = CPU.registers().pc;
    const SOD: bool = CPU.latches().sod;
    assert_eq!(PC, 0x1234);
    assert_eq!(u8::from(SOD), 1);
    assert_eq!(*CPU.registers(), REGISTERS);
    assert_eq!(*CPU.latches(), DIRTY_LATCHES);
}

#[test]
fn mutable_accessors_change_the_owned_state() {
    let mut cpu = I8085::default();
    *cpu.registers_mut() = REGISTERS;
    *cpu.latches_mut() = DIRTY_LATCHES;
    assert_eq!(*cpu.registers(), REGISTERS);
    assert_eq!(*cpu.latches(), DIRTY_LATCHES);
}

#[test]
fn reset_matches_the_table_and_clears_halt() {
    let mut cpu = I8085::new(REGISTERS, DIRTY_LATCHES);
    cpu.reset();
    assert_eq!(cpu.registers().pc, 0x0000);
    assert!(!cpu.latches().interrupt_enable);
    assert!(!cpu.latches().rst7_5_pending);
    assert!(!cpu.latches().trap_pending);
    assert!(!cpu.latches().sod);
    assert!(cpu.latches().mask5_5);
    assert!(cpu.latches().mask6_5);
    assert!(cpu.latches().mask7_5);
    assert!(!cpu.latches().halted);
}

#[test]
fn reset_preserves_registers_flags_and_sp_by_library_choice() {
    let mut cpu = I8085::new(REGISTERS, DIRTY_LATCHES);
    cpu.reset();
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0000,
            ..REGISTERS
        }
    );
}

#[test]
fn reset_clears_ei_delay_unverified_and_trap_bookkeeping_by_library_choice() {
    let mut cpu = I8085::new(REGISTERS, DIRTY_LATCHES);
    cpu.reset();
    assert!(!cpu.latches().ei_delay); // Unverified reset/EI interaction.
    assert_eq!(*cpu.latches(), RESET_LATCHES);
    cpu.reset();
    assert_eq!(*cpu.latches(), RESET_LATCHES);
    assert_eq!(
        *cpu.registers(),
        Registers {
            pc: 0x0000,
            ..REGISTERS
        }
    );
}
