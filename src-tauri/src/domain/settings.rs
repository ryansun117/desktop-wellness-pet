use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub water_enabled: bool,
    pub water_interval_minutes: u32,
    pub stand_enabled: bool,
    pub stand_interval_minutes: u32,
    pub snooze_duration_minutes: u32,
    pub quiet_hours_enabled: bool,
    pub quiet_hours_start: String,
    pub quiet_hours_end: String,
    pub always_on_top: bool,
    pub launch_at_login: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            water_enabled: true,
            water_interval_minutes: 45,
            stand_enabled: true,
            stand_interval_minutes: 60,
            snooze_duration_minutes: 10,
            quiet_hours_enabled: true,
            quiet_hours_start: "22:00".into(),
            quiet_hours_end: "08:00".into(),
            always_on_top: true,
            launch_at_login: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub field: String,
    pub message: String,
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), Vec<ValidationIssue>> {
        let mut issues = Vec::new();
        validate_minutes(
            "waterIntervalMinutes",
            self.water_interval_minutes,
            1,
            24 * 60,
            &mut issues,
        );
        validate_minutes(
            "standIntervalMinutes",
            self.stand_interval_minutes,
            1,
            24 * 60,
            &mut issues,
        );
        validate_minutes(
            "snoozeDurationMinutes",
            self.snooze_duration_minutes,
            1,
            8 * 60,
            &mut issues,
        );

        let start = chrono::NaiveTime::parse_from_str(&self.quiet_hours_start, "%H:%M");
        let end = chrono::NaiveTime::parse_from_str(&self.quiet_hours_end, "%H:%M");
        if start.is_err() {
            issues.push(ValidationIssue {
                field: "quietHoursStart".into(),
                message: "Use a valid 24-hour time such as 22:00.".into(),
            });
        }
        if end.is_err() {
            issues.push(ValidationIssue {
                field: "quietHoursEnd".into(),
                message: "Use a valid 24-hour time such as 08:00.".into(),
            });
        }
        if let (Ok(start), Ok(end)) = (start, end) {
            if start == end {
                issues.push(ValidationIssue {
                    field: "quietHoursEnd".into(),
                    message: "Quiet-hours start and end must differ.".into(),
                });
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

fn validate_minutes(
    field: &str,
    value: u32,
    minimum: u32,
    maximum: u32,
    issues: &mut Vec<ValidationIssue>,
) {
    if !(minimum..=maximum).contains(&value) {
        issues.push(ValidationIssue {
            field: field.into(),
            message: format!("Enter a value from {minimum} to {maximum} minutes."),
        });
    }
}
