use crate::domain::TimerPhase;

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

pub fn tray_icon_is_template() -> bool {
    false
}

pub fn should_show_main_for_second_instance(phase: TimerPhase) -> bool {
    phase != TimerPhase::Resting
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

    #[test]
    fn windows_tray_uses_a_colored_icon() {
        assert!(!tray_icon_is_template());
    }

    #[test]
    fn second_instance_does_not_replace_a_break_overlay() {
        assert!(should_show_main_for_second_instance(TimerPhase::Idle));
        assert!(should_show_main_for_second_instance(TimerPhase::Working));
        assert!(should_show_main_for_second_instance(TimerPhase::Paused));
        assert!(!should_show_main_for_second_instance(TimerPhase::Resting));
    }
}
