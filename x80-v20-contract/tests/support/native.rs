//! Mock operations, not instruction decoding or native execution.
//! Design: decision-9 / plan §3. Every NEC citation here means the pinned
//! uPD70108/uPD70116 User's Manual, Stock No. 500350, October 1986.

use super::assumptions;
use x80_core::i8080::{Latches, Registers, I8080};

// NEC p. 3-3 PSW diagram; p. 8-1 Fig. 8-2 (IE and BRK).
pub const IE: u16 = 1 << 9;
pub const BRK: u16 = 1 << 8;
pub const MD: u16 = 1 << 15;
const DIR: u16 = 1 << 10;

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
}

impl Native {
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
        }
    }

    /// NEC p. 8-1 Fig. 8-1: AL=A, BW=HL, CW=BC, DW=DE, BP=SP.
    pub fn copy_in(self, mut latches: Latches) -> I8080 {
        let mut r = Registers {
            sp: self.bp,
            pc: self.pc,
            ..Registers::default()
        };
        r.set_psw(((self.aw & 0xFF) << 8) | (self.psw & 0xFF));
        r.set_hl(self.bw);
        r.set_bc(self.cw);
        r.set_de(self.dw);
        latches.interrupt_enable = self.psw & IE != 0;
        I8080::new(r, latches)
    }

    /// NEC p. 8-1 Figs 8-1/8-2. AH and native-only state stay untouched.
    pub fn copy_out(&mut self, cpu: &I8080) {
        let r = cpu.registers();
        self.aw = (self.aw & 0xFF00) | u16::from(r.a);
        self.bw = r.hl();
        self.cw = r.bc();
        self.dw = r.de();
        self.bp = r.sp;
        self.pc = r.pc;
        self.psw = (self.psw & !(0xFF | IE)) | (r.psw() & 0xFF);
        if cpu.latches().interrupt_enable {
            self.psw |= IE;
        }
    }

    /// NEC p. 12-169 / p. 8-2: push PSW, PS, PC; MD=0; vector.
    /// `pc` is already the native continuation supplied by the caller.
    pub fn brkem(&mut self, memory: &mut Memory, vector: u8) {
        assert_ne!(self.psw & MD, 0, "BRKEM requires native mode");
        self.push_frame(memory, "brkem");
        self.psw &= !MD;
        if !assumptions::BRKEM_PRESERVES_IE {
            self.psw &= !IE;
        }
        if !assumptions::BRKEM_PRESERVES_BRK {
            self.psw &= !BRK;
        }
        self.vector(memory, vector);
    }

    /// NEC p. 6-1 and p. 12-176 / Fig. 8-6 p. 8-4: saved MD=0,
    /// then IE=0, BRK=0, MD=1, vector into a native routine.
    pub fn enter_service(&mut self, memory: &mut Memory, source: &'static str, vector: u8) {
        assert_eq!(self.psw & MD, 0);
        self.push_frame(memory, source);
        self.psw = (self.psw & !(IE | BRK)) | MD;
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

    /// Test payload, deliberately no x86 bytes/execution. Changes mapped
    /// and native-only registers and live flags before the RETI operation.
    pub fn mock_routine(&mut self, saved_psw: u16) {
        self.aw = (self.aw & 0xFF00) | 0xC7;
        self.cw = 0xABCD;
        self.ix = 0xDEAD;
        // Fixture policy: invert CY, P, AC, Z, S, BRK, IE, DIR and V
        // relative to the saved PSW (NEC p. 3-3 diagram). MD stays native,
        // hence differs from saved MD=0; reserved bits stay untouched.
        self.psw = (saved_psw ^ 0x0FD5) | MD;
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
