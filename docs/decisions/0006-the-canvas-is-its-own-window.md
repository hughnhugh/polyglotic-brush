# 0006 On Windows the canvas is its own frameless window, owned by the host's, on a DirectComposition swap chain

- **Status:** accepted
- **Date:** 2026-11-21
- **Decided by:** the user with Claude, 2026-11-20 (polyglotic-hq project 520's plan, section 2; DECISIONS #232)

## Situation

A flip-model child window inside Qt's translucent window doesn't composite (QTBUG-105537). The engine also wants the pen's messages on its own thread.

## Decision

`brush-host` makes a `WS_POPUP` window with `WS_EX_NOREDIRECTIONBITMAP`, owned by the host's window, never activated; wgpu's `Dx12SwapchainKind::DxgiFromVisual` gives it a premultiplied-alpha swap chain. Click-through is `WS_EX_TRANSPARENT | WS_EX_LAYERED`, switched by the host. Window, messages and frames are one thread.

## Why

Transparency that composites, and no Python or Qt between the pen and the GPU.

## What would reverse it, and how

M0 showing it flickers or mis-orders over the caption window (then: an opaque paper pane and Qt-drawn pointer ink), or Qt fixing child composition. Only `brush-host/src/overlay.rs` changes.
