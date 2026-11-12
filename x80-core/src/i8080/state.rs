//! The 8080 register file and execution latches (plan §2.2, decision-5
//! items 1 and 2).
//!
//! Primary source: Intel 98-153B (September 1975), checked against the page
//! images. p. 2-2, "Registers": the program counter, the stack pointer and six
//! 8-bit general purpose registers "arranged in pairs, referred to as B,C; D,E;
//! and H,L"; the temporary pair W,Z "is not program addressable" and is not
//! modelled. p. 2-3: the accumulator and the 5-bit flag register.

use super::Flags;

/// The 8080 programmer-visible registers.
///
/// `Default` is all zeros with every flag clear. That is a library choice, not
/// a hardware claim: 98-153B p. 2-13 says the register contents are
/// indeterminate at power-up.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Registers {
    /// Accumulator.
    pub a: u8,
    /// Condition flags.
    pub flags: Flags,
    /// B, the high-order register of pair BC.
    pub b: u8,
    /// C, the low-order register of pair BC.
    pub c: u8,
    /// D, the high-order register of pair DE.
    pub d: u8,
    /// E, the low-order register of pair DE.
    pub e: u8,
    /// H, the high-order register of pair HL.
    pub h: u8,
    /// L, the low-order register of pair HL.
    pub l: u8,
    /// Stack pointer.
    pub sp: u16,
    /// Program counter.
    pub pc: u16,
}

// Register pairs, 98-153B p. 4-3, "rp": "B represents the B,C pair with B as the
// high-order register and C as the low-order register"; likewise D,E and H,L.
impl Registers {
    /// Pair BC: B in the high byte, C in the low byte.
    pub const fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    /// Sets B to the high byte and C to the low byte of `value`.
    pub fn set_bc(&mut self, value: u16) {
        [self.b, self.c] = value.to_be_bytes();
    }

    /// Pair DE: D in the high byte, E in the low byte.
    pub const fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    /// Sets D to the high byte and E to the low byte of `value`.
    pub fn set_de(&mut self, value: u16) {
        [self.d, self.e] = value.to_be_bytes();
    }

    /// Pair HL: H in the high byte, L in the low byte.
    pub const fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    /// Sets H to the high byte and L to the low byte of `value`.
    pub fn set_hl(&mut self, value: u16) {
        [self.h, self.l] = value.to_be_bytes();
    }

    /// The processor status word: A in the high byte, the flag byte in the low
    /// byte.
    ///
    /// The PSW "comprises the contents of the accumulator and the current
    /// settings of the condition flags" (9800301-04 p. 3-45). The 16-bit
    /// packing with A high is derived from the stack order: PUSH PSW stores A
    /// at SP - 1, where PUSH rp stores the high-order register, and the flag
    /// byte at SP - 2 (98-153B p. 4-13). The flag byte holds S, Z, AC, P and
    /// CY in bits 7, 6, 4, 2 and 0, with bit 1 set and bits 3 and 5 clear
    /// (p. 4-13 "FLAG WORD"; 9800301-04 p. 3-45).
    pub const fn psw(&self) -> u16 {
        u16::from_be_bytes([self.a, self.flags.to_byte()])
    }

    /// Sets A from the high byte and the five flags from bits 7, 6, 4, 2 and 0
    /// of the low byte of `value`.
    ///
    /// Bits 1, 3 and 5 are not stored, because the CPU has only five flag
    /// bits (98-153B p. 2-3). A later [`psw`](Self::psw) reads them back as 1,
    /// 0 and 0. The transfer list of POP PSW on p. 4-13 likewise assigns only
    /// those five bits; the instruction itself is witnessed with its
    /// implementation.
    pub fn set_psw(&mut self, value: u16) {
        let [a, flags] = value.to_be_bytes();
        self.a = a;
        self.flags = Flags::from_byte(flags);
    }
}

/// The 8080 execution latches that persist between calls (decision-5).
///
/// `Default` is interrupts disabled, no EI delay pending and not halted: a
/// library choice, not a hardware claim. 98-153B documents no power-up state
/// for INTE or halt; p. 5-14 covers RESET only. Their update rules are fixed
/// with the instructions and the interrupt sequence that change
/// them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Latches {
    /// The interrupt-enable flip-flop, INTE (98-153B p. 5-14, pin definitions).
    pub interrupt_enable: bool,
    /// The delay after EI is pending. EI enables the interrupt system
    /// "following the execution of the next instruction" (98-153B p. 4-14); an
    /// interrupt is eligible only once the delay has expired (decision-5
    /// item 4).
    pub ei_delay: bool,
    /// The CPU is in the halt state entered by HLT (98-153B p. 2-13, "Halt
    /// Sequences").
    pub halted: bool,
}
