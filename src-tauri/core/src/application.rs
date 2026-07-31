use crate::{
    domain::{AppSnapshot, TimerAction, TimerEngine, TimerPhase},
    window_state::DisplayEvent,
};

pub struct TimerActionOutcome {
    pub snapshot: AppSnapshot,
    pub previous_phase: TimerPhase,
    pub display_event: Option<DisplayEvent>,
    pub settings_changed: bool,
}

pub fn apply_timer_action(
    engine: &mut TimerEngine,
    action: TimerAction,
    current_time: u64,
) -> TimerActionOutcome {
    let previous_phase = engine.snapshot().phase;
    let changed = engine.dispatch(action, current_time);
    let display_event = changed.then(|| DisplayEvent::Timer {
        action,
        previous_phase,
        next_phase: engine.snapshot().phase,
    });
    let settings_changed = display_event
        .filter(|event| event.requests_widget_visibility())
        .is_some_and(|_| {
            let mut settings = engine.snapshot().settings;
            if settings.widget_visible {
                false
            } else {
                settings.widget_visible = true;
                engine.update_settings(settings, current_time)
            }
        });
    TimerActionOutcome {
        snapshot: engine.snapshot(),
        previous_phase,
        display_event,
        settings_changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AppSettings;

    #[test]
    fn start_and_resume_align_the_persisted_widget_setting() {
        let mut engine = TimerEngine::new(AppSettings {
            widget_visible: false,
            ..AppSettings::default()
        });
        let started = apply_timer_action(&mut engine, TimerAction::Start, 0);
        assert!(started.settings_changed);
        assert!(started.snapshot.settings.widget_visible);

        apply_timer_action(&mut engine, TimerAction::Pause, 1_000);
        let mut settings = engine.snapshot().settings;
        settings.widget_visible = false;
        engine.update_settings(settings, 1_000);

        let resumed = apply_timer_action(&mut engine, TimerAction::Resume, 2_000);
        assert!(resumed.settings_changed);
        assert!(resumed.snapshot.settings.widget_visible);
    }
}
