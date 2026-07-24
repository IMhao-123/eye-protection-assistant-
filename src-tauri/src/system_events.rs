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

#[cfg(target_os = "macos")]
pub fn register(app: &tauri::AppHandle) {
    use std::ptr::NonNull;

    use block2::RcBlock;
    use objc2_app_kit::{
        NSApplicationDidChangeScreenParametersNotification, NSWorkspace,
        NSWorkspaceDidWakeNotification, NSWorkspaceScreensDidSleepNotification,
        NSWorkspaceScreensDidWakeNotification, NSWorkspaceSessionDidBecomeActiveNotification,
        NSWorkspaceSessionDidResignActiveNotification, NSWorkspaceWillSleepNotification,
    };
    use objc2_foundation::{NSNotification, NSNotificationCenter, NSNotificationName};

    fn observe(
        center: &NSNotificationCenter,
        name: &NSNotificationName,
        app: &tauri::AppHandle,
        event: SystemEvent,
    ) {
        let app_handle = app.clone();
        let callback = RcBlock::new(move |_notification: NonNull<NSNotification>| {
            let event_handle = app_handle.clone();
            let scheduling_handle = app_handle.clone();
            if let Err(error) = scheduling_handle.run_on_main_thread(move || {
                super::handle_system_event(&event_handle, event);
            }) {
                eprintln!("failed to schedule macOS system event: {error}");
            }
        });
        unsafe {
            let observer = center.addObserverForName_object_queue_usingBlock(
                Some(name),
                None,
                None,
                &callback,
            );
            std::mem::forget(observer);
        }
    }

    let workspace = NSWorkspace::sharedWorkspace();
    let workspace_center = workspace.notificationCenter();
    observe(
        &workspace_center,
        unsafe { NSWorkspaceWillSleepNotification },
        app,
        SystemEvent::Suspend,
    );
    observe(
        &workspace_center,
        unsafe { NSWorkspaceSessionDidResignActiveNotification },
        app,
        SystemEvent::Suspend,
    );
    observe(
        &workspace_center,
        unsafe { NSWorkspaceScreensDidSleepNotification },
        app,
        SystemEvent::Suspend,
    );
    observe(
        &workspace_center,
        unsafe { NSWorkspaceDidWakeNotification },
        app,
        SystemEvent::Resume,
    );
    observe(
        &workspace_center,
        unsafe { NSWorkspaceSessionDidBecomeActiveNotification },
        app,
        SystemEvent::Resume,
    );
    observe(
        &workspace_center,
        unsafe { NSWorkspaceScreensDidWakeNotification },
        app,
        SystemEvent::Resume,
    );

    let default_center = NSNotificationCenter::defaultCenter();
    observe(
        &default_center,
        unsafe { NSApplicationDidChangeScreenParametersNotification },
        app,
        SystemEvent::ScreensChanged,
    );
}

#[cfg(not(target_os = "macos"))]
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
}
