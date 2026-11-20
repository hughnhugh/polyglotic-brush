# 0001 The picture is drawn on the learner's own device

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the user with Claude, 2026-11-20 (polyglotic-hq project 520's plan, section 2; DECISIONS #232)

## Situation

A server could draw the canvas and stream it. From Victoria only Australia and New Zealand would be under the ~50 ms at which inking feels attached to the pen (polyglotic-hq 520: round trips of 92-268 ms elsewhere).

## Decision

No server draws live ink. The engine runs on the learner's device; strokes stay there unless the learner shares them.

## Why

Costs nothing per hour, works offline, keeps strokes private.

## What would reverse it, and how

Nothing foreseeable for live ink. A server may later re-render *stored* pieces (thumbnails): that is an addition, not a reversal.
