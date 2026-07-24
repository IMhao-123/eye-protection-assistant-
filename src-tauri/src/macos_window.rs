#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakWindowPolicy {
    pub level: i64,
    pub joins_all_spaces: bool,
    pub full_screen_auxiliary: bool,
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
        full_screen_auxiliary: true,
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

#[cfg(target_os = "macos")]
pub fn present_break_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    window
        .with_webview(move |webview| unsafe {
            let window: &NSWindow = &*webview.ns_window().cast();
            let policy = break_window_policy();
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
            window.setCollectionBehavior(behavior);
            window.setLevel(policy.level as isize);
            window.setHasShadow(false);
            window.setHidesOnDeactivate(false);
            window.setCanHide(false);
            window.setIgnoresMouseEvents(!policy.accepts_mouse_events);
            if policy.orders_front_regardless {
                window.orderFrontRegardless();
            }
            window.makeKeyAndOrderFront(None);
        })
        .map_err(|error| error.to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn present_break_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
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
        assert!(policy.full_screen_auxiliary);
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
}
