//! Seven stored 8085 flags and the PSW image (decision-18 §4).
//!
//! D = die analysis (8085; exact part unverified); S = documented by second
//! source (CA80C85B), unverified for Intel 8085; L = adopted library rule,
//! exact hardware effect unverified (decision-17/18; PROVENANCE §1).
//! Image D/S: W p. 145; Tundra 9000000.MD302.01 Fig. 2-9 p. 2-15;
//! independently KV circuit/bus drivers and FC storage (one die origin).
//! L unstated-effect policy (decision-18 §4): PUSH, data transfer, CMA,
//! CMC/STC, branches, calls/returns, RIM/SIM, DI/EI, NOP/HLT and ordinary
//! interrupt entry leave the independent V/K latches untouched.

/// The seven independent 8085 flags (decision-18 §4).
///
/// Evidence labels: D = die analysis (8085; exact part unverified);
/// S = documented by second source (CA80C85B), unverified for Intel 8085;
/// L = adopted library rule, exact hardware effect unverified.
///
/// `Default` clears all flags; reset preserves them (L construction/reset
/// rules, decision-18 §4 and decision-12 rule 9; no silicon guarantee).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Flags {
    /// Sign, PSW bit 7 (205775-002 p. 5-15, FLAG WORD).
    pub sign: bool,
    /// Zero, PSW bit 6 (same figure).
    pub zero: bool,
    /// K/X5, PSW bit 5: D/S image (decision-18 §4, W p. 145/T Fig. 2-9,
    /// KV circuit/FC storage). Updated arithmetic uses V XOR result sign;
    /// INX/DCX and POP have independent rules, so this is an independent latch.
    pub k: bool,
    /// Auxiliary carry, PSW bit 4 (same figure).
    pub auxiliary_carry: bool,
    /// Parity, PSW bit 2 (same figure).
    pub parity: bool,
    /// Signed overflow V, PSW bit 1: D/S image (decision-18 §4,
    /// W p. 145/T Fig. 2-9; KV circuit/FC storage). Individual update rules
    /// retain their D/S/L qualifications; POP can load this independently.
    pub overflow: bool,
    /// Carry, PSW bit 0 (same figure).
    pub carry: bool,
}

impl Flags {
    /// D/S: S Z K AC 0 P V CY (decision-18 §4, sources in module docs).
    pub(crate) const fn to_byte(self) -> u8 {
        ((self.sign as u8) << 7)
            | ((self.zero as u8) << 6)
            | ((self.k as u8) << 5)
            | ((self.auxiliary_carry as u8) << 4)
            | ((self.parity as u8) << 2)
            | ((self.overflow as u8) << 1)
            | self.carry as u8
    }

    /// L: POP/set_psw load seven independent flags and discard bit 3
    /// (decision-18 §4, following KV circuit's explicit hidden databus loads;
    /// FC storage/ALU bus_to_flags share that origin, exact hardware unverified).
    pub(crate) const fn from_byte(byte: u8) -> Self {
        Self {
            sign: byte & 0x80 != 0,
            zero: byte & 0x40 != 0,
            k: byte & 0x20 != 0,
            overflow: byte & 0x02 != 0,
            auxiliary_carry: byte & 0x10 != 0,
            parity: byte & 0x04 != 0,
            carry: byte & 0x01 != 0,
        }
    }
}

// Model-owned flag access (decision-16 rule 3): Intel 205775-002 (January 1983),
// §5.5 p. 5-3.
impl crate::intel::FlagRules for Flags {
    fn sign(self) -> bool {
        self.sign
    }

    fn zero(self) -> bool {
        self.zero
    }

    fn parity(self) -> bool {
        self.parity
    }

    fn carry(self) -> bool {
        self.carry
    }

    // AC access for DAA: 205775-002 (January 1983), §5.5 p. 5-3;
    // second witness: 9800301-04 (May 1981), Ch. 1 pp. 1-11/1-12.
    fn auxiliary_carry(self) -> bool {
        self.auxiliary_carry
    }

    // Carry-only updates: 205775-002 (January 1983) §5.6.3 pp. 5-11/5-12;
    // 9800301-04 (May 1981) Ch. 3 pp. 3-45/3-46, 3-49, 3-53/3-54,
    // 3-12/3-63. Witnesses: i8085_logical. Other stored flags survive.
    // Also DAD/INR/DCR: 205775-002 §5.6.2 pp. 5-8/5-9;
    // 9800301-04 Ch. 3 pp. 3-20/3-21/3-25. Witness: i8085_arithmetic.
    // L: CMA/CMC/STC and all other unstated effects preserve V/K
    // (decision-18 §4 documented-instruction table; PROVENANCE §1).
    fn set_carry(&mut self, carry: bool) {
        self.carry = carry;
    }

    // PSW image: 205775-002 (January 1983) §5.6.5 pp. 5-15/5-16;
    // second witness: 9800301-04 (May 1981) Ch. 3 pp. 3-43/3-45.
    // Hidden image D/S and input loading L: decision-18 §4, module sources.
    // Design: decision-16 rule 3; witnesses: i8085_registers and
    // i8085_stack_io_control. Delegate to the existing model conversion.
    fn to_byte(self) -> u8 {
        Flags::to_byte(self)
    }

    fn from_byte(byte: u8) -> Self {
        Flags::from_byte(byte)
    }

    // Model-owned ALU facts (decision-16 rule 3). The cited primitives in
    // alu retain their independent arithmetic and logical witnesses.
    fn result(value: u8, auxiliary_carry: bool, carry: bool) -> Self {
        super::alu::result_flags(value, auxiliary_carry, carry)
    }

    // Addition/INR flags: 205775-002 (January 1983), §5.5 p. 5-3;
    // §5.6.2 ADD/ADC/INR pp. 5-6 to 5-8.
    // Second witness: 9800301-04 (May 1981), Ch. 1 pp. 1-10 to 1-12,
    // Ch. 3 pp. 3-2 to 3-5, 3-25. Independent witnesses: i8085_arithmetic,
    // i8085_daa. DCR retains the model's subtract derivation below.
    fn add(left: u8, right: u8, carry: bool) -> (u8, Self) {
        super::alu::add(left, right, carry)
    }

    fn subtract(left: u8, right: u8, borrow: bool) -> (u8, Self) {
        super::alu::subtract(left, right, borrow)
    }

    fn and(a: u8, operand: u8) -> (u8, Self) {
        super::alu::and(a, operand)
    }

    // decision-19 pair-step hook; K D/S, V preservation L (decision-18 §4).
    fn pair_step(&mut self, old: u16, _result: u16, decrement: bool) {
        super::alu::pair_step(self, old, decrement);
    }

    // decision-19 rotate hook; V update / K preservation L (decision-18 §4).
    fn rotate(&mut self, opcode: u8, old_a: u8, _old_carry: bool, result: u8, _carry: bool) {
        super::alu::rotate(self, old_a, result, opcode & 8 != 0);
    }

    // decision-19 DAD hook; V S / die-reading reconciliation, K L (§4).
    fn dad(&mut self, left: u16, right: u16, result: u16) {
        super::alu::dad(self, left, right, result);
    }

    // decision-19 DAA hook; combined-correction V/K L (decision-18 §4).
    fn daa(&mut self, old_a: u8, correction: u8, result: u8) {
        super::alu::daa(self, old_a, correction, result);
    }
}
