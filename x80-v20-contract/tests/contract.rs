//! Vectors: decision-9 / plan §3. NEC Stock No. 500350, October 1986,
//! rendered pages checked; independent literal states, frames and access
//! counts live in vectors/mode-switch.txt (source map in README.md).
//! 8080 payloads use Intel 98-153B Ch. 4 pp. 4-4/4-5, 4-11, 4-13/4-14;
//! the uPD8080AF equivalence is ASSUMED in PROVENANCE.md §1.
#![forbid(unsafe_code)]

mod support;

use support::adapter::{Adapter, Lines, Outcome, Refusal};
use support::native::{Memory, SingleStepRefused, IE, MD};
use support::vectors::{check_format, configuration, fixture, run_vector};
use x80_core::MemoryAccess;

macro_rules! vector_tests {
    ($($name:ident),+ $(,)?) => { $(
        #[test]
        fn $name() { run_vector(stringify!($name)); }
    )+ };
}

vector_tests!(
    brkem_retem,
    calln_reti_retem,
    int_enabled_running,
    int_disabled_running,
    nmi_running,
    hlt_int_disabled,
    hlt_int_disabled_retem,
    hlt_int_disabled_calln,
    hlt_int_enabled,
    hlt_nmi,
    reset_running,
    int_low_running,
    int_held_running,
    nmi_over_int_and_hold,
    calln_consumes_ei_delay
);

#[test]
#[should_panic(expected = "unsupported vector format")]
fn format_1_vectors_are_rejected() {
    // Format 1 counted the adapter's escape reads in `access`.
    check_format("# old\nformat x80-v20-contract-1\n");
}

#[test]
fn brkem_maps_every_register_before_any_8080_instruction() {
    // NEC p. 8-1 Figs 8-1/8-2, p. 12-169; literal independent witness.
    let a = fixture("brkem_retem");
    let cpu = a.native.copy_in();
    let r = cpu.registers();
    assert_eq!(
        (r.a, r.h, r.l, r.b, r.c, r.d, r.e),
        (0x11, 0xB2, 0x23, 0xC4, 0x45, 0xD6, 0x67)
    );
    assert_eq!((r.sp, r.pc, r.psw()), (0x9000, 0x0100, 0x1192));
    // After BRKEM: IE = 0 (ASSUMPTION R2, decision-9 amendment), BRK = 0
    // (precondition; R2's BRK clear is not observable here).
    assert_eq!(
        (a.native.sp, a.native.ps, a.native.psw),
        (0x7FFA, 0x2000, 0x7092)
    );
    assert!(!cpu.latches().interrupt_enable);
    assert_eq!(
        &a.memory.bytes[0x37FFA..0x38000],
        &[0x45, 0x23, 0x00, 0x10, 0x92, 0xF2]
    );
}

#[test]
fn brkem_with_brk_set_is_refused_without_effect() {
    // decision-9 amendment 2026-11-15, following the principle of decision-23
    // §3 (x86jit): the native core refuses BRKEM with BRK = 1 before any
    // effect. NEC p. 6-3's single-step rule covers BRKEM literally, but no page
    // settles how it combines with the switch into emulation mode and R2's BRK
    // clear (ASSUMED). Interface rule, not a hardware fact. F392 is format 1's
    // start.
    let (mut native, mut memory, vector) = configuration("brkem_retem");
    native.psw = 0xF392;
    let before = native;
    let image = memory.bytes.clone();
    assert_eq!(native.brkem(&mut memory, vector), Err(SingleStepRefused));
    assert_eq!(native, before);
    assert!(memory.bytes == image, "memory changed");
    assert!(memory.frames.is_empty());
    // Entry is refused, so no adapter exists and x80 never runs.
    assert_eq!(
        Adapter::enter(native, memory, vector).err(),
        Some(Refusal::NativeSingleStep)
    );
}

#[test]
fn non_ed_refusal_is_typed_and_keeps_emulation_mode() {
    // decision-9 amendment 2026-11-15 / decision-3: a non-ED Unsupported is
    // a typed refusal. MD stays 0, nothing is decoded natively, and each
    // later run at that PS:PC refuses again.
    let mut a = fixture("brkem_retem");
    a.memory.write(0x2000, 0x111, 0xDD);
    assert_eq!(a.run(9, Lines::default()), Ok(Outcome::Budget));
    for _ in 0..2 {
        let count = a.accesses.len();
        assert_eq!(
            a.run(1, Lines::default()),
            Err(Refusal::Opcode {
                pc: 0x111,
                opcode: 0xDD,
            })
        );
        // Fixture only: this mock re-runs x80 on every retry, so x80 fetches
        // DD again. A native-side port answers later runs itself until the
        // next hand-back and sees no further x80 fetch.
        assert_eq!(a.accesses.len() - count, 1);
        assert_eq!(
            (a.native.aw, a.native.bw, a.native.pc, a.native.psw),
            (0xA1A6, 0x1234, 0x111, 0x7093)
        );
        assert_eq!(a.native.psw & MD, 0);
        assert_eq!(a.boundaries.last(), Some(&a.native));
        assert_eq!(a.memory.frames.len(), 1);
        assert!(a.services.is_empty());
    }
}

#[test]
fn ed_escape_is_decoded_by_the_native_side_without_reading_past_ed() {
    // Option B (decision-9 amendment 2026-11-15): x80's bus sees one ED
    // opcode fetch and no operand fetch. The native side reads ED 00 itself
    // and reports its unknown instruction; NEC pp. 12-176/12-177 license only
    // ED ED nn and ED FD. MD stays 0 and the registers were written back.
    let mut a = fixture("brkem_retem");
    a.memory.write(0x2000, 0x111, 0xED);
    a.memory.write(0x2000, 0x112, 0x00);
    assert_eq!(a.run(9, Lines::default()), Ok(Outcome::Budget));
    let count = a.accesses.len();
    assert_eq!(
        a.run(1, Lines::default()),
        Err(Refusal::NativeUnknown {
            pc: 0x111,
            bytes: [0xED, 0x00],
        })
    );
    let new = &a.accesses[count..];
    assert_eq!(new.len(), 1);
    assert_eq!(
        (new[0].kind, new[0].segment, new[0].offset, new[0].value),
        (MemoryAccess::OpcodeFetch, 0x2000, 0x111, 0xED)
    );
    assert!(!a.accesses.iter().any(|x| x.offset == 0x112));
    assert_eq!(
        (a.native.aw, a.native.bw, a.native.cw, a.native.dw),
        (0xA1A6, 0x1234, 0x1234, 0x5678)
    );
    assert_eq!(
        (a.native.bp, a.native.pc, a.native.psw),
        (0x9000, 0x111, 0x7093)
    );
    assert_eq!(a.native.psw & MD, 0);
    assert_eq!(a.boundaries.last(), Some(&a.native));
    assert_eq!(a.memory.frames.len(), 1);
    assert!(a.services.is_empty());
}

#[test]
fn halted_lives_in_the_native_model_across_hand_backs() {
    // decision-9 amendment 2026-11-15: halted is in the view. A slice ends
    // right after HLT on its budget (x80 has not yet returned Halt). Every
    // later run rebuilds x80 from the native model alone; it must stay in
    // standby (NEC pp. 8-2, 9-1) and fetch nothing until a wake event.
    let mut a = fixture("hlt_int_enabled");
    assert_eq!(a.run(0x0B, Lines::default()), Ok(Outcome::Budget));
    assert_eq!(a.native.pc, 0x113);
    assert!(a.native.halted);
    let count = a.accesses.len();
    // A zero-step slice is a hand-back too; it must not clear the latch.
    assert_eq!(a.run(0, Lines::default()), Ok(Outcome::Budget));
    for _ in 0..2 {
        assert_eq!(a.run(1, Lines::default()), Ok(Outcome::Halt));
    }
    assert_eq!(a.accesses.len(), count);
    assert!(a.native.halted);
    assert_eq!(a.native.pc, 0x113);
    let int = Lines {
        int: true,
        vector: 0x41,
        ..Lines::default()
    };
    assert_eq!(a.run(1, int), Ok(Outcome::Serviced));
    assert_eq!(a.memory.frames[1].words, [0x113, 0x2000, 0x7293]);
    assert!(!a.native.halted);
    a.until(Outcome::Retem);
}

#[test]
fn ei_delay_assumption_is_observed_by_host_interrupt_acceptance() {
    // ASSUMPTION (PROVENANCE §1), not NEC-confirmed emulation behavior.
    let mut a = fixture("brkem_retem");
    for (i, byte) in [0xF3, 0xFB, 0x00, 0xED, 0xFD].into_iter().enumerate() {
        a.memory.write(0x2000, 0x100 + i as u16, byte);
    }
    assert_eq!(a.run(2, Lines::default()), Ok(Outcome::Budget));
    assert!(a.native.ei_delay);
    // The delay is exchanged verbatim; a zero-step slice never clears it.
    assert_eq!(a.run(0, Lines::default()), Ok(Outcome::Budget));
    assert!(a.native.ei_delay);
    let int = Lines {
        int: true,
        vector: 0x41,
        ..Lines::default()
    };
    assert_eq!(a.run(1, int), Ok(Outcome::Budget));
    assert!(a.services.is_empty());
    assert_eq!(a.native.pc, 0x103);
    assert_eq!(a.run(1, int), Ok(Outcome::Serviced));
    assert_eq!(a.memory.frames[1].words, [0x103, 0x2000, 0x7292]);
    assert_eq!(a.native.psw & (MD | IE), IE);
    a.until(Outcome::Retem);
}

#[test]
fn retem_drops_the_8080_latches() {
    // Fixture policy (assumptions.rs, README): after RETEM no emulation
    // state remains, so the mock drops the EI delay that EI left pending.
    // The format's `final` record has no latch field, hence this test.
    let mut a = fixture("brkem_retem");
    for (i, byte) in [0xFB, 0xED, 0xFD].into_iter().enumerate() {
        a.memory.write(0x2000, 0x111 + i as u16, byte);
    }
    assert_eq!(a.run(0x0A, Lines::default()), Ok(Outcome::Budget));
    assert!(a.native.ei_delay);
    assert_eq!(a.run(1, Lines::default()), Ok(Outcome::Retem));
    assert!(a.ended);
    assert!(!a.native.ei_delay);
    assert!(!a.native.halted);
}

#[test]
fn offset_wrap_and_io_high_byte_are_fixture_assumptions() {
    // ASSUMPTIONS: offset wrap inside the segment and the 8080 IN/OUT high
    // address byte (decision-9 amendment 2026-11-15, PROVENANCE §1 to-confirm
    // list); not NEC confirmations.
    let mut a = fixture("brkem_retem");
    a.native.pc = 0xFFFF;
    a.memory.write(0x2000, 0xFFFF, 0x3E); // MVI A,5A across segment end
    a.memory.write(0x2000, 0, 0x5A);
    for (i, byte) in [0xDB, 0x23, 0xD3, 0x24, 0xED, 0xFD].into_iter().enumerate() {
        a.memory.write(0x2000, 1 + i as u16, byte);
    }
    assert_eq!(a.run(1, Lines::default()), Ok(Outcome::Budget));
    assert_eq!((a.native.aw, a.native.pc, a.native.ps), (0xA15A, 1, 0x2000));
    a.until(Outcome::Retem);
    assert_eq!(a.ports, [(0x0023, 0xA6, false), (0x0024, 0xA6, true)]);
    assert_eq!(a.native.aw, 0xA1A6);
    // NEC p. 10-1 Fig. 10-1: 20-bit physical address, aliases in flat RAM.
    assert_eq!(Memory::physical(0x1234, 0x5678), 0x179B8);
    let mut m = Memory::new();
    m.write(0x1234, 0x10, 0xA5);
    assert_eq!(m.read(0x1235, 0), 0xA5);
}

#[test]
fn reset_copies_boundary_state_before_native_reset_and_ends_the_loop() {
    // NEC pp. 8-4, 7-1: abort, then native reset. Undefined fields retained
    // solely by fixture policy; do not turn them into hardware expectations.
    let mut a = fixture("reset_running");
    assert_eq!(a.run(10, Lines::default()), Ok(Outcome::Budget));
    assert_eq!(
        a.run(
            1,
            Lines {
                reset: true,
                ..Lines::default()
            }
        ),
        Ok(Outcome::Reset)
    );
    let before = a.boundaries.last().unwrap();
    assert_eq!(
        (before.aw, before.pc, before.psw, before.sp),
        (0xA1A6, 0x112, 0x7293, 0x7FFA)
    );
    assert_eq!(
        (a.native.ps, a.native.pc, a.native.psw),
        (0xFFFF, 0, 0xF093)
    );
    assert_eq!(a.run(1, Lines::default()), Err(Refusal::Ended));
}

#[test]
fn hold_inhibits_enabled_int_in_standby_without_using_the_masked_wake_rule() {
    // NEC p. 6-2 inhibits INT acceptance during hold; pp. 8-2/9-1's
    // unserviced resume applies to disabled interrupts. Hold bus cycles
    // themselves are deliberately outside this boundary fixture.
    let mut a = fixture("hlt_int_enabled");
    a.until(Outcome::Halt);
    assert_eq!(a.native.pc, 0x113);
    assert_eq!(
        a.run(
            1,
            Lines {
                int: true,
                hold: true,
                vector: 0x41,
                ..Lines::default()
            }
        ),
        Ok(Outcome::Halt)
    );
    assert_eq!(a.native.pc, 0x113);
    assert!(a.services.is_empty());
    assert_eq!(
        a.run(
            1,
            Lines {
                int: true,
                vector: 0x41,
                ..Lines::default()
            }
        ),
        Ok(Outcome::Serviced)
    );
    assert_eq!(a.memory.frames[1].words, [0x113, 0x2000, 0x7293]);
    a.until(Outcome::Retem);
}
