use eye_care_core::platform_policy::{break_window_policy, widget_window_policy};

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
