use chrono::{DateTime, Utc};
use tauri::{AppHandle, LogicalSize, Manager, State};
use tauri_plugin_autostart::ManagerExt as _;

use crate::{
    domain::{AppSettings, AppSnapshot, ReminderKind},
    error::AppError,
};

use super::{events, AppRuntime};

#[tauri::command]
pub fn get_app_snapshot(state: State<'_, AppRuntime>) -> Result<AppSnapshot, AppError> {
    state.service.snapshot()
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppRuntime>) -> Result<AppSettings, AppError> {
    Ok(state.service.snapshot()?.settings)
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppRuntime>,
    settings: AppSettings,
) -> Result<AppSnapshot, AppError> {
    apply_settings(&app, &state, settings)
}

#[tauri::command]
pub fn complete_reminder(
    app: AppHandle,
    state: State<'_, AppRuntime>,
    kind: ReminderKind,
) -> Result<AppSnapshot, AppError> {
    let snapshot = state.service.complete(kind)?;
    events::publish_snapshot(&app, "reminder-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn snooze_reminder(
    app: AppHandle,
    state: State<'_, AppRuntime>,
    kind: ReminderKind,
) -> Result<AppSnapshot, AppError> {
    let snapshot = state.service.snooze(kind)?;
    events::publish_snapshot(&app, "reminder-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn skip_reminder(
    app: AppHandle,
    state: State<'_, AppRuntime>,
    kind: ReminderKind,
) -> Result<AppSnapshot, AppError> {
    let snapshot = state.service.skip(kind)?;
    events::publish_snapshot(&app, "reminder-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn pause_reminders(
    app: AppHandle,
    state: State<'_, AppRuntime>,
    until: Option<DateTime<Utc>>,
) -> Result<AppSnapshot, AppError> {
    let snapshot = state.service.pause(until)?;
    events::publish_snapshot(&app, "pause-state-changed", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn resume_reminders(
    app: AppHandle,
    state: State<'_, AppRuntime>,
) -> Result<AppSnapshot, AppError> {
    let snapshot = state.service.resume()?;
    events::publish_snapshot(&app, "pause-state-changed", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn reset_settings(
    app: AppHandle,
    state: State<'_, AppRuntime>,
) -> Result<AppSnapshot, AppError> {
    let current = state.service.snapshot()?;
    let defaults = AppSettings::default();
    sync_native_settings(&app, &current.settings, &defaults)?;
    let snapshot = state.service.reset()?;
    events::publish_snapshot(&app, "settings-updated", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn show_pet(app: AppHandle) -> Result<(), AppError> {
    let window = main_window(&app)?;
    window.show().map_err(window_error)?;
    window.set_focus().map_err(window_error)
}

#[tauri::command]
pub fn hide_pet(app: AppHandle) -> Result<(), AppError> {
    main_window(&app)?.hide().map_err(window_error)
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), AppError> {
    let window = main_window(&app)?;
    window.set_resizable(true).map_err(window_error)?;
    window
        .set_size(LogicalSize::new(420.0, 620.0))
        .map_err(window_error)?;
    window.show().map_err(window_error)?;
    window.set_focus().map_err(window_error)
}

#[tauri::command]
pub fn close_settings(app: AppHandle) -> Result<(), AppError> {
    let window = main_window(&app)?;
    window
        .set_size(LogicalSize::new(240.0, 260.0))
        .map_err(window_error)?;
    window.set_resizable(false).map_err(window_error)
}

#[tauri::command]
pub fn set_always_on_top(
    app: AppHandle,
    state: State<'_, AppRuntime>,
    enabled: bool,
) -> Result<AppSnapshot, AppError> {
    let mut settings = state.service.snapshot()?.settings;
    settings.always_on_top = enabled;
    apply_settings(&app, &state, settings)
}

#[tauri::command]
pub fn set_launch_at_login(
    app: AppHandle,
    state: State<'_, AppRuntime>,
    enabled: bool,
) -> Result<AppSnapshot, AppError> {
    let mut settings = state.service.snapshot()?.settings;
    settings.launch_at_login = enabled;
    apply_settings(&app, &state, settings)
}

fn apply_settings(
    app: &AppHandle,
    state: &State<'_, AppRuntime>,
    settings: AppSettings,
) -> Result<AppSnapshot, AppError> {
    super::AppService::validate_settings(&settings)?;
    let current = state.service.snapshot()?;
    sync_native_settings(app, &current.settings, &settings)?;
    let snapshot = state.service.update_settings(settings)?;
    events::publish_snapshot(app, "settings-updated", &snapshot);
    Ok(snapshot)
}

fn sync_native_settings(
    app: &AppHandle,
    old: &AppSettings,
    new: &AppSettings,
) -> Result<(), AppError> {
    if old.launch_at_login != new.launch_at_login {
        let result = if new.launch_at_login {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        };
        result.map_err(|error| AppError::Autostart(error.to_string()))?;
    }
    if old.always_on_top != new.always_on_top {
        main_window(app)?
            .set_always_on_top(new.always_on_top)
            .map_err(window_error)?;
    }
    Ok(())
}

fn main_window(app: &AppHandle) -> Result<tauri::WebviewWindow, AppError> {
    app.get_webview_window("main")
        .ok_or_else(|| AppError::Window("the pet window is unavailable".into()))
}

fn window_error(error: tauri::Error) -> AppError {
    AppError::Window(error.to_string())
}
