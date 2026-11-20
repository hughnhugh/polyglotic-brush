# 0005 Brush footprints are swept on the GPU from a tip model; no bristle physics on a second processor

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the user with Claude, 2026-11-20 (polyglotic-hq project 520's plan, section 2; DECISIONS #232)

## Situation

The same patent's claims 11 and 16 cover bristles simulated on one processor and paint on another.

## Decision

The tip's footprint is computed from the stroke's samples in `brush-core` (plain arithmetic, deterministic) and swept by the GPU. There is no per-bristle physical simulation running on the CPU while the GPU simulates paint.

## Why

Clear of the claims on their face; simpler; replays exactly.

## What would reverse it, and how

The developer's bristle model, after the claims are read in full and a new record says how it stays clear.
