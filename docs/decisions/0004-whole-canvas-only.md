# 0004 The whole canvas is simulated every step - never a region that follows the brush

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the user with Claude, 2026-11-20 (polyglotic-hq project 520's plan, section 2; DECISIONS #232)

## Situation

Adobe's US 9,600,907 (active to about 2034-11-25) claims simulating paint only in a brush-following region sized by the device's speed (claims 1, 11, 16). Its full text, dependent claims and foreign filings have not been read yet (polyglotic-hq 520's check).

## Decision

Every simulation step covers the whole grid. **No tiles, no active area, no dirty rectangle that follows the brush or is sized to the device** - not as an optimisation, not as a fallback. A slow device gets a smaller grid (512, 1024, 2048) or fewer steps a frame, never a smaller region.

## Why

It keeps clear of the claims on their face, and the whole canvas is affordable (`docs/M0.md`'s numbers).

## What would reverse it, and how

Claude's or a lawyer's full reading of the claims saying a scheme is clear. Until that is written here as a new record, a pull request adding any region-limited simulation is refused in review. The check: `FlowSim::step` dispatches the whole grid and takes no rectangle.
