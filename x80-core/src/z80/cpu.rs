//! Ownership, boundary sequence and prefix dispatch (decisions 13–15).
//!
//! The dispatcher advances PC past the fetched prefix/key bytes. Executors
//! own all remaining operand reads and PC changes; INDEX displacement is
//! fetched by the executor with `fetch_operand`, before any immediate byte.
//! Only INDEX_CB carries an already-fetched displacement in `Decoded`.
//! Complete instruction costs are retired once, never once per prefix.
//! Repeating blocks return Path::Taken and rewind PC by two in the executor
//! (decision-15 rule 11; UM008011-0816 rev. 11, LDIR p. 132;
//! 03-0029-01, 1977, Table 7.0-3 p. 46). No loop special case is needed.

use super::opcodes::{self, Group, Instruction};
use super::{
    acknowledge, arithmetic16, arithmetic8, bit_set_reset_test, call_return, exchange_block_search,
    general_arithmetic_cpu_control, input_output, jump, load16, load8, nmi, rotate_shift, Inputs,
    Latches, Registers,
};
use crate::decode::Path;
use crate::progress::Progress;
use crate::{Budget, Bus, Exit, MemoryAccess, Report, Unsupported};

/// Model-owned table context; no prefix semantics enter decode.rs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Table {
    Main,
    Cb,
    Ed,
    Index,
    IndexCb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IndexRegister {
    Ix,
    Iy,
}

/// Final context shared by all eleven executors (decision-13 rules 3–4).
/// PC is already past the fetched key byte, including d/op for INDEX_CB.
/// `instruction.length` is the WHOLE length, not a remaining-byte count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Decoded {
    pub(super) table: Table,
    pub(super) opcode: u8,
    pub(super) index: Option<IndexRegister>,
    /// Only INDEX_CB. INDEX executors fetch d themselves when applicable.
    pub(super) displacement: Option<u8>,
    pub(super) start_pc: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Interrupt {
    Nmi,
    Int,
}

/// A Z80 CPU's registers and execution latches (decision-13 rule 1).
///
/// `Default` uses library construction values, not a hardware power-up
/// state (decision-15 rule 12). Only NOP executes during group staging.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Z80 {
    registers: Registers,
    latches: Latches,
}

impl Z80 {
    /// Owns the supplied state unchanged.
    pub const fn new(registers: Registers, latches: Latches) -> Self {
        Self { registers, latches }
    }

    /// The register file.
    pub const fn registers(&self) -> &Registers {
        &self.registers
    }

    /// The register file, for changes between calls.
    pub fn registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    /// The execution and sampler latches.
    pub const fn latches(&self) -> &Latches {
        &self.latches
    }

    /// The execution and sampler latches, for changes between calls.
    pub fn latches_mut(&mut self) -> &mut Latches {
        &mut self.latches
    }

    /// Applies RESET (decision-15 rule 12).
    ///
    /// Clears IFF1/IFF2, PC, I and R; selects IM 0 (Zilog UM008011-0816,
    /// revision 11, August 2016, RESET p. 6 and Table 1 p. 18, PDF 20/32;
    /// 03-0029-01, 1977, RESET p. 9 and Fig. 8.0-1 p. 56, PDF 15/62).
    /// Leaves halt (UM HALT p. 181, PDF 195).
    ///
    /// Preserving SP, both general/AF register sets, IX, IY and flags is a
    /// library choice: those reset descriptions do not specify their values.
    /// Clearing EI delay and the parity-note latch and setting NMI history
    /// to not asserted are library bookkeeping choices, not hardware claims.
    /// No bus access or poll occurs, and no elapsed cycles are reported.
    pub fn reset(&mut self) {
        self.registers.pc = 0;
        self.registers.i = 0;
        self.registers.r = 0;
        self.latches = Latches::default();
    }

    /// Runs until a budget limit or another exit (decision-15 rules 1–3).
    ///
    /// Each boundary checks budget, polls stop/NMI/INT once each in that
    /// order, honors stop, samples the NMI edge, selects NMI before eligible
    /// INT, checks halt, then fetches and executes. Stop and Unsupported
    /// preserve pre-sampling CPU state; Halt keeps samples. Bus reads and
    /// their device side effects cannot be undone. Budget does no polling.
    /// Started instructions complete with their whole cost, even on overshoot.
    ///
    /// During staging, documented encodings in groups not yet landed return
    /// Unsupported with exactly their table's hole reads (decision-5 item 7;
    /// decision-14 rule 4). Interrupt seams currently decline acceptance.
    pub fn run<S: Bus + Inputs>(&mut self, system: &mut S, budget: Budget) -> Report {
        let mut progress = Progress::default();
        loop {
            if progress.reached(budget) {
                return progress.report(Exit::Budget);
            }
            let stop = system.stop_requested();
            let nmi = system.nmi();
            let int = system.int();
            if stop {
                return progress.report(Exit::Stop);
            }
            // Save AFTER all polls and BEFORE sampling (decision-15 rule 2;
            // decision-12 rule 2 clarification). Restore even an INT refusal.
            let boundary_registers = self.registers;
            let boundary_latches = self.latches;
            let nmi_edge = self.sample_nmi(nmi);
            match self.pending_interrupt(nmi_edge, int) {
                Some(Interrupt::Nmi) => {
                    if let Some(states) = nmi::accept(self, system) {
                        progress.retire(states);
                        continue;
                    }
                }
                Some(Interrupt::Int) => {
                    if let Some(result) = acknowledge::accept(self, system) {
                        match result {
                            Ok(states) => progress.retire(states),
                            Err(unsupported) => {
                                self.registers = boundary_registers;
                                self.latches = boundary_latches;
                                return progress.report(Exit::Unsupported(unsupported));
                            }
                        }
                        continue;
                    }
                }
                None => {}
            }
            if self.latches.halted {
                return progress.report(Exit::Halt);
            }
            // Consume the OLD delay before execute, so EI can re-arm it.
            // NMI acceptance bypasses this advance (rule 7's unverified
            // EI/NMI interaction). Expire the old parity note, keeping P/V
            // itself; LD A,I/R may set a new note (rule 10, unverified).
            self.latches.ei_delay = false;
            self.latches.parity_note_pending = false;
            match self.execute(system) {
                Ok(states) => progress.retire(states),
                Err(unsupported) => {
                    self.registers = boundary_registers;
                    self.latches = boundary_latches;
                    return progress.report(Exit::Unsupported(unsupported));
                }
            }
        }
    }

    /// Executes at most one instruction or one accepted interrupt, with an
    /// unlimited cycle budget (decision-5 item 8; decision-15 rule 3).
    pub fn step<S: Bus + Inputs>(&mut self, system: &mut S) -> Report {
        self.run(
            system,
            Budget {
                steps: 1,
                cycles: u64::MAX,
            },
        )
    }

    /// API boundary sampler, not within-instruction edge capture (rule 2).
    /// False-to-true represents the active-low negative edge: UM008011-0816
    /// rev. 11, Pin Functions p. 6; 03-0029-01 (1977), §3 p. 8.
    fn sample_nmi(&mut self, asserted: bool) -> bool {
        let edge = asserted && !self.latches.nmi_asserted;
        self.latches.nmi_asserted = asserted;
        edge
    }

    /// NMI priority and independence of IFF1/delay: decision-15 rules 3/7;
    /// UM pp. 6, 14, 183; 03-0029-01 §3 p. 8, §4 p. 18, §8 p. 55.
    fn pending_interrupt(&self, nmi_edge: bool, int: bool) -> Option<Interrupt> {
        if nmi_edge {
            Some(Interrupt::Nmi)
        } else if int && self.latches.iff1 && !self.latches.ei_delay {
            Some(Interrupt::Int)
        } else {
            None
        }
    }

    /// Fetch one remaining instruction-stream operand and advance PC.
    /// INDEX executors call this for d BEFORE any immediate operand; no d
    /// is present for pair/index-only forms. INDEX_CB d is already decoded.
    /// OperandFetch is an API distinction (rule 5); displacement: UM p. 3,
    /// BIT b,(IX+d) p. 247; 03-0029-01 §2 p. 4.
    /// Wrapping is derived from the 16-bit PC width (UM p. 3).
    pub(super) fn fetch_operand<S: Bus>(&mut self, system: &mut S) -> u8 {
        let byte = system.read(self.registers.pc, MemoryAccess::OperandFetch);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        byte
    }

    /// One M1 refresh increment; also called ONCE by NMI after its discarded
    /// fetch and by INT after its acknowledge. Preserve programmable bit 7.
    /// Decision-15 rule 6: UM Registers p. 3; 03-0029-01 §2 p. 4, §3 p. 7.
    /// Per-M1 count is derived, including the interrupt fetches (UM Figs 9/10
    /// pp. 13–14; TM §4 pp. 16, 18). No refresh bus access is exposed.
    pub(super) fn increment_r(&mut self) {
        self.registers.r = (self.registers.r & 0x80) | (self.registers.r.wrapping_add(1) & 0x7F);
    }

    /// Acceptance seams call this ONLY on successful acceptance, after all
    /// refusal validation and before service. Do not consume the EI delay.
    /// Decision-15 rule 10's unverified boundary interpretation of the note:
    /// UM LD A,I / LD A,R pp. 94–95; 03-0002-01 (January 1978) pp. 48–49.
    #[allow(
        dead_code,
        reason = "final helper for separately landing interrupt seams"
    )]
    pub(super) fn consume_parity_note(&mut self) {
        if self.latches.parity_note_pending {
            self.registers.flags.parity_overflow = false;
            self.latches.parity_note_pending = false;
        }
    }

    fn fetch_opcode<S: Bus>(&mut self, system: &mut S) -> u8 {
        let byte = system.read(self.registers.pc, MemoryAccess::OpcodeFetch);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        self.increment_r();
        byte
    }

    /// Only prefix/key bytes are read here. Layouts: decision-13 rules 3–4;
    /// UM Table 6 p. 42, RL m p. 221; TM Tables 7.0-1/7 pp. 44/50.
    /// DD/FD CB d op uses two M1s and two operand reads: decision-15 rule 5's
    /// UNVERIFIED fourth-byte classification (UM BIT p. 247; TM §3 p. 7).
    /// Diagnostics are assembled without scanning past the refused key.
    fn fetch_decode<S: Bus>(&mut self, system: &mut S) -> (Decoded, Unsupported) {
        let start_pc = self.registers.pc;
        let first = self.fetch_opcode(system);
        let mut decoded = Decoded {
            table: Table::Main,
            opcode: first,
            index: None,
            displacement: None,
            start_pc,
        };
        let mut unsupported = Unsupported {
            pc: start_pc,
            opcode: first,
            acknowledge: false,
            preceding: [0; 3],
            preceding_len: 0,
        };
        match first {
            0xCB | 0xED => {
                decoded.table = if first == 0xCB { Table::Cb } else { Table::Ed };
                decoded.opcode = self.fetch_opcode(system);
                unsupported.preceding = [first, 0, 0];
                unsupported.preceding_len = 1;
            }
            0xDD | 0xFD => {
                decoded.table = Table::Index;
                decoded.index = Some(if first == 0xDD {
                    IndexRegister::Ix
                } else {
                    IndexRegister::Iy
                });
                decoded.opcode = self.fetch_opcode(system);
                unsupported.preceding = [first, 0, 0];
                unsupported.preceding_len = 1;
                if decoded.opcode == 0xCB {
                    decoded.table = Table::IndexCb;
                    let displacement = self.fetch_operand(system);
                    decoded.displacement = Some(displacement);
                    decoded.opcode = self.fetch_operand(system);
                    unsupported.preceding = [first, 0xCB, displacement];
                    unsupported.preceding_len = 3;
                }
            }
            _ => {}
        }
        unsupported.opcode = decoded.opcode;
        (decoded, unsupported)
    }

    fn execute<S: Bus>(&mut self, system: &mut S) -> Result<u8, Unsupported> {
        let (decoded, unsupported) = self.fetch_decode(system);
        let table = match decoded.table {
            Table::Main => &opcodes::MAIN,
            Table::Cb => &opcodes::CB,
            Table::Ed => &opcodes::ED,
            Table::Index => &opcodes::INDEX,
            Table::IndexCb => &opcodes::INDEX_CB,
        };
        let Some(instruction) = &table[usize::from(decoded.opcode)] else {
            return Err(unsupported);
        };
        // ONE group-owned switch enables each executor. No group bead needs
        // to edit this dispatcher. General control admits NOP during staging.
        let landed = match instruction.group {
            Group::Load8 => load8::LANDED,
            Group::Load16 => load16::LANDED,
            Group::ExchangeBlockSearch => exchange_block_search::LANDED,
            Group::Arithmetic8 => arithmetic8::LANDED,
            Group::GeneralArithmeticCpuControl => {
                general_arithmetic_cpu_control::is_landed(&decoded)
            }
            Group::Arithmetic16 => arithmetic16::LANDED,
            Group::RotateShift => rotate_shift::LANDED,
            Group::BitSetResetTest => bit_set_reset_test::LANDED,
            Group::Jump => jump::LANDED,
            Group::CallReturn => call_return::LANDED,
            Group::InputOutput => input_output::LANDED,
        };
        if !landed {
            return Err(unsupported);
        }
        let path = self.execute_group(system, instruction, &decoded);
        Ok(instruction.states.of(path))
    }

    fn execute_group<S: Bus>(
        &mut self,
        system: &mut S,
        instruction: &Instruction,
        decoded: &Decoded,
    ) -> Path {
        match instruction.group {
            Group::Load8 => load8::execute(self, system, instruction, decoded),
            Group::Load16 => load16::execute(self, system, instruction, decoded),
            Group::ExchangeBlockSearch => {
                exchange_block_search::execute(self, system, instruction, decoded)
            }
            Group::Arithmetic8 => arithmetic8::execute(self, system, instruction, decoded),
            Group::GeneralArithmeticCpuControl => {
                general_arithmetic_cpu_control::execute(self, system, instruction, decoded)
            }
            Group::Arithmetic16 => arithmetic16::execute(self, system, instruction, decoded),
            Group::RotateShift => rotate_shift::execute(self, system, instruction, decoded),
            Group::BitSetResetTest => {
                bit_set_reset_test::execute(self, system, instruction, decoded)
            }
            Group::Jump => jump::execute(self, system, instruction, decoded),
            Group::CallReturn => call_return::execute(self, system, instruction, decoded),
            Group::InputOutput => input_output::execute(self, system, instruction, decoded),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Decoded, IndexRegister, Interrupt, Table, Z80};
    use crate::{Bus, MemoryAccess};

    // A fixed trace suffices for decoder-only seams. It accepts exactly four
    // instruction bytes starting at FFFEh; expectations below are literals.
    struct FetchBus {
        bytes: [u8; 4],
        calls: [(u16, MemoryAccess); 4],
        count: usize,
    }

    impl FetchBus {
        fn new(bytes: [u8; 4]) -> Self {
            Self {
                bytes,
                calls: [(0, MemoryAccess::Data); 4],
                count: 0,
            }
        }
    }

    impl Bus for FetchBus {
        fn read(&mut self, address: u16, kind: MemoryAccess) -> u8 {
            self.calls[self.count] = (address, kind);
            self.count += 1;
            self.bytes[usize::from(address.wrapping_sub(0xFFFE))]
        }
        fn write(&mut self, _: u16, _: u8, _: MemoryAccess) {
            panic!("unexpected write");
        }
        fn input(&mut self, _: u8) -> u8 {
            panic!("unexpected input");
        }
        fn output(&mut self, _: u8, _: u8) {
            panic!("unexpected output");
        }
    }

    // Decision-13 rules 3–4: UM008011-0816 rev. 11, Table 6 p. 42,
    // RL m p. 221; TM 03-0029-01 (1977), Tables 7.0-1/7 pp. 44/50.
    // R: decision-15 rule 6, UM p. 3; TM §2 p. 4, §3 p. 7. Fourth-byte
    // indexed-CB access is rule 5's UNVERIFIED reading, not a stated fact.
    #[test]
    fn decoded_context_pc_and_refresh_are_final_for_all_table_forms() {
        use MemoryAccess::{OpcodeFetch as Op, OperandFetch as Arg};
        for (bytes, table, opcode, index, displacement, pc, r, calls) in [
            (
                [0x00, 0xA5, 0x5A, 0x3C],
                Table::Main,
                0,
                None,
                None,
                0xFFFF,
                0x80,
                &[(0xFFFE, Op)][..],
            ),
            (
                [0xCB, 0x00, 0xA5, 0x5A],
                Table::Cb,
                0,
                None,
                None,
                0,
                0x81,
                &[(0xFFFE, Op), (0xFFFF, Op)][..],
            ),
            (
                [0xED, 0x44, 0xA5, 0x5A],
                Table::Ed,
                0x44,
                None,
                None,
                0,
                0x81,
                &[(0xFFFE, Op), (0xFFFF, Op)][..],
            ),
            (
                [0xDD, 0x21, 0xA5, 0x5A],
                Table::Index,
                0x21,
                Some(IndexRegister::Ix),
                None,
                0,
                0x81,
                &[(0xFFFE, Op), (0xFFFF, Op)][..],
            ),
            (
                [0xFD, 0x46, 0x80, 0x5A],
                Table::Index,
                0x46,
                Some(IndexRegister::Iy),
                None,
                0,
                0x81,
                &[(0xFFFE, Op), (0xFFFF, Op)][..],
            ),
            (
                [0xDD, 0xCB, 0x80, 0x46],
                Table::IndexCb,
                0x46,
                Some(IndexRegister::Ix),
                Some(0x80),
                2,
                0x81,
                &[(0xFFFE, Op), (0xFFFF, Op), (0, Arg), (1, Arg)][..],
            ),
            (
                [0xFD, 0xCB, 0x7F, 0x06],
                Table::IndexCb,
                0x06,
                Some(IndexRegister::Iy),
                Some(0x7F),
                2,
                0x81,
                &[(0xFFFE, Op), (0xFFFF, Op), (0, Arg), (1, Arg)][..],
            ),
        ] {
            let mut cpu = Z80::default();
            cpu.registers.pc = 0xFFFE;
            cpu.registers.r = 0xFF;
            let mut bus = FetchBus::new(bytes);
            let (decoded, _) = cpu.fetch_decode(&mut bus);
            assert_eq!(
                decoded,
                Decoded {
                    table,
                    opcode,
                    index,
                    displacement,
                    start_pc: 0xFFFE
                }
            );
            assert_eq!((cpu.registers.pc, cpu.registers.r), (pc, r));
            assert_eq!(&bus.calls[..bus.count], calls);
        }
    }

    // Decision-15 rule 5, UM LD (IX+d),n p. 87; TM Table 7.0-1 p. 44:
    // prefix/key precede d then n. OperandFetch itself is an API distinction.
    #[test]
    fn index_executor_fetches_displacement_then_immediate_without_refresh() {
        let mut cpu = Z80::default();
        cpu.registers.pc = 0xFFFE;
        cpu.registers.r = 0xFF;
        let mut bus = FetchBus::new([0xDD, 0x36, 0x80, 0xA5]);
        let (decoded, _) = cpu.fetch_decode(&mut bus);
        assert_eq!(decoded.displacement, None);
        assert_eq!(cpu.registers.pc, 0);
        assert_eq!(cpu.fetch_operand(&mut bus), 0x80);
        assert_eq!(cpu.fetch_operand(&mut bus), 0xA5);
        assert_eq!((cpu.registers.pc, cpu.registers.r), (2, 0x81));
        assert_eq!(
            bus.calls,
            [
                (0xFFFE, MemoryAccess::OpcodeFetch),
                (0xFFFF, MemoryAccess::OpcodeFetch),
                (0, MemoryAccess::OperandFetch),
                (1, MemoryAccess::OperandFetch),
            ]
        );
    }

    // Decision-15 rules 2–3/7: UM Pin Functions p. 6, HALT Exit p. 14,
    // EI p. 183; TM §3 p. 8, §4 p. 18, §8 p. 55. Selection and committed
    // sampler history are tested independently of acceptance staging.
    #[test]
    fn nmi_edges_rearm_and_have_priority_independent_of_iff_and_delay() {
        let mut cpu = Z80::default();
        for iff1 in [false, true] {
            for delay in [false, true] {
                cpu.latches.iff1 = iff1;
                cpu.latches.ei_delay = delay;
                assert_eq!(cpu.pending_interrupt(true, true), Some(Interrupt::Nmi));
                assert_eq!(cpu.pending_interrupt(true, false), Some(Interrupt::Nmi));
                assert_eq!(
                    cpu.pending_interrupt(false, true),
                    if iff1 && !delay {
                        Some(Interrupt::Int)
                    } else {
                        None
                    }
                );
                assert_eq!(cpu.pending_interrupt(false, false), None);
            }
        }
        assert!(cpu.sample_nmi(true));
        assert!(!cpu.sample_nmi(true));
        assert!(!cpu.sample_nmi(false));
        assert!(cpu.sample_nmi(true));
    }

    // Decision-15 rule 10: UNVERIFIED interpretation of UM LD A,I/R's
    // interrupt note pp. 94–95; ALPM 03-0002-01 (Jan. 1978) pp. 48–49.
    #[test]
    fn successful_interrupt_helper_consumes_only_pending_parity_note() {
        for pending in [false, true] {
            let mut cpu = Z80::default();
            cpu.registers.flags = super::super::Flags::from_byte(0xD7);
            cpu.latches.parity_note_pending = pending;
            cpu.latches.ei_delay = true;
            let mut expected = cpu.clone();
            expected.registers.flags.parity_overflow = !pending;
            expected.latches.parity_note_pending = false;
            cpu.consume_parity_note();
            assert_eq!(cpu, expected);
        }
    }
}
