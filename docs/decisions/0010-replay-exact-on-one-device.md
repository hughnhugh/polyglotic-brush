# 0010 A replay is exact on one device and build; across GPUs it is the same to the eye, not to the bit

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the user with Claude, 2026-11-20 (polyglotic-hq project 520's plan, section 2; DECISIONS #232)

## Situation

Float shaders differ by vendor and driver.

## Decision

Everything that must be exact - stroke checks, ink used, ordering - is computed from the stroke data in `brush-core` (fixed point, no floats parsed), never read back from pixels. Golden pictures compare with a tolerance.

## Why

A server can check a stroke without a GPU, and tests don't depend on a driver.

## What would reverse it, and how

A need for bit-exact pictures across devices: a software rasteriser for that one use.
