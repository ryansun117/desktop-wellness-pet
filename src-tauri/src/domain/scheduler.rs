use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use super::{quiet_hours_end, AppSettings, ReminderKind, ReminderSnapshot, ReminderState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedReminderState {
    pub water: ReminderState,
    pub stand: ReminderState,
    pub last_water_completed_at: Option<DateTime<Utc>>,
    pub last_stand_completed_at: Option<DateTime<Utc>>,
    pub paused_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub paused_indefinitely: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub settings: AppSettings,
    pub water: ReminderSnapshot,
    pub stand: ReminderSnapshot,
    pub current_reminder: Option<ReminderKind>,
    pub paused: bool,
    pub paused_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvaluationAction {
    pub kind: ReminderKind,
    pub emit_due_event: bool,
    pub send_notification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderEngine {
    settings: AppSettings,
    state: PersistedReminderState,
}

impl ReminderEngine {
    pub fn new(
        now: DateTime<Utc>,
        settings: AppSettings,
    ) -> Result<Self, Vec<super::ValidationIssue>> {
        settings.validate()?;
        let state = PersistedReminderState {
            water: initial_state(now, settings.water_enabled, settings.water_interval_minutes),
            stand: initial_state(now, settings.stand_enabled, settings.stand_interval_minutes),
            last_water_completed_at: None,
            last_stand_completed_at: None,
            paused_until: None,
            paused_indefinitely: false,
        };
        Ok(Self { settings, state })
    }

    pub fn restore(
        settings: AppSettings,
        state: PersistedReminderState,
    ) -> Result<Self, Vec<super::ValidationIssue>> {
        settings.validate()?;
        Ok(Self { settings, state })
    }

    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn persisted_state(&self) -> &PersistedReminderState {
        &self.state
    }

    pub fn snapshot(&self, now: DateTime<Utc>) -> AppSnapshot {
        AppSnapshot {
            settings: self.settings.clone(),
            water: ReminderSnapshot {
                state: self.state.water.clone(),
                last_completed_at: self.state.last_water_completed_at,
            },
            stand: ReminderSnapshot {
                state: self.state.stand.clone(),
                last_completed_at: self.state.last_stand_completed_at,
            },
            current_reminder: self.current_reminder(),
            paused: self.is_paused(now),
            paused_until: self.state.paused_until,
        }
    }

    pub fn evaluate(&mut self, now: DateTime<Utc>, timezone: Tz) -> Vec<EvaluationAction> {
        if self.state.paused_until.is_some_and(|until| now >= until) {
            self.state.paused_until = None;
        }
        if self.is_paused(now) {
            return Vec::new();
        }

        let quiet_end = quiet_hours_end(now, &self.settings, timezone);
        let mut actions = Vec::new();
        for kind in ReminderKind::ALL {
            let slot = self.slot_mut(kind);
            let due_at = match slot {
                ReminderState::Scheduled { due_at } => Some(*due_at),
                ReminderState::Snoozed { until } => Some(*until),
                _ => None,
            };
            if let Some(due_at) = due_at.filter(|due| now >= *due) {
                if let Some(until) = quiet_end {
                    *slot = ReminderState::Scheduled { due_at: until };
                } else {
                    *slot = ReminderState::Due {
                        due_since: due_at,
                        event_emitted: false,
                        notification_emitted: false,
                    };
                }
            }

            if let ReminderState::Due {
                event_emitted,
                notification_emitted,
                ..
            } = slot
            {
                let action = EvaluationAction {
                    kind,
                    emit_due_event: !*event_emitted,
                    send_notification: !*notification_emitted,
                };
                *event_emitted = true;
                *notification_emitted = true;
                if action.emit_due_event || action.send_notification {
                    actions.push(action);
                }
            }
        }
        actions
    }

    pub fn complete(&mut self, kind: ReminderKind, now: DateTime<Utc>) {
        *self.last_completed_mut(kind) = Some(now);
        self.schedule_normal(kind, now);
    }

    pub fn snooze(&mut self, kind: ReminderKind, now: DateTime<Utc>) {
        let until = now + Duration::minutes(i64::from(self.settings.snooze_duration_minutes));
        *self.slot_mut(kind) = ReminderState::Snoozed { until };
    }

    pub fn skip(&mut self, kind: ReminderKind, now: DateTime<Utc>) {
        self.schedule_normal(kind, now);
    }

    pub fn pause(&mut self, until: Option<DateTime<Utc>>) {
        self.state.paused_until = until;
        self.state.paused_indefinitely = until.is_none();
    }

    pub fn resume(&mut self) {
        self.state.paused_until = None;
        self.state.paused_indefinitely = false;
    }

    pub fn update_settings(
        &mut self,
        settings: AppSettings,
        now: DateTime<Utc>,
    ) -> Result<(), Vec<super::ValidationIssue>> {
        settings.validate()?;
        update_kind(
            &mut self.state.water,
            self.settings.water_enabled,
            settings.water_enabled,
            self.settings.water_interval_minutes,
            settings.water_interval_minutes,
            now,
        );
        update_kind(
            &mut self.state.stand,
            self.settings.stand_enabled,
            settings.stand_enabled,
            self.settings.stand_interval_minutes,
            settings.stand_interval_minutes,
            now,
        );
        self.settings = settings;
        Ok(())
    }

    pub fn reset(&mut self, now: DateTime<Utc>) {
        *self = Self::new(now, AppSettings::default()).expect("default settings are valid");
    }

    fn current_reminder(&self) -> Option<ReminderKind> {
        let water = due_since(&self.state.water).map(|date| (date, ReminderKind::Water));
        let stand = due_since(&self.state.stand).map(|date| (date, ReminderKind::Stand));
        match (water, stand) {
            (Some(water), Some(stand)) => Some(if water.0 <= stand.0 { water.1 } else { stand.1 }),
            (Some((_, kind)), None) | (None, Some((_, kind))) => Some(kind),
            (None, None) => None,
        }
    }

    fn is_paused(&self, now: DateTime<Utc>) -> bool {
        self.state.paused_indefinitely || self.state.paused_until.is_some_and(|until| now < until)
    }

    fn schedule_normal(&mut self, kind: ReminderKind, now: DateTime<Utc>) {
        let (enabled, minutes) = match kind {
            ReminderKind::Water => (
                self.settings.water_enabled,
                self.settings.water_interval_minutes,
            ),
            ReminderKind::Stand => (
                self.settings.stand_enabled,
                self.settings.stand_interval_minutes,
            ),
        };
        *self.slot_mut(kind) = initial_state(now, enabled, minutes);
    }

    fn slot_mut(&mut self, kind: ReminderKind) -> &mut ReminderState {
        match kind {
            ReminderKind::Water => &mut self.state.water,
            ReminderKind::Stand => &mut self.state.stand,
        }
    }

    fn last_completed_mut(&mut self, kind: ReminderKind) -> &mut Option<DateTime<Utc>> {
        match kind {
            ReminderKind::Water => &mut self.state.last_water_completed_at,
            ReminderKind::Stand => &mut self.state.last_stand_completed_at,
        }
    }
}

fn initial_state(now: DateTime<Utc>, enabled: bool, interval_minutes: u32) -> ReminderState {
    if enabled {
        ReminderState::Scheduled {
            due_at: now + Duration::minutes(i64::from(interval_minutes)),
        }
    } else {
        ReminderState::Disabled
    }
}

fn due_since(state: &ReminderState) -> Option<DateTime<Utc>> {
    match state {
        ReminderState::Due { due_since, .. } => Some(*due_since),
        _ => None,
    }
}

fn update_kind(
    state: &mut ReminderState,
    was_enabled: bool,
    is_enabled: bool,
    old_interval: u32,
    new_interval: u32,
    now: DateTime<Utc>,
) {
    if !is_enabled {
        *state = ReminderState::Disabled;
    } else if !was_enabled || old_interval != new_interval {
        *state = initial_state(now, true, new_interval);
    }
}
