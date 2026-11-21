// Copyright 2026 Polyglotic
// SPDX-License-Identifier: Apache-2.0

//! A report: lines printed as they come and, when asked, saved to a file.
//! Numbers and adapter names only.

use std::io::Write;

#[derive(Debug)]
pub struct Report {
    lines: Vec<String>,
    out: Option<String>,
}

impl Report {
    pub fn new(title: &str, out: Option<String>) -> Self {
        let mut report = Self { lines: Vec::new(), out };
        report.line(format!(
            "{title} {} (interface {})",
            env!("CARGO_PKG_VERSION"),
            brush_host::ffi::BRUSH_ABI_VERSION
        ));
        report
    }

    pub fn line(&mut self, line: String) {
        println!("{line}");
        let _ = std::io::stdout().flush();
        self.lines.push(line);
    }

    /// Several lines at once: only the Windows commands (the overlay, the pen) have them.
    #[cfg(windows)]
    pub fn lines(&mut self, lines: impl IntoIterator<Item = String>) {
        for line in lines {
            self.line(line);
        }
    }

    /// Save the report, if a file was asked for.
    pub fn finish(self) -> Result<(), String> {
        let Some(path) = self.out else {
            return Ok(());
        };
        let text = self.lines.join("\n") + "\n";
        std::fs::write(&path, text).map_err(|e| format!("the report wasn't saved to {path}: {e}"))?;
        println!("saved: {path}");
        Ok(())
    }
}
