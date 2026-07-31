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
const PBT_APMRESUMESUSPEND_VALUE: usize = 0x7;
const WTS_SESSION_LOCK_VALUE: usize = 0x7;
const WTS_SESSION_UNLOCK_VALUE: usize = 0x8;
const DUPLICATE_EVENT_WINDOW_MS: u64 = 500;

pub fn map_windows_message(message: u32, event_code: usize) -> Option<SystemEvent> {
    match (message, event_code) {
        (WM_WTSSESSION_CHANGE_VALUE, WTS_SESSION_LOCK_VALUE)
        | (WM_POWERBROADCAST_VALUE, PBT_APMSUSPEND_VALUE) => Some(SystemEvent::Suspend),
        (WM_WTSSESSION_CHANGE_VALUE, WTS_SESSION_UNLOCK_VALUE)
        | (WM_POWERBROADCAST_VALUE, PBT_APMRESUMESUSPEND_VALUE) => Some(SystemEvent::Resume),
        (WM_DISPLAYCHANGE_VALUE, _) | (WM_DEVICECHANGE_VALUE, _) => {
            Some(SystemEvent::ScreensChanged)
        }
        _ => None,
    }
}

#[derive(Default)]
pub struct EventDeduplicator {
    last: Option<(SystemEvent, u64)>,
}

impl EventDeduplicator {
    pub fn accept(&mut self, event: SystemEvent, now_ms: u64) -> bool {
        let duplicate = self.last.is_some_and(|(previous, previous_ms)| {
            previous == event && now_ms.saturating_sub(previous_ms) <= DUPLICATE_EVENT_WINDOW_MS
        });
        if !duplicate {
            self.last = Some((event, now_ms));
        }
        !duplicate
    }
}

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
        assert_eq!(map_windows_message(0x0218, 0x12), None);
        assert_eq!(map_windows_message(0x0218, 0x7), Some(SystemEvent::Resume));
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
