//! The 8080 model type and its run loop (decision-2, decision-5, decision-6).
//!
//! It owns the register file and the execution latches. `run` and `step`
//! follow the boundary sequence of decision-5 item 5; fetched opcodes go
//! through the opcode map to their group's module. Eligible external
//! interrupts execute an acknowledged RST in place of an opcode fetch.

use super::opcodes::{Group, OPCODES};
use super::{arithmetic, branch, data_transfer, logical, stack_io_control};
use super::{Inputs, Latches, Registers};
use crate::progress::Progress;
use crate::{Budget, Bus, Exit, MemoryAccess, Report, Unsupported};

/// An Intel 8080 CPU: its registers and execution latches.
///
/// The embedder's bus and input lines are borrowed per call, never owned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct I8080 {
    registers: Registers,
    latches: Latches,
}

impl I8080 {
    /// A CPU with the given registers and execution latches.
    pub const fn new(registers: Registers, latches: Latches) -> Self {
        Self { registers, latches }
    }

    /// The register file.
    pub const fn registers(&self) -> &Registers {
        &self.registers
    }

    /// The register file, for the embedder to change between calls.
    pub fn registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    /// The execution latches.
    pub const fn latches(&self) -> &Latches {
        &self.latches
    }

    /// The execution latches, for the embedder to change between calls.
    pub fn latches_mut(&mut self) -> &mut Latches {
        &mut self.latches
    }

    /// Applies RESET (decision-5 item 9).
    ///
    /// Clears PC, interrupt enable and halt, preserving flags, A, the other
    /// working registers and SP: Intel 98-153B (September 1975), Ch. 2,
    /// "Halt Sequences" and "Start-up of the 8080 CPU", p. 2-13 (PDF 27),
    /// Figs. 2-12/2-13, pp. 2-14/2-15 (PDF 28/29); Ch. 5, 8080A pin
    /// definitions, RESET/INTE, p. 5-14 (PDF 76). Intel 9800301-04 (May
    /// 1981), Ch. 3, EI/HLT, pp. 3-23/3-24 also confirms PC and interrupt
    /// disable on reset and restart from halt.
    ///
    /// Clearing the EI delay is an unverified model choice: those RESET
    /// descriptions do not specify its effect on a pending EI. This does
    /// not initialize indeterminate power-up registers; they keep their
    /// supplied values (98-153B p. 2-13). No bus access or input poll occurs.
    pub fn reset(&mut self) {
        self.registers.pc = 0;
        self.latches.interrupt_enable = false;
        self.latches.ei_delay = false;
        self.latches.halted = false;
    }

    /// Executes instructions until `budget` is spent or another exit occurs
    /// (decision-2, decision-5).
    ///
    /// At each instruction boundary, in this order (decision-5 item 5):
    ///
    /// 1. If `budget.steps` steps or `budget.cycles` cycles have been spent,
    ///    return [`Exit::Budget`] with no poll and no bus access. A zero limit
    ///    executes nothing.
    /// 2. Call [`stop_requested`](crate::Control::stop_requested) and then
    ///    [`interrupt_requested`](Inputs::interrupt_requested), once each,
    ///    whatever their results and the CPU state. A stop request returns
    ///    [`Exit::Stop`].
    /// 3. If INT is high, interrupt enable is set and the EI delay has expired,
    ///    accept the interrupt, leaving halt and clearing interrupt enable.
    ///    Read [`MemoryAccess::InterruptAcknowledge`] at PC without incrementing
    ///    it. RST n pushes that PC and executes as this boundary's instruction
    ///    (one step, 11 states). Any other byte returns [`Exit::Unsupported`]
    ///    with `acknowledge` true, no progress and the boundary state restored
    ///    (decision-5 items 4, 6 and 8). No opcode fetch follows in this boundary.
    /// 4. If the CPU is halted, return [`Exit::Halt`] without fetching.
    /// 5. Read the opcode at PC as [`MemoryAccess::OpcodeFetch`] and execute
    ///    the instruction. An opcode that the 8080 manual does not document
    ///    (decision-3) returns
    ///    [`Exit::Unsupported`] with `acknowledge` false. That exit adds no
    ///    step and no cycle and changes no state: only the opcode read has
    ///    reached the bus.
    ///
    /// The report carries the steps and cycles (clock states) of the
    /// instructions completed by this call, including those before an exit
    /// other than the budget. An instruction once started completes, so the
    /// cycles may exceed `budget.cycles`. Both counters saturate.
    ///
    /// ```
    /// use x80_core::i8080::{Inputs, I8080};
    /// use x80_core::{Budget, Bus, Control, Exit, MemoryAccess};
    ///
    /// /// 64 KiB of zero bytes, which is NOP (00000000) on the 8080.
    /// struct Ram(Vec<u8>);
    ///
    /// impl Bus for Ram {
    ///     fn read(&mut self, address: u16, _kind: MemoryAccess) -> u8 {
    ///         self.0[usize::from(address)]
    ///     }
    ///     fn write(&mut self, address: u16, value: u8, _kind: MemoryAccess) {
    ///         self.0[usize::from(address)] = value;
    ///     }
    ///     fn input(&mut self, _port: u8) -> u8 {
    ///         0
    ///     }
    ///     fn output(&mut self, _port: u8, _value: u8) {}
    /// }
    /// impl Control for Ram {}
    /// impl Inputs for Ram {}
    ///
    /// let mut ram = Ram(vec![0; 0x1_0000]);
    /// let mut cpu = I8080::default();
    /// let report = cpu.run(&mut ram, Budget { steps: 10, cycles: 18 });
    /// // NOP takes 4 states: the fifth NOP starts at 16 cycles and overshoots.
    /// assert_eq!((report.steps, report.cycles), (5, 20));
    /// assert_eq!(report.exit, Exit::Budget);
    /// assert_eq!(cpu.registers().pc, 5);
    /// ```
    pub fn run<S: Bus + Inputs>(&mut self, system: &mut S, budget: Budget) -> Report {
        let mut progress = Progress::default();
        loop {
            // 1. Budget (decision-2): no poll and no bus access.
            if progress.reached(budget) {
                return progress.report(Exit::Budget);
            }
            // 2. The input lines, once each and in this order (decision-5
            // item 4).
            let stop = system.stop_requested();
            let interrupt = system.interrupt_requested();
            if stop {
                return progress.report(Exit::Stop);
            }
            // 3. Acknowledge replaces fetch, even while halted (decision-5
            // items 4/5). INT is a level; disabled requests are not retained.
            // 98-153B Ch. 2 Interrupt Sequences p. 2-11, Halt Sequences
            // p. 2-13; Ch. 5 INT/INTE p. 5-14.
            if interrupt && self.latches.interrupt_enable && !self.latches.ei_delay {
                match self.acknowledge(system) {
                    Ok(states) => progress.retire(states),
                    Err(unsupported) => return progress.report(Exit::Unsupported(unsupported)),
                }
                continue;
            }
            // 4. Halt: return at once, never spin (decision-2). The halt state
            // is left only by an interrupt or RESET (98-153B p. 2-13, "Halt
            // Sequences").
            if self.latches.halted {
                return progress.report(Exit::Halt);
            }
            // 5. Fetch and execute.
            match self.execute(system) {
                Ok((states, opcode)) => {
                    // decision-5 item 4; 98-153B Ch. 4 EI p. 4-14,
                    // 9800301-04 Ch. 3 EI p. 3-23: enable only after the
                    // following instruction completes. EI itself arms the
                    // delay; no exit without completion advances it.
                    // EI;EI rearms it: simplest consistent model, unverified
                    // by these pages. DI clears enable and delay itself.
                    if opcode != 0xFB {
                        self.latches.ei_delay = false;
                    }
                    progress.retire(states);
                }
                Err(unsupported) => return progress.report(Exit::Unsupported(unsupported)),
            }
        }
    }

    /// Executes at most one instruction: [`run`](Self::run) with the budget
    /// `{ steps: 1, cycles: u64::MAX }` (decision-5 item 8).
    pub fn step<S: Bus + Inputs>(&mut self, system: &mut S) -> Report {
        self.run(
            system,
            Budget {
                steps: 1,
                cycles: u64::MAX,
            },
        )
    }

    /// Boundary step 3 (decision-5 items 4, 6, 8). Intel 98-153B
    /// (September 1975), Ch. 5 INTE p. 5-14 (PDF 76): clear INTE at T1,
    /// before the acknowledge read. Ch. 2 Interrupt Sequences/Fig. 2-8,
    /// p. 2-11 (PDF 25): PC on the address bus, no increment; push that PC,
    /// high at SP-1 then low at SP-2. Halt Sequences p. 2-13 (PDF 27)
    /// permits waking from halt. The return PC is then the address after HLT
    /// (Intel 9800301-04, May 1981, Ch. 3 HLT p. 3-24, PDF 78).
    fn acknowledge<S: Bus>(&mut self, system: &mut S) -> Result<u8, Unsupported> {
        let pc = self.registers.pc;
        let boundary_latches = self.latches;
        self.latches.interrupt_enable = false;
        self.latches.halted = false;
        let opcode = system.read(pc, MemoryAccess::InterruptAcknowledge);
        // RST's 11NNN111 encoding: 98-153B Ch. 2 pp. 2-18/2-19;
        // 9800301-04 Ch. 3 RST pp. 3-54/3-55 (PDF 108/109).
        if opcode & 0xC7 != 0xC7 {
            // Deliberate departure from hardware: decision-5 item 6 restores
            // the boundary state for an unsupported acknowledge. Only the
            // acknowledge read has reached the bus; registers are untouched.
            self.latches = boundary_latches;
            return Err(Unsupported {
                pc,
                opcode,
                acknowledge: true,
            });
        }
        branch::restart(self, system, opcode, pc);
        // Fig. 2-8 p. 2-11 explicitly shows M1 T1..T5, M2 T1..T3,
        // M3 T1..T3: 5+3+3 = 11 states, including the acknowledge, with
        // no extra fetch. RST's table pp. 2-18/2-19 and Stack note 16
        // p. 2-20 agree; 9800301-04 Ch. 3 p. 3-55 gives 11 (8080).
        // Ch. 7 pp. 7-2/7-3 (PDF 186/187) confirms INTE reset and the
        // unincremented return PC. This is one instruction (item 8).
        Ok(11)
    }

    /// Boundary step 5: reads the opcode at PC and executes its instruction.
    ///
    /// Returns the states spent and opcode, or the unsupported exit with no state
    /// changed. Every documented opcode has mandatory state totals and an
    /// executor that completes it; only an undocumented byte is unsupported
    /// (decision-5 item 7's interim path is removed).
    fn execute<S: Bus>(&mut self, system: &mut S) -> Result<(u8, u8), Unsupported> {
        let pc = self.registers.pc;
        let opcode = system.read(pc, MemoryAccess::OpcodeFetch);
        let unsupported = Unsupported {
            pc,
            opcode,
            acknowledge: false,
        };
        // Not documented as an instruction by 98-153B (decision-3).
        let Some(instruction) = OPCODES[usize::from(opcode)] else {
            return Err(unsupported);
        };
        let path = match instruction.group {
            Group::DataTransfer => data_transfer::execute(self, system, opcode, instruction),
            Group::Arithmetic => arithmetic::execute(self, system, opcode, instruction),
            Group::Logical => logical::execute(self, system, opcode, instruction),
            Group::Branch => branch::execute(self, system, opcode, instruction),
            Group::StackIoControl => stack_io_control::execute(self, system, opcode, instruction),
        };
        Ok((instruction.states.of(path), opcode))
    }
}

impl Default for I8080 {
    /// All registers and flags zero, interrupts disabled, no EI delay pending
    /// and not halted.
    ///
    /// A library choice, not a hardware claim: 98-153B p. 2-13 says the
    /// register contents are indeterminate at power-up (decision-5 item 2).
    fn default() -> Self {
        Self::new(Registers::default(), Latches::default())
    }
}
