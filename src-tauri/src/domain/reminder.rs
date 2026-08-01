use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ReminderState {
    Disabled,
    Scheduled {
        due_at: DateTime<Utc>,
    },
    Due {
        due_since: DateTime<Utc>,
        event_emitted: bool,
        notification_emitted: bool,
    },
    Snoozed {
        until: DateTime<Utc>,
    },
}

impl ReminderState {
    pub fn due_at(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Scheduled { due_at } => Some(*due_at),
            Self::Snoozed { until } => Some(*until),
            Self::Due { due_since, .. } => Some(*due_since),
            Self::Disabled => None,
        }
    }

    pub fn is_due(&self) -> bool {
        matches!(self, Self::Due { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderSnapshot {
    pub state: ReminderState,
    pub last_completed_at: Option<DateTime<Utc>>,
}
