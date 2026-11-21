# 0013 The engine is open source, under Apache-2.0

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the user, 2026-11-21 (polyglotic-hq DECISIONS #254), as the app and the site are (#121); built by polyglotic-hq project 589

## Situation

The repository began private, its files headed "All rights reserved" as a
placeholder until a licence was chosen. Its CI could not run on GitHub's
machines while it was private (0012). The app and the website are already
public under Apache-2.0.

## Decision

- **The licence is Apache-2.0:** `LICENSE` (the licence's own text),
  `NOTICE`, and `license = "Apache-2.0"` for every crate (the workspace's
  `Cargo.toml`).
- **Every Rust and WGSL file starts with** `// Copyright 2026 Polyglotic`
  and `// SPDX-License-Identifier: Apache-2.0`; the C header, written by
  cbindgen, with the same in one line. CI fails a file without it.
- **The crates the engine is built from** are listed with their licences in
  `THIRD_PARTY_NOTICES.md`, written by `tools/third-party-notices.py` from
  `cargo deny list` and `cargo metadata`; CI fails when it is stale.
  `cargo deny check` now checks our own crates' licence too.
- **The history is published as it was,** with every commit's author and
  committer the account's GitHub address.

## Why

- Free CI on Windows, Linux and macOS (0014).
- The same terms as the app that loads the engine.
- Apache-2.0's patent grant, and every dependency's licence is compatible
  with it (all are MIT, Apache-2.0, BSD-2-Clause, ISC, Zlib or Unicode-3.0).
- The whole-canvas rule (0004) is easier to show kept when anyone can read
  the simulation.

## What would reverse it, and how

Code already published under Apache-2.0 stays available under it. A later
change of licence applies only to what is written after it: a new record,
`LICENSE` and `NOTICE` replaced, and the headers changed in one commit.
