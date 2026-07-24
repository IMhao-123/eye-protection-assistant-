use crate::domain::{TimerAction, TimerPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkSurface {
    Main,
    Widget,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowPlan {
    pub show_main: bool,
    pub show_widget: bool,
    pub show_break: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayEvent {
    Timer {
        action: TimerAction,
        previous_phase: TimerPhase,
        next_phase: TimerPhase,
    },
    ShowMain,
    CloseMain {
        phase: TimerPhase,
        widget_enabled: bool,
    },
    SetWidgetVisibility {
        visible: bool,
        phase: TimerPhase,
    },
}

impl DisplayEvent {
    pub fn requires_window_sync(self) -> bool {
        match self {
            Self::Timer {
                action,
                previous_phase,
                next_phase,
            } => {
                action == TimerAction::Stop
                    || previous_phase == TimerPhase::Resting
                    || next_phase == TimerPhase::Resting
                    || self.requests_widget_visibility()
            }
            Self::ShowMain | Self::CloseMain { .. } | Self::SetWidgetVisibility { .. } => true,
        }
    }

    pub fn requests_widget_visibility(self) -> bool {
        matches!(
            self,
            Self::Timer {
                action: TimerAction::Start,
                next_phase: TimerPhase::Working,
                ..
            } | Self::Timer {
                action: TimerAction::Resume | TimerAction::TogglePause,
                previous_phase: TimerPhase::Paused,
                next_phase: TimerPhase::Working,
            } | Self::SetWidgetVisibility { visible: true, .. }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowCoordinator {
    work_surface: WorkSurface,
    surface_before_main: Option<WorkSurface>,
}

impl Default for WindowCoordinator {
    fn default() -> Self {
        Self {
            work_surface: WorkSurface::Main,
            surface_before_main: None,
        }
    }
}

impl WindowCoordinator {
    pub fn apply(&mut self, event: DisplayEvent) {
        match event {
            DisplayEvent::Timer {
                action: TimerAction::Start,
                next_phase: TimerPhase::Working,
                ..
            }
            | DisplayEvent::Timer {
                action: TimerAction::Resume,
                previous_phase: TimerPhase::Paused,
                next_phase: TimerPhase::Working,
            }
            | DisplayEvent::Timer {
                action: TimerAction::TogglePause,
                previous_phase: TimerPhase::Paused,
                next_phase: TimerPhase::Working,
            } => {
                self.work_surface = WorkSurface::Widget;
                self.surface_before_main = None;
            }
            DisplayEvent::Timer {
                action: TimerAction::Stop,
                next_phase: TimerPhase::Idle,
                ..
            } if self.work_surface == WorkSurface::Widget => {
                self.work_surface = WorkSurface::Hidden;
                self.surface_before_main = None;
            }
            DisplayEvent::ShowMain => {
                if self.work_surface != WorkSurface::Main {
                    self.surface_before_main = Some(self.work_surface);
                }
                self.work_surface = WorkSurface::Main;
            }
            DisplayEvent::CloseMain {
                phase: TimerPhase::Working | TimerPhase::Paused,
                widget_enabled,
            } => {
                self.work_surface = self
                    .surface_before_main
                    .take()
                    .unwrap_or(if widget_enabled {
                        WorkSurface::Widget
                    } else {
                        WorkSurface::Hidden
                    });
            }
            DisplayEvent::CloseMain { .. } => {
                self.work_surface = WorkSurface::Hidden;
                self.surface_before_main = None;
            }
            DisplayEvent::SetWidgetVisibility {
                visible: true,
                phase: TimerPhase::Working | TimerPhase::Paused,
            } => {
                self.work_surface = WorkSurface::Widget;
                self.surface_before_main = None;
            }
            DisplayEvent::SetWidgetVisibility { visible: false, .. }
                if self.work_surface == WorkSurface::Widget =>
            {
                self.work_surface = WorkSurface::Hidden;
                self.surface_before_main = None;
            }
            DisplayEvent::SetWidgetVisibility { visible: false, .. }
                if self.work_surface == WorkSurface::Main =>
            {
                self.surface_before_main = Some(WorkSurface::Hidden);
            }
            DisplayEvent::Timer { .. } | DisplayEvent::SetWidgetVisibility { .. } => {}
        }
    }

    pub fn plan(&self, phase: TimerPhase) -> WindowPlan {
        if phase == TimerPhase::Resting {
            return WindowPlan {
                show_main: false,
                show_widget: false,
                show_break: true,
            };
        }

        WindowPlan {
            show_main: self.work_surface == WorkSurface::Main,
            show_widget: matches!(phase, TimerPhase::Working | TimerPhase::Paused)
                && self.work_surface == WorkSurface::Widget,
            show_break: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timer(
        action: TimerAction,
        previous_phase: TimerPhase,
        next_phase: TimerPhase,
    ) -> DisplayEvent {
        DisplayEvent::Timer {
            action,
            previous_phase,
            next_phase,
        }
    }

    #[test]
    fn start_and_resume_choose_the_widget_exclusively() {
        let mut coordinator = WindowCoordinator::default();
        coordinator.apply(timer(
            TimerAction::Start,
            TimerPhase::Idle,
            TimerPhase::Working,
        ));
        assert_eq!(
            coordinator.plan(TimerPhase::Working),
            WindowPlan {
                show_main: false,
                show_widget: true,
                show_break: false,
            }
        );

        coordinator.apply(DisplayEvent::ShowMain);
        coordinator.apply(timer(
            TimerAction::Resume,
            TimerPhase::Paused,
            TimerPhase::Working,
        ));
        assert!(coordinator.plan(TimerPhase::Working).show_widget);
        assert!(!coordinator.plan(TimerPhase::Working).show_main);
    }

    #[test]
    fn hiding_or_closing_the_widget_keeps_both_work_windows_quiet() {
        let mut coordinator = WindowCoordinator::default();
        coordinator.apply(timer(
            TimerAction::Start,
            TimerPhase::Idle,
            TimerPhase::Working,
        ));
        coordinator.apply(DisplayEvent::SetWidgetVisibility {
            visible: false,
            phase: TimerPhase::Working,
        });

        assert_eq!(
            coordinator.plan(TimerPhase::Working),
            WindowPlan {
                show_main: false,
                show_widget: false,
                show_break: false,
            }
        );
    }

    #[test]
    fn opening_main_hides_widget_and_closing_main_restores_it_when_enabled() {
        let mut coordinator = WindowCoordinator::default();
        coordinator.apply(timer(
            TimerAction::Start,
            TimerPhase::Idle,
            TimerPhase::Working,
        ));
        coordinator.apply(DisplayEvent::ShowMain);
        let main_plan = coordinator.plan(TimerPhase::Working);
        assert!(main_plan.show_main);
        assert!(!main_plan.show_widget);

        coordinator.apply(DisplayEvent::CloseMain {
            phase: TimerPhase::Working,
            widget_enabled: true,
        });
        let widget_plan = coordinator.plan(TimerPhase::Working);
        assert!(!widget_plan.show_main);
        assert!(widget_plan.show_widget);
    }

    #[test]
    fn stopping_keeps_main_visibility_and_never_opens_it_from_widget() {
        let mut main = WindowCoordinator::default();
        main.apply(timer(
            TimerAction::Stop,
            TimerPhase::Working,
            TimerPhase::Idle,
        ));
        assert!(main.plan(TimerPhase::Idle).show_main);

        let mut widget = WindowCoordinator::default();
        widget.apply(timer(
            TimerAction::Start,
            TimerPhase::Idle,
            TimerPhase::Working,
        ));
        widget.apply(timer(
            TimerAction::Stop,
            TimerPhase::Working,
            TimerPhase::Idle,
        ));
        let stopped = widget.plan(TimerPhase::Idle);
        assert!(!stopped.show_main);
        assert!(!stopped.show_widget);
    }

    #[test]
    fn rest_temporarily_replaces_and_then_restores_each_work_surface() {
        for surface in [WorkSurface::Main, WorkSurface::Widget, WorkSurface::Hidden] {
            let coordinator = WindowCoordinator {
                work_surface: surface,
                surface_before_main: None,
            };
            let resting = coordinator.plan(TimerPhase::Resting);
            assert!(!resting.show_main);
            assert!(!resting.show_widget);
            assert!(resting.show_break);

            let restored = coordinator.plan(TimerPhase::Working);
            assert_eq!(restored.show_main, surface == WorkSurface::Main);
            assert_eq!(restored.show_widget, surface == WorkSurface::Widget);
            assert!(!restored.show_break);
        }
    }

    #[test]
    fn every_plan_keeps_main_and_widget_mutually_exclusive() {
        for surface in [WorkSurface::Main, WorkSurface::Widget, WorkSurface::Hidden] {
            for phase in [
                TimerPhase::Idle,
                TimerPhase::Working,
                TimerPhase::Paused,
                TimerPhase::Resting,
            ] {
                let plan = WindowCoordinator {
                    work_surface: surface,
                    surface_before_main: None,
                }
                .plan(phase);
                assert!(!(plan.show_main && plan.show_widget));
            }
        }
    }

    #[test]
    fn main_window_restores_an_explicitly_hidden_surface() {
        let mut coordinator = WindowCoordinator::default();
        coordinator.apply(timer(
            TimerAction::Start,
            TimerPhase::Idle,
            TimerPhase::Working,
        ));
        coordinator.apply(DisplayEvent::SetWidgetVisibility {
            visible: false,
            phase: TimerPhase::Working,
        });
        coordinator.apply(DisplayEvent::ShowMain);
        coordinator.apply(DisplayEvent::CloseMain {
            phase: TimerPhase::Working,
            widget_enabled: false,
        });

        assert_eq!(
            coordinator.plan(TimerPhase::Working),
            WindowPlan {
                show_main: false,
                show_widget: false,
                show_break: false,
            }
        );
    }

    #[test]
    fn only_display_changes_request_native_window_synchronization() {
        assert!(
            !timer(TimerAction::Tick, TimerPhase::Working, TimerPhase::Working,)
                .requires_window_sync()
        );
        assert!(
            !timer(TimerAction::Sleep, TimerPhase::Working, TimerPhase::Working,)
                .requires_window_sync()
        );
        assert!(
            !timer(TimerAction::Pause, TimerPhase::Working, TimerPhase::Paused,)
                .requires_window_sync()
        );
        assert!(
            timer(TimerAction::Tick, TimerPhase::Working, TimerPhase::Resting,)
                .requires_window_sync()
        );
        assert!(DisplayEvent::ShowMain.requires_window_sync());
        assert!(DisplayEvent::SetWidgetVisibility {
            visible: false,
            phase: TimerPhase::Working,
        }
        .requires_window_sync());
    }

    #[test]
    fn start_and_resume_are_explicit_widget_visibility_requests() {
        assert!(
            timer(TimerAction::Start, TimerPhase::Idle, TimerPhase::Working,)
                .requests_widget_visibility()
        );
        assert!(
            timer(TimerAction::Resume, TimerPhase::Paused, TimerPhase::Working,)
                .requests_widget_visibility()
        );
        assert!(timer(
            TimerAction::TogglePause,
            TimerPhase::Paused,
            TimerPhase::Working,
        )
        .requests_widget_visibility());
        assert!(
            !timer(TimerAction::Pause, TimerPhase::Working, TimerPhase::Paused,)
                .requests_widget_visibility()
        );
        assert!(
            !timer(TimerAction::Tick, TimerPhase::Working, TimerPhase::Working,)
                .requests_widget_visibility()
        );
    }
}
