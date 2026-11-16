# x80

> **LLM-powered project.** Most of the code, tests and documentation in x80 are
> written by LLM coding agents, working under the maintainer's direction. Every
> change goes through the same source rules and test gates described below, but
> if you use x80 in your own project, be aware of how it is made and review it as
> you would any third-party code you depend on.

An interpreter library for the Intel 8080 family of CPUs, written in pure Rust.
It emulates the Intel 8080, the Intel 8085 and the Zilog Z80. It also covers the
8080 emulation mode of the NEC V20/V30, working together with the companion x86
emulator `x86jit`.

x80 emulates CPUs, not computers. Your program supplies memory, I/O devices,
interrupt sources, timing and file loading; x80 executes the instructions.

- **No dependencies.** `#![no_std]`, no heap allocation and no `unsafe` code.
- **Complete instruction sets.** Every documented instruction on every CPU, with its
  cycle count (T-states).
- **Traceable behaviour.** Documented behaviour comes from manufacturer manuals and
  datasheets, cited down to the page, and each fact is checked by a test.
  Undocumented behaviour is implemented only when a written source and an
  independent hardware measurement agree. Where they do not settle a detail, the library's choice is
  marked as such in [PROVENANCE.md](https://github.com/unemu-org/x80/blob/main/PROVENANCE.md).
- **Deterministic.** Given the same memory and device responses, a program ends in
  the same state whether it runs in one call, one instruction at a time or in
  slices of any size.
- **No guessing.** A run returns why it stopped: its budget ran out, the host asked
  it to stop, the CPU halted, or it met a byte with no reliable documentation
  (`Unsupported`, with the byte and its address).

## Status

All three CPUs are complete. The 8085 and Z80 also run their undocumented
instructions. [status](https://github.com/unemu-org/x80/blob/main/backlog/docs/status.md) has the detailed capability list.

| CPU | What is emulated |
| --- | --- |
| Intel 8080 | All 244 documented opcodes, interrupts (RST), the EI delay, HLT and RESET. The twelve undocumented opcodes return `Unsupported`, because no reliable source describes them. |
| Intel 8085 | All 256 opcodes: the documented set, RIM/SIM and the ten undocumented instructions. All five interrupt inputs (TRAP, RST 7.5/6.5/5.5, INTR with RST or CALL), the serial SID/SOD lines, and the undocumented V and K flags. |
| Zilog Z80 (NMOS) | Every opcode, undocumented ones included: IXH/IXL/IYH/IYL, SLL, the duplicate ED and indexed CB forms. Undocumented flag bits 3 and 5, the hidden WZ (MEMPTR) register, and the internal Q value that SCF and CCF read. NMI and interrupt modes 0, 1 and 2. 16-bit I/O port addresses. |
| NEC V20/V30, 8080 mode | These x86 CPUs can switch into an 8080-compatible mode. x80 runs the 8080 side, x86jit runs the x86 side, and your program connects them ([design record](https://github.com/unemu-org/x80/blob/main/backlog/decisions/decision-9%20-%20NEC-V20-V30-8080-emulation-mode-is-the-i8080-model-with-a-V20-profile.md)). Both projects pass a shared set of hand-over test cases; x86jit runs 12 of the 15, and the 3 NMI cases wait for NMI support there. Cycle counts are Intel 8080 timings, not V20 clock counts. |

Each CPU is its own type: `i8080::I8080`, `i8085::I8085` and `z80::Z80`. The crate
root holds what they share: `Bus`, `MemoryAccess`, `Control`, `Budget`, `Report`,
`Exit` and `Unsupported`.

## Verification

Well-known CPU test programs run in an optional release-mode check, separate from
the normal test suite ([how to run them](https://github.com/unemu-org/x80/blob/main/backlog/docs/commands.md#oracle-runs)):

| Test program | Result |
| --- | --- |
| 8080PRE | passes on the 8080 and the 8085 |
| 8080EXM | all 25 test groups pass |
| 8085EXM | all 24 checked groups pass; one long ALU group has no trusted expected result, so it is reported, not checked |
| Z80 prelim | passes |
| ZEXDOC, ZEXALL | all 67 groups pass in each |
| z80test v1.2a | all six variants pass all 156 checked groups; four tests aimed at NEC and ST clones are skipped |

These programs check the implementation; none of them is the sole source of any
behaviour, and nothing is taken from other emulators' code or documentation. [PROVENANCE.md](https://github.com/unemu-org/x80/blob/main/PROVENANCE.md)
lists every source, the test runs, and the details that remain unconfirmed.

## Example

```rust
use x80_core::i8080::{Inputs, I8080};
use x80_core::{Budget, Bus, Control, Exit, MemoryAccess};

/// 64 KiB of RAM and one output port. Your program owns all of it.
struct System {
    memory: [u8; 0x10000],
    last_output: Option<(u8, u8)>,
}

impl Bus for System {
    fn read(&mut self, address: u16, _kind: MemoryAccess) -> u8 {
        self.memory[usize::from(address)]
    }
    fn write(&mut self, address: u16, value: u8, _kind: MemoryAccess) {
        self.memory[usize::from(address)] = value;
    }
    fn input(&mut self, _port: u8) -> u8 {
        0xFF
    }
    fn output(&mut self, port: u8, value: u8) {
        self.last_output = Some((port, value));
    }
}

impl Control for System {} // never asks the CPU to stop early
impl Inputs for System {} // the interrupt line stays low

let mut system = System { memory: [0; 0x10000], last_output: None };
// MVI A,2Ah ; OUT 01h ; HLT
system.memory[..5].copy_from_slice(&[0x3E, 0x2A, 0xD3, 0x01, 0x76]);

let mut cpu = I8080::default();
let report = cpu.run(&mut system, Budget { steps: 1_000, cycles: u64::MAX });

assert_eq!(report.exit, Exit::Halt);
assert_eq!((report.steps, report.cycles), (3, 24)); // 7 + 10 + 7 T-states
assert_eq!(cpu.registers().a, 0x2A);
assert_eq!(system.last_output, Some((0x01, 0x2A)));
```

Every memory access says what it is: an opcode fetch, an operand fetch, a data or
stack access, or an interrupt acknowledge. `step` runs at most one instruction.
`run` checks the budget before each instruction and always finishes an instruction
it has started. It can therefore exceed the cycle limit by part of one instruction,
but never the step limit. A halted CPU returns `Halt` at once without using cycles,
so your program decides how time passes while the CPU waits. The 8085 and Z80 work
the same way, each with its own `Inputs` trait for its interrupt lines. To see the
full 16-bit port address on the Z80, implement `Bus::input16` and `Bus::output16`.

## Not included

- Whole machines, devices, file loaders, operating-system services and ROMs. These
  belong to the program that uses x80. JIT compilation and x86 execution are out
  of scope; the V20/V30's own x86 mode belongs to x86jit.
- Bus-cycle or pin-level timing: wait states, READY, HOLD, and the Z80's BUSREQ and
  WAIT. Cycle counts are per instruction.
- Other chips: CMOS Z80s, other makers' versions and later derivatives.
- The 8080's undocumented opcodes, until reliable evidence describes them. A few
  interrupt responses outside the supported set also return `Unsupported`: a non-RST
  byte on the 8080, anything but RST or CALL on the 8085, a non-RST instruction in
  Z80 interrupt mode 0, and an odd vector in Z80 interrupt mode 2.

[deferred.md](https://github.com/unemu-org/x80/blob/main/backlog/docs/deferred.md) explains each of these and what would
change it.

## Workspace

| Crate | Role |
| --- | --- |
| `x80-core` | The library. No dependencies, `no_std`. |
| `x80-oracle` | Command-line runner for the CPU test programs. Not published. |
| `x80-v20-contract` | The shared V20/V30 hand-over test cases, with a stand-in for the x86 side. Test-only, not published. |

`x80-core` is on [crates.io](https://crates.io/crates/x80-core): add it with
`cargo add x80-core`, or `x80-core = "0.1"` under `[dependencies]`. It needs Rust 1.95
or newer (edition 2021). The CPU test programs and manuals are not stored in this
repository: a script in the `oracles/` submodule downloads them and checks their
hashes.

## Building and testing

```sh
cargo build --workspace --offline --locked
cargo nextest run --workspace --all-features --offline --locked
cargo test --workspace --doc --all-features --offline --locked
cargo clippy --workspace --all-targets --all-features --offline --locked -- -D warnings
cargo fmt --all --check
```

Without cargo-nextest, `cargo test --workspace --all-features --offline --locked`
runs everything. A Nix development shell provides the pinned toolchain
([setup](https://github.com/unemu-org/x80/blob/main/backlog/docs/dev-setup.md)). [commands](https://github.com/unemu-org/x80/blob/main/backlog/docs/commands.md) covers
the minimum Rust version check, the licence check and the CPU test programs.

## Documentation

- [plan](https://github.com/unemu-org/x80/blob/main/backlog/docs/plan.md): goals, design and history
- [architecture](https://github.com/unemu-org/x80/blob/main/backlog/docs/architecture.md): code layout
- [design records](https://github.com/unemu-org/x80/tree/main/backlog/decisions): why the API and behaviour are as they are
- [PROVENANCE.md](https://github.com/unemu-org/x80/blob/main/PROVENANCE.md): sources, citations and what is still unconfirmed
- [glossary](https://github.com/unemu-org/x80/blob/main/backlog/docs/glossary.md): terms used in the docs
- [AGENTS.md](https://github.com/unemu-org/x80/blob/main/AGENTS.md): development workflow, written for the coding agents

## Contributing

x80 is developed in the open at <https://github.com/unemu-org/x80>, by the
maintainer working with LLM coding agents. Pull requests are disabled while the
project matures. Issues are very welcome: bug reports, questions, hardware
measurements and corrections to a cited source. Conversations about them stay
open on GitHub. Planned work is tracked with beads (`bd`), whose database lives
in the same repository; `bd bootstrap` restores it in a clone.

## License

Dual-licensed under [MIT](https://github.com/unemu-org/x80/blob/main/LICENSE-MIT) or [Apache-2.0](https://github.com/unemu-org/x80/blob/main/LICENSE-APACHE), at your option.
