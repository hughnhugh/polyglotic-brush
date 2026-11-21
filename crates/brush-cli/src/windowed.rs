// Copyright 2026 Polyglotic
// SPDX-License-Identifier: Apache-2.0

//! The commands that open the canvas's window (Windows).

use std::time::{Duration, Instant};

use brush_host::overlay::{Input, Options, Overlay};

use crate::args::Args;
use crate::report::Report;

fn rect(args: &mut Args, default: (i32, i32, i32, i32)) -> Result<(i32, i32, i32, i32), String> {
    match args.numbers::<i32>("--rect")? {
        None => Ok(default),
        Some(v) if v.len() == 4 && v[2] > 0 && v[3] > 0 => Ok((v[0], v[1], v[2], v[3])),
        Some(_) => Err("--rect wants X,Y,W,H".into()),
    }
}

/// Keep the canvas open for `seconds`, or until it is closed. With `script`,
/// post a made-up stroke at 240 samples a second: a looping curve whose
/// pressure swells, lifted for a moment every two seconds.
fn hold(overlay: &Overlay, size: (i32, i32), seconds: f64, script: bool) {
    const HZ: f64 = 240.0;
    let start = Instant::now();
    let mut i = 0u64;
    while overlay.is_open() {
        let t = i as f64 / HZ;
        if t >= seconds {
            break;
        }
        if script {
            let (w, h) = (f64::from(size.0), f64::from(size.1));
            let x = w / 2.0 + 0.42 * w * (std::f64::consts::TAU * 0.35 * t).sin();
            let y = h / 2.0 + 0.36 * h * (std::f64::consts::TAU * 0.55 * t).sin();
            let pressure = 0.5 + 0.5 * (3.0 * t).sin();
            overlay.inject(x as i16, y as i16, pressure as f32, t % 2.0 < 1.8);
        }
        i += 1;
        let next = start + Duration::from_secs_f64(i as f64 / HZ);
        std::thread::sleep(next.saturating_duration_since(Instant::now()));
    }
}

pub fn overlay(args: &mut Args) -> Result<(), String> {
    let owner = match args.value("--owner") {
        None => 0,
        Some(word) => word.parse::<isize>().map_err(|_| format!("--owner wants a window's number, not '{word}'"))?,
    };
    let rect = rect(args, (100, 100, 800, 300))?;
    let seconds = f64::from(args.number("--seconds")?.unwrap_or(10));
    let input = match args.value("--input").as_deref() {
        Some("on") => Input::On,
        None | Some("off") => Input::Off,
        Some(other) => return Err(format!("--input is on or off, not '{other}'")),
    };
    let sim = match args.numbers::<u32>("--sim")? {
        None => None,
        Some(v) if v.len() == 2 => Some((v[0], v[1])),
        Some(_) => return Err("--sim wants GRID,STEPS".into()),
    };
    let tint = match args.value("--tint") {
        None => 0.0,
        Some(word) => word.parse::<f32>().map_err(|_| format!("--tint wants 0.0 to 1.0, not '{word}'"))?,
    };
    let script = args.flag("--script");
    let options = Options { owner, rect, input, on_top: true, sim, adapter: args.value("--adapter"), tint };
    let mut report = Report::new("brush overlay", args.value("--out"));
    report.line(format!(
        "asked: owner {owner} rect {rect:?} input {input:?} sim {sim:?} script {script} for {seconds} s"
    ));
    let overlay = Overlay::open(options)?;
    report.line(format!("canvas window: {}", overlay.hwnd()));
    hold(&overlay, (rect.2, rect.3), seconds, script);
    report.lines(overlay.close().lines());
    report.finish()
}

pub fn pen(args: &mut Args) -> Result<(), String> {
    let rect = rect(args, (160, 120, 1280, 720))?;
    let seconds = f64::from(args.number("--seconds")?.unwrap_or(60));
    let mut report = Report::new("brush pen", args.value("--out").or_else(|| Some("brush-pen-report.txt".into())));
    println!("A lightly washed rectangle is the canvas. Draw on it with the pen: slow lines,");
    println!("fast lines, light and heavy, tilted both ways. Right-click it to finish early.");
    let options = Options { rect, input: Input::On, tint: 0.14, ..Options::default() };
    let overlay = Overlay::open(options)?;
    hold(&overlay, (rect.2, rect.3), seconds, false);
    let stats = overlay.close();
    report.lines(stats.lines());
    if stats.pen_samples == 0 && stats.mouse_samples > 0 {
        report.line(
            "note: the strokes arrived as a MOUSE, not a pen: no pressure or tilt. On a Wacom tablet, \
             turn on \"Use Windows Ink\" in Wacom Tablet Properties (the pen's Mapping tab) and run again."
                .into(),
        );
    }
    report.finish()
}

pub fn floor(args: &mut Args) -> Result<(), String> {
    let mut report = Report::new("brush floor", args.value("--out").or_else(|| Some("brush-floor-report.txt".into())));
    report.line("== 1. the simulation on every adapter ==".into());
    crate::bench_into(&mut report, true, None, &[512, 1024, 2048], Duration::from_millis(3000))?;

    // The floor is the integrated adapter: draw the canvas with it when there
    // is one, even beside a faster card.
    let instance = brush_engine::gpu::instance(wgpu::Backends::DX12);
    let integrated = brush_engine::gpu::adapters(&instance)
        .iter()
        .map(wgpu::Adapter::get_info)
        .find(|info| info.device_type == wgpu::DeviceType::IntegratedGpu)
        .map(|info| info.name);
    drop(instance);
    match &integrated {
        Some(name) => report.line(format!("the canvas is drawn by the integrated adapter: {name}")),
        None => report.line("no integrated adapter here: the canvas is drawn by the system's choice".into()),
    }

    let rect = (120, 120, 960, 360);
    let runs: [(&str, Option<(u32, u32)>); 3] = [
        ("== 2. the canvas alone, 6 seconds ==", None),
        ("== 3. the canvas with a 512 simulation, 1 step a frame, 6 seconds ==", Some((512, 1))),
        ("== 4. the canvas with a 1024 simulation, 1 step a frame, 6 seconds ==", Some((1024, 1))),
    ];
    for (title, sim) in runs {
        report.line(title.into());
        let options =
            Options { rect, input: Input::Off, sim, tint: 0.14, adapter: integrated.clone(), ..Options::default() };
        match Overlay::open(options) {
            Ok(overlay) => {
                hold(&overlay, (rect.2, rect.3), 6.0, true);
                report.lines(overlay.close().lines());
            }
            Err(why) => report.line(format!("the canvas didn't open: {why}")),
        }
    }
    report.finish()
}
