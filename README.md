# x80

A guest-agnostic interpreter for the Intel 8080 family, written in pure Rust.
It covers the Intel 8080, the Intel 8085 and the Zilog Z80. Through an embedder's
adapter and the sibling `x86jit` project, it also runs the 8080 emulation mode of
the NEC V20/V30.

The embedder owns the machine: memory, port devices, interrupt sources,
scheduling and file formats. `x80-core` executes instructions and nothing else.

- **No dependencies.** `#![no_std]`, no allocation and `#![forbid(unsafe_code)]`.
- **Every documented instruction on every model**, with the instruction's T-state total.
- **Cited behaviour.** Every hardware fact cites its pinned source and page and has a
  witness test. Undocumented facts need two independent keys; where a key is
  missing, the gap is a labelled library rule ([PROVENANCE.md](PROVENANCE.md)).
- **Deterministic.** Given the same bus and input responses, a program ends with the
  same registers, memory, bus trace and totals whether it runs whole, step by step
  or in any budget split.
- **Typed exits.** Every return carries a non-exhaustive `Exit`: budget, host stop,
  halt, or `Unsupported` with the refused byte and its PC.

## Status

Feature complete for its planned scope: milestones M0 to M5 and the undocumented
tracks are done. [status](backlog/docs/status.md) has the full capability snapshot.

| Model | Coverage |
| --- | --- |
| `i8080` | All 244 documented opcodes, INT with RST acceptance, EI delay, HLT and RESET. The twelve undocumented bytes return `Unsupported`: no source qualifies them. |
| `i8085` | All 256 fetched bytes: the documented set, RIM/SIM and the ten undocumented instructions. Five interrupt sources (TRAP, RST 7.5/6.5/5.5, INTR with RST or CALL). Serial SID/SOD lines. Independently stored V and K flags. |
| `z80` | Zilog NMOS. Every normally fetched encoding, undocumented forms included: index halves, SLL, ED aliases and indexed CB copies. Full F/F' with bits 3 and 5, retained WZ (MEMPTR) and the Q latch for SCF/CCF. NMI, IM 0 (RST), IM 1 and IM 2. 16-bit port addresses. |
| NEC V20/V30 8080 mode | The `i8080` model under an integration profile ([decision-9](<backlog/decisions/decision-9 - NEC-V20-V30-8080-emulation-mode-is-the-i8080-model-with-a-V20-profile.md>)). x80 stops on the ED escape and the native core (x86jit) decodes RETEM and CALLN. Both projects run the same contract vectors: x80 all 15 cases against a mock native side, x86jit its 12 non-NMI cases. The adapter is the embedder's. Timings are Intel T-states, not V20 clocks. |

Each model is its own type: `i8080::I8080`, `i8085::I8085` and `z80::Z80`.
The crate root holds only the model-neutral contract: `Bus`, `MemoryAccess`,
`Control`, `Budget`, `Report`, `Exit` and `Unsupported`.

## Verification

Pinned third-party CPU exercisers (GPL-2.0-or-later, and MIT for z80test) run in
an optional release-mode campaign, outside the ordinary gates
([oracle runs](backlog/docs/commands.md#oracle-runs)):

| Oracle | Result |
| --- | --- |
| 8080PRE | PASS on the `i8080` and `i8085` models |
| 8080EXM | 25/25 groups |
| 8085EXM | 24/24 judged groups (the long ALU CRC is reported, not judged) |
| Z80 prelim | PASS |
| ZEXDOC, ZEXALL | 67/67 groups each |
| z80test v1.2a | 156/156 judged groups in each of the six variants (four NEC/ST diagnostics skipped) |

An exerciser can falsify a fact, but it cannot license one. Documented behaviour
comes from pinned primary sources. An undocumented hardware fact needs a pinned
written statement plus an independent silicon confirmation
([decision-17](<backlog/decisions/decision-17 - Two-key-evidence-rule-for-undocumented-behaviour.md>)).
Where a key is missing, the 8085 and Z80 use an explicitly labelled library rule,
not a hardware claim. [PROVENANCE.md](PROVENANCE.md) records the sources, the
oracle runs and, for each milestone, the facts that remain derived, assumed or
unverified. No behaviour comes from another emulator's source or documentation.

## Example

```rust
use x80_core::i8080::{Inputs, I8080};
use x80_core::{Budget, Bus, Control, Exit, MemoryAccess};

/// 64 KiB of RAM and one output port: the embedder owns all of it.
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

impl Control for System {} // no host stop requests
impl Inputs for System {} // INT stays low

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

Every memory read and write carries a `MemoryAccess` kind: opcode fetch, operand
fetch, data, stack or interrupt acknowledge. `step` runs at most one instruction.
`run` checks the budget before each instruction and finishes any instruction it
starts. Reported cycles can therefore exceed the cycle limit by less than the last
instruction's cost, while steps never exceed the step limit. A halted CPU returns
`Halt` at once without spending cycles, so the embedder advances time while halted.
The other models follow the same shape, each with its own `Inputs` trait for its
interrupt lines. A Z80 embedder overrides `Bus::input16` and `Bus::output16` to
see the full 16-bit port address.

## Out of scope

- Machines, devices, loaders, file formats, OS services and ROMs. These belong to
  embedders. JIT compilation and x86 execution are non-goals; NEC V20/V30 native
  mode is x86jit's.
- Bus-cycle stepping, pin-level timing, wait states, READY, HOLD and the Z80's
  BUSREQ and WAIT. Totals are per instruction.
- Other parts: Z80 CMOS and other makers' parts, second sources and later
  derivatives.
- Undocumented 8080 bytes return `Unsupported` until evidence meeting decision-17
  admits them. Interrupt acknowledges outside the API contract also return
  `Unsupported`: a non-RST byte on the 8080, a byte other than RST or CALL on the
  8085, a non-RST instruction in Z80 IM 0, and an odd IM 2 vector.

[deferred.md](backlog/docs/deferred.md) lists each deferral and what would
reopen it.

## Workspace

| Crate | Role |
| --- | --- |
| `x80-core` | The library. Zero dependencies, `no_std`. |
| `x80-oracle` | Unpublished host that runs the CPU exercisers offline. |
| `x80-v20-contract` | Unpublished, test-only V20/V30 contract vectors and a mock native side. |

`x80-core` is not on crates.io yet; depend on it by path. MSRV is Rust 1.95,
edition 2021. The exercisers and manuals are fetch-only pins in the `oracles/`
submodule; they are not redistributed here.

## Building and testing

```sh
cargo build --workspace --offline --locked
cargo nextest run --workspace --all-features --offline --locked
cargo test --workspace --doc --all-features --offline --locked
cargo clippy --workspace --all-targets --all-features --offline --locked -- -D warnings
cargo fmt --all --check
```

Without nextest, `cargo test --workspace --all-features --offline --locked` runs
everything. A Nix development shell pins the toolchain
([setup](backlog/docs/dev-setup.md)); [commands](backlog/docs/commands.md) covers
MSRV, cargo-deny and the oracle runs.

## Documentation

- [plan](backlog/docs/plan.md): goals, design and milestones
- [architecture](backlog/docs/architecture.md): layout and module map
- [decisions](backlog/decisions/): accepted design records
- [PROVENANCE.md](PROVENANCE.md): sources, citations and evidence labels
- [glossary](backlog/docs/glossary.md): family terms
- [AGENTS.md](AGENTS.md): instructions for coding agents

Open work is tracked with beads (`bd`, prefix `x80`); Backlog.md holds docs and
decisions.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
