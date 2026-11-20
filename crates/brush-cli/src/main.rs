// Copyright (c) 2026 Polyglotic. All rights reserved.

//! `brush`: the engine from the command line. M0's commands measure; they
//! write numbers and adapter names, and nothing about the person or the PC.

mod args;
mod report;
#[cfg(windows)]
mod windowed;

use std::time::Duration;

use args::Args;
use report::Report;

const HELP: &str = "\
brush: Polyglotic's canvas engine, M0's measurements

  brush adapters
      List the graphics adapters.
  brush bench [--adapter NAME | --all] [--grids 512,1024,2048] [--budget-ms 3000] [--out FILE]
      Time the whole-canvas simulation on an adapter (the system's choice,
      one whose name contains NAME, or every adapter).
  brush overlay [--owner HWND] [--rect X,Y,W,H] [--seconds 10] [--input on|off]
                [--script] [--sim GRID,STEPS] [--adapter NAME] [--tint 0.0] [--out FILE]
      Open the transparent canvas and measure its frames. --script draws a
      made-up stroke. A right click on the canvas ends it early.
  brush pen [--seconds 60] [--rect X,Y,W,H] [--out FILE]
      The pen test: a lightly washed canvas that takes the pen. Draw on it;
      the report says what arrived (pressure, tilt, rate) and how long each
      sample took to reach Present.
  brush floor [--out FILE]
      The two-minute test for a PC with integrated graphics: every adapter
      benched, then the canvas on screen for 20 seconds.
";

fn main() -> std::process::ExitCode {
    let mut args = Args::from_env();
    let command = args.command();
    let outcome = match command.as_deref() {
        Some("adapters") => adapters(),
        Some("bench") => bench(&mut args),
        #[cfg(windows)]
        Some("overlay") => windowed::overlay(&mut args),
        #[cfg(windows)]
        Some("pen") => windowed::pen(&mut args),
        #[cfg(windows)]
        Some("floor") => windowed::floor(&mut args),
        #[cfg(not(windows))]
        Some("overlay" | "pen" | "floor") => Err("this command needs Windows".into()),
        _ => {
            print!("{HELP}");
            return std::process::ExitCode::from(2);
        }
    };
    match outcome {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("brush: {why}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn adapters() -> Result<(), String> {
    let instance = brush_engine::gpu::instance(wgpu::Backends::all());
    for adapter in brush_engine::gpu::adapters(&instance) {
        println!("{}", brush_engine::gpu::describe(&adapter));
    }
    Ok(())
}

fn bench(args: &mut Args) -> Result<(), String> {
    let all = args.flag("--all");
    let name = args.value("--adapter");
    let grids = args.numbers("--grids")?.unwrap_or_else(|| vec![512, 1024, 2048]);
    let budget = Duration::from_millis(u64::from(args.number("--budget-ms")?.unwrap_or(3000)));
    let mut report = Report::new("brush bench", args.value("--out"));
    bench_into(&mut report, all, name.as_deref(), &grids, budget)?;
    report.finish()
}

/// Bench the chosen adapters into `report`.
pub(crate) fn bench_into(report: &mut Report, all: bool, name: Option<&str>, grids: &[u32], budget: Duration) -> Result<(), String> {
    let instance = brush_engine::gpu::instance(wgpu::Backends::all());
    let every = brush_engine::gpu::adapters(&instance);
    let wanted: Vec<&wgpu::Adapter> = if all {
        every.iter().collect()
    } else if let Some(name) = name {
        let name = name.to_lowercase();
        every.iter().filter(|a| a.get_info().name.to_lowercase().contains(&name)).take(1).collect()
    } else {
        let pick = every
            .iter()
            .find(|a| a.get_info().device_type == wgpu::DeviceType::DiscreteGpu)
            .or_else(|| every.iter().find(|a| a.get_info().device_type == wgpu::DeviceType::IntegratedGpu))
            .or(every.first());
        pick.into_iter().collect()
    };
    if wanted.is_empty() {
        return Err("no adapter matches".into());
    }
    for adapter in wanted {
        report.line(format!("adapter: {}", brush_engine::gpu::describe(adapter)));
        let (device, queue) = match brush_engine::gpu::open(adapter) {
            Ok(pair) => pair,
            Err(why) => {
                report.line(format!("  no device: {why}"));
                continue;
            }
        };
        for &n in grids {
            let row = brush_engine::bench(&device, &queue, n, budget);
            report.line(format!(
                "  {n} x {n}: flow step {:.3} ms; flow + lit picture {:.3} ms; steps in half a frame: {:.1} at 60 Hz, {:.1} at 120 Hz ({} steps a run; inked {:.4})",
                row.flow_ms,
                row.both_ms,
                row.steps_per_frame(60.0),
                row.steps_per_frame(120.0),
                row.steps,
                row.inked
            ));
        }
    }
    Ok(())
}
