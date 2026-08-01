# Rust learning notes

These notes point to the concepts as they are used in Wellness Pet rather than presenting a separate tutorial.

## Ownership and borrowing

[`scheduler.rs`](src-tauri/src/domain/scheduler.rs) owns `AppSettings` and `PersistedReminderState`. Methods borrow `&self` for snapshots and `&mut self` for transitions. `slot_mut` returns a focused mutable borrow so state changes cannot accidentally mutate both reminder kinds.

## Enums and pattern matching

[`reminder.rs`](src-tauri/src/domain/reminder.rs) models disabled, scheduled, due, and snoozed as an enum. Invalid combinations are harder to represent than they would be with several booleans. `match` expressions select timestamps and behavior for each variant and for `ReminderKind`.

## `Result` and error types

Settings validation returns `Result<(), Vec<ValidationIssue>>`. Infrastructure and commands return [`AppError`](src-tauri/src/error.rs), a `thiserror` enum that also serializes for React. Recoverable failures travel through `?` instead of `unwrap` in application paths.

## Serde

Settings, reminder states, persisted state, snapshots, and errors derive Serde traits. Camel-case wire names keep TypeScript idiomatic while Rust fields remain snake case. Tagged enum serialization gives the frontend a safe `status` discriminator.

## Clock injection

[`time.rs`](src-tauri/src/domain/time.rs) defines `Clock`, `SystemClock`, and `FixedClock`. `AppService` receives `Arc<dyn Clock>`, while pure domain operations take explicit timestamps. Tests never wait for real time.

## `Arc` and `Mutex`

[`app_service.rs`](src-tauri/src/application/app_service.rs) wraps the engine in `Mutex` to serialize transitions. The service is held in `Arc` so the scheduler, commands, and lifecycle handlers share ownership safely. Notification and event work happens after evaluation releases the guard.

## Async tasks

[`application/mod.rs`](src-tauri/src/application/mod.rs) starts one Tauri async task containing a Tokio interval. The interval is only a wake-up mechanism; UTC timestamps remain the source of truth.

## Tauri commands

[`commands.rs`](src-tauri/src/application/commands.rs) shows `#[tauri::command]` functions accepting typed input and managed state. Commands delegate domain behavior to `AppService`, then return a complete `AppSnapshot`.

## Tauri events

[`events.rs`](src-tauri/src/application/events.rs) emits the small public event set. [`useAppSnapshot.ts`](src/hooks/useAppSnapshot.ts) registers listeners before fetching the startup snapshot and cleans all listeners up on unmount.

