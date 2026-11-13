---
id: decision-17
title: Two-key evidence rule for undocumented behaviour
date: '2026-11-13'
status: accepted
---

Decider: maintainer (2026-11-13). Draft of the maintainer's binding direction of
2026-11-13; epic `x80-mn3`.

## Context

Decisions 3, 11 and 14 restrict encodings to the documented set. Implementing
undocumented behaviour needs a source policy that can admit written analysis
while requiring confirmation from real silicon. Document class alone does not
establish independence or resolve errors: a second-source document may copy an
earlier analysis, and measurements may distinguish parts within a family.

Research context: `undoc-sources-fable.txt` (2026-11-13), the supplied S1–S15
inventory of candidate sources, classes and URLs. It is context, not a source
pin or authority for an individual hardware fact. Later beads must inspect and
pin the sources themselves, settle their origins and silicon scope, and record
per-fact evidence before implementation.

## Choice

- Documented behaviour keeps the existing rule: derive it only from cited primary
  sources (document/revision, section or table, and page), pin revision and SHA-256,
  and give every implemented fact an independent witness test.
- Undocumented behaviour uses the **two-key rule**. Each fact needs both:

  1. A written statement of the fact from a pinned source of class A
     (manufacturer or licensee/second-source document), B (die analysis or
     hardware measurement) or C (third-party written analysis).
  2. Independent confirmation from real silicon: class B, or class A of a
     different origin.

  Pin both sources by revision/date and SHA-256, with the exact section/table/page
  or equivalent locator. For changing web material, pin a dated snapshot.
  Independence means different origin, not different publisher; a copy counts
  once. Class C alone never licenses a fact. Independent witness tests remain
  required and do not replace the second key.
- Each fact carries one label — "documented by manufacturer", "documented by
  second source (part)", "die analysis (chip)", "hardware-measured (part, by
  whom)" or "third-party analysis" — and names the silicon it holds for. Retain
  the supporting sources' classes and origins. Anything else stays "unverified";
  do not extend a part-specific finding to a family without evidence.
- When classes disagree, silicon evidence prevails over a derivative or erroneous
  document. Record both readings and the ruling; do not silently discard the
  losing reading.
- An oracle alone never licenses a fact. A CRC confirms a stated fact; it cannot
  state one. A program whose expected values come from identified hardware
  (for example, z80test) may supply the independent silicon confirmation for
  facts its groups discriminate, once pinned with that provenance. It does not
  confirm cases its groups cannot distinguish. Unidentified-hardware CRCs
  (ZEXALL, ZEXDOC) remain falsifiers only.
- Never read another emulator's source or documentation. HDL replicas and
  netlist simulators are implementations; only their die-derived data
  (schematics, netlists) is evidence.
- Scope and order: implement undocumented behaviour for the 8085 first, then the
  Zilog NMOS Z80, under epic `x80-mn3`. It is always on for those models, with no
  embedder profile. Later per-model records will supersede decisions 11 and 14
  in part; this record does not supersede them or admit individual encodings,
  flag rules or state changes.
- The 8080 keeps `Unsupported` under decision-3 until a qualifying source is
  found and a later per-model record admits the behaviour. Preserve decision-9's
  V20 ED escape. Documented behaviour and its evidence rules are unchanged.

## Alternatives

- Documents only. Rejected: it cannot cover facts absent from manufacturer or
  second-source documents, and document rank alone cannot resolve copied errors.
- Class C licenses alone, with a label. Rejected: a written third-party claim
  without independent silicon confirmation is insufficient evidence.

## Consequences

- AGENTS.md and PROVENANCE.md distinguish the documented rule from the two-key
  rule. Labels expose the kind and silicon scope of each admitted fact.
- Source pinning and per-model records precede implementation. Candidate research
  and oracle eligibility do not themselves establish a fact or approve a pin.
- Independence must be traced to origin; mirrors, copied tables and analyses of
  the same die data do not supply separate keys merely by changing publisher.
- The 8085 and Zilog NMOS Z80 work is planned, not deferred behind an optional
  profile. The 8080 remains research-only for undocumented behaviour; CMOS and
  other Z80 parts need their own evidence and scope decision.

## Revisit when

New evidence changes a source's origin, silicon identification or reliability;
independent silicon findings conflict; or a later per-model record needs to
extend the named silicon scope. Such records must state their own rulings and
the parts of decisions 11 or 14 they supersede.
