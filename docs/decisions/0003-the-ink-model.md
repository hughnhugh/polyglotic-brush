# 0003 The ink is a MoXi-style model: surface water, flow in the paper, fixed pigment, a height field for light

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the user with Claude, 2026-11-20 (polyglotic-hq project 520's plan, section 2; DECISIONS #232)

## Situation

East Asian ink in absorbent paper has a published model (Chu and Tai 2005) that is one cheap fixed-step pass.

## Decision

A D2Q9 lattice-Boltzmann flow in the paper with partial bounce-back from the fibres; pigment carried and fixed; a height field lit in the look. M0 has the flow pass and a first look only (`shaders/`).

## Why

Published, cheap, and Adobe's patent on a MoXi-like model (US 8,296,668) lapsed 2024-10-23.

## What would reverse it, and how

A better model from the brush developer. `brush-sim` is one crate behind one small surface (`FlowSim`); it becomes a trait when there is a second implementation.
