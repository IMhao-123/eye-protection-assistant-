#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakWindowPolicy {
    pub always_on_top: bool,
    pub skip_taskbar: bool,
    pub accepts_mouse_events: bool,
    pub takes_focus: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetWindowPolicy {
    pub always_on_top: bool,
    pub skip_taskbar: bool,
    pub takes_focus: bool,
}

pub fn break_window_policy() -> BreakWindowPolicy {
    BreakWindowPolicy {
        always_on_top: true,
        skip_taskbar: true,
        accepts_mouse_events: true,
        takes_focus: true,
    }
}

pub fn widget_window_policy() -> WidgetWindowPolicy {
    WidgetWindowPolicy {
        always_on_top: true,
        skip_taskbar: true,
        takes_focus: false,
    }
}

#[cfg(target_os = "windows")]
fn place_above_normal_windows(window: &tauri::WebviewWindow, activate: bool) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };

    let hwnd = window.hwnd().map_err(|error| error.to_string())?.0;
    let mut flags = SWP_NOMOVE | SWP_NOSIZE;
    if !activate {
        flags |= SWP_NOACTIVATE;
    }
    if unsafe { SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, flags) } == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn place_above_normal_windows(
    _window: &tauri::WebviewWindow,
    _activate: bool,
) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn show_without_activation(window: &tauri::WebviewWindow) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNOACTIVATE};

    let hwnd = window.hwnd().map_err(|error| error.to_string())?.0;
    unsafe {
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn show_without_activation(window: &tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())
}

pub fn present_break_window(
    window: &tauri::WebviewWindow,
    _screen_index: usize,
    _screen_name: Option<String>,
) -> Result<(), String> {
    window
        .set_always_on_top(break_window_policy().always_on_top)
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    place_above_normal_windows(window, true)?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn dismiss_break_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|error| error.to_string())
}

pub fn present_widget_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    window
        .set_always_on_top(widget_window_policy().always_on_top)
        .map_err(|error| error.to_string())?;
    show_without_activation(window)?;
    place_above_normal_windows(window, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn break_overlay_policy_is_interactive_and_above_normal_windows() {
        let policy = break_window_policy();
        assert!(policy.always_on_top);
        assert!(policy.skip_taskbar);
        assert!(policy.accepts_mouse_events);
        assert!(policy.takes_focus);
    }

    #[test]
    fn widget_policy_keeps_the_capsule_visible_without_taking_focus() {
        let policy = widget_window_policy();
        assert!(policy.always_on_top);
        assert!(policy.skip_taskbar);
        assert!(!policy.takes_focus);
    }
}
