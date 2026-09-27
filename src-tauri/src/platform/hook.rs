use windows_sys::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT,
    LLKHF_INJECTED, LLKHF_UP, MSG, WH_KEYBOARD_LL,
};

static HOOK: std::sync::atomic::AtomicPtr<std::ffi::c_void> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

pub fn install() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        crate::ctrl_gap::ensure_worker();
        std::thread::spawn(|| unsafe {
            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(proc),
                GetModuleHandleW(std::ptr::null()) as HINSTANCE,
                0,
            );
            if hook.is_null() {
                return;
            }
            HOOK.store(hook, std::sync::atomic::Ordering::Release);
            let mut msg = std::mem::zeroed::<MSG>();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {}
        });
    });
}

unsafe extern "system" fn proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let info = &*(lparam as *const KBDLLHOOKSTRUCT);
        let injected = (info.flags & (LLKHF_INJECTED | 0x02)) != 0;
        let accept = crate::ctrl_gap::should_treat_as_physical(
            injected,
            crate::test_args::is_active(),
            crate::ctrl_gap::is_sending(),
        );
        if accept {
            let was_down = crate::ctrl_gap::physical();
            let down = (info.flags & LLKHF_UP) == 0;
            let swallow = crate::ctrl_gap::on_hook_key(info.vkCode as u16, down);
            if was_down && !crate::ctrl_gap::physical() {
                // フックの中で送ると固まるので、別スレッドで上げる。
                std::thread::spawn(super::input::release_control);
            }
            if swallow {
                return 1;
            }
        }
    }
    CallNextHookEx(
        HOOK.load(std::sync::atomic::Ordering::Acquire) as HHOOK,
        code,
        wparam,
        lparam,
    )
}
