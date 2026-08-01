mod app_service;
mod commands;
pub(crate) mod events;

use std::{sync::Arc, time::Duration};

use chrono_tz::Tz;
use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::ManagerExt as _;

use crate::{
    domain::{Clock, SystemClock},
    infrastructure::{persistence::PersistenceStore, tray, window_state::WindowStateStore},
};

pub use app_service::AppService;

pub struct AppRuntime {
    pub service: Arc<AppService>,
    pub window_state: WindowStateStore,
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_snapshot,
            commands::get_settings,
            commands::update_settings,
            commands::complete_reminder,
            commands::snooze_reminder,
            commands::skip_reminder,
            commands::pause_reminders,
            commands::resume_reminders,
            commands::reset_settings,
            commands::show_pet,
            commands::hide_pet,
            commands::open_settings,
            commands::close_settings,
            commands::set_always_on_top,
            commands::set_launch_at_login,
        ])
        .setup(|app| {
            tracing_subscriber::fmt()
                .with_target(false)
                .compact()
                .init();
            let app_data = app.path().app_data_dir()?;
            let clock: Arc<dyn Clock> = Arc::new(SystemClock);
            let store = PersistenceStore::new(app_data.join("state.json"));
            let (engine, warning) = store.load_engine(clock.now());
            if let Some(warning) = warning {
                tracing::warn!("Recovering with default local state: {warning}");
            }
            let timezone = local_timezone();
            let service = Arc::new(AppService::new(engine, store, clock, timezone));
            let runtime = AppRuntime {
                service: Arc::clone(&service),
                window_state: WindowStateStore::new(app_data.join("window-state.json")),
            };
            app.manage(runtime);

            if let Some(window) = app.get_webview_window("main") {
                let snapshot = service.snapshot()?;
                window.set_always_on_top(snapshot.settings.always_on_top)?;
                if let Err(error) =
                    WindowStateStore::restore(&window, &app_data.join("window-state.json"))
                {
                    tracing::warn!("Could not restore window position: {error}");
                }
            }
            sync_autostart(app.handle(), service.snapshot()?.settings.launch_at_login);
            tray::build(app.handle())?;
            start_scheduler(app.handle().clone(), service);
            Ok(())
        })
        .build(tauri::generate_context!());

    match app {
        Ok(app) => app.run(handle_run_event),
        Err(error) => eprintln!("Wellness Pet failed to start: {error}"),
    }
}

fn start_scheduler(app: tauri::AppHandle, service: Arc<AppService>) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            events::evaluate_and_publish(&app, &service);
        }
    });
}

fn handle_run_event(app: &tauri::AppHandle, event: RunEvent) {
    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } if label == "main" => {
            api.prevent_close();
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::Moved(position),
            ..
        } if label == "main" => {
            let runtime = app.state::<AppRuntime>();
            if let Err(error) = runtime.window_state.save(position) {
                tracing::warn!("Could not save window position: {error}");
            }
        }
        RunEvent::WindowEvent {
            event: WindowEvent::Focused(true),
            ..
        }
        | RunEvent::Resumed => {
            let runtime = app.state::<AppRuntime>();
            events::evaluate_and_publish(app, &runtime.service);
        }
        RunEvent::Reopen { .. } => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        _ => {}
    }
}

fn local_timezone() -> Tz {
    iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse().ok())
        .unwrap_or_else(|| {
            tracing::warn!("Could not detect the local timezone; using UTC");
            chrono_tz::UTC
        })
}

fn sync_autostart(app: &tauri::AppHandle, enabled: bool) {
    let result = if enabled {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    };
    if let Err(error) = result {
        tracing::warn!("Could not synchronize launch-at-login: {error}");
    }
}
