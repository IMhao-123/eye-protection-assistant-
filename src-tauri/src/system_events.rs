pub use eye_care_core::system_events::{timer_action, SystemEvent};

#[cfg(target_os = "windows")]
mod windows {
    use std::{
        ptr::{null, null_mut},
        sync::{
            atomic::{AtomicIsize, Ordering},
            Mutex, OnceLock,
        },
        time::Instant,
    };

    use tauri::AppHandle;
    use windows_sys::Win32::{
        Foundation::{FreeLibrary, HMODULE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            PostMessageW, PostQuitMessage, RegisterClassW, TranslateMessage, WM_CLOSE, WM_DESTROY,
            WNDCLASSW,
        },
    };

    use eye_care_core::system_events::{map_windows_message, EventDeduplicator, SystemEvent};

    static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
    static STARTED_AT: OnceLock<Instant> = OnceLock::new();
    static DEDUPLICATOR: OnceLock<Mutex<EventDeduplicator>> = OnceLock::new();
    static EVENT_WINDOW: AtomicIsize = AtomicIsize::new(0);

    type WtsRegisterSessionNotification = unsafe extern "system" fn(HWND, u32) -> i32;
    type WtsUnregisterSessionNotification = unsafe extern "system" fn(HWND) -> i32;

    struct WtsNotifications {
        module: HMODULE,
        unregister: WtsUnregisterSessionNotification,
    }

    unsafe fn register_wts_notifications(hwnd: HWND) -> Option<WtsNotifications> {
        let library_name = "wtsapi32.dll\0".encode_utf16().collect::<Vec<_>>();
        let module = LoadLibraryW(library_name.as_ptr());
        if module.is_null() {
            return None;
        }
        let register: Option<WtsRegisterSessionNotification> = std::mem::transmute(GetProcAddress(
            module,
            c"WTSRegisterSessionNotification".as_ptr().cast(),
        ));
        let unregister: Option<WtsUnregisterSessionNotification> = std::mem::transmute(
            GetProcAddress(module, c"WTSUnRegisterSessionNotification".as_ptr().cast()),
        );
        let (Some(register), Some(unregister)) = (register, unregister) else {
            let _ = FreeLibrary(module);
            return None;
        };
        if register(hwnd, 0) == 0 {
            let _ = FreeLibrary(module);
            return None;
        }
        Some(WtsNotifications { module, unregister })
    }

    fn dispatch(event: SystemEvent) {
        let now_ms = STARTED_AT
            .get_or_init(Instant::now)
            .elapsed()
            .as_millis()
            .min(u64::MAX as u128) as u64;
        if DEDUPLICATOR
            .get_or_init(|| Mutex::new(EventDeduplicator::default()))
            .lock()
            .is_ok_and(|mut deduplicator| !deduplicator.accept(event, now_ms))
        {
            return;
        }
        let Some(app) = APP_HANDLE.get().cloned() else {
            return;
        };
        let scheduling_handle = app.clone();
        if let Err(error) = scheduling_handle.run_on_main_thread(move || {
            super::super::handle_system_event(&app, event);
        }) {
            eprintln!("failed to schedule Windows system event: {error}");
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if let Some(event) = map_windows_message(message, wparam) {
            dispatch(event);
        }
        if message == WM_DESTROY {
            PostQuitMessage(0);
            return 0;
        }
        DefWindowProcW(hwnd, message, wparam, lparam)
    }

    unsafe fn run_message_loop() -> Result<(), String> {
        let class_name = "EyeCareAssistantSystemEvents\0"
            .encode_utf16()
            .collect::<Vec<_>>();
        let instance = GetModuleHandleW(null());
        if instance.is_null() {
            return Err("无法取得 Windows 应用实例".to_string());
        }
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: class_name.as_ptr(),
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err("无法注册 Windows 系统事件窗口".to_string());
        }
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if hwnd.is_null() {
            return Err("无法创建 Windows 系统事件窗口".to_string());
        }
        EVENT_WINDOW.store(hwnd as isize, Ordering::Release);
        let wts_notifications = register_wts_notifications(hwnd);
        if wts_notifications.is_none() {
            eprintln!("Windows WTS 会话通知不可用，锁屏事件将等待用户验收");
        }

        let mut message = std::mem::zeroed();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        EVENT_WINDOW.store(0, Ordering::Release);

        if let Some(notifications) = wts_notifications {
            let _ = (notifications.unregister)(hwnd);
            let _ = FreeLibrary(notifications.module);
        }
        let _ = DestroyWindow(hwnd);
        Ok(())
    }

    pub fn register(app: &AppHandle) {
        if APP_HANDLE.set(app.clone()).is_err() {
            return;
        }
        if let Err(error) = std::thread::Builder::new()
            .name("windows-system-events".to_string())
            .spawn(|| unsafe {
                if let Err(error) = run_message_loop() {
                    eprintln!("failed to register Windows system events: {error}");
                }
            })
        {
            eprintln!("failed to start Windows system event listener: {error}");
        }
    }

    pub fn shutdown() {
        let hwnd = EVENT_WINDOW.swap(0, Ordering::AcqRel);
        if hwnd != 0 {
            unsafe {
                let _ = PostMessageW(hwnd as HWND, WM_CLOSE, 0, 0);
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows::{register, shutdown};

#[cfg(not(target_os = "windows"))]
pub fn register(_app: &tauri::AppHandle) {}

#[cfg(not(target_os = "windows"))]
pub fn shutdown() {}
