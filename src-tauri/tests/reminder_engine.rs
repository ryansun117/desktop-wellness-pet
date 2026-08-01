use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::{America::Los_Angeles, UTC};
use wellness_pet_lib::domain::{
    quiet_hours_end, AppSettings, Clock, FixedClock, ReminderEngine, ReminderKind, ReminderState,
};

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 1, hour, minute, 0)
        .single()
        .unwrap()
}

fn engine(now: DateTime<Utc>) -> ReminderEngine {
    ReminderEngine::new(now, AppSettings::default()).unwrap()
}

fn due_at(state: &ReminderState) -> DateTime<Utc> {
    state.due_at().unwrap()
}

#[test]
fn first_launch_schedules_water_and_standing() {
    let now = at(12, 0);
    let engine = engine(now);
    let snapshot = engine.snapshot(now);
    assert_eq!(due_at(&snapshot.water.state), now + Duration::minutes(45));
    assert_eq!(due_at(&snapshot.stand.state), now + Duration::minutes(60));
}

#[test]
fn reminder_not_yet_due_produces_no_action() {
    let now = at(12, 0);
    assert!(engine(now)
        .evaluate(now + Duration::minutes(44), UTC)
        .is_empty());
}

#[test]
fn water_and_stand_become_due() {
    let now = at(12, 0);
    let mut engine = engine(now);
    let water = engine.evaluate(now + Duration::minutes(45), UTC);
    assert_eq!(water.len(), 1);
    assert_eq!(water[0].kind, ReminderKind::Water);

    let stand = engine.evaluate(now + Duration::minutes(60), UTC);
    assert_eq!(stand.len(), 1);
    assert_eq!(stand[0].kind, ReminderKind::Stand);
}

#[test]
fn completing_records_completion_and_schedules_from_completion_time() {
    let now = at(12, 0);
    let completed = now + Duration::minutes(50);
    let mut engine = engine(now);
    engine.complete(ReminderKind::Water, completed);
    let snapshot = engine.snapshot(completed);
    assert_eq!(snapshot.water.last_completed_at, Some(completed));
    assert_eq!(
        due_at(&snapshot.water.state),
        completed + Duration::minutes(45)
    );
}

#[test]
fn snooze_uses_configured_duration() {
    let now = at(12, 0);
    let mut engine = engine(now);
    engine.snooze(ReminderKind::Water, now);
    assert_eq!(
        due_at(&engine.snapshot(now).water.state),
        now + Duration::minutes(10)
    );
}

#[test]
fn skip_does_not_record_completion() {
    let now = at(12, 0);
    let mut engine = engine(now);
    engine.skip(ReminderKind::Stand, now);
    let stand = engine.snapshot(now).stand;
    assert_eq!(stand.last_completed_at, None);
    assert_eq!(due_at(&stand.state), now + Duration::minutes(60));
}

#[test]
fn overnight_quiet_hours_end_next_morning() {
    let settings = AppSettings::default();
    let now = Los_Angeles
        .with_ymd_and_hms(2026, 8, 1, 23, 0, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let end = quiet_hours_end(now, &settings, Los_Angeles).unwrap();
    let expected = Los_Angeles
        .with_ymd_and_hms(2026, 8, 2, 8, 0, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc);
    assert_eq!(end, expected);
}

#[test]
fn daytime_is_outside_overnight_quiet_hours() {
    let settings = AppSettings::default();
    let now = Los_Angeles
        .with_ymd_and_hms(2026, 8, 1, 12, 0, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc);
    assert_eq!(quiet_hours_end(now, &settings, Los_Angeles), None);
}

#[test]
fn due_during_quiet_hours_is_postponed() {
    let start = Los_Angeles
        .with_ymd_and_hms(2026, 8, 1, 21, 0, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let mut engine = engine(start);
    let evaluation = start + Duration::minutes(90);
    assert!(engine.evaluate(evaluation, Los_Angeles).is_empty());
    let expected = Los_Angeles
        .with_ymd_and_hms(2026, 8, 2, 8, 0, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc);
    assert_eq!(due_at(&engine.snapshot(evaluation).water.state), expected);
}

#[test]
fn long_sleep_creates_one_occurrence_without_a_backlog() {
    let now = at(12, 0);
    let mut engine = engine(now);
    let actions = engine.evaluate(now + Duration::days(3), UTC);
    assert_eq!(actions.len(), 2);
    assert!(engine.evaluate(now + Duration::days(3), UTC).is_empty());
}

#[test]
fn duplicate_events_and_notifications_are_prevented() {
    let now = at(12, 0);
    let mut engine = engine(now);
    let first = engine.evaluate(now + Duration::minutes(61), UTC);
    assert!(first.iter().all(|action| action.emit_due_event));
    assert!(first.iter().all(|action| action.send_notification));
    assert!(engine.evaluate(now + Duration::minutes(62), UTC).is_empty());
}

#[test]
fn disabled_reminders_do_not_fire() {
    let now = at(12, 0);
    let settings = AppSettings {
        water_enabled: false,
        stand_enabled: false,
        ..AppSettings::default()
    };
    let mut engine = ReminderEngine::new(now, settings).unwrap();
    assert!(matches!(
        engine.snapshot(now).water.state,
        ReminderState::Disabled
    ));
    assert!(engine.evaluate(now + Duration::days(1), UTC).is_empty());
}

#[test]
fn both_overdue_are_preserved_and_presented_sequentially() {
    let now = at(12, 0);
    let overdue = now + Duration::minutes(61);
    let mut engine = engine(now);
    engine.evaluate(overdue, UTC);
    assert_eq!(
        engine.snapshot(overdue).current_reminder,
        Some(ReminderKind::Water)
    );
    engine.complete(ReminderKind::Water, overdue);
    assert_eq!(
        engine.snapshot(overdue).current_reminder,
        Some(ReminderKind::Stand)
    );
}

#[test]
fn interval_changes_rebase_the_next_due_time() {
    let now = at(12, 0);
    let changed_at = now + Duration::minutes(10);
    let mut engine = engine(now);
    let settings = AppSettings {
        water_interval_minutes: 20,
        ..AppSettings::default()
    };
    engine.update_settings(settings, changed_at).unwrap();
    assert_eq!(
        due_at(&engine.snapshot(changed_at).water.state),
        changed_at + Duration::minutes(20)
    );
}

#[test]
fn enabling_and_disabling_updates_state() {
    let now = at(12, 0);
    let mut engine = engine(now);
    let disabled = AppSettings {
        water_enabled: false,
        ..AppSettings::default()
    };
    engine.update_settings(disabled, now).unwrap();
    assert!(matches!(
        engine.snapshot(now).water.state,
        ReminderState::Disabled
    ));
    engine.update_settings(AppSettings::default(), now).unwrap();
    assert_eq!(
        due_at(&engine.snapshot(now).water.state),
        now + Duration::minutes(45)
    );
}

#[test]
fn indefinite_pause_and_resume_suppress_then_reveal_due_items() {
    let now = at(12, 0);
    let late = now + Duration::minutes(61);
    let mut engine = engine(now);
    engine.pause(None);
    assert!(engine.evaluate(late, UTC).is_empty());
    assert!(engine.snapshot(late).paused);
    engine.resume();
    assert_eq!(engine.evaluate(late, UTC).len(), 2);
}

#[test]
fn timed_pause_expires_automatically() {
    let now = at(12, 0);
    let mut engine = engine(now);
    engine.pause(Some(now + Duration::minutes(30)));
    assert!(engine.snapshot(now).paused);
    engine.evaluate(now + Duration::minutes(31), UTC);
    assert!(!engine.snapshot(now + Duration::minutes(31)).paused);
}

#[test]
fn invalid_settings_are_rejected_with_fields() {
    let settings = AppSettings {
        water_interval_minutes: 0,
        quiet_hours_start: "bad".into(),
        ..AppSettings::default()
    };
    let issues = settings.validate().unwrap_err();
    assert!(issues
        .iter()
        .any(|issue| issue.field == "waterIntervalMinutes"));
    assert!(issues.iter().any(|issue| issue.field == "quietHoursStart"));
}

#[test]
fn spring_forward_nonexistent_quiet_end_moves_to_first_valid_minute() {
    let settings = AppSettings {
        quiet_hours_start: "01:00".into(),
        quiet_hours_end: "02:30".into(),
        ..AppSettings::default()
    };
    let now = Los_Angeles
        .with_ymd_and_hms(2026, 3, 8, 1, 30, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let end = quiet_hours_end(now, &settings, Los_Angeles).unwrap();
    assert_eq!(end.with_timezone(&Los_Angeles).hour(), 3);
}

#[test]
fn fixed_clock_is_deterministic() {
    let expected = at(12, 0);
    assert_eq!(FixedClock::new(expected).now(), expected);
}

use chrono::Timelike;
