use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::domain::{AppSettings, RecoverableAppError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn clamp_widget_position(
    position: WidgetPosition,
    screens: &[ScreenRect],
    widget_width: u32,
    widget_height: u32,
) -> WidgetPosition {
    let Some(screen) = screens
        .iter()
        .find(|screen| {
            position.x < screen.x.saturating_add_unsigned(screen.width)
                && position.y < screen.y.saturating_add_unsigned(screen.height)
                && position.x.saturating_add_unsigned(widget_width) > screen.x
                && position.y.saturating_add_unsigned(widget_height) > screen.y
        })
        .or_else(|| screens.first())
    else {
        return position;
    };
    let max_x = screen
        .x
        .saturating_add_unsigned(screen.width.saturating_sub(widget_width));
    let max_y = screen
        .y
        .saturating_add_unsigned(screen.height.saturating_sub(widget_height));
    WidgetPosition {
        x: position.x.clamp(screen.x, max_x),
        y: position.y.clamp(screen.y, max_y),
    }
}

pub fn load_settings(path: &Path) -> (AppSettings, Option<RecoverableAppError>) {
    let Ok(contents) = fs::read_to_string(path) else {
        return (AppSettings::default(), None);
    };
    match serde_json::from_str::<AppSettings>(&contents) {
        Ok(settings) => (settings.validate(), None),
        Err(error) => {
            let backup = path.with_extension("corrupt.json");
            let _ = fs::rename(path, &backup);
            (
                AppSettings::default(),
                Some(RecoverableAppError {
                    code: "settings_recovered".into(),
                    message: format!("设置文件损坏，已恢复默认值：{error}"),
                }),
            )
        }
    }
}

pub fn save_settings(path: &Path, settings: &AppSettings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temp = path.with_extension("tmp");
    let json = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(&temp, json).map_err(|error| error.to_string())?;
    fs::rename(&temp, path).map_err(|error| error.to_string())
}

pub fn load_widget_position(path: &Path) -> Option<WidgetPosition> {
    fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
}

pub fn save_widget_position(path: &Path, position: WidgetPosition) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let json = serde_json::to_vec(&position).map_err(|error| error.to_string())?;
    fs::write(path, json).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupted_settings_are_backed_up_and_recovered() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("settings.json");
        fs::write(&path, "{not-json").expect("write fixture");
        let (settings, error) = load_settings(&path);
        assert_eq!(settings, AppSettings::default());
        assert_eq!(error.expect("recovery error").code, "settings_recovered");
        assert!(path.with_extension("corrupt.json").exists());
    }

    #[test]
    fn settings_round_trip_atomically() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("nested/settings.json");
        let settings = AppSettings {
            work_minutes: 42,
            ..AppSettings::default()
        };
        save_settings(&path, &settings).expect("save settings");
        assert_eq!(load_settings(&path).0.work_minutes, 42);
    }

    #[test]
    fn widget_position_is_clamped_back_to_a_visible_screen() {
        let screens = [ScreenRect {
            x: 100,
            y: 50,
            width: 1200,
            height: 800,
        }];
        assert_eq!(
            clamp_widget_position(WidgetPosition { x: 4_000, y: -500 }, &screens, 248, 72,),
            WidgetPosition { x: 1052, y: 50 }
        );
    }

    #[test]
    fn widget_position_round_trips() {
        let directory = tempfile::tempdir().expect("temp directory");
        let path = directory.path().join("widget-position.json");
        let position = WidgetPosition { x: -300, y: 120 };
        save_widget_position(&path, position).expect("save widget position");
        assert_eq!(load_widget_position(&path), Some(position));
    }
}
