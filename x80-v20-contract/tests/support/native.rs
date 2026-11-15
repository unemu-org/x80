//! Mock operations, not native instruction execution. The only bytes the mock
//! decodes are ED FD and ED ED nn after an `EdEscape` hand-back (x86jit:
//! `I8080Handback::EdEscape`; option B).
//! Design: decision-9 and its amendments / plan §3. Every NEC citation here
//! means the pinned uPD70108/uPD70116 User's Manual, Stock No. 500350,
//! October 1986.

use super::assumptions;
use x80_core::i8080::{Latches, Registers, I8080};

// NEC p. 3-3 PSW diagram; p. 8-1 Fig. 8-2 (IE and BRK).
pub const IE: u16 = 1 << 9;
pub const BRK: u16 = 1 << 8;
pub const MD: u16 = 1 << 15;
const DIR: u16 = 1 << 10;

/// The native context plus the two 8080 latches the native core keeps for
/// the view exchange (decision-9 amendment, 2026-11-15). IE is PSW bit 9.
/// `ei_delay` and `halted` are stored verbatim from each copy-out and are
/// the only source of x80's latches at each copy-in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Native {
    pub aw: u16,
    pub bw: u16,
    pub cw: u16,
    pub dw: u16,
    pub bp: u16,
    pub sp: u16,
    pub ix: u16,
    pub iy: u16,
    pub ps: u16,
    pub ss: u16,
    pub ds0: u16,
    pub ds1: u16,
    pub pc: u16,
    pub psw: u16,
    pub ei_delay: bool,
    pub halted: bool,
}

/// The native core's typed refusal of BRKEM with BRK = 1, before any effect
/// (decision-9 amendment, 2026-11-15, following the principle of decision-23
/// §3 (x86jit)). NEC p. 6-3's general single-step rule covers BRKEM
/// literally, but no page settles how it combines with BRKEM's switch into
/// emulation mode and with R2's BRK clear (ASSUMED). An interface rule, not a
/// hardware fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SingleStepRefused;

/// What the native core does with an `EdEscape` hand-back (option B).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escape {
    /// ED FD: RETEM completed; emulation ended.
    Retem,
    /// ED ED nn: CALLN completed; a native routine is entered.
    Calln,
    /// Any other ED form: the native core's unknown instruction. Nothing
    /// changes and MD stays 0.
    Unknown { bytes: [u8; 2] },
}

impl Native {
    /// Fourteen words; neither 8080 latch is pending.
    pub fn from_words(w: &[u16]) -> Self {
        let &[aw, bw, cw, dw, bp, sp, ix, iy, ps, ss, ds0, ds1, pc, psw] = w else {
            panic!("native state needs fourteen words");
        };
        Self {
            aw,
            bw,
            cw,
            dw,
            bp,
            sp,
            ix,
            iy,
            ps,
            ss,
            ds0,
            ds1,
            pc,
            psw,
            ei_delay: false,
            halted: false,
        }
    }

    /// The view exchange into x80. NEC p. 8-1 Fig. 8-1: AL=A, BW=HL, CW=BC,
    /// DW=DE, BP=SP; Fig. 8-2 and p. 9-1: IE. Registers and every latch come
    /// from this model alone; the adapter keeps no 8080 state of its own.
    pub fn copy_in(&self) -> I8080 {
        let mut r = Registers {
            sp: self.bp,
            pc: self.pc,
            ..Registers::default()
        };
        r.set_psw(((self.aw & 0xFF) << 8) | (self.psw & 0xFF));
        r.set_hl(self.bw);
        r.set_bc(self.cw);
        r.set_de(self.dw);
        // Exhaustive on purpose: a new x80 latch must be added to the view.
        let latches = Latches {
            interrupt_enable: self.psw & IE != 0,
            ei_delay: self.ei_delay,
            halted: self.halted,
        };
        I8080::new(r, latches)
    }

    /// The view exchange out of x80, at every x80 exit. NEC p. 8-1
    /// Figs 8-1/8-2. AH, segments and native-only state stay untouched; the
    /// EI delay and halted are stored verbatim.
    pub fn copy_out(&mut self, cpu: &I8080) {
        let r = cpu.registers();
        self.aw = (self.aw & 0xFF00) | u16::from(r.a);
        self.bw = r.hl();
        self.cw = r.bc();
        self.dw = r.de();
        self.bp = r.sp;
        self.pc = r.pc;
        self.psw = (self.psw & !(0xFF | IE)) | (r.psw() & 0xFF);
        let latches = cpu.latches();
        if latches.interrupt_enable {
            self.psw |= IE;
        }
        self.ei_delay = latches.ei_delay;
        self.halted = latches.halted;
    }

    /// NEC p. 12-169 / p. 8-2: push PSW, PS, PC; MD=0; vector. IE and BRK
    /// follow the BRKEM/CALLN ASSUMPTION (R2: cleared, as on p. 6-1).
    /// `pc` is already the native continuation supplied by the caller.
    /// BRK = 1 is refused first, with no effect on state or memory.
    pub fn brkem(&mut self, memory: &mut Memory, vector: u8) -> Result<(), SingleStepRefused> {
        assert_ne!(self.psw & MD, 0, "BRKEM requires native mode");
        if self.psw & BRK != 0 {
            return Err(SingleStepRefused);
        }
        self.push_frame(memory, "brkem");
        self.psw &= !MD;
        if assumptions::BRKEM_CALLN_CLEAR_IE_BRK {
            self.psw &= !(IE | BRK);
        }
        self.vector(memory, vector);
        Ok(())
    }

    /// Option B (decision-9 amendment, 2026-11-15): x80 stopped on ED at
    /// PS:PC and the adapter handed back `EdEscape`. As its next unit the
    /// native core fetches ED at PS:PC again from its own memory, so these
    /// reads never reach x80's bus: ED FD is RETEM (p. 12-177), ED ED nn is
    /// CALLN (p. 12-176), each one native step. Frames and vectors are read
    /// from memory now. Any other ED form changes nothing.
    pub fn escape(&mut self, memory: &mut Memory) -> Escape {
        assert_eq!(self.psw & MD, 0, "an escape needs emulation mode");
        assert!(!self.halted, "a halted 8080 core cannot stop on an opcode");
        let first = memory.read(self.ps, self.pc);
        assert_eq!(first, 0xED, "x80 stopped on ED at PS:PC");
        let second = memory.read(self.ps, offset_add(self.pc, 1));
        match second {
            0xFD => {
                // NEC p. 12-177: restore original native PSW, discarding
                // emulated flags; registers carry out.
                self.return_frame(memory);
                self.leave_emulation();
                Escape::Retem
            }
            0xED => {
                // NEC p. 12-176; the saved continuation is ASSUMED.
                let vector = memory.read(self.ps, offset_add(self.pc, 2));
                self.pc = offset_add(self.pc, i32::from(assumptions::CALLN_PC_ADVANCE));
                self.enter_service(memory, "calln", vector);
                Escape::Calln
            }
            second => Escape::Unknown {
                bytes: [first, second],
            },
        }
    }

    /// INT/NMI: NEC p. 8-4 ("same as in native mode") and p. 6-1; CALLN: p. 12-176. Saved
    /// MD=0, then MD=1, vector into a native routine. INT and NMI clear IE
    /// and BRK as read on p. 6-1; for CALLN that is the BRKEM/CALLN
    /// ASSUMPTION (R2), since p. 12-176 lists only MD.
    pub fn enter_service(&mut self, memory: &mut Memory, source: &'static str, vector: u8) {
        assert_eq!(self.psw & MD, 0);
        self.push_frame(memory, source);
        let clear = match source {
            "int" | "nmi" => true,
            "calln" => assumptions::BRKEM_CALLN_CLEAR_IE_BRK,
            other => panic!("unknown service source {other}"),
        };
        if clear {
            self.psw &= !(IE | BRK);
        }
        if source == "calln" && assumptions::CALLN_CONSUMES_EI_DELAY {
            self.ei_delay = false;
        }
        // NEC pp. 8-2, 9-1: service ends standby; RETI returns after HLT.
        self.halted = false;
        self.psw |= MD;
        self.vector(memory, vector);
    }

    // NEC p. 6-1 vector layout: low-address word PC, high-address word PS.
    fn vector(&mut self, memory: &Memory, vector: u8) {
        let offset = u16::from(vector) * 4;
        self.pc = memory.word(0, offset);
        self.ps = memory.word(0, offset + 2);
    }

    // NEC p. 6-1, p. 12-169, p. 12-176; SS:SP backing p. 10-1.
    // Little-endian word placement: NEC p. 4-1 Fig. 4-2. This log records
    // architectural word writes, not byte bus timing or prefetch behavior.
    fn push_frame(&mut self, memory: &mut Memory, source: &'static str) {
        for value in [self.psw, self.ps, self.pc] {
            self.sp = offset_add(self.sp, -2);
            memory.put_word(self.ss, self.sp, value);
        }
        memory.frames.push(Frame {
            source: source.into(),
            ss: self.ss,
            sp: self.sp,
            words: [self.pc, self.ps, self.psw],
        });
    }

    /// RETI (p. 12-168) in an excursion from emulation, or RETEM
    /// (p. 12-177): PC, PS, full PSW restored, SP += 6.
    /// The original BRKEM frame restores native flags on RETEM.
    pub fn return_frame(&mut self, memory: &Memory) {
        self.pc = memory.word(self.ss, self.sp);
        self.ps = memory.word(self.ss, offset_add(self.sp, 2));
        self.psw = memory.word(self.ss, offset_add(self.sp, 4));
        self.sp = offset_add(self.sp, 6);
    }

    /// Fixture policy after RETEM or RESET: no 8080 state remains, so the
    /// mock drops both latches. Not a hardware claim.
    fn leave_emulation(&mut self) {
        self.ei_delay = false;
        self.halted = false;
    }

    /// Test payload, deliberately no x86 bytes/execution. Changes mapped
    /// and native-only registers and live flags before the RETI operation.
    pub fn mock_routine(&mut self, saved_psw: u16) {
        self.aw = (self.aw & 0xFF00) | 0xC7;
        self.cw = 0xABCD;
        self.ix = 0xDEAD;
        // Fixture policy: invert CY, P, AC, Z, S, IE, DIR and V relative to
        // the saved PSW (NEC p. 3-3 diagram). MD stays native, hence differs
        // from saved MD=0; reserved bits stay untouched. BRK stays as service
        // entry left it (0): a RETI that starts with BRK = 1 and re-enters
        // 8080 mode is refused natively (decision-9 amendment, 2026-11-15).
        self.psw = (saved_psw ^ 0x0ED5) | MD;
    }

    /// NEC p. 8-4 RESET aborts emulation; p. 7-1 lists defined native
    /// reset values. All other registers/flags are undefined: retaining
    /// them is solely a deterministic fixture policy, not a hardware claim.
    pub fn reset(&mut self) {
        self.pc = 0;
        self.ps = 0xFFFF;
        self.ss = 0;
        self.ds0 = 0;
        self.ds1 = 0;
        self.psw = (self.psw & !(IE | BRK | DIR)) | MD;
        self.leave_emulation();
    }
}

pub fn offset_add(offset: u16, amount: i32) -> u16 {
    if assumptions::WRAP_OFFSETS {
        offset.wrapping_add_signed(amount as i16)
    } else {
        u16::try_from(i32::from(offset) + amount).expect("offset wrap refused by assumption")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub source: String,
    pub ss: u16,
    pub sp: u16,
    /// Increasing addresses: PC, PS, PSW. Writes happened in reverse order.
    pub words: [u16; 3],
}

pub struct Memory {
    pub bytes: Vec<u8>,
    pub frames: Vec<Frame>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            bytes: vec![0xCC; 1 << 20],
            frames: Vec::new(),
        }
    }

    /// NEC p. 10-1 / Fig. 10-1: segment*16 + offset, 20 address bits.
    pub fn physical(segment: u16, offset: u16) -> usize {
        ((usize::from(segment) << 4) + usize::from(offset)) & 0xF_FFFF
    }

    pub fn read(&self, segment: u16, offset: u16) -> u8 {
        self.bytes[Self::physical(segment, offset)]
    }

    pub fn write(&mut self, segment: u16, offset: u16, value: u8) {
        self.bytes[Self::physical(segment, offset)] = value;
    }

    pub fn word(&self, segment: u16, offset: u16) -> u16 {
        u16::from_le_bytes([
            self.read(segment, offset),
            self.read(segment, offset_add(offset, 1)),
        ])
    }

    pub fn put_word(&mut self, segment: u16, offset: u16, word: u16) {
        let [lo, hi] = word.to_le_bytes();
        self.write(segment, offset, lo);
        self.write(segment, offset_add(offset, 1), hi);
    }
}
