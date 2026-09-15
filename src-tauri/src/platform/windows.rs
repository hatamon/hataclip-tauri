use super::Point;
use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetGUIThreadInfo, GetWindowRect, GetWindowThreadProcessId,
    SetForegroundWindow, GUITHREADINFO,
};

#[derive(Clone, Copy)]
pub struct Foreground {
    hwnd: isize,
}

pub fn anchor_position() -> Option<Point> {
    caret_position().or_else(cursor_position).or_else(foreground_window_position)
}

pub fn capture_foreground() -> Option<Foreground> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            None
        } else {
            Some(Foreground { hwnd: hwnd as isize })
        }
    }
}

pub fn restore_foreground(fg: &Foreground) -> bool {
    unsafe { SetForegroundWindow(fg.hwnd as HWND) != 0 }
}

fn caret_position() -> Option<Point> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let thread_id = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        let mut info: GUITHREADINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;
        if GetGUIThreadInfo(thread_id, &mut info) == 0 || info.hwndCaret.is_null() {
            return None;
        }
        let mut point = POINT {
            x: info.rcCaret.left,
            y: info.rcCaret.bottom,
        };
        if ClientToScreen(info.hwndCaret, &mut point) == 0 {
            return None;
        }
        Some(Point {
            x: point.x,
            y: point.y,
        })
    }
}

fn cursor_position() -> Option<Point> {
    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut point) == 0 {
            None
        } else {
            Some(Point {
                x: point.x,
                y: point.y,
            })
        }
    }
}

fn foreground_window_position() -> Option<Point> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetWindowRect(hwnd, &mut rect) == 0 {
            return None;
        }
        Some(Point {
            x: rect.left + 8,
            y: rect.bottom.saturating_sub(40),
        })
    }
}
