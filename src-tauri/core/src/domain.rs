use serde::{Deserialize, Serialize};

pub const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerPhase {
    Idle,
    Working,
    Paused,
    Resting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SleepPolicy {
    RestartCycle,
    PauseResume,
    RealTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppLanguage {
    System,
    Zh,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorScheme {
    Original,
    MorningLake,
    GraphiteLime,
    MistBlueCoral,
    PorcelainForest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub version: u32,
    pub work_minutes: u32,
    pub rest_seconds: u32,
    pub skip_confirmation: bool,
    pub sleep_policy: SleepPolicy,
    pub language: AppLanguage,
    pub theme: ThemePreference,
    pub color_scheme: ColorScheme,
    pub sound_enabled: bool,
    pub notification_enabled: bool,
    pub launch_at_login: bool,
    pub widget_visible: bool,
    pub break_message: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            work_minutes: 20,
            rest_seconds: 20,
            skip_confirmation: true,
            sleep_policy: SleepPolicy::RestartCycle,
            language: AppLanguage::System,
            theme: ThemePreference::System,
            color_scheme: ColorScheme::MistBlueCoral,
            sound_enabled: true,
            notification_enabled: true,
            launch_at_login: false,
            widget_visible: true,
            break_message: "请眺望远方，让眼睛放松。".into(),
        }
    }
}

impl AppSettings {
    pub fn validate(mut self) -> Self {
        self.version = SETTINGS_VERSION;
        self.work_minutes = self.work_minutes.clamp(1, 120);
        self.rest_seconds = self.rest_seconds.clamp(5, 300);
        self.break_message = self.break_message.trim().chars().take(120).collect();
        if self.break_message.is_empty() {
            self.break_message = AppSettings::default().break_message;
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipConfirmationState {
    None,
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverableAppError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub phase: TimerPhase,
    pub seconds_remaining: u32,
    pub skip_confirmation: SkipConfirmationState,
    pub settings: AppSettings,
    pub recoverable_error: Option<RecoverableAppError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimerAction {
    Start,
    Pause,
    Resume,
    TogglePause,
    Stop,
    RequestSkip,
    ConfirmSkip,
    CancelSkip,
    Sleep,
    Wake,
    Tick,
}

#[derive(Debug, Clone)]
pub struct TimerEngine {
    snapshot: AppSnapshot,
    deadline_ms: Option<u64>,
    paused_remaining_ms: u64,
    suspended_remaining_ms: Option<u64>,
}

impl TimerEngine {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            snapshot: AppSnapshot {
                phase: TimerPhase::Idle,
                seconds_remaining: 0,
                skip_confirmation: SkipConfirmationState::None,
                settings: settings.validate(),
                recoverable_error: None,
            },
            deadline_ms: None,
            paused_remaining_ms: 0,
            suspended_remaining_ms: None,
        }
    }

    pub fn snapshot(&self) -> AppSnapshot {
        self.snapshot.clone()
    }

    pub fn update_settings(&mut self, settings: AppSettings, now_ms: u64) -> bool {
        let settings = settings.validate();
        if settings == self.snapshot.settings {
            return false;
        }
        self.snapshot.settings = settings;
        if self.snapshot.phase == TimerPhase::Idle {
            self.snapshot.seconds_remaining = 0;
        } else {
            self.refresh_remaining(now_ms);
        }
        true
    }

    pub fn set_recoverable_error(&mut self, error: Option<RecoverableAppError>) {
        self.snapshot.recoverable_error = error;
    }

    pub fn dispatch(&mut self, action: TimerAction, now_ms: u64) -> bool {
        match action {
            TimerAction::Start if self.snapshot.phase == TimerPhase::Idle => {
                self.start_work(now_ms);
                true
            }
            TimerAction::Pause if self.snapshot.phase == TimerPhase::Working => {
                self.pause(now_ms);
                true
            }
            TimerAction::Resume if self.snapshot.phase == TimerPhase::Paused => {
                self.resume(now_ms);
                true
            }
            TimerAction::TogglePause if self.snapshot.phase == TimerPhase::Working => {
                self.pause(now_ms);
                true
            }
            TimerAction::TogglePause if self.snapshot.phase == TimerPhase::Paused => {
                self.resume(now_ms);
                true
            }
            TimerAction::Stop if self.snapshot.phase != TimerPhase::Idle => {
                self.stop();
                true
            }
            TimerAction::RequestSkip if self.snapshot.phase == TimerPhase::Resting => {
                if self.snapshot.settings.skip_confirmation {
                    if self.snapshot.skip_confirmation == SkipConfirmationState::Pending {
                        return false;
                    }
                    self.snapshot.skip_confirmation = SkipConfirmationState::Pending;
                } else {
                    self.start_work(now_ms);
                }
                true
            }
            TimerAction::ConfirmSkip
                if self.snapshot.phase == TimerPhase::Resting
                    && self.snapshot.skip_confirmation == SkipConfirmationState::Pending =>
            {
                self.start_work(now_ms);
                true
            }
            TimerAction::CancelSkip
                if self.snapshot.skip_confirmation == SkipConfirmationState::Pending =>
            {
                self.snapshot.skip_confirmation = SkipConfirmationState::None;
                true
            }
            TimerAction::Sleep
                if self.snapshot.phase != TimerPhase::Idle
                    && self.suspended_remaining_ms.is_none() =>
            {
                self.refresh_remaining(now_ms);
                self.suspended_remaining_ms = Some(self.current_remaining_ms(now_ms));
                true
            }
            TimerAction::Wake
                if self.snapshot.phase != TimerPhase::Idle
                    && self.suspended_remaining_ms.is_some() =>
            {
                self.handle_wake(now_ms)
            }
            TimerAction::Tick => self.tick(now_ms),
            _ => false,
        }
    }

    fn start_work(&mut self, now_ms: u64) {
        let duration_ms = self.snapshot.settings.work_minutes as u64 * 60_000;
        self.snapshot.phase = TimerPhase::Working;
        self.snapshot.seconds_remaining = (duration_ms / 1000) as u32;
        self.snapshot.skip_confirmation = SkipConfirmationState::None;
        self.deadline_ms = Some(now_ms.saturating_add(duration_ms));
        self.paused_remaining_ms = 0;
        self.suspended_remaining_ms = None;
    }

    fn start_rest(&mut self, now_ms: u64) {
        let duration_ms = self.snapshot.settings.rest_seconds as u64 * 1000;
        self.snapshot.phase = TimerPhase::Resting;
        self.snapshot.seconds_remaining = self.snapshot.settings.rest_seconds;
        self.snapshot.skip_confirmation = SkipConfirmationState::None;
        self.deadline_ms = Some(now_ms.saturating_add(duration_ms));
        self.paused_remaining_ms = 0;
    }

    fn pause(&mut self, now_ms: u64) {
        self.paused_remaining_ms = self.current_remaining_ms(now_ms);
        self.snapshot.seconds_remaining = ceil_seconds(self.paused_remaining_ms);
        self.snapshot.phase = TimerPhase::Paused;
        self.deadline_ms = None;
    }

    fn resume(&mut self, now_ms: u64) {
        self.snapshot.phase = TimerPhase::Working;
        self.deadline_ms = Some(now_ms.saturating_add(self.paused_remaining_ms));
    }

    fn stop(&mut self) {
        self.snapshot.phase = TimerPhase::Idle;
        self.snapshot.seconds_remaining = 0;
        self.snapshot.skip_confirmation = SkipConfirmationState::None;
        self.deadline_ms = None;
        self.paused_remaining_ms = 0;
        self.suspended_remaining_ms = None;
    }

    fn tick(&mut self, now_ms: u64) -> bool {
        if self.suspended_remaining_ms.is_some() {
            return false;
        }
        if !matches!(
            self.snapshot.phase,
            TimerPhase::Working | TimerPhase::Resting
        ) {
            return false;
        }
        let before = self.snapshot.clone();
        if self.current_remaining_ms(now_ms) == 0 {
            if self.snapshot.phase == TimerPhase::Working {
                self.start_rest(now_ms);
            } else {
                self.start_work(now_ms);
            }
        } else {
            self.refresh_remaining(now_ms);
        }
        self.snapshot != before
    }

    fn handle_wake(&mut self, now_ms: u64) -> bool {
        let remaining = self.suspended_remaining_ms.take();
        match self.snapshot.settings.sleep_policy {
            SleepPolicy::RestartCycle => {
                self.start_work(now_ms);
                true
            }
            SleepPolicy::PauseResume => {
                if let Some(remaining_ms) = remaining {
                    if self.snapshot.phase == TimerPhase::Paused {
                        self.paused_remaining_ms = remaining_ms;
                    } else {
                        self.deadline_ms = Some(now_ms.saturating_add(remaining_ms));
                    }
                    self.snapshot.seconds_remaining = ceil_seconds(remaining_ms);
                    true
                } else {
                    false
                }
            }
            SleepPolicy::RealTime => self.advance_with_real_time(now_ms),
        }
    }

    fn advance_with_real_time(&mut self, now_ms: u64) -> bool {
        if !matches!(
            self.snapshot.phase,
            TimerPhase::Working | TimerPhase::Resting
        ) {
            return false;
        }
        let Some(deadline_ms) = self.deadline_ms else {
            return false;
        };
        let before = self.snapshot.clone();
        if now_ms < deadline_ms {
            self.refresh_remaining(now_ms);
            return self.snapshot != before;
        }

        let work_ms = self.snapshot.settings.work_minutes as u64 * 60_000;
        let rest_ms = self.snapshot.settings.rest_seconds as u64 * 1_000;
        let (first_phase, first_duration, second_phase, second_duration) = match self.snapshot.phase
        {
            TimerPhase::Working => (TimerPhase::Resting, rest_ms, TimerPhase::Working, work_ms),
            TimerPhase::Resting => (TimerPhase::Working, work_ms, TimerPhase::Resting, rest_ms),
            TimerPhase::Idle | TimerPhase::Paused => unreachable!(),
        };
        let cycle_duration = first_duration.saturating_add(second_duration);
        let cycle_offset = now_ms.saturating_sub(deadline_ms) % cycle_duration;
        let (phase, remaining_ms) = if cycle_offset < first_duration {
            (first_phase, first_duration - cycle_offset)
        } else {
            (
                second_phase,
                second_duration - (cycle_offset - first_duration),
            )
        };

        self.snapshot.phase = phase;
        self.snapshot.seconds_remaining = ceil_seconds(remaining_ms);
        self.snapshot.skip_confirmation = SkipConfirmationState::None;
        self.deadline_ms = Some(now_ms.saturating_add(remaining_ms));
        self.paused_remaining_ms = 0;
        self.snapshot != before
    }

    fn current_remaining_ms(&self, now_ms: u64) -> u64 {
        if let Some(remaining_ms) = self.suspended_remaining_ms {
            return remaining_ms;
        }
        match self.snapshot.phase {
            TimerPhase::Paused => self.paused_remaining_ms,
            TimerPhase::Working | TimerPhase::Resting => self
                .deadline_ms
                .map(|deadline| deadline.saturating_sub(now_ms))
                .unwrap_or(0),
            TimerPhase::Idle => 0,
        }
    }

    fn refresh_remaining(&mut self, now_ms: u64) {
        self.snapshot.seconds_remaining = ceil_seconds(self.current_remaining_ms(now_ms));
    }
}

fn ceil_seconds(milliseconds: u64) -> u32 {
    milliseconds.div_ceil(1000).min(u32::MAX as u64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_with_short_cycle() -> TimerEngine {
        TimerEngine::new(AppSettings {
            work_minutes: 1,
            rest_seconds: 5,
            ..AppSettings::default()
        })
    }

    #[test]
    fn color_scheme_uses_stable_serialized_identifiers() {
        for (scheme, identifier) in [
            (ColorScheme::Original, "\"original\""),
            (ColorScheme::MorningLake, "\"morning_lake\""),
            (ColorScheme::GraphiteLime, "\"graphite_lime\""),
            (ColorScheme::MistBlueCoral, "\"mist_blue_coral\""),
            (ColorScheme::PorcelainForest, "\"porcelain_forest\""),
        ] {
            assert_eq!(
                serde_json::to_string(&scheme).expect("serialize color scheme"),
                identifier
            );
        }
        assert_eq!(
            serde_json::from_str::<ColorScheme>("\"original\"").expect("deserialize color scheme"),
            ColorScheme::Original
        );
    }

    #[test]
    fn defaults_follow_the_twenty_twenty_rule() {
        let settings = AppSettings::default();
        assert_eq!(settings.work_minutes, 20);
        assert_eq!(settings.rest_seconds, 20);
        assert!(settings.skip_confirmation);
        assert_eq!(settings.sleep_policy, SleepPolicy::RestartCycle);
        assert_eq!(settings.color_scheme, ColorScheme::MistBlueCoral);
    }

    #[test]
    fn start_pause_resume_uses_deadlines_without_drift() {
        let mut engine = engine_with_short_cycle();
        assert!(engine.dispatch(TimerAction::Start, 1_000));
        assert_eq!(engine.snapshot().seconds_remaining, 60);
        engine.dispatch(TimerAction::Tick, 11_250);
        assert_eq!(engine.snapshot().seconds_remaining, 50);
        engine.dispatch(TimerAction::Pause, 11_250);
        engine.dispatch(TimerAction::Tick, 41_250);
        assert_eq!(engine.snapshot().seconds_remaining, 50);
        engine.dispatch(TimerAction::Resume, 41_250);
        engine.dispatch(TimerAction::Tick, 51_250);
        assert_eq!(engine.snapshot().seconds_remaining, 40);
    }

    #[test]
    fn work_and_rest_transition_once_at_deadline() {
        let mut engine = engine_with_short_cycle();
        engine.dispatch(TimerAction::Start, 0);
        assert!(engine.dispatch(TimerAction::Tick, 60_000));
        assert_eq!(engine.snapshot().phase, TimerPhase::Resting);
        assert_eq!(engine.snapshot().seconds_remaining, 5);
        assert!(!engine.dispatch(TimerAction::Tick, 60_000));
        engine.dispatch(TimerAction::Tick, 65_000);
        assert_eq!(engine.snapshot().phase, TimerPhase::Working);
        assert_eq!(engine.snapshot().seconds_remaining, 60);
    }

    #[test]
    fn skip_requires_confirmation_by_default() {
        let mut engine = engine_with_short_cycle();
        engine.dispatch(TimerAction::Start, 0);
        engine.dispatch(TimerAction::Tick, 60_000);
        engine.dispatch(TimerAction::RequestSkip, 61_000);
        assert_eq!(
            engine.snapshot().skip_confirmation,
            SkipConfirmationState::Pending
        );
        engine.dispatch(TimerAction::CancelSkip, 61_000);
        assert_eq!(engine.snapshot().phase, TimerPhase::Resting);
        engine.dispatch(TimerAction::RequestSkip, 62_000);
        engine.dispatch(TimerAction::ConfirmSkip, 62_000);
        assert_eq!(engine.snapshot().phase, TimerPhase::Working);
    }

    #[test]
    fn settings_are_clamped_and_empty_message_is_recovered() {
        let settings = AppSettings {
            work_minutes: 0,
            rest_seconds: 900,
            break_message: "  ".into(),
            ..AppSettings::default()
        }
        .validate();
        assert_eq!(settings.work_minutes, 1);
        assert_eq!(settings.rest_seconds, 300);
        assert!(!settings.break_message.is_empty());
    }

    #[test]
    fn wake_policies_are_deterministic() {
        let mut restart = engine_with_short_cycle();
        restart.dispatch(TimerAction::Start, 0);
        restart.dispatch(TimerAction::Sleep, 20_000);
        restart.dispatch(TimerAction::Wake, 90_000);
        assert_eq!(restart.snapshot().seconds_remaining, 60);

        let mut resume = TimerEngine::new(AppSettings {
            work_minutes: 1,
            sleep_policy: SleepPolicy::PauseResume,
            ..AppSettings::default()
        });
        resume.dispatch(TimerAction::Start, 0);
        resume.dispatch(TimerAction::Sleep, 20_000);
        resume.dispatch(TimerAction::Wake, 90_000);
        assert_eq!(resume.snapshot().seconds_remaining, 40);

        let mut real_time = TimerEngine::new(AppSettings {
            work_minutes: 1,
            rest_seconds: 20,
            sleep_policy: SleepPolicy::RealTime,
            ..AppSettings::default()
        });
        real_time.dispatch(TimerAction::Start, 0);
        real_time.dispatch(TimerAction::Sleep, 20_000);
        real_time.dispatch(TimerAction::Wake, 90_000);
        assert_eq!(real_time.snapshot().phase, TimerPhase::Working);
        assert_eq!(real_time.snapshot().seconds_remaining, 50);
    }

    #[test]
    fn real_time_policy_updates_within_the_current_phase() {
        let mut engine = TimerEngine::new(AppSettings {
            work_minutes: 1,
            rest_seconds: 20,
            sleep_policy: SleepPolicy::RealTime,
            ..AppSettings::default()
        });
        engine.dispatch(TimerAction::Start, 0);
        engine.dispatch(TimerAction::Sleep, 20_000);
        engine.dispatch(TimerAction::Wake, 40_000);

        assert_eq!(engine.snapshot().phase, TimerPhase::Working);
        assert_eq!(engine.snapshot().seconds_remaining, 20);
    }

    #[test]
    fn real_time_policy_catches_up_across_multiple_phases() {
        let mut engine = TimerEngine::new(AppSettings {
            work_minutes: 1,
            rest_seconds: 20,
            sleep_policy: SleepPolicy::RealTime,
            ..AppSettings::default()
        });
        engine.dispatch(TimerAction::Start, 0);
        engine.dispatch(TimerAction::Sleep, 20_000);
        engine.dispatch(TimerAction::Wake, 200_000);

        assert_eq!(engine.snapshot().phase, TimerPhase::Working);
        assert_eq!(engine.snapshot().seconds_remaining, 20);
    }

    #[test]
    fn real_time_policy_catches_up_from_a_rest_phase() {
        let mut engine = TimerEngine::new(AppSettings {
            work_minutes: 1,
            rest_seconds: 20,
            sleep_policy: SleepPolicy::RealTime,
            ..AppSettings::default()
        });
        engine.dispatch(TimerAction::Start, 0);
        engine.dispatch(TimerAction::Tick, 60_000);
        engine.dispatch(TimerAction::Sleep, 65_000);
        engine.dispatch(TimerAction::Wake, 230_000);

        assert_eq!(engine.snapshot().phase, TimerPhase::Resting);
        assert_eq!(engine.snapshot().seconds_remaining, 10);
    }

    #[test]
    fn pause_resume_policy_freezes_ticks_while_the_session_is_inactive() {
        let mut engine = TimerEngine::new(AppSettings {
            work_minutes: 1,
            sleep_policy: SleepPolicy::PauseResume,
            ..AppSettings::default()
        });
        engine.dispatch(TimerAction::Start, 0);
        assert!(engine.dispatch(TimerAction::Sleep, 20_000));
        assert!(!engine.dispatch(TimerAction::Tick, 90_000));
        assert_eq!(engine.snapshot().phase, TimerPhase::Working);
        assert_eq!(engine.snapshot().seconds_remaining, 40);

        assert!(engine.dispatch(TimerAction::Wake, 90_000));
        engine.dispatch(TimerAction::Tick, 100_000);
        assert_eq!(engine.snapshot().seconds_remaining, 30);
    }

    #[test]
    fn duplicate_suspend_and_resume_events_are_ignored() {
        let mut engine = engine_with_short_cycle();
        engine.dispatch(TimerAction::Start, 0);
        assert!(engine.dispatch(TimerAction::Sleep, 20_000));
        assert!(!engine.dispatch(TimerAction::Sleep, 21_000));
        assert!(engine.dispatch(TimerAction::Wake, 90_000));
        assert!(!engine.dispatch(TimerAction::Wake, 91_000));
    }
}
