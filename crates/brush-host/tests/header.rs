// Copyright (c) 2026 Polyglotic. All rights reserved.

//! `include/brush_host.h` is what `src/ffi.rs` says it is.
//!
//! To write the header again after changing the interface:
//! `BRUSH_WRITE_HEADER=1 cargo test -p brush-host --test header`

use std::path::Path;

#[test]
fn the_header_is_current() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = cbindgen::Config {
        language: cbindgen::Language::C,
        include_guard: Some("POLYGLOTIC_BRUSH_HOST_H".into()),
        header: Some("/* Copyright (c) 2026 Polyglotic. All rights reserved. */\n/* Written by cbindgen from crates/brush-host/src/ffi.rs. Don't edit. */".into()),
        cpp_compat: true,
        documentation: true,
        ..Default::default()
    };
    let mut made = Vec::new();
    cbindgen::Builder::new()
        .with_config(config)
        .with_src(dir.join("src/ffi.rs"))
        .generate()
        .expect("cbindgen reads ffi.rs")
        .write(&mut made);
    let made = String::from_utf8(made).expect("the header is text").replace("\r\n", "\n");

    let path = dir.join("../../include/brush_host.h");
    if std::env::var_os("BRUSH_WRITE_HEADER").is_some() {
        std::fs::create_dir_all(dir.join("../../include")).expect("the include folder is made");
        std::fs::write(&path, &made).expect("the header is written");
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
    assert!(kept == made, "include/brush_host.h is stale: run this test with BRUSH_WRITE_HEADER=1 and commit it");
}
