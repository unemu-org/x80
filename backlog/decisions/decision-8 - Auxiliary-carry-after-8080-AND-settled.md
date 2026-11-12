---
id: decision-8
title: Auxiliary carry after 8080 AND settled
date: '2026-11-12'
status: accepted
---

Decider: maintainer (2026-11-12). Supersedes decision-7, which kept the same
reading as an unverified interim. Bead `x80-ecy.2`.

## Context

The two pinned Intel sources disagree on the auxiliary carry (AC) after the 8080
logical AND instructions (PROVENANCE.md §1):

- 98-153B p. 4-9: ANI "The CY and AC flags are cleared"; ANA (p. 4-8) only says
  CY is cleared.
- 9800301-04 p. 1-12: "The 8080 logical AND instructions set the flag to reflect
  the logical OR of bit 3 of the values involved in the AND operation".

decision-7 adopted the second reading as unverified until an oracle could be run.
The pinned 8080EXM exerciser (PROVENANCE.md §2) now runs in the `x80-oracle`
harness. Evidence, 2026-11-12:

- MEASURED: with the 9800301-04 reading, 8080EXM passes all 25 groups, including
  "aluop nn" (CRC 9e922f9e) and "aluop <b,c,d,e,h,l,m,a>" (cf762c86).
- MEASURED: with "AC cleared" for ANA and ANI (a throwaway build, not committed),
  only those two groups fail, with 7799ea9d and b3491c2a.
- READ: the exerciser author's results page, pinned fetch-only in `oracles/`
  (`8080-exercisers/8080-8085-cpu-exerciser-results.pdf`, pp. 3-4, "Results"): the
  expected CRCs are the ones reported for Intel 8080A (four chips), NEC, NS, TI,
  KR580, Tesla and Samsung parts; the AMD 9080A and 8080A rows report 7799EA9D
  and B3491C2A, with a contributor note that AMD's ANA/ANI flags match ORA and
  XRA. These are contributor measurements on a web page, not Intel documentation.

## Choice

- For the `i8080` model (the Intel 8080 as described by 98-153B), ANA r, ANA M
  and ANI set AC to bit 3 of (A OR operand) and clear CY, per 9800301-04 p. 1-12.
  The reading is no longer interim.
- The authority is the primary source, 9800301-04 p. 1-12. The oracle does not
  license it; it falsifies the competing reading, 98-153B p. 4-9 applied to AND,
  for the parts whose results the page reports as Intel 8080A.
- The code drops the "provisional" labels and cites this record and the oracle
  evidence. Its witnesses stay as written.
- AMD-like behaviour (AC cleared) is not represented. Under decision-6 it could
  become a variant only once a pinned primary source documents it; the results
  page is oracle evidence, not such a source.

## Alternatives

- Keep decision-7 as interim. Rejected by the maintainer: the oracle distinguishes
  the readings, and the CRC table is now pinned and cited.
- Settle without pinning the results page. Rejected: the CRCs' provenance would
  rest on an unpinned web page.

## Consequences

- PROVENANCE.md's disagreement row moves from interim to settled, citing both
  sources and the oracle evidence.
- 98-153B p. 4-9's ANI wording stays recorded as a known disagreement in the
  primary source.

## Revisit when

A pinned Intel source states otherwise, or an AMD or other second-source variant
is brought into scope (decision-6).
