# 0007 The host loads the engine through a C interface

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the user with Claude, 2026-11-20 (polyglotic-hq project 520's plan, section 2; DECISIONS #232)

## Situation

The desktop app is Python and Qt; later hosts are Swift, Kotlin, the web, and the developer's own tools.

## Decision

`brush-host` exports C functions (`include/brush_host.h`, written by cbindgen; a test fails when it is stale). A canvas is an opaque pointer; the host is never called back, it polls; `BRUSH_ABI_VERSION` rises on a breaking change.

## Why

One stable surface for every language, with no tie to a Python version.

## What would reverse it, and how

A second Python host needing richer types: PyO3 beside it, not instead.
