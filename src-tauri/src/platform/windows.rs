use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow};

#[derive(Clone, Copy)]
pub struct Foreground {
    hwnd: isize,
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
