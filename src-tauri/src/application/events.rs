use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt as _;

use crate::{
    domain::{AppSnapshot, ReminderKind},
    infrastructure::tray,
};

use super::AppService;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReminderDueEvent {
    kind: ReminderKind,
    snapshot: AppSnapshot,
}

pub fn evaluate_and_publish(app: &AppHandle, service: &Arc<AppService>) {
    match service.evaluate() {
        Ok((snapshot, actions)) => {
            for action in actions {
                if action.emit_due_event {
                    let payload = ReminderDueEvent {
                        kind: action.kind,
                        snapshot: snapshot.clone(),
                    };
                    if let Err(error) = app.emit("reminder-due", payload) {
                        tracing::warn!("Could not emit reminder event: {error}");
                    }
                }
                if action.send_notification {
                    let body = match action.kind {
                        ReminderKind::Water => "Time for some water.",
                        ReminderKind::Stand => "Time to stand and stretch.",
                    };
                    if let Err(error) = app
                        .notification()
                        .builder()
                        .title("Wellness Pet")
                        .body(body)
                        .show()
                    {
                        tracing::warn!("Native notification unavailable: {error}");
                    }
                }
            }
            tray::refresh(app, &snapshot);
        }
        Err(error) => tracing::error!("Reminder evaluation failed: {error}"),
    }
}

pub fn publish_snapshot(app: &AppHandle, event: &str, snapshot: &AppSnapshot) {
    if let Err(error) = app.emit(event, snapshot) {
        tracing::warn!("Could not emit {event}: {error}");
    }
    tray::refresh(app, snapshot);
}
