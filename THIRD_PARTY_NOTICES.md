# Third-party notices

The brush engine is built from the crates below, each under its own licence:
the licence expression its authors declare, as Cargo reads it. This
repository holds none of their code; Cargo fetches them, at the versions in
`Cargo.lock`, when it builds. A program built from it (`brush`,
`brush_host.dll`) contains some of them, and whoever passes such a program
on passes on their licence texts and notices with it.

Every licence here is one `deny.toml` allows (MIT, Apache-2.0, BSD-2-Clause,
ISC, Zlib, Unicode-3.0), and each may be combined with this repository's
Apache-2.0. The list covers every platform and feature, not only what one
build uses. Tools used only in development aren't in any program built from
here and aren't listed: cbindgen (MPL-2.0) writes `include/brush_host.h` in
a test.

Written by `tools/third-party-notices.py` from `cargo deny list` and
`cargo metadata`; CI checks it is current.

| Crate | Version | Licence | Source |
|---|---|---|---|
| allocator-api2 | 0.2.21 | MIT OR Apache-2.0 | <https://github.com/zakarumych/allocator-api2> |
| android_system_properties | 0.1.6 | MIT OR Apache-2.0 | <https://github.com/nical/android_system_properties> |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 | <https://github.com/bluss/arrayvec> |
| ash | 0.38.0+1.3.281 | MIT OR Apache-2.0 | <https://github.com/ash-rs/ash> |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | <https://github.com/cuviper/autocfg> |
| bit-set | 0.10.0 | Apache-2.0 OR MIT | <https://github.com/contain-rs/bit-set> |
| bit-vec | 0.9.1 | Apache-2.0 OR MIT | <https://github.com/contain-rs/bit-vec> |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | <https://github.com/bitflags/bitflags> |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | <https://github.com/Lokathor/bytemuck> |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT | <https://github.com/Lokathor/bytemuck> |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | <https://github.com/rust-lang/cfg-if> |
| cfg_aliases | 0.2.2 | MIT | <https://github.com/katharostech/cfg_aliases> |
| codespan-reporting | 0.13.1 | Apache-2.0 | <https://github.com/brendanzab/codespan> |
| crunchy | 0.2.4 | MIT | <https://github.com/eira-fransham/crunchy> |
| document-features | 0.2.12 | MIT OR Apache-2.0 | <https://github.com/slint-ui/document-features> |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | <https://github.com/indexmap-rs/equivalent> |
| foldhash | 0.2.0 | Zlib | <https://github.com/orlp/foldhash> |
| gpu-allocator | 0.28.0 | MIT OR Apache-2.0 | <https://github.com/Traverse-Research/gpu-allocator> |
| half | 2.7.1 | MIT OR Apache-2.0 | <https://github.com/VoidStarKat/half-rs> |
| hashbrown | 0.16.1 | MIT OR Apache-2.0 | <https://github.com/rust-lang/hashbrown> |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | <https://github.com/rust-lang/hashbrown> |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | <https://github.com/indexmap-rs/indexmap> |
| libc | 0.2.191 | MIT OR Apache-2.0 | <https://github.com/rust-lang/libc> |
| libloading | 0.8.9 | ISC | <https://github.com/nagisa/rust_libloading/> |
| libm | 0.2.16 | MIT | <https://github.com/rust-lang/compiler-builtins> |
| litrs | 1.0.1 | MIT OR Apache-2.0 | <https://github.com/LukasKalbertodt/litrs> |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | <https://github.com/Amanieu/parking_lot> |
| log | 0.4.35 | MIT OR Apache-2.0 | <https://github.com/rust-lang/log> |
| naga | 30.0.2 | MIT OR Apache-2.0 | <https://github.com/gfx-rs/wgpu> |
| naga-types | 30.0.2 | MIT OR Apache-2.0 | <https://github.com/gfx-rs/wgpu> |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | <https://github.com/rust-num/num-traits> |
| objc2 | 0.6.5 | MIT | <https://github.com/madsmtm/objc2> |
| objc2-core-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://github.com/madsmtm/objc2> |
| objc2-encode | 4.1.0 | MIT | <https://github.com/madsmtm/objc2> |
| objc2-foundation | 0.3.2 | MIT | <https://github.com/madsmtm/objc2> |
| objc2-metal | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://github.com/madsmtm/objc2> |
| objc2-quartz-core | 0.3.2 | Zlib OR Apache-2.0 OR MIT | <https://github.com/madsmtm/objc2> |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | <https://github.com/matklad/once_cell> |
| ordered-float | 5.5.1 | MIT | <https://github.com/reem/rust-ordered-float> |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | <https://github.com/Amanieu/parking_lot> |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | <https://github.com/Amanieu/parking_lot> |
| pollster | 1.0.3 | MIT OR Apache-2.0 | <https://github.com/zesterer/pollster> |
| portable-atomic | 1.15.0 | Apache-2.0 OR MIT | <https://github.com/taiki-e/portable-atomic> |
| portable-atomic-util | 0.2.8 | Apache-2.0 OR MIT | <https://github.com/taiki-e/portable-atomic-util> |
| presser | 0.3.1 | MIT OR Apache-2.0 | <https://github.com/EmbarkStudios/presser> |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | <https://github.com/dtolnay/proc-macro2> |
| profiling | 1.0.18 | MIT OR Apache-2.0 | <https://github.com/aclysma/profiling> |
| quote | 1.0.47 | MIT OR Apache-2.0 | <https://github.com/dtolnay/quote> |
| range-alloc | 0.1.5 | MIT OR Apache-2.0 | <https://github.com/gfx-rs/range-alloc> |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | <https://github.com/rust-windowing/raw-window-handle> |
| raw-window-metal | 1.1.0 | MIT OR Apache-2.0 | <https://github.com/rust-windowing/raw-window-metal> |
| redox_syscall | 0.5.18 | MIT | <https://gitlab.redox-os.org/redox-os/syscall> |
| renderdoc-sys | 1.1.0 | MIT OR Apache-2.0 | <https://github.com/ebkalderon/renderdoc-rs> |
| rustc-hash | 1.1.0 | Apache-2.0/MIT | <https://github.com/rust-lang-nursery/rustc-hash> |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | <https://github.com/bluss/scopeguard> |
| smallvec | 1.16.6 | MIT OR Apache-2.0 | <https://github.com/smallvec/smallvec> |
| spirv | 0.4.0+sdk-1.4.341.0 | Apache-2.0 | <https://github.com/gfx-rs/rspirv> |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | <https://github.com/nvzqz/static-assertions-rs> |
| syn | 2.0.119 | MIT OR Apache-2.0 | <https://github.com/dtolnay/syn> |
| syn | 3.0.10 | MIT OR Apache-2.0 | <https://github.com/dtolnay/syn> |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | <https://github.com/dtolnay/thiserror> |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | <https://github.com/dtolnay/thiserror> |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | <https://github.com/dtolnay/unicode-ident> |
| unicode-width | 0.2.3 | MIT OR Apache-2.0 | <https://github.com/unicode-rs/unicode-width> |
| wgpu | 30.0.2 | MIT OR Apache-2.0 | <https://github.com/gfx-rs/wgpu> |
| wgpu-core | 30.0.2 | MIT OR Apache-2.0 | <https://github.com/gfx-rs/wgpu> |
| wgpu-core-deps-windows-linux-android | 30.0.2 | MIT OR Apache-2.0 | <https://github.com/gfx-rs/wgpu> |
| wgpu-hal | 30.0.2 | MIT OR Apache-2.0 | <https://github.com/gfx-rs/wgpu> |
| wgpu-naga-bridge | 30.0.2 | MIT OR Apache-2.0 | <https://github.com/gfx-rs/wgpu> |
| wgpu-types | 30.0.2 | MIT OR Apache-2.0 | <https://github.com/gfx-rs/wgpu> |
| windows | 0.62.2 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-collections | 0.3.2 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-core | 0.62.2 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-future | 0.3.2 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-implement | 0.60.2 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-interface | 0.59.3 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-numerics | 0.3.1 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-result | 0.4.1 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-strings | 0.5.1 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| windows-threading | 0.2.1 | MIT OR Apache-2.0 | <https://github.com/microsoft/windows-rs> |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | <https://github.com/google/zerocopy> |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | <https://github.com/google/zerocopy> |
