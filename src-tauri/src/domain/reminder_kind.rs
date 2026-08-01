use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReminderKind {
    Water,
    Stand,
}

impl ReminderKind {
    pub const ALL: [Self; 2] = [Self::Water, Self::Stand];
}
