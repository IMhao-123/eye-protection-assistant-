use crate::domain::TimerAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemEvent {
    Suspend,
    Resume,
    ScreensChanged,
}

pub fn timer_action(event: SystemEvent) -> Option<TimerAction> {
    match event {
        SystemEvent::Suspend => Some(TimerAction::Sleep),
        SystemEvent::Resume => Some(TimerAction::Wake),
        SystemEvent::ScreensChanged => None,
    }
}

const WM_DISPLAYCHANGE_VALUE: u32 = 0x007E;
const WM_DEVICECHANGE_VALUE: u32 = 0x0219;
const WM_POWERBROADCAST_VALUE: u32 = 0x0218;
const WM_WTSSESSION_CHANGE_VALUE: u32 = 0x02B1;
const PBT_APMSUSPEND_VALUE: usize = 0x4;
const PBT_APMRESUMEAUTOMATIC_VALUE: usize = 0x12;
const WTS_SESSION_LOCK_VALUE: usize = 0x7;
const WTS_SESSION_UNLOCK_VALUE: usize = 0x8;
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const DUPLICATE_EVENT_WINDOW_MS: u64 = 500;

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn map_windows_message(message: u32, event_code: usize) -> Option<SystemEvent> {
    match (message, event_code) {
        (WM_WTSSESSION_CHANGE_VALUE, WTS_SESSION_LOCK_VALUE)
        | (WM_POWERBROADCAST_VALUE, PBT_APMSUSPEND_VALUE) => Some(SystemEvent::Suspend),
        (WM_WTSSESSION_CHANGE_VALUE, WTS_SESSION_UNLOCK_VALUE)
        | (WM_POWERBROADCAST_VALUE, PBT_APMRESUMEAUTOMATIC_VALUE) => Some(SystemEvent::Resume),
        (WM_DISPLAYCHANGE_VALUE, _) => Some(SystemEvent::ScreensChanged),
        (WM_DEVICECHANGE_VALUE, _) => Some(SystemEvent::ScreensChanged),
        _ => None,
    }
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[derive(Default)]
struct EventDeduplicator {
    last: Option<(SystemEvent, u64)>,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
impl EventDeduplicator {
    fn accept(&mut self, event: SystemEvent, now_ms: u64) -> bool {
        let duplicate = self.last.is_some_and(|(previous, previous_ms)| {
            previous == event && now_ms.saturating_sub(previous_ms) <= DUPLICATE_EVENT_WINDOW_MS
        });
        if !duplicate {
            self.last = Some((event, now_ms));
        }
        !duplicate
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::{
        ptr::{null, null_mut},
        sync::{LazyLock, Mutex, OnceLock},
        time::Instant,
    };

    use tauri::AppHandle;
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{
            LibraryLoader::GetModuleHandleW,
            RemoteDesktop::{
                WTSRegisterSessionNotification, WTSUnRegisterSessionNotification,
                NOTIFY_FOR_THIS_SESSION,
            },
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            PostQuitMessage, RegisterClassW, TranslateMessage, WM_DESTROY, WNDCLASSW,
        },
    };

    use super::{map_windows_message, EventDeduplicator, SystemEvent};

    static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
    static STARTED_AT: OnceLock<Instant> = OnceLock::new();
    static DEDUPLICATOR: LazyLock<Mutex<EventDeduplicator>> =
        LazyLock::new(|| Mutex::new(EventDeduplicator::default()));

    fn dispatch(event: SystemEvent) {
        let now_ms = STARTED_AT
            .get_or_init(Instant::now)
            .elapsed()
            .as_millis()
            .min(u64::MAX as u128) as u64;
        if DEDUPLICATOR
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
        if WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) == 0 {
            let _ = DestroyWindow(hwnd);
            return Err("无法监听 Windows 锁屏事件".to_string());
        }

        let mut message = std::mem::zeroed();
        while GetMessageW(&mut message, null_mut(), 0, 0) > 0 {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        let _ = WTSUnRegisterSessionNotification(hwnd);
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
}

#[cfg(target_os = "windows")]
pub use windows::register;

#[cfg(not(target_os = "windows"))]
pub fn register(_app: &tauri::AppHandle) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_real_session_events_drive_the_timer_engine() {
        assert_eq!(timer_action(SystemEvent::Suspend), Some(TimerAction::Sleep));
        assert_eq!(timer_action(SystemEvent::Resume), Some(TimerAction::Wake));
        assert_eq!(timer_action(SystemEvent::ScreensChanged), None);
    }

    #[test]
    fn windows_messages_map_to_existing_domain_events() {
        assert_eq!(map_windows_message(0x02B1, 0x7), Some(SystemEvent::Suspend));
        assert_eq!(map_windows_message(0x02B1, 0x8), Some(SystemEvent::Resume));
        assert_eq!(map_windows_message(0x0218, 0x4), Some(SystemEvent::Suspend));
        assert_eq!(map_windows_message(0x0218, 0x12), Some(SystemEvent::Resume));
        assert_eq!(
            map_windows_message(0x007E, 0),
            Some(SystemEvent::ScreensChanged)
        );
        assert_eq!(
            map_windows_message(0x0219, 0x0007),
            Some(SystemEvent::ScreensChanged)
        );
        assert_eq!(map_windows_message(0x000F, 0), None);
    }

    #[test]
    fn duplicate_native_events_are_suppressed_only_within_the_burst_window() {
        let mut deduplicator = EventDeduplicator::default();
        assert!(deduplicator.accept(SystemEvent::Suspend, 1000));
        assert!(!deduplicator.accept(SystemEvent::Suspend, 1100));
        assert!(deduplicator.accept(SystemEvent::Resume, 1200));
        assert!(deduplicator.accept(SystemEvent::Resume, 1801));
    }
}
