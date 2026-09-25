use crate::text::browser_page;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{CloseHandle, HWND, MAX_PATH};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, SetForegroundWindow,
};

/// タイトルからページを取り出す相手。それ以外はプロセス名だけで区別する。
const BROWSERS: [&str; 8] = [
    "chrome", "msedge", "firefox", "brave", "vivaldi", "opera", "chromium", "zen",
];

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
            Some(Foreground {
                hwnd: hwnd as isize,
            })
        }
    }
}

pub fn restore_foreground(fg: &Foreground) -> bool {
    unsafe { SetForegroundWindow(fg.hwnd as HWND) != 0 }
}

/// 貼り付け先を表す目印。ブラウザは開いているページまで見る。
pub fn context_key(fg: &Foreground) -> Option<String> {
    let hwnd = fg.hwnd as HWND;
    let process = process_name(hwnd)?;
    if BROWSERS.contains(&process.as_str()) {
        if let Some(page) = window_title(hwnd).as_deref().and_then(browser_page) {
            return Some(format!("{process}|{page}"));
        }
    }
    Some(process)
}

/// UI Automation のコントロール種別名。取れなければ空。
pub fn focused_control() -> String {
    let Ok(automation) = uiautomation::UIAutomation::new() else {
        return String::new();
    };
    let Ok(element) = automation.get_focused_element() else {
        return String::new();
    };
    let Ok(kind) = element.get_control_type() else {
        return String::new();
    };
    format!("{kind}")
}

pub fn app_and_title(fg: &Foreground) -> (String, String) {
    let hwnd = fg.hwnd as HWND;
    (
        process_name(hwnd).unwrap_or_default(),
        window_title(hwnd).unwrap_or_default(),
    )
}

fn process_name(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return None;
        }
        let mut buffer = [0u16; MAX_PATH as usize];
        let mut length = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length);
        CloseHandle(process);
        if ok == 0 || length == 0 {
            return None;
        }
        let path = OsString::from_wide(&buffer[..length as usize]);
        Path::new(&path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_lowercase())
    }
}

fn window_title(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut buffer = [0u16; 512];
        let length = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        if length <= 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buffer[..length as usize]))
    }
}

/// 物理的に押したままの修飾キー。コピーの SendInput で離すと、次の Ctrl+7 が 7 だけになる。
pub(crate) fn modifiers_held() -> (bool, bool) {
    unsafe {
        let ctrl = GetAsyncKeyState(VK_CONTROL as i32) as u16 & 0x8000 != 0;
        let shift = GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000 != 0;
        (ctrl, shift)
    }
}
