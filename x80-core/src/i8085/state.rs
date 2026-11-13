//! Model-owned state (decision-10; decision-12 rules 2, 6, 8 and 9).
//!
//! Register set: Intel 205775-002 (January 1983) §2.2.1/§2.2.2 p. 2-2;
//! second source 231718-001 (September 1987), handbook p. 16-14.

use super::Flags;

/// The 8085 programmer-visible registers (205775-002 Ch. 2, p. 2-2).
///
/// `Default` is zero with all flags clear, a library choice. Reset preserves
/// these values except PC by library choice (decision-12 rule 9): the sources
/// describe indeterminate reset results (205775-002 §2.3.7 p. 2-18;
/// 231718-001 Table 1, handbook p. 16-13).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Registers {
    /// Accumulator.
    pub a: u8,
    /// The seven independent condition flags (decision-18 §4).
    pub flags: Flags,
    /// High byte of BC.
    pub b: u8,
    /// Low byte of BC.
    pub c: u8,
    /// High byte of DE.
    pub d: u8,
    /// Low byte of DE.
    pub e: u8,
    /// High byte of HL.
    pub h: u8,
    /// Low byte of HL.
    pub l: u8,
    /// Stack pointer.
    pub sp: u16,
    /// Program counter.
    pub pc: u16,
}

// Pair names: 205775-002 p. 2-2. First register high, second low:
// §5.4 p. 5-3, Register Indirect. Design: decision-10, model-owned facts.
impl Registers {
    /// BC, with B high and C low.
    pub const fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    /// Sets B high and C low.
    pub fn set_bc(&mut self, value: u16) {
        [self.b, self.c] = value.to_be_bytes();
    }

    /// DE, with D high and E low.
    pub const fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    /// Sets D high and E low.
    pub fn set_de(&mut self, value: u16) {
        [self.d, self.e] = value.to_be_bytes();
    }

    /// HL, with H high and L low.
    pub const fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    /// Sets H high and L low.
    pub fn set_hl(&mut self, value: u16) {
        [self.h, self.l] = value.to_be_bytes();
    }

    /// A high, flag byte low: an API packing derived from PUSH PSW's stack
    /// order (205775-002 §5.6.5 p. 5-15; 9800301-04 p. 3-45).
    /// D/S image S Z K AC 0 P V CY (decision-18 §4; flags module sources).
    pub const fn psw(&self) -> u16 {
        u16::from_be_bytes([self.a, self.flags.to_byte()])
    }

    /// L: sets A high and loads seven independent flags, discarding bit 3
    /// (decision-18 §4 POP rule; flags module sources and qualifications).
    pub fn set_psw(&mut self, value: u16) {
        let [a, flags] = value.to_be_bytes();
        self.a = a;
        self.flags = Flags::from_byte(flags);
    }
}

/// Persistent 8085 execution and boundary-sampler state (decision-12).
///
/// `Default` is a library choice: masks set, TRAP armed for a rising sample,
/// all other fields false. It is not a hardware power-up claim. The sampler
/// uses consecutive boundary levels, an API choice that can miss shorter
/// pulses (rule 2); only TRAP and RST 7.5 need prior levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Latches {
    /// Interrupt enable (205775-002 p. 5-17, EI/DI; rule 6).
    pub interrupt_enable: bool,
    /// Suppresses interrupt acceptance at the next boundary after EI or DI
    /// (205775-002 p. 5-17; decision-12 rule 6, unverified reading, TRAP
    /// included). Expires after the next completed instruction unless re-armed.
    /// Reset clearing it is unverified (rule 9).
    pub ei_delay: bool,
    /// Halt state (205775-002 §2.3.6 p. 2-16; rule 7).
    pub halted: bool,
    /// RST 5.5 disabled when true (205775-002 p. 5-18, SIM; rule 8).
    pub mask5_5: bool,
    /// RST 6.5 disabled when true (same page and rule).
    pub mask6_5: bool,
    /// RST 7.5 disabled when true (same page and rule).
    pub mask7_5: bool,
    /// RST 7.5 flip-flop, set on a rising edge even when masked; cleared by
    /// acceptance, SIM bit 4 or reset (205775-002 p. 5-18; rule 2).
    pub rst7_5_pending: bool,
    /// TRAP edge retained until eligible while high, or cleared by a low
    /// sample (rule 2; 231718-001 handbook p. 16-15, Fig. 4).
    pub trap_pending: bool,
    /// Ready for a new TRAP rising edge; acceptance disarms until a low
    /// sample (rule 2; 231718-001 handbook p. 16-15).
    pub trap_armed: bool,
    /// Interrupt enable saved at TRAP acceptance (rule 4;
    /// 231718-001 handbook p. 16-15; 205775-002 p. 5-17, RIM).
    pub trap_saved_interrupt_enable: bool,
    /// First RIM after TRAP must use the saved enable, then clear this flag
    /// (rule 8; same pages).
    pub trap_rim_pending: bool,
    /// SOD output latch (205775-002 p. 5-18, SIM; rules 8 and 9).
    pub sod: bool,
    /// Last boundary TRAP level, for edge detection (rule 2 API choice).
    pub last_trap: bool,
    /// Last boundary RST 7.5 level, for edge detection (rule 2 API choice).
    pub last_rst7_5: bool,
}

impl Default for Latches {
    fn default() -> Self {
        Self {
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
        }
    }
}
