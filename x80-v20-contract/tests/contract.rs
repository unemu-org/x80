//! Vectors: decision-9 / plan §3. NEC Stock No. 500350, October 1986,
//! rendered pages checked; independent literal states, frames and access
//! counts live in vectors/mode-switch.txt (source map in README.md).
//! 8080 payloads use Intel 98-153B Ch. 4 pp. 4-4/4-5, 4-11, 4-13/4-14;
//! the uPD8080AF equivalence is ASSUMED in PROVENANCE.md §1.
#![forbid(unsafe_code)]

mod support;

use support::adapter::{Lines, Outcome, Refusal};
use support::native::{Memory, IE, MD};
use support::vectors::{fixture, run_vector};

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
    nmi_over_int_and_hold
);

#[test]
fn brkem_maps_every_register_before_any_8080_instruction() {
    // NEC p. 8-1 Figs 8-1/8-2, p. 12-169; literal independent witness.
    let a = fixture("brkem_retem");
    let r = a.cpu.registers();
    assert_eq!(
        (r.a, r.h, r.l, r.b, r.c, r.d, r.e),
        (0x11, 0xB2, 0x23, 0xC4, 0x45, 0xD6, 0x67)
    );
    assert_eq!((r.sp, r.pc, r.psw()), (0x9000, 0x0100, 0x1192));
    // ASSUMPTION R2 (decision-9 amendment): BRKEM clears IE and BRK.
    assert_eq!(
        (a.native.sp, a.native.ps, a.native.psw),
        (0x7FFA, 0x2000, 0x7092)
    );
    assert!(!a.cpu.latches().interrupt_enable);
    assert_eq!(
        &a.memory.bytes[0x37FFA..0x38000],
        &[0x45, 0x23, 0x00, 0x10, 0x92, 0xF3]
    );
}

#[test]
fn refusals_copy_out_without_consuming_an_unknown_encoding() {
    // decision-9 / decision-5 Unsupported; NEC p. 12-176/12-177 only
    // licenses ED ED nn and ED FD. Neither refusal is a native transfer.
    for (bytes, refusal, reads) in [
        (
            &[0xDD][..],
            Refusal::Opcode {
                pc: 0x111,
                opcode: 0xDD,
            },
            1,
        ),
        (
            &[0xED, 0x00][..],
            Refusal::Escape {
                pc: 0x111,
                second: 0,
            },
            2,
        ),
    ] {
        let mut a = fixture("brkem_retem");
        for (i, &byte) in bytes.iter().enumerate() {
            a.memory.write(0x2000, 0x111 + i as u16, byte);
        }
        assert_eq!(a.run(9, Lines::default()), Ok(Outcome::Budget));
        let count = a.accesses.len();
        assert_eq!(a.run(1, Lines::default()), Err(refusal));
        assert_eq!(a.accesses.len() - count, reads);
        assert_eq!(
            (a.native.aw, a.native.bw, a.native.pc, a.native.psw),
            (0xA1A6, 0x1234, 0x111, 0x7093)
        );
        assert_eq!(a.boundaries.last(), Some(&a.native));
        assert_eq!(a.memory.frames.len(), 1);
        assert!(a.services.is_empty());
    }
}

#[test]
fn ei_delay_assumption_is_observed_by_host_interrupt_acceptance() {
    // ASSUMPTION (PROVENANCE §1), not NEC-confirmed emulation behavior.
    let mut a = fixture("brkem_retem");
    for (i, byte) in [0xF3, 0xFB, 0x00, 0xED, 0xFD].into_iter().enumerate() {
        a.memory.write(0x2000, 0x100 + i as u16, byte);
    }
    assert_eq!(a.run(2, Lines::default()), Ok(Outcome::Budget));
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
fn offset_wrap_and_io_high_byte_are_fixture_assumptions() {
    // ASSUMPTIONS, PROVENANCE §1 / §3, plan §3; not NEC confirmations.
    let mut a = fixture("brkem_retem");
    a.native.pc = 0xFFFF;
    a.cpu.registers_mut().pc = 0xFFFF;
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
