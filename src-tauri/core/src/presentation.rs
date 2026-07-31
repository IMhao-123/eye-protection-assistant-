use crate::domain::{AppSnapshot, TimerPhase};

pub struct TrayMenuPresentation {
    pub action_text: &'static str,
    pub action_enabled: bool,
    pub stop_enabled: bool,
    pub widget_text: &'static str,
    pub widget_enabled: bool,
}

pub fn tray_menu_presentation(snapshot: &AppSnapshot) -> TrayMenuPresentation {
    let (action_text, action_enabled) = match snapshot.phase {
        TimerPhase::Idle => ("开始专注", true),
        TimerPhase::Working => ("暂停计时", true),
        TimerPhase::Paused => ("继续计时", true),
        TimerPhase::Resting => ("正在休息", false),
    };
    TrayMenuPresentation {
        action_text,
        action_enabled,
        stop_enabled: snapshot.phase != TimerPhase::Idle,
        widget_text: if snapshot.settings.widget_visible {
            "隐藏计时胶囊"
        } else {
            "显示计时胶囊"
        },
        widget_enabled: matches!(snapshot.phase, TimerPhase::Working | TimerPhase::Paused),
    }
}

pub fn tray_status_text(snapshot: &AppSnapshot) -> String {
    let minutes = snapshot.seconds_remaining / 60;
    let seconds = snapshot.seconds_remaining % 60;
    match snapshot.phase {
        TimerPhase::Idle => "准备就绪".into(),
        TimerPhase::Working => format!("专注中 · {minutes}:{seconds:02}"),
        TimerPhase::Paused => format!("已暂停 · {minutes}:{seconds:02}"),
        TimerPhase::Resting => format!("休息中 · {minutes}:{seconds:02}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AppSettings, TimerAction, TimerEngine};

    #[test]
    fn tray_status_reflects_every_timer_phase() {
        let mut engine = TimerEngine::new(AppSettings::default());
        assert_eq!(tray_status_text(&engine.snapshot()), "准备就绪");
        engine.dispatch(TimerAction::Start, 0);
        assert_eq!(tray_status_text(&engine.snapshot()), "专注中 · 20:00");
        engine.dispatch(TimerAction::Pause, 1_000);
        assert_eq!(tray_status_text(&engine.snapshot()), "已暂停 · 19:59");
    }

    #[test]
    fn tray_uses_one_unambiguous_context_action() {
        let mut snapshot = TimerEngine::new(AppSettings::default()).snapshot();
        assert_eq!(tray_menu_presentation(&snapshot).action_text, "开始专注");

        snapshot.phase = TimerPhase::Working;
        assert_eq!(tray_menu_presentation(&snapshot).action_text, "暂停计时");
        assert_eq!(
            tray_menu_presentation(&snapshot).widget_text,
            "隐藏计时胶囊"
        );

        snapshot.phase = TimerPhase::Paused;
        assert_eq!(tray_menu_presentation(&snapshot).action_text, "继续计时");

        snapshot.phase = TimerPhase::Resting;
        let resting = tray_menu_presentation(&snapshot);
        assert_eq!(resting.action_text, "正在休息");
        assert!(!resting.action_enabled);
    }
}
