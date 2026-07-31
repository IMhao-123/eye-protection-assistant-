#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakWindowPolicy {
    pub level: i64,
    pub joins_all_spaces: bool,
    pub joins_all_applications: bool,
    pub full_screen_auxiliary: bool,
    pub uses_nonactivating_panel: bool,
    pub requires_accessory_application: bool,
    pub makes_key_window: bool,
    pub keeps_host_content_view_attached: bool,
    pub uses_native_screen_frame: bool,
    pub reapplies_outer_frame_after_creation: bool,
    pub stationary: bool,
    pub ignores_window_cycle: bool,
    pub accepts_mouse_events: bool,
    pub orders_front_regardless: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WidgetWindowPolicy {
    pub joins_all_spaces: bool,
    pub full_screen_auxiliary: bool,
    pub stationary: bool,
    pub ignores_window_cycle: bool,
    pub orders_front_regardless: bool,
}

pub fn break_window_policy() -> BreakWindowPolicy {
    BreakWindowPolicy {
        level: 1000,
        joins_all_spaces: true,
        joins_all_applications: true,
        full_screen_auxiliary: true,
        uses_nonactivating_panel: true,
        requires_accessory_application: true,
        makes_key_window: false,
        keeps_host_content_view_attached: true,
        uses_native_screen_frame: true,
        reapplies_outer_frame_after_creation: true,
        stationary: true,
        ignores_window_cycle: true,
        accepts_mouse_events: true,
        orders_front_regardless: true,
    }
}

pub fn widget_window_policy() -> WidgetWindowPolicy {
    WidgetWindowPolicy {
        joins_all_spaces: true,
        full_screen_auxiliary: true,
        stationary: true,
        ignores_window_cycle: true,
        orders_front_regardless: true,
    }
}

fn choose_break_screen_index(
    screen_names: &[String],
    requested_index: usize,
    requested_name: Option<&str>,
) -> Option<usize> {
    if screen_names.is_empty() {
        return None;
    }
    let fallback = requested_index.min(screen_names.len() - 1);
    let Some(requested_name) = requested_name else {
        return Some(fallback);
    };
    let matches = screen_names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| (name == requested_name).then_some(index))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [only] => Some(*only),
        many if many.contains(&fallback) => Some(fallback),
        _ => Some(fallback),
    }
}

#[cfg(target_os = "macos")]
thread_local! {
    static BREAK_PANELS: std::cell::RefCell<
        std::collections::HashMap<String, objc2::rc::Retained<objc2_app_kit::NSPanel>>
    > = std::cell::RefCell::new(std::collections::HashMap::new());
}

#[cfg(target_os = "macos")]
const BREAK_PANEL_OVERSCAN_POINTS: f64 = 16.0;

#[cfg(target_os = "macos")]
pub fn present_break_window(
    window: &tauri::WebviewWindow,
    screen_index: usize,
    screen_name: Option<String>,
) -> Result<(), String> {
    use std::sync::{Arc, Mutex};

    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{
        NSBackingStoreType, NSPanel, NSScreen, NSView, NSWindow, NSWindowCollectionBehavior,
        NSWindowStyleMask,
    };
    use objc2_foundation::NSString;

    let label = window.label().to_string();
    let outcome = Arc::new(Mutex::new(Ok(())));
    let callback_outcome = Arc::clone(&outcome);
    window
        .with_webview(move |webview| unsafe {
            let result = BREAK_PANELS.with(|panels| -> Result<(), String> {
                let host: &NSWindow = &*webview.ns_window().cast();
                let policy = break_window_policy();
                let mut panels = panels.borrow_mut();
                let mtm = MainThreadMarker::new()
                    .ok_or_else(|| "休息遮罩必须在 macOS 主线程创建".to_string())?;
                let screens = NSScreen::screens(mtm);
                if screens.is_empty() {
                    return Err("休息遮罩找不到可用显示器".to_string());
                }
                let screen_names = (0..screens.len())
                    .map(|index| screens.objectAtIndex(index).localizedName().to_string())
                    .collect::<Vec<_>>();
                let selected_index =
                    choose_break_screen_index(&screen_names, screen_index, screen_name.as_deref())
                        .ok_or_else(|| "休息遮罩找不到可用显示器".to_string())?;
                let screen = screens.objectAtIndex(selected_index);
                let mut target_frame = screen.frame();
                target_frame.origin.x -= BREAK_PANEL_OVERSCAN_POINTS;
                target_frame.origin.y -= BREAK_PANEL_OVERSCAN_POINTS;
                target_frame.size.width += BREAK_PANEL_OVERSCAN_POINTS * 2.0;
                target_frame.size.height += BREAK_PANEL_OVERSCAN_POINTS * 2.0;

                if let Some(panel) = panels.get(&label) {
                    panel.setFrame_display(target_frame, true);
                    if policy.orders_front_regardless {
                        panel.orderFrontRegardless();
                    }
                    return Ok(());
                }

                let content_view = host
                    .contentView()
                    .ok_or_else(|| "休息遮罩缺少 WebView 内容".to_string())?;
                let style = if policy.uses_nonactivating_panel {
                    NSWindowStyleMask::NonactivatingPanel
                } else {
                    NSWindowStyleMask::Borderless
                };
                let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
                    NSPanel::alloc(mtm),
                    target_frame,
                    style,
                    NSBackingStoreType::Buffered,
                    false,
                );
                let mut behavior = NSWindowCollectionBehavior::Default;
                if policy.joins_all_spaces {
                    behavior |= NSWindowCollectionBehavior::CanJoinAllSpaces;
                }
                if policy.joins_all_applications {
                    behavior |= NSWindowCollectionBehavior::CanJoinAllApplications;
                }
                if policy.full_screen_auxiliary {
                    behavior |= NSWindowCollectionBehavior::FullScreenAuxiliary;
                }
                if policy.stationary {
                    behavior |= NSWindowCollectionBehavior::Stationary;
                }
                if policy.ignores_window_cycle {
                    behavior |= NSWindowCollectionBehavior::IgnoresCycle;
                }

                let placeholder = NSView::initWithFrame(NSView::alloc(mtm), content_view.frame());
                host.setContentView(Some(&placeholder));
                panel.setContentView(Some(&content_view));
                panel.setTitle(&NSString::from_str("护眼休息"));
                panel.setCollectionBehavior(behavior);
                panel.setLevel(policy.level as isize);
                panel.setFloatingPanel(true);
                panel.setBecomesKeyOnlyIfNeeded(true);
                panel.setHasShadow(false);
                panel.setHidesOnDeactivate(false);
                panel.setCanHide(false);
                panel.setIgnoresMouseEvents(!policy.accepts_mouse_events);
                if policy.reapplies_outer_frame_after_creation {
                    panel.setFrame_display(target_frame, true);
                }
                if policy.orders_front_regardless {
                    panel.orderFrontRegardless();
                }
                panels.insert(label.clone(), panel);
                Ok(())
            });
            if let Ok(mut outcome) = callback_outcome.lock() {
                *outcome = result;
            }
        })
        .map_err(|error| error.to_string())?;
    let result = outcome
        .lock()
        .map_err(|_| "休息遮罩结果锁已损坏".to_string())?
        .clone();
    result
}

#[cfg(target_os = "macos")]
pub fn dismiss_break_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    use std::sync::{Arc, Mutex};

    use objc2_app_kit::NSWindow;

    let label = window.label().to_string();
    let outcome = Arc::new(Mutex::new(Ok(())));
    let callback_outcome = Arc::clone(&outcome);
    window
        .with_webview(move |webview| unsafe {
            let result = BREAK_PANELS.with(|panels| -> Result<(), String> {
                let Some(panel) = panels.borrow_mut().remove(&label) else {
                    return Ok(());
                };
                let host: &NSWindow = &*webview.ns_window().cast();
                panel.orderOut(None);
                if let Some(content_view) = panel.contentView() {
                    panel.setContentView(None);
                    host.setContentView(Some(&content_view));
                }
                panel.close();
                Ok(())
            });
            if let Ok(mut outcome) = callback_outcome.lock() {
                *outcome = result;
            }
        })
        .map_err(|error| error.to_string())?;
    let result = outcome
        .lock()
        .map_err(|_| "休息遮罩结果锁已损坏".to_string())?
        .clone();
    result
}

#[cfg(not(target_os = "macos"))]
pub fn present_break_window(
    window: &tauri::WebviewWindow,
    _screen_index: usize,
    _screen_name: Option<String>,
) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn dismiss_break_window(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn present_widget_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    use objc2_app_kit::{NSColor, NSWindow, NSWindowCollectionBehavior};

    window
        .with_webview(move |webview| unsafe {
            let window: &NSWindow = &*webview.ns_window().cast();
            let policy = widget_window_policy();
            let mut behavior = window.collectionBehavior();
            if policy.joins_all_spaces {
                behavior |= NSWindowCollectionBehavior::CanJoinAllSpaces;
            }
            if policy.full_screen_auxiliary {
                behavior |= NSWindowCollectionBehavior::FullScreenAuxiliary;
            }
            if policy.stationary {
                behavior |= NSWindowCollectionBehavior::Stationary;
            }
            if policy.ignores_window_cycle {
                behavior |= NSWindowCollectionBehavior::IgnoresCycle;
            }
            let clear = NSColor::clearColor();
            window.setCollectionBehavior(behavior);
            window.setHasShadow(false);
            window.setOpaque(false);
            window.setBackgroundColor(Some(&clear));
            window.setHidesOnDeactivate(false);
            window.setCanHide(false);
            if policy.orders_front_regardless {
                window.orderFrontRegardless();
            }
        })
        .map_err(|error| error.to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn present_widget_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn break_overlay_policy_covers_full_screen_spaces_and_input() {
        let policy = break_window_policy();
        assert_eq!(policy.level, 1000);
        assert!(policy.joins_all_spaces);
        assert!(policy.joins_all_applications);
        assert!(policy.full_screen_auxiliary);
        assert!(policy.uses_nonactivating_panel);
        assert!(policy.requires_accessory_application);
        assert!(!policy.makes_key_window);
        assert!(policy.keeps_host_content_view_attached);
        assert!(policy.uses_native_screen_frame);
        assert!(policy.reapplies_outer_frame_after_creation);
        assert!(policy.stationary);
        assert!(policy.ignores_window_cycle);
        assert!(policy.accepts_mouse_events);
        assert!(policy.orders_front_regardless);
    }

    #[test]
    fn widget_policy_keeps_the_capsule_visible_without_taking_focus() {
        let policy = widget_window_policy();
        assert!(policy.joins_all_spaces);
        assert!(policy.full_screen_auxiliary);
        assert!(policy.stationary);
        assert!(policy.ignores_window_cycle);
        assert!(policy.orders_front_regardless);
    }

    #[test]
    fn duplicate_monitor_names_keep_each_break_overlay_on_its_requested_screen() {
        let names = vec!["Studio Display".to_string(), "Studio Display".to_string()];
        assert_eq!(
            choose_break_screen_index(&names, 0, Some("Studio Display")),
            Some(0)
        );
        assert_eq!(
            choose_break_screen_index(&names, 1, Some("Studio Display")),
            Some(1)
        );
        assert_eq!(choose_break_screen_index(&names, 4, None), Some(1));
        assert_eq!(choose_break_screen_index(&[], 0, None), None);
    }
}
