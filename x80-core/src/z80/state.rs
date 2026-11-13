//! Model-owned state (decision-13 rule 1; decision-15 rules 2, 7, 10, 12).
//!
//! Zilog UM008011-0816 (revision 11, August 2016), CPU Register / Registers
//! pp. 2–3 (PDF 16–17); second witness 03-0029-01 (1977) §2.1 pp. 3–5,
//! Fig. 2.0-2 p. 4 (PDF 9–11). Pages checked as rendered images.
//! The primary's Fig. 2 repeats B' in C's alternate column; its p. 3 pair
//! text and the second witness's diagram identify BC'.

use super::Flags;

/// The Z80 programmer-visible registers (UM008011-0816 pp. 2–3).
///
/// Alternate registers use the `_alternate` suffix for the manual's prime.
/// `Default` is zero with all flags clear, a library construction choice,
/// not a hardware power-up claim (decision-15 rule 12).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Registers {
    /// Accumulator A.
    pub a: u8,
    /// F, stored as eight independent bits (decision-20 §4, library choice).
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
    /// Alternate accumulator A'.
    pub a_alternate: u8,
    /// Alternate F', with the same library image as F.
    pub flags_alternate: Flags,
    /// High byte of BC'.
    pub b_alternate: u8,
    /// Low byte of BC'.
    pub c_alternate: u8,
    /// High byte of DE'.
    pub d_alternate: u8,
    /// Low byte of DE'.
    pub e_alternate: u8,
    /// High byte of HL'.
    pub h_alternate: u8,
    /// Low byte of HL'.
    pub l_alternate: u8,
    /// Index register IX.
    pub ix: u16,
    /// Index register IY.
    pub iy: u16,
    /// Stack pointer.
    pub sp: u16,
    /// Program counter.
    pub pc: u16,
    /// Interrupt page address register.
    pub i: u8,
    /// Memory refresh register, including its programmable bit 7.
    pub r: u8,
}

// Pair layout: UM pp. 2–3; 03-0029-01 §2.1, Fig. 2.0-2 p. 4.
// Model-owned packing, including the decision-14 rule 5 F/F' library image.
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

    /// AF, with A high and all eight F bits low (decision-20 §4).
    pub const fn af(&self) -> u16 {
        u16::from_be_bytes([self.a, self.flags.to_byte()])
    }

    /// Sets A high and all eight independent F bits low (decision-20 §4).
    pub fn set_af(&mut self, value: u16) {
        let [a, flags] = value.to_be_bytes();
        self.a = a;
        self.flags = Flags::from_byte(flags);
    }

    /// BC', with B' high and C' low.
    pub const fn bc_alternate(&self) -> u16 {
        u16::from_be_bytes([self.b_alternate, self.c_alternate])
    }

    /// Sets B' high and C' low.
    pub fn set_bc_alternate(&mut self, value: u16) {
        [self.b_alternate, self.c_alternate] = value.to_be_bytes();
    }

    /// DE', with D' high and E' low.
    pub const fn de_alternate(&self) -> u16 {
        u16::from_be_bytes([self.d_alternate, self.e_alternate])
    }

    /// Sets D' high and E' low.
    pub fn set_de_alternate(&mut self, value: u16) {
        [self.d_alternate, self.e_alternate] = value.to_be_bytes();
    }

    /// HL', with H' high and L' low.
    pub const fn hl_alternate(&self) -> u16 {
        u16::from_be_bytes([self.h_alternate, self.l_alternate])
    }

    /// Sets H' high and L' low.
    pub fn set_hl_alternate(&mut self, value: u16) {
        [self.h_alternate, self.l_alternate] = value.to_be_bytes();
    }

    /// AF', with A' high and all eight F' bits low (decision-20 §4).
    pub const fn af_alternate(&self) -> u16 {
        u16::from_be_bytes([self.a_alternate, self.flags_alternate.to_byte()])
    }

    /// Sets A' high and all eight independent F' bits low (decision-20 §4).
    pub fn set_af_alternate(&mut self, value: u16) {
        let [a, flags] = value.to_be_bytes();
        self.a_alternate = a;
        self.flags_alternate = Flags::from_byte(flags);
    }
}

/// Z80 maskable interrupt mode (decision-15 rules 9/12).
///
/// Mode numbers: UM008011-0816 RESET p. 6; 03-0029-01 §8 pp. 56–57.
/// `Default` selects Mode 0 by library construction choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InterruptMode {
    /// Interrupt mode 0.
    #[default]
    Mode0,
    /// Interrupt mode 1.
    Mode1,
    /// Interrupt mode 2.
    Mode2,
}

/// Persistent Z80 execution and boundary-sampler state (decision-15).
///
/// `Default` clears every bool and selects IM 0. These are library
/// construction values, not a hardware power-up claim (rule 12).
/// The boundary loop owns sampling and expiry; group and interrupt
/// executors own their instruction-specific changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Latches {
    /// Maskable interrupt enable (UM Table 1 p. 18; rules 7/12).
    pub iff1: bool,
    /// Saved interrupt enable (UM Table 1 pp. 18–19; rules 7/12).
    pub iff2: bool,
    /// Selected maskable interrupt mode (rule 9).
    pub interrupt_mode: InterruptMode,
    /// One-instruction inhibition after EI (UM p. 18; rule 7).
    /// EI/EI, DI and NMI interactions are unverified readings in rule 7.
    pub ei_delay: bool,
    /// Halt state (UM HALT p. 181; rule 7).
    pub halted: bool,
    /// Last sampled NMI request, true when asserted (pin Low). Consecutive
    /// false/true samples detect the negative edge (UM p. 6; rule 2 API
    /// sampler). Initial/reset false is a library choice; short pulses
    /// wholly between boundary polls can be missed.
    pub nmi_asserted: bool,
    /// Pending LD A,I / LD A,R parity-note correction (UM pp. 94–95).
    /// Mapping the note to acceptance at the next boundary, for either
    /// interrupt kind, is an unverified reading (rule 10). Reset clears
    /// this bookkeeping by library choice (rule 12).
    pub parity_note_pending: bool,
}
