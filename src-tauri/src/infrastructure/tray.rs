use chrono::Local;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::{
    application::{events, AppRuntime},
    domain::{AppSnapshot, ReminderState},
};

pub struct TrayState {
    water: MenuItem<tauri::Wry>,
    stand: MenuItem<tauri::Wry>,
    pause: MenuItem<tauri::Wry>,
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Pet", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Pet", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Open Settings…", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause Reminders", true, None::<&str>)?;
    let water = MenuItem::with_id(app, "next-water", "Next water: —", false, None::<&str>)?;
    let stand = MenuItem::with_id(app, "next-stand", "Next stand: —", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Wellness Pet", true, None::<&str>)?;
    let separator_one = PredefinedMenuItem::separator(app)?;
    let separator_two = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &hide,
            &settings,
            &separator_one,
            &pause,
            &water,
            &stand,
            &separator_two,
            &quit,
        ],
    )?;
    app.manage(TrayState {
        water,
        stand,
        pause,
    });

    TrayIconBuilder::with_id("wellness-pet")
        .menu(&menu)
        .tooltip("Wellness Pet")
        .icon_as_template(true)
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            // The generated bundle always includes an icon; this fallback is only for unusual tests.
            tauri::image::Image::new(&[0, 0, 0, 0], 1, 1)
        }))
        .on_menu_event(handle_menu)
        .build(app)?;

    if let Ok(snapshot) = app.state::<AppRuntime>().service.snapshot() {
        refresh(app, &snapshot);
    }
    Ok(())
}

pub fn refresh(app: &AppHandle, snapshot: &AppSnapshot) {
    let Some(state) = app.try_state::<TrayState>() else {
        return;
    };
    let _ = state
        .water
        .set_text(format!("Next water: {}", describe(&snapshot.water.state)));
    let _ = state
        .stand
        .set_text(format!("Next stand: {}", describe(&snapshot.stand.state)));
    let _ = state.pause.set_text(if snapshot.paused {
        "Resume Reminders"
    } else {
        "Pause Reminders"
    });
}

fn describe(state: &ReminderState) -> String {
    match state {
        ReminderState::Disabled => "disabled".into(),
        ReminderState::Due { .. } => "due now".into(),
        ReminderState::Scheduled { due_at } => due_at
            .with_timezone(&Local)
            .format("%a %-I:%M %p")
            .to_string(),
        ReminderState::Snoozed { until } => format!(
            "snoozed until {}",
            until.with_timezone(&Local).format("%-I:%M %p")
        ),
    }
}

fn handle_menu(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();
    let window = app.get_webview_window("main");
    match id {
        "show" => {
            if let Some(window) = window {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = window {
                let _ = window.hide();
            }
        }
        "settings" => {
            if let Some(window) = window {
                let _ = window.set_resizable(true);
                let _ = window.set_size(tauri::LogicalSize::new(420.0, 620.0));
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "pause" => {
            let runtime = app.state::<AppRuntime>();
            let result = runtime.service.snapshot().and_then(|snapshot| {
                if snapshot.paused {
                    runtime.service.resume()
                } else {
                    runtime.service.pause(None)
                }
            });
            match result {
                Ok(snapshot) => events::publish_snapshot(app, "pause-state-changed", &snapshot),
                Err(error) => tracing::warn!("Tray pause action failed: {error}"),
            }
        }
        "quit" => app.exit(0),
        _ => {}
    }
}
