//! Keeps the notch in Windows' topmost z-order band for the life of the session.
//!
//! `tao` (the windowing crate under Tauri 2) only calls `SetWindowPos` when its own
//! `ALWAYS_ON_TOP` flag changes value. That flag is `true` from window creation (`tauri.conf.json`
//! sets `alwaysOnTop`), so a later `window.set_always_on_top(true)` is a no-op — it never
//! reasserts anything with the OS. The `WS_EX_TOPMOST` extended-style bit and the window's actual
//! position in the z-order can disagree: the bit can survive while the window has sunk behind
//! ordinary windows anyway. See https://github.com/vinzdg/codenotch/issues/304 for the measurement
//! that pinned this down (60 of 60 samples with the notch behind an ordinary window, style bit
//! still set) and the two-contributor diagnosis this module implements.

use tauri::{AppHandle, Manager, WebviewWindow};

/// True once the notch has left the topmost band, by either failure mode the issue describes:
/// its own `WS_EX_TOPMOST` bit cleared outright, or the bit surviving while an ordinary window
/// still sits in front of it in the real z-order. A single top-to-bottom walk answers both —
/// if the notch's own bit is already clear there is nothing to enumerate for.
#[cfg(windows)]
pub fn is_out_of_topmost_band(window: &WebviewWindow) -> bool {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowLongPtrW, IsWindowVisible, GWL_EXSTYLE, WS_EX_TOPMOST,
    };

    // `window.hwnd()` comes back typed against whatever `windows` version Tauri itself pulled in,
    // which can differ from the one this crate depends on directly (see `notchmenu.rs`'s
    // `give_back` for the same workaround) — so the raw pointer is carried across as `isize` and
    // rebuilt into this crate's own `HWND` before it touches any Win32 call below.
    let Ok(raw) = window.hwnd() else { return false };
    let hwnd = HWND(raw.0 as _);

    let ex_style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    if (ex_style as u32 & WS_EX_TOPMOST.0) == 0 {
        return true;
    }

    struct State {
        target: isize,
        found: bool,
        band_broken: bool,
    }
    unsafe extern "system" fn cb(hwnd: HWND, l: LPARAM) -> BOOL {
        let state = &mut *(l.0 as *mut State);
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }
        if hwnd.0 as isize == state.target {
            state.found = true;
            return BOOL(0);
        }
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        if (ex as u32 & WS_EX_TOPMOST.0) == 0 {
            state.band_broken = true;
            return BOOL(0);
        }
        BOOL(1)
    }
    let mut state = State { target: hwnd.0 as isize, found: false, band_broken: false };
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(&mut state as *mut _ as isize));
    }
    !state.found || state.band_broken
}

#[cfg(not(windows))]
pub fn is_out_of_topmost_band(_window: &WebviewWindow) -> bool {
    false
}

/// Reasserts the notch at the top of the z-order with a direct Win32 call, bypassing `tao`'s
/// diffed `set_always_on_top` entirely. Also what `dropzones.rs` uses to lift the notch back over
/// the drop-zone overlay during a carry — `notch.set_always_on_top(true)` there was the same no-op.
#[cfg(windows)]
pub fn reassert(window: &WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };
    let Ok(raw) = window.hwnd() else { return };
    let hwnd = HWND(raw.0 as _);
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
}

#[cfg(not(windows))]
pub fn reassert(_window: &WebviewWindow) {}

/// How often the notch is checked against the topmost band. Validated on #304 at this cadence —
/// a session logged three assertions, not one per tick, so 2s does not chase transient state.
const TOPMOST_POLL_MS: u64 = 2000;

/// Polled rather than hooked, same reasoning as `start_work_area_watch`: nothing hands us an event
/// for "another process just changed our z-order," so there is nothing to subscribe to.
#[cfg(windows)]
pub fn start_watchdog(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(TOPMOST_POLL_MS));
        // Mid-drag the carry has its own topmost handling via dropzones::show; do not fight it.
        if crate::DRAGGING.load(std::sync::atomic::Ordering::SeqCst) {
            continue;
        }
        let Some(w) = app.get_webview_window("notch") else { continue };
        if is_out_of_topmost_band(&w) {
            reassert(&w);
            crate::applog("topmost watchdog: notch had left the topmost band, reasserted");
        }
    });
}

#[cfg(not(windows))]
pub fn start_watchdog(_app: AppHandle) {}
