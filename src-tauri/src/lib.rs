mod system_events;
mod windows_window;

use std::{
    path::PathBuf,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use eye_care_core::{
    application::{apply_timer_action as apply_core_timer_action, TimerActionOutcome},
    domain::{AppSettings, AppSnapshot, TimerAction, TimerEngine, TimerPhase},
    persistence::{self, ScreenRect, WidgetPosition},
    platform_policy::{
        break_window_policy, should_show_main_for_second_instance, tray_icon_is_template,
        widget_window_policy,
    },
    presentation::{tray_menu_presentation, tray_status_text},
    window_state::{DisplayEvent, WindowCoordinator, WindowPlan},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};
use tauri_plugin_notification::NotificationExt;

const SNAPSHOT_EVENT: &str = "app://snapshot-changed";
const WIDGET_LOGICAL_WIDTH: u32 = 248;
const WIDGET_LOGICAL_HEIGHT: u32 = 72;

struct RuntimeState {
    engine: Mutex<TimerEngine>,
    settings_path: Mutex<Option<PathBuf>>,
    widget_position_path: Mutex<Option<PathBuf>>,
    widget_position: Mutex<Option<WidgetPosition>>,
    tray_status: Mutex<Option<MenuItem<tauri::Wry>>>,
    tray_action: Mutex<Option<MenuItem<tauri::Wry>>>,
    tray_stop: Mutex<Option<MenuItem<tauri::Wry>>>,
    tray_widget: Mutex<Option<MenuItem<tauri::Wry>>>,
    windows: Mutex<WindowCoordinator>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            engine: Mutex::new(TimerEngine::new(AppSettings::default())),
            settings_path: Mutex::new(None),
            widget_position_path: Mutex::new(None),
            widget_position: Mutex::new(None),
            tray_status: Mutex::new(None),
            tray_action: Mutex::new(None),
            tray_stop: Mutex::new(None),
            tray_widget: Mutex::new(None),
            windows: Mutex::new(WindowCoordinator::default()),
        }
    }
}

#[tauri::command]
fn get_app_snapshot(state: tauri::State<'_, RuntimeState>) -> Result<AppSnapshot, String> {
    state
        .engine
        .lock()
        .map(|engine| engine.snapshot())
        .map_err(|_| "计时状态暂时不可用".to_string())
}

#[tauri::command]
fn dispatch_timer_action(
    app: AppHandle,
    state: tauri::State<'_, RuntimeState>,
    action: TimerAction,
) -> Result<AppSnapshot, String> {
    let outcome = apply_timer_action(&state, action, now_ms())?;
    if outcome.settings_changed {
        persist_snapshot_settings(&state, &outcome.snapshot)?;
    }
    if let Some(event) = outcome.display_event {
        publish_snapshot(&app, &outcome.snapshot, outcome.previous_phase, Some(event));
    }
    Ok(outcome.snapshot)
}

#[tauri::command]
fn update_settings(
    app: AppHandle,
    state: tauri::State<'_, RuntimeState>,
    settings: AppSettings,
) -> Result<AppSnapshot, String> {
    let (snapshot, widget_visibility_changed) = {
        let mut engine = state
            .engine
            .lock()
            .map_err(|_| "设置暂时不可用".to_string())?;
        let previous_widget_visibility = engine.snapshot().settings.widget_visible;
        engine.update_settings(settings, now_ms());
        let snapshot = engine.snapshot();
        (
            snapshot.clone(),
            previous_widget_visibility != snapshot.settings.widget_visible,
        )
    };
    if let Some(path) = state
        .settings_path
        .lock()
        .map_err(|_| "设置路径暂时不可用".to_string())?
        .clone()
    {
        persistence::save_settings(&path, &snapshot.settings)?;
    }
    let display_event = widget_visibility_changed.then_some(DisplayEvent::SetWidgetVisibility {
        visible: snapshot.settings.widget_visible,
        phase: snapshot.phase,
    });
    publish_snapshot(&app, &snapshot, snapshot.phase, display_event);
    Ok(snapshot)
}

#[tauri::command]
fn set_widget_visibility(
    app: AppHandle,
    state: tauri::State<'_, RuntimeState>,
    visible: bool,
) -> Result<AppSnapshot, String> {
    let snapshot = {
        let mut engine = state
            .engine
            .lock()
            .map_err(|_| "设置暂时不可用".to_string())?;
        let mut settings = engine.snapshot().settings;
        settings.widget_visible = visible;
        engine.update_settings(settings, now_ms());
        engine.snapshot()
    };
    persist_snapshot_settings(&state, &snapshot)?;
    publish_snapshot(
        &app,
        &snapshot,
        snapshot.phase,
        Some(DisplayEvent::SetWidgetVisibility {
            visible,
            phase: snapshot.phase,
        }),
    );
    Ok(snapshot)
}

fn persist_snapshot_settings(state: &RuntimeState, snapshot: &AppSnapshot) -> Result<(), String> {
    if let Some(path) = state
        .settings_path
        .lock()
        .map_err(|_| "设置路径暂时不可用".to_string())?
        .clone()
    {
        persistence::save_settings(&path, &snapshot.settings)?;
    }
    Ok(())
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    let state = app.state::<RuntimeState>();
    let snapshot = state
        .engine
        .lock()
        .map_err(|_| "计时状态暂时不可用".to_string())?
        .snapshot();
    apply_display_event(&state, DisplayEvent::ShowMain)?;
    sync_windows(&app, &snapshot)
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    system_events::shutdown();
    app.exit(0);
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

fn perform_action(app: &AppHandle, action: TimerAction) {
    let state = app.state::<RuntimeState>();
    match apply_timer_action(&state, action, now_ms()) {
        Ok(outcome) => {
            if outcome.settings_changed {
                if let Err(error) = persist_snapshot_settings(&state, &outcome.snapshot) {
                    eprintln!("failed to save widget preference: {error}");
                }
            }
            if let Some(event) = outcome.display_event {
                publish_snapshot(app, &outcome.snapshot, outcome.previous_phase, Some(event));
            }
        }
        Err(error) => eprintln!("failed to update timer: {error}"),
    }
}

fn apply_timer_action(
    state: &RuntimeState,
    action: TimerAction,
    current_time: u64,
) -> Result<TimerActionOutcome, String> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| "计时状态暂时不可用".to_string())?;
    Ok(apply_core_timer_action(&mut engine, action, current_time))
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn handle_system_event(app: &AppHandle, event: system_events::SystemEvent) {
    if let Some(action) = system_events::timer_action(event) {
        perform_action(app, action);
        return;
    }

    let state = app.state::<RuntimeState>();
    let snapshot = state.engine.lock().map(|engine| engine.snapshot());
    if let Ok(snapshot) = snapshot {
        if let Err(error) = sync_windows(app, &snapshot) {
            eprintln!("failed to react to display configuration change: {error}");
        }
    }
}

fn publish_snapshot(
    app: &AppHandle,
    snapshot: &AppSnapshot,
    previous_phase: TimerPhase,
    display_event: Option<DisplayEvent>,
) {
    let state = app.state::<RuntimeState>();
    let should_sync_windows = display_event.is_some_and(DisplayEvent::requires_window_sync);
    if let Some(event) = display_event {
        if let Err(error) = apply_display_event(&state, event) {
            eprintln!("failed to update window state: {error}");
        }
    }
    if let Ok(status) = state.tray_status.lock() {
        if let Some(status) = status.as_ref() {
            if let Err(error) = status.set_text(tray_status_text(snapshot)) {
                eprintln!("failed to update tray status: {error}");
            }
        }
    }
    update_tray_controls(&state, snapshot);
    if let Err(error) = app.emit(SNAPSHOT_EVENT, snapshot) {
        eprintln!("failed to emit snapshot: {error}");
    }
    if should_sync_windows {
        if let Err(error) = sync_windows(app, snapshot) {
            eprintln!("failed to synchronize windows: {error}");
        }
    }
    if snapshot.phase != previous_phase && snapshot.settings.notification_enabled {
        let (title, body) = match snapshot.phase {
            TimerPhase::Resting => ("该休息了", snapshot.settings.break_message.as_str()),
            TimerPhase::Working if previous_phase == TimerPhase::Resting => {
                ("休息完成", "下一轮专注已经开始。")
            }
            _ => ("", ""),
        };
        if !title.is_empty() {
            if let Err(error) = app.notification().builder().title(title).body(body).show() {
                eprintln!("failed to show notification: {error}");
            }
        }
    }
}

fn apply_display_event(
    state: &tauri::State<'_, RuntimeState>,
    event: DisplayEvent,
) -> Result<(), String> {
    state
        .windows
        .lock()
        .map(|mut windows| windows.apply(event))
        .map_err(|_| "窗口状态暂时不可用".to_string())
}

fn current_window_plan(
    state: &tauri::State<'_, RuntimeState>,
    phase: TimerPhase,
) -> Result<WindowPlan, String> {
    state
        .windows
        .lock()
        .map(|windows| windows.plan(phase))
        .map_err(|_| "窗口状态暂时不可用".to_string())
}

fn update_tray_controls(state: &tauri::State<'_, RuntimeState>, snapshot: &AppSnapshot) {
    let menu = tray_menu_presentation(snapshot);
    if let Ok(action) = state.tray_action.lock() {
        if let Some(action) = action.as_ref() {
            let _ = action.set_text(menu.action_text);
            let _ = action.set_enabled(menu.action_enabled);
        }
    }
    if let Ok(stop) = state.tray_stop.lock() {
        if let Some(stop) = stop.as_ref() {
            let _ = stop.set_enabled(menu.stop_enabled);
        }
    }
    if let Ok(widget) = state.tray_widget.lock() {
        if let Some(widget) = widget.as_ref() {
            let _ = widget.set_text(menu.widget_text);
            let _ = widget.set_enabled(menu.widget_enabled);
        }
    }
}

fn handle_second_instance(app: &AppHandle) {
    let state = app.state::<RuntimeState>();
    let phase = state.engine.lock().map(|engine| engine.snapshot().phase);
    if phase.is_ok_and(should_show_main_for_second_instance) {
        let _ = show_main_window(app.clone());
    }
}

fn ensure_widget(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("widget") {
        return Ok(window);
    }
    let widget_policy = widget_window_policy();
    let builder = WebviewWindowBuilder::new(
        app,
        "widget",
        WebviewUrl::App("index.html?view=widget".into()),
    )
    .title("护眼助手计时")
    .inner_size(WIDGET_LOGICAL_WIDTH as f64, WIDGET_LOGICAL_HEIGHT as f64)
    .min_inner_size(WIDGET_LOGICAL_WIDTH as f64, WIDGET_LOGICAL_HEIGHT as f64)
    .max_inner_size(WIDGET_LOGICAL_WIDTH as f64, WIDGET_LOGICAL_HEIGHT as f64)
    .decorations(false)
    .shadow(false)
    .resizable(false)
    .always_on_top(widget_policy.always_on_top)
    .skip_taskbar(widget_policy.skip_taskbar)
    .visible(false);
    #[cfg(target_os = "windows")]
    let builder = builder.transparent(true);
    let window = builder.build().map_err(|error| error.to_string())?;
    let state = app.state::<RuntimeState>();
    if let Ok(position) = state.widget_position.lock() {
        if let Some(position) = *position {
            window
                .set_position(PhysicalPosition::new(position.x, position.y))
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(window)
}

fn clamp_widget_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    let position = window.outer_position().map_err(|error| error.to_string())?;
    let available = window
        .available_monitors()
        .map_err(|error| error.to_string())?;
    let primary = window
        .primary_monitor()
        .map_err(|error| error.to_string())?;
    let mut screens = Vec::with_capacity(available.len());
    if let Some(monitor) = primary {
        screens.push(ScreenRect {
            x: monitor.position().x,
            y: monitor.position().y,
            width: monitor.size().width,
            height: monitor.size().height,
        });
    }
    for monitor in available {
        let screen = ScreenRect {
            x: monitor.position().x,
            y: monitor.position().y,
            width: monitor.size().width,
            height: monitor.size().height,
        };
        if !screens.contains(&screen) {
            screens.push(screen);
        }
    }
    let scale_factor = window.scale_factor().map_err(|error| error.to_string())?;
    let (widget_width, widget_height) = persistence::logical_size_to_physical(
        WIDGET_LOGICAL_WIDTH,
        WIDGET_LOGICAL_HEIGHT,
        scale_factor,
    );
    let clamped = persistence::clamp_widget_position(
        WidgetPosition {
            x: position.x,
            y: position.y,
        },
        &screens,
        widget_width,
        widget_height,
    );
    if clamped.x != position.x || clamped.y != position.y {
        window
            .set_position(PhysicalPosition::new(clamped.x, clamped.y))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn sync_windows(app: &AppHandle, snapshot: &AppSnapshot) -> Result<(), String> {
    let state = app.state::<RuntimeState>();
    let plan = current_window_plan(&state, snapshot.phase)?;
    let widget = ensure_widget(app)?;
    let main = app.get_webview_window("main");

    if plan.show_break {
        widget.hide().map_err(|error| error.to_string())?;
        if let Some(main) = main.as_ref() {
            main.hide().map_err(|error| error.to_string())?;
        }
        show_break_windows(app, main.as_ref())?;
        return Ok(());
    }

    close_extra_break_windows(app, 0);
    if plan.show_main {
        widget.hide().map_err(|error| error.to_string())?;
        if let Some(main) = main.as_ref() {
            main.unminimize().map_err(|error| error.to_string())?;
            main.show().map_err(|error| error.to_string())?;
            main.set_focus().map_err(|error| error.to_string())?;
        }
    } else if plan.show_widget {
        if let Some(main) = main.as_ref() {
            main.hide().map_err(|error| error.to_string())?;
        }
        clamp_widget_window(&widget)?;
        windows_window::present_widget_window(&widget)?;
    } else {
        widget.hide().map_err(|e| e.to_string())?;
        if let Some(main) = main.as_ref() {
            main.hide().map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn show_break_windows(app: &AppHandle, main: Option<&tauri::WebviewWindow>) -> Result<(), String> {
    let monitors = if let Some(main) = main {
        main.available_monitors()
            .map_err(|error| error.to_string())?
    } else {
        return Err("主窗口不存在，无法读取显示器".to_string());
    };
    let break_policy = break_window_policy();
    for (index, monitor) in monitors.iter().enumerate() {
        let label = format!("break-{index}");
        let window = if let Some(window) = app.get_webview_window(&label) {
            window
        } else {
            WebviewWindowBuilder::new(
                app,
                &label,
                WebviewUrl::App(format!("index.html?view=break&monitor={index}").into()),
            )
            .title("护眼休息")
            .decorations(false)
            .shadow(false)
            .resizable(false)
            .always_on_top(break_policy.always_on_top)
            .skip_taskbar(break_policy.skip_taskbar)
            .visible(false)
            .build()
            .map_err(|error| error.to_string())?
        };
        window
            .set_position(PhysicalPosition::new(
                monitor.position().x,
                monitor.position().y,
            ))
            .map_err(|error| error.to_string())?;
        window
            .set_size(PhysicalSize::new(
                monitor.size().width,
                monitor.size().height,
            ))
            .map_err(|error| error.to_string())?;
        windows_window::present_break_window(&window, index, monitor.name().cloned())?;
    }
    close_extra_break_windows(app, monitors.len());
    Ok(())
}

fn close_extra_break_windows(app: &AppHandle, keep: usize) {
    for (label, window) in app.webview_windows() {
        if let Some(index) = label
            .strip_prefix("break-")
            .and_then(|value| value.parse::<usize>().ok())
        {
            if index >= keep {
                if let Err(error) = windows_window::dismiss_break_window(&window) {
                    eprintln!("failed to dismiss {label}: {error}");
                }
                if let Err(error) = window.close() {
                    eprintln!("failed to close {label}: {error}");
                }
            }
        }
    }
}

fn configure_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "准备就绪", false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "打开主页面", true, None::<&str>)?;
    let action = MenuItem::with_id(app, "timer_action", "开始专注", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "停止当前循环", false, None::<&str>)?;
    let widget = MenuItem::with_id(app, "widget", "显示计时胶囊", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, Some("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(app, &[&status, &show, &action, &stop, &widget, &quit])?;

    let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/trayTemplate.png"))?;
    let builder = TrayIconBuilder::new()
        .tooltip("护眼助手")
        .icon(tray_icon)
        .icon_as_template(tray_icon_is_template())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                let _ = show_main_window(app.clone());
            }
            "timer_action" => {
                let phase = app
                    .state::<RuntimeState>()
                    .engine
                    .lock()
                    .map(|engine| engine.snapshot().phase)
                    .unwrap_or(TimerPhase::Idle);
                match phase {
                    TimerPhase::Idle => perform_action(app, TimerAction::Start),
                    TimerPhase::Working | TimerPhase::Paused => {
                        perform_action(app, TimerAction::TogglePause);
                    }
                    TimerPhase::Resting => {}
                }
            }
            "stop" => perform_action(app, TimerAction::Stop),
            "widget" => {
                let state = app.state::<RuntimeState>();
                if let Ok(mut engine) = state.engine.lock() {
                    let mut settings = engine.snapshot().settings;
                    settings.widget_visible = !settings.widget_visible;
                    engine.update_settings(settings, now_ms());
                    let snapshot = engine.snapshot();
                    drop(engine);
                    if let Err(error) = persist_snapshot_settings(&state, &snapshot) {
                        eprintln!("failed to save widget preference: {error}");
                    }
                    publish_snapshot(
                        app,
                        &snapshot,
                        snapshot.phase,
                        Some(DisplayEvent::SetWidgetVisibility {
                            visible: snapshot.settings.widget_visible,
                            phase: snapshot.phase,
                        }),
                    );
                };
            }
            "quit" => {
                system_events::shutdown();
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                let _ = show_main_window(tray.app_handle().clone());
            }
        });
    builder.build(app)?;
    if let Ok(mut tray_status) = app.state::<RuntimeState>().tray_status.lock() {
        *tray_status = Some(status);
    }
    if let Ok(mut tray_action) = app.state::<RuntimeState>().tray_action.lock() {
        *tray_action = Some(action);
    }
    if let Ok(mut tray_stop) = app.state::<RuntimeState>().tray_stop.lock() {
        *tray_stop = Some(stop);
    }
    if let Ok(mut tray_widget) = app.state::<RuntimeState>().tray_widget.lock() {
        *tray_widget = Some(widget);
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            handle_second_instance(app);
        }))
        .manage(RuntimeState::default())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .setup(|app| {
            let settings_path = app
                .path()
                .app_data_dir()
                .map_err(|error| error.to_string())?
                .join("settings.json");
            let widget_position_path = settings_path.with_file_name("widget-position.json");
            let (settings, recovery_error) = persistence::load_settings(&settings_path);
            let widget_position = persistence::load_widget_position(&widget_position_path);
            {
                let state = app.state::<RuntimeState>();
                if let Ok(mut path) = state.settings_path.lock() {
                    *path = Some(settings_path);
                }
                if let Ok(mut path) = state.widget_position_path.lock() {
                    *path = Some(widget_position_path);
                }
                if let Ok(mut position) = state.widget_position.lock() {
                    *position = widget_position;
                }
                if let Ok(mut engine) = state.engine.lock() {
                    *engine = TimerEngine::new(settings);
                    engine.set_recoverable_error(recovery_error);
                };
            }
            ensure_widget(app.handle()).map_err(std::io::Error::other)?;
            configure_tray(app)?;
            system_events::register(app.handle());
            let state = app.state::<RuntimeState>();
            if let Ok(engine) = state.engine.lock() {
                let snapshot = engine.snapshot();
                drop(engine);
                publish_snapshot(app.handle(), &snapshot, snapshot.phase, None);
                sync_windows(app.handle(), &snapshot).map_err(std::io::Error::other)?;
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    perform_action(&handle, TimerAction::Tick);
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } if window.label() == "main" => {
                api.prevent_close();
                let state = window.state::<RuntimeState>();
                let snapshot = state.engine.lock().map(|engine| engine.snapshot());
                if let Ok(snapshot) = snapshot {
                    let event = DisplayEvent::CloseMain {
                        phase: snapshot.phase,
                        widget_enabled: snapshot.settings.widget_visible,
                    };
                    if let Err(error) = apply_display_event(&state, event)
                        .and_then(|_| sync_windows(window.app_handle(), &snapshot))
                    {
                        eprintln!("failed to close main window cleanly: {error}");
                    }
                }
            }
            WindowEvent::Moved(position) if window.label() == "widget" => {
                let state = window.state::<RuntimeState>();
                let value = WidgetPosition {
                    x: position.x,
                    y: position.y,
                };
                if let Ok(mut stored) = state.widget_position.lock() {
                    *stored = Some(value);
                }
                if let Ok(path) = state.widget_position_path.lock() {
                    if let Some(path) = path.as_ref() {
                        if let Err(error) = persistence::save_widget_position(path, value) {
                            eprintln!("failed to save widget position: {error}");
                        }
                    }
                };
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_app_snapshot,
            dispatch_timer_action,
            update_settings,
            set_widget_visibility,
            show_main_window,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("failed to run eye-care assistant");
}
