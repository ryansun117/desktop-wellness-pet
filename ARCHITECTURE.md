# Wellness Pet architecture

## System overview

Wellness Pet is one local Tauri process with a React webview. Rust owns durable application truth. The webview can disappear or reload without affecting reminder correctness.

## Responsibility split

React renders the pet, reminder bubble, settings form, and presentation-only celebration timer. It loads a complete Rust snapshot at startup, subscribes to a small event set, and invokes typed commands. It does not calculate due reminders.

Rust validates settings; advances reminders; handles quiet hours, pausing, missed time, and duplicate suppression; writes local state; runs the scheduler; and integrates with macOS windows, notifications, menu-bar controls, and autostart.

## Layers

### Domain

[`src-tauri/src/domain`](src-tauri/src/domain) has no Tauri dependency. `ReminderEngine` contains two explicit `ReminderState` values, settings, completion timestamps, and pause state. Operations receive timestamps from a `Clock`, making tests immediate and deterministic.

### Application

[`src-tauri/src/application/app_service.rs`](src-tauri/src/application/app_service.rs) is the single mutation boundary. `AppService` owns `Mutex<ReminderEngine>`, a clock, timezone, and persistence store. Commands in [`commands.rs`](src-tauri/src/application/commands.rs) call the same service methods as tray actions. [`events.rs`](src-tauri/src/application/events.rs) publishes snapshots and performs recoverable notification delivery.

### Infrastructure

[`src-tauri/src/infrastructure`](src-tauri/src/infrastructure) implements atomic JSON persistence, menu-bar state, and window position storage/restoration. Tauri-specific code does not leak into the domain.

## Command boundary

Typed commands expose snapshot/settings retrieval, setting updates, Done/Snooze/Skip, pause/resume, reset, show/hide, compact/expanded window modes, always-on-top, and launch-at-login. `AppError` serializes stable error codes and structured validation details.

## Event flow

The frontend registers listeners first and then requests `get_app_snapshot`, preventing a startup race. Rust emits:

- `reminder-due`
- `reminder-updated`
- `settings-updated`
- `pause-state-changed`

Payloads contain complete snapshots (the due event additionally identifies the kind), so the UI can replace rather than reconstruct state.

## Reminder state machine

```text
Disabled ──enable──► Scheduled
Scheduled ──due──► Due ──Done/Skip──► Scheduled
                         └─Snooze──► Snoozed ──due──► Due
any enabled state ──disable──► Disabled
```

Done records completion and schedules from completion time. Skip schedules normally without completion. Snooze schedules from the action time. Quiet hours replace an imminent due transition with one scheduled at quiet-hours end.

## Persistence

`PersistedAppState` includes a schema version, strongly typed settings, explicit reminder states, completion timestamps, duplicate markers, and pause state. Writes use a same-directory temporary file, flush it, and atomically rename it. Missing, malformed, invalid, or unsupported state falls back safely to defaults and logs a warning. Unknown JSON fields are ignored by Serde for forward compatibility.

## Scheduler and lifecycle

One scheduler task is created during application setup and never replaced when settings change. Every 15 seconds it evaluates absolute UTC timestamps. The same path runs when the pet window regains focus or Tauri's event loop resumes. A late timer therefore produces one current overdue occurrence per kind, never one item per missed interval.

## Duplicate prevention

Each `Due` state persists `eventEmitted` and `notificationEmitted`. Evaluation marks both before returning side effects and persists the transition before publication. Later polls return no actions for that occurrence. The frontend startup snapshot still reveals a due reminder if an event was missed.

## Locking

`AppService` uses one `Mutex` because all reminder mutations must be serialized. The guarded object is small. Persistence remains inside the critical section to preserve mutation/write order and prevent an older concurrent write from replacing newer state. Notification and event I/O occurs after the lock is released.

## Error handling

Expected failures become `AppError`, never raw panics at the command boundary. Notification failure is logged and the in-app reminder remains available. Corrupt persistence recovers to defaults. Window and autostart errors are returned to the UI.

## Future Windows and Linux support

The domain, application service, command payloads, and JSON persistence are platform-neutral. Port work is limited mainly to transparent-window behavior, tray conventions, autostart details, notification permission UX, and per-platform CI. `macOSPrivateApi` is isolated to Tauri configuration and can be removed or conditionally configured for other distributions.

