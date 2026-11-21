// Copyright 2026 Polyglotic
// SPDX-License-Identifier: Apache-2.0

//! The canvas on Windows: a frameless, transparent window of its own, owned
//! by the host's window, drawn through a DirectComposition swap chain
//! (`docs/decisions/0006`).
//!
//! The window, its messages and its frames all live on one thread that the
//! engine starts, so nothing of the host's (no Python, no Qt) sits between
//! the pen and the GPU. The host talks to it by posting messages and reads
//! numbers back; it is never called back.
//!
//! Each frame: wait until the swap chain takes a frame, read every message
//! that has arrived (so input is read as late as it can be), draw, present.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::num::NonZeroIsize;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use brush_core::{Sample, Source};
use brush_engine::Engine;
use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetCompositionTimingInfo, DWM_TIMING_INFO};
use windows::Win32::Graphics::Dxgi::{IDXGISwapChain3, DXGI_FRAME_STATISTICS};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows::Win32::UI::HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows::Win32::UI::Input::Pointer::{
    GetPointerInfo, GetPointerPenInfoHistory, GetPointerType, POINTER_FLAG_INCONTACT, POINTER_INFO, POINTER_PEN_INFO,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetWindowLongPtrW, IsWindow, PeekMessageW,
    PostMessageW, RegisterClassW, SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, CREATESTRUCTW,
    GWLP_USERDATA, GWL_EXSTYLE, HWND_TOPMOST, MA_NOACTIVATE, MSG, PM_NOREMOVE, PM_REMOVE, PT_PEN, PT_TOUCH,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE, WM_APP,
    WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_NCCREATE, WM_POINTERDOWN,
    WM_POINTERUP, WM_POINTERUPDATE, WM_RBUTTONUP, WM_SIZE, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

use crate::stats::Stats;

/// Whether the canvas takes the pen and the mouse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// Clicks pass through to whatever is underneath; nothing is drawn.
    Off,
    /// The canvas takes the pen and the mouse and draws.
    On,
}

/// How to open a canvas.
#[derive(Clone, Debug)]
pub struct Options {
    /// The host's window, which the canvas stays above; 0 for none.
    pub owner: isize,
    /// Left, top, width, height in screen pixels.
    pub rect: (i32, i32, i32, i32),
    pub input: Input,
    /// Keep above every ordinary window, as the caption window is.
    pub on_top: bool,
    /// A whole-canvas simulation to step each frame: its grid and the steps a frame.
    pub sim: Option<(u32, u32)>,
    /// Draw with the adapter whose name contains this; otherwise the system's choice.
    pub adapter: Option<String>,
    /// A white wash over the canvas, 0.0 (none) to 1.0, so a test can see where it is.
    pub tint: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            owner: 0,
            rect: (100, 100, 800, 300),
            input: Input::Off,
            on_top: true,
            sim: None,
            adapter: None,
            tint: 0.0,
        }
    }
}

/// Samples kept before a new stroke clears the canvas (M0 only: pointer ink
/// gets lifetimes in M1).
const KEPT_SAMPLES: usize = 4000;

const WM_BRUSH_SAMPLE: u32 = WM_APP + 1;
const WM_BRUSH_INPUT: u32 = WM_APP + 2;
const WM_BRUSH_RECT: u32 = WM_APP + 3;
const WM_BRUSH_CLOSE: u32 = WM_APP + 4;
const WM_BRUSH_SHOW: u32 = WM_APP + 5;

/// What the host's threads and the canvas's thread share.
#[derive(Debug, Default)]
struct Shared {
    hwnd: AtomicIsize,
    stats: Mutex<Stats>,
    /// When each injected sample was posted, oldest first.
    injected: Mutex<VecDeque<i64>>,
    rect: Mutex<(i32, i32, i32, i32)>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// An open canvas. Dropping it closes the window.
#[derive(Debug)]
pub struct Overlay {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

impl Overlay {
    /// Open the canvas on a thread of its own, and wait until its window is up.
    ///
    /// # Errors
    /// In words, when the window, the adapter or a transparent target can't be had.
    pub fn open(options: Options) -> Result<Self, String> {
        let shared = Arc::new(Shared::default());
        *lock(&shared.rect) = options.rect;
        let (ready, is_ready) = mpsc::channel();
        let theirs = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("brush-canvas".into())
            .spawn(move || {
                if let Err(why) = run(&options, &theirs, &ready) {
                    lock(&theirs.stats).notes.push(why.clone());
                    let _ = ready.send(Err(why));
                }
            })
            .map_err(|e| format!("the canvas's thread didn't start: {e}"))?;
        // The canvas's window is owned by the host's, and Windows sends the
        // owner's thread messages while it is made. If that thread is the one
        // waiting here, it must keep answering them, or both wait for ever.
        loop {
            match is_ready.try_recv() {
                Ok(Ok(())) => return Ok(Self { shared, thread: Some(thread) }),
                Ok(Err(why)) => return Err(why),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err("the canvas's thread ended before its window was up".into())
                }
                Err(mpsc::TryRecvError::Empty) => answer_sent_messages(),
            }
        }
    }

    /// The canvas's window, as an `HWND`'s number.
    #[must_use]
    pub fn hwnd(&self) -> isize {
        self.shared.hwnd.load(Ordering::SeqCst)
    }

    fn post(&self, message: u32, wparam: usize, lparam: isize) {
        let hwnd = HWND(self.hwnd() as *mut _);
        // SAFETY: posting to a window handle is sound whatever its state; a
        // window that has gone makes the call fail, which is ignored.
        let _ = unsafe { PostMessageW(Some(hwnd), message, WPARAM(wparam), LPARAM(lparam)) };
    }

    /// Take the pen and the mouse, or let them through.
    pub fn set_input(&self, input: Input) {
        self.post(WM_BRUSH_INPUT, usize::from(input == Input::On), 0);
    }

    /// Show or hide the canvas. Windows hides an owned window only when its
    /// owner is minimised, so a host that hides its window says so here. A
    /// hidden canvas draws nothing.
    pub fn set_visible(&self, visible: bool) {
        self.post(WM_BRUSH_SHOW, usize::from(visible), 0);
    }

    /// Move and size the canvas, in screen pixels.
    pub fn set_rect(&self, rect: (i32, i32, i32, i32)) {
        *lock(&self.shared.rect) = rect;
        self.post(WM_BRUSH_RECT, 0, 0);
    }

    /// Post a made-up sample at a point inside the canvas, stamped now: for
    /// tests, which have no pen. `pressure` is 0.0 to 1.0; `down` false lifts.
    pub fn inject(&self, x: i16, y: i16, pressure: f32, down: bool) {
        lock(&self.shared.injected).push_back(now());
        let pressure = (pressure.clamp(0.0, 1.0) * 65535.0) as usize;
        let packed = (x as u16 as isize) | ((y as u16 as isize) << 16);
        self.post(WM_BRUSH_SAMPLE, (pressure << 1) | usize::from(down), packed);
    }

    /// The measurements so far.
    #[must_use]
    pub fn stats(&self) -> Stats {
        lock(&self.shared.stats).clone()
    }

    /// Whether the canvas's thread is still running (it ends when the window
    /// is closed, by the host, by its owner going, or by a right click).
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.thread.as_ref().is_some_and(|t| !t.is_finished())
    }

    /// Close the window and return the final measurements.
    #[must_use]
    pub fn close(mut self) -> Stats {
        self.shut();
        self.stats()
    }

    fn shut(&mut self) {
        if let Some(thread) = self.thread.take() {
            self.post(WM_BRUSH_CLOSE, 0, 0);
            // As in `open`: destroying an owned window can send to this thread.
            while !thread.is_finished() {
                answer_sent_messages();
            }
            let _ = thread.join();
        }
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        self.shut();
    }
}

/// Let this thread answer messages other threads have *sent* it (posted ones
/// stay in its queue for the host's own loop), then rest a millisecond.
fn answer_sent_messages() {
    let mut message = MSG::default();
    // SAFETY: `message` is a live local. With PM_NOREMOVE nothing is taken
    // from the queue; the call's effect is to run pending sent messages.
    let _ = unsafe { PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE) };
    std::thread::sleep(std::time::Duration::from_millis(1));
}

/// The performance counter now.
fn now() -> i64 {
    let mut ticks = 0;
    // SAFETY: the pointer is to a live local; the call can't fail on any
    // Windows this runs on.
    let _ = unsafe { QueryPerformanceCounter(&mut ticks) };
    ticks
}

/// The performance counter's ticks a second.
fn frequency() -> i64 {
    let mut per_second = 0;
    // SAFETY: as in `now`.
    let _ = unsafe { QueryPerformanceFrequency(&mut per_second) };
    per_second.max(1)
}

/// A sample waiting for the next frame.
#[derive(Clone, Copy, Debug)]
struct Pending {
    /// Starts a new stroke.
    begin: bool,
    sample: Sample,
    /// The sample's own time, in performance-counter ticks.
    at: i64,
}

/// What the window's messages have gathered since the last frame.
#[derive(Debug, Default)]
struct Window {
    pending: Vec<Pending>,
    drawing: bool,
    stroke_began: i64,
    resized: Option<(u32, u32)>,
    close: bool,
    hidden: bool,
    pen: u64,
    mouse: u64,
    touch: u64,
    injected: u64,
    pressure: Option<(u32, u32)>,
    tilt_x: Option<(i32, i32)>,
    tilt_y: Option<(i32, i32)>,
    /// Ticks the pen has spent down, for its sample rate.
    pen_down_ticks: i64,
    pen_last: Option<i64>,
    shared: Option<Arc<Shared>>,
    ticks_per_second: i64,
}

fn widen<T: Copy + PartialOrd>(range: &mut Option<(T, T)>, value: T) {
    *range = Some(match *range {
        None => (value, value),
        Some((lo, hi)) => (if value < lo { value } else { lo }, if value > hi { value } else { hi }),
    });
}

impl Window {
    /// Take one reading at a point in the window. `contact` false ends the stroke.
    fn take(&mut self, (x, y): (f32, f32), pressure: f32, tilt: (f32, f32), at: i64, source: Source, contact: bool) {
        if !contact {
            self.drawing = false;
            self.pen_last = None;
            return;
        }
        let begin = !self.drawing;
        if begin {
            self.drawing = true;
            self.stroke_began = at;
        }
        let micros = (at - self.stroke_began).max(0) * 1_000_000 / self.ticks_per_second.max(1);
        let sample = Sample::from_pixels(x, y, pressure, micros.min(i64::from(u32::MAX)) as u32, source)
            .with_tilt(tilt.0, tilt.1);
        self.pending.push(Pending { begin, sample, at });
    }
}

/// The low and high halves of a mouse message's `LPARAM`, as signed pixels.
fn point(lparam: LPARAM) -> (f32, f32) {
    let x = (lparam.0 & 0xffff) as u16 as i16;
    let y = ((lparam.0 >> 16) & 0xffff) as u16 as i16;
    (f32::from(x), f32::from(y))
}

/// Read a pointer message: every pen reading since the last one, with its
/// pressure, tilt and own time. True if the message was ours to handle.
fn pointer(hwnd: HWND, window: &mut Window, wparam: WPARAM) -> bool {
    let id = (wparam.0 & 0xffff) as u32;
    let mut kind = Default::default();
    // SAFETY: `kind` is a live local the call writes to.
    if unsafe { GetPointerType(id, &mut kind) }.is_err() {
        return false;
    }
    let client = |screen: POINT| {
        let mut p = screen;
        // SAFETY: `p` is a live local; `hwnd` is the window this message came to.
        let _ = unsafe { ScreenToClient(hwnd, &mut p) };
        (p.x as f32, p.y as f32)
    };
    if kind == PT_PEN {
        let mut history = [POINTER_PEN_INFO::default(); 128];
        let mut count = history.len() as u32;
        // SAFETY: `history` has room for `count` entries, as the call requires.
        if unsafe { GetPointerPenInfoHistory(id, &mut count, Some(history.as_mut_ptr())) }.is_err() {
            return false;
        }
        // Newest first from Windows; a stroke wants oldest first.
        for info in history[..(count as usize).min(history.len())].iter().rev() {
            let contact = info.pointerInfo.pointerFlags.0 & POINTER_FLAG_INCONTACT.0 != 0;
            let at = match info.pointerInfo.PerformanceCount {
                0 => now(),
                ticks => ticks as i64,
            };
            let (x, y) = client(info.pointerInfo.ptPixelLocation);
            if contact {
                window.pen += 1;
                widen(&mut window.pressure, info.pressure);
                widen(&mut window.tilt_x, info.tiltX);
                widen(&mut window.tilt_y, info.tiltY);
                if let Some(last) = window.pen_last {
                    window.pen_down_ticks += (at - last).max(0);
                }
                window.pen_last = Some(at);
            }
            let pressure = info.pressure as f32 / 1024.0;
            window.take((x, y), pressure, (info.tiltX as f32, info.tiltY as f32), at, Source::Pen, contact);
        }
        return true;
    }
    if kind == PT_TOUCH {
        let mut info = POINTER_INFO::default();
        // SAFETY: `info` is a live local the call writes to.
        if unsafe { GetPointerInfo(id, &mut info) }.is_err() {
            return false;
        }
        let contact = info.pointerFlags.0 & POINTER_FLAG_INCONTACT.0 != 0;
        let (x, y) = client(info.ptPixelLocation);
        window.touch += u64::from(contact);
        window.take((x, y), 0.5, (0.0, 0.0), info.PerformanceCount as i64, Source::Touch, contact);
        return true;
    }
    false
}

/// The window's messages. They only gather; the frame loop draws.
unsafe extern "system" fn messages(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if message == WM_NCCREATE {
        // SAFETY: for WM_NCCREATE, `lparam` points to the CREATESTRUCTW of
        // this window, whose lpCreateParams is the pointer `run` passed.
        let create = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
        // SAFETY: `hwnd` is the window being created, on its own thread.
        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize) };
    }
    // SAFETY: `hwnd` is a live window on this thread.
    let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const RefCell<Window>;
    // SAFETY: the pointer is null before WM_NCCREATE, and afterwards points to
    // the `RefCell` that `run` keeps alive until after the window is destroyed.
    let Some(state) = (unsafe { state.as_ref() }) else {
        // SAFETY: forwarding the message's own arguments unchanged.
        return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
    };
    let Ok(mut window) = state.try_borrow_mut() else {
        // A message sent while another is being handled: Windows' default will do.
        // SAFETY: forwarding the message's own arguments unchanged.
        return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
    };
    match message {
        WM_POINTERDOWN | WM_POINTERUPDATE | WM_POINTERUP => {
            if pointer(hwnd, &mut window, wparam) {
                // Handled: Windows makes no mouse messages from this pen.
                return LRESULT(0);
            }
        }
        WM_LBUTTONDOWN | WM_MOUSEMOVE | WM_LBUTTONUP => {
            let down = message != WM_LBUTTONUP && wparam.0 & 0x0001 != 0;
            if message == WM_LBUTTONDOWN {
                // SAFETY: `hwnd` is this thread's window.
                unsafe { SetCapture(hwnd) };
            } else if message == WM_LBUTTONUP {
                // SAFETY: releasing capture is sound whether or not it is held.
                let _ = unsafe { ReleaseCapture() };
            }
            let (x, y) = point(lparam);
            window.mouse += u64::from(down);
            window.take((x, y), 0.5, (0.0, 0.0), now(), Source::Mouse, down);
            return LRESULT(0);
        }
        WM_BRUSH_SAMPLE => {
            let posted = window.shared.as_ref().and_then(|shared| lock(&shared.injected).pop_front());
            let down = wparam.0 & 1 != 0;
            let pressure = (wparam.0 >> 1) as f32 / 65535.0;
            let (x, y) = point(lparam);
            window.injected += u64::from(down);
            window.take((x, y), pressure, (0.0, 0.0), posted.unwrap_or_else(now), Source::Mouse, down);
            return LRESULT(0);
        }
        WM_BRUSH_INPUT => {
            // SAFETY: `hwnd` is this thread's window; the style bits are valid.
            unsafe {
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let through = (WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0) as isize;
                let wanted = if wparam.0 == 1 { style & !through } else { style | through };
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, wanted);
                let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED;
                let _ = SetWindowPos(hwnd, None, 0, 0, 0, 0, flags);
            }
            return LRESULT(0);
        }
        WM_BRUSH_SHOW => {
            window.hidden = wparam.0 == 0;
            // SAFETY: `hwnd` is this thread's window.
            let _ = unsafe { ShowWindow(hwnd, if window.hidden { SW_HIDE } else { SW_SHOWNOACTIVATE }) };
            return LRESULT(0);
        }
        WM_BRUSH_RECT => {
            let rect = window.shared.as_ref().map(|shared| *lock(&shared.rect));
            // The state's borrow ends first: SetWindowPos sends WM_SIZE back here.
            drop(window);
            if let Some((x, y, w, h)) = rect {
                // SAFETY: `hwnd` is this thread's window.
                let _ = unsafe { SetWindowPos(hwnd, None, x, y, w, h, SWP_NOZORDER | SWP_NOACTIVATE) };
            }
            return LRESULT(0);
        }
        WM_SIZE => {
            let (w, h) = ((lparam.0 & 0xffff) as u32, ((lparam.0 >> 16) & 0xffff) as u32);
            window.resized = Some((w.max(1), h.max(1)));
            return LRESULT(0);
        }
        // Drawing must never take the keyboard from the game or the video.
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
        WM_BRUSH_CLOSE | WM_RBUTTONUP | WM_DESTROY => {
            window.close = true;
            return LRESULT(0);
        }
        _ => {}
    }
    drop(window);
    // SAFETY: forwarding the message's own arguments unchanged.
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

/// The compositor's refresh rate, or 0.0 if Windows won't say.
fn refresh_hz() -> f64 {
    let mut info = DWM_TIMING_INFO { cbSize: std::mem::size_of::<DWM_TIMING_INFO>() as u32, ..Default::default() };
    // SAFETY: `info` is a live local with its size set, as the call requires.
    if unsafe { DwmGetCompositionTimingInfo(HWND::default(), &mut info) }.is_err() {
        return 0.0;
    }
    match info.rateRefresh.uiDenominator {
        0 => 0.0,
        d => f64::from(info.rateRefresh.uiNumerator) / f64::from(d),
    }
}

/// The DXGI swap chain under wgpu's surface, for its frame statistics.
fn swap_chain(surface: &wgpu::Surface<'_>) -> Option<IDXGISwapChain3> {
    // SAFETY: the swap chain is only asked for statistics; nothing wgpu owns
    // is presented, resized or released through it.
    let hal = unsafe { surface.as_hal::<wgpu::hal::api::Dx12>() }?;
    hal.swap_chain()
}

/// Pump every message waiting for this thread.
fn pump() {
    let mut message = MSG::default();
    // SAFETY: `message` is a live local; the calls are the standard pump.
    unsafe {
        while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

/// The canvas's thread: the window, then frames until it closes.
fn run(options: &Options, shared: &Arc<Shared>, ready: &mpsc::Sender<Result<(), String>>) -> Result<(), String> {
    let per_second = frequency();
    let state = Box::new(RefCell::new(Window {
        shared: Some(Arc::clone(shared)),
        ticks_per_second: per_second,
        ..Window::default()
    }));

    // SAFETY: plain Win32 window creation on this thread. The class's
    // procedure is `messages`; the create parameter is `state`, which lives
    // until this function returns, after the window is destroyed.
    let hwnd = unsafe {
        // Pixels are real pixels on every monitor.
        let _ = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let module = GetModuleHandleW(None).map_err(|e| format!("no module handle: {e}"))?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(messages),
            hInstance: module.into(),
            lpszClassName: w!("PolygloticBrushCanvas"),
            ..Default::default()
        };
        // Registering twice fails harmlessly: the class is already there.
        RegisterClassW(&class);
        // No redirection bitmap: the window is drawn only by its composition
        // visual, which is what makes it transparent.
        let mut style = WS_EX_NOREDIRECTIONBITMAP | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;
        if options.on_top {
            style |= WS_EX_TOPMOST;
        }
        if options.input == Input::Off {
            style |= WS_EX_TRANSPARENT | WS_EX_LAYERED;
        }
        let owner = (options.owner != 0).then_some(HWND(options.owner as *mut _));
        let (x, y, w, h) = options.rect;
        CreateWindowExW(
            style,
            class.lpszClassName,
            w!("Polyglotic canvas"),
            WS_POPUP,
            x,
            y,
            w,
            h,
            owner,
            None,
            Some(module.into()),
            Some(std::ptr::from_ref(&*state).cast()),
        )
        .map_err(|e| format!("the canvas's window wasn't made: {e}"))?
    };
    shared.hwnd.store(hwnd.0 as isize, Ordering::SeqCst);

    let result = frames(options, shared, ready, hwnd, &state, per_second);

    // SAFETY: `hwnd` was made on this thread; destroying a window that has
    // already gone fails harmlessly.
    unsafe {
        if IsWindow(Some(hwnd)).as_bool() {
            let _ = DestroyWindow(hwnd);
        }
    }
    pump();
    result
}

/// Everything that needs the GPU: set up, then the frame loop.
fn frames(
    options: &Options,
    shared: &Arc<Shared>,
    ready: &mpsc::Sender<Result<(), String>>,
    hwnd: HWND,
    state: &RefCell<Window>,
    per_second: i64,
) -> Result<(), String> {
    let instance = brush_engine::gpu::instance(wgpu::Backends::DX12);
    let handle = NonZeroIsize::new(hwnd.0 as isize).ok_or("the canvas's window has no handle")?;
    let target = wgpu::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: Some(wgpu::rwh::RawDisplayHandle::Windows(wgpu::rwh::WindowsDisplayHandle::new())),
        raw_window_handle: wgpu::rwh::RawWindowHandle::Win32(wgpu::rwh::Win32WindowHandle::new(handle)),
    };
    // SAFETY: `hwnd` is a live window, and the surface is dropped (at the end
    // of this function) before `run` destroys it.
    let surface = unsafe { instance.create_surface_unsafe(target) }.map_err(|e| format!("no surface: {e}"))?;

    let named = options.adapter.as_ref().and_then(|want| {
        let want = want.to_lowercase();
        brush_engine::gpu::adapters(&instance)
            .into_iter()
            .find(|a| a.get_info().name.to_lowercase().contains(&want) && a.is_surface_supported(&surface))
    });
    let adapter = match named {
        Some(adapter) => adapter,
        None => pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .map_err(|e| format!("no adapter can draw to the canvas: {e}"))?,
    };
    let (device, queue) = brush_engine::gpu::open(&adapter).map_err(|e| format!("the adapter gave no device: {e}"))?;

    let caps = surface.get_capabilities(&adapter);
    if !caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
        return Err(format!("the canvas can't be transparent here: alpha modes {:?}", caps.alpha_modes));
    }
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|f| *f == wgpu::TextureFormat::Bgra8Unorm)
        .or_else(|| caps.formats.first().copied())
        .ok_or("the surface offers no format")?;
    let (_, _, w, h) = options.rect;
    let mut config = surface
        .get_default_config(&adapter, w.max(1) as u32, h.max(1) as u32)
        .ok_or("the surface has no default configuration")?;
    config.format = format;
    config.alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
    config.present_mode = wgpu::PresentMode::Fifo;
    config.desired_maximum_frame_latency = 1;
    surface.configure(&device, &config);
    let mut chain = swap_chain(&surface);

    let mut engine = Engine::new(&device, &queue, format, options.sim.map(|(grid, _)| grid));
    engine.set_backdrop(options.tint);
    let sim_steps = options.sim.map_or(0, |(_, steps)| steps);

    {
        let mut stats = lock(&shared.stats);
        stats.adapter = brush_engine::gpu::describe(&adapter);
        stats.format = format!("{format:?}");
        stats.alpha = format!("{:?}", config.alpha_mode);
        stats.size = (config.width, config.height);
        stats.refresh_hz = refresh_hz();
        if chain.is_none() {
            stats.notes.push("no DXGI swap chain to read frame statistics from".into());
        }
    }
    // SAFETY: `hwnd` is this thread's window.
    let _ = unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
    if options.on_top {
        // SAFETY: as above; only the z-order changes.
        let _ = unsafe { SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) };
    }
    let _ = ready.send(Ok(()));

    let micros = |ticks: i64| (ticks.max(0) as u128 * 1_000_000 / per_second as u128) as u64;
    let mut last_present: Option<i64> = None;
    // Presents whose sync time DXGI hasn't reported yet: its count, its samples' times.
    let mut awaiting: VecDeque<(u32, Vec<i64>)> = VecDeque::new();
    let mut first_counts: Option<(u32, u32)> = None;

    loop {
        if state.borrow().hidden {
            // Nothing to draw: wait for messages without touching the GPU.
            pump();
            if state.borrow().close {
                break;
            }
            last_present = None;
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }
        let asked = now();
        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => Some(frame),
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                drop(chain.take());
                surface.configure(&device, &config);
                chain = swap_chain(&surface);
                None
            }
            _ => None,
        };
        let got = now();
        // Input is read after the wait, so the frame draws the newest samples.
        pump();
        let (pending, resized, close) = {
            let mut window = state.borrow_mut();
            (std::mem::take(&mut window.pending), window.resized.take(), window.close)
        };
        if close {
            break;
        }
        let Some(frame) = frame else {
            lock(&shared.stats).refused += 1;
            std::thread::sleep(std::time::Duration::from_millis(2));
            continue;
        };
        let times: Vec<i64> = pending.iter().map(|p| p.at).collect();
        for p in pending {
            if p.begin {
                // M0 keeps every stroke and lays the ink out again each frame, so
                // a long test starts afresh rather than grow without end.
                if engine.samples() > KEPT_SAMPLES {
                    engine.clear();
                }
                engine.stroke_begin();
            }
            engine.stroke_add(p.sample);
        }
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let commands = engine.frame(&device, &queue, &view, (config.width, config.height), sim_steps);
        queue.submit([commands]);
        // The view holds the swap chain's buffer, which a resize must find free.
        drop(view);
        queue.present(frame);
        let presented = now();

        let mut synced: Vec<u64> = Vec::new();
        let mut counts: Option<(u32, u32)> = None;
        if let Some(chain) = chain.as_ref() {
            // SAFETY: plain reads of the swap chain's counters.
            let (mine, statistics) = unsafe {
                let mut statistics = DXGI_FRAME_STATISTICS::default();
                let ok = chain.GetFrameStatistics(&mut statistics).is_ok();
                (chain.GetLastPresentCount().ok(), ok.then_some(statistics))
            };
            if let Some(mine) = mine {
                if !times.is_empty() {
                    awaiting.push_back((mine, times.clone()));
                }
            }
            if let Some(s) = statistics {
                counts = Some((s.PresentCount, s.PresentRefreshCount));
                while awaiting.front().is_some_and(|(count, _)| *count <= s.PresentCount) {
                    if let Some((count, sample_times)) = awaiting.pop_front() {
                        if count == s.PresentCount {
                            synced.extend(sample_times.iter().map(|t| micros(s.SyncQPCTime - t)));
                        }
                    }
                }
            }
            while awaiting.len() > 64 {
                awaiting.pop_front();
            }
        }

        let mut stats = lock(&shared.stats);
        stats.frames += 1;
        stats.acquire_wait.push_micros(micros(got - asked));
        stats.frame_work.push_micros(micros(presented - got));
        if let Some(last) = last_present {
            stats.frame_interval.push_micros(micros(presented - last));
        }
        last_present = Some(presented);
        for t in &times {
            stats.input_to_present.push_micros(micros(presented - t));
        }
        for s in synced {
            stats.input_to_sync.push_micros(s);
        }
        if let Some((presents, refreshes)) = counts {
            // DXGI's counters read zero until the first frame has been shown.
            if refreshes > 0 {
                let (p0, r0) = *first_counts.get_or_insert((presents, refreshes));
                stats.dxgi_presents = Some(presents.wrapping_sub(p0));
                stats.dxgi_refreshes = Some(refreshes.wrapping_sub(r0));
            }
        }
        {
            let window = state.borrow();
            stats.pen_samples = window.pen;
            stats.mouse_samples = window.mouse;
            stats.touch_samples = window.touch;
            stats.injected_samples = window.injected;
            stats.pressure = window.pressure;
            stats.tilt_x = window.tilt_x;
            stats.tilt_y = window.tilt_y;
            if window.pen_down_ticks > 0 {
                stats.pen_rate_hz = window.pen as f64 * per_second as f64 / window.pen_down_ticks as f64;
            }
        }
        drop(stats);

        if let Some((w, h)) = resized {
            if (w, h) != (config.width, config.height) {
                config.width = w;
                config.height = h;
                drop(chain.take());
                surface.configure(&device, &config);
                chain = swap_chain(&surface);
                engine.resized();
                lock(&shared.stats).size = (w, h);
            }
        }
    }
    Ok(())
}
