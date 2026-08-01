use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{AppSettings, PersistedReminderState, ReminderEngine},
    error::AppError,
};

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedAppState {
    pub schema_version: u32,
    pub settings: AppSettings,
    pub reminder_state: PersistedReminderState,
}

#[derive(Debug, Clone)]
pub struct PersistenceStore {
    path: PathBuf,
}

impl PersistenceStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_engine(&self, now: DateTime<Utc>) -> (ReminderEngine, Option<AppError>) {
        match self.load() {
            Ok(Some(persisted)) if persisted.schema_version == SCHEMA_VERSION => {
                match ReminderEngine::restore(persisted.settings, persisted.reminder_state) {
                    Ok(engine) => (engine, None),
                    Err(issues) => (default_engine(now), Some(AppError::InvalidSettings(issues))),
                }
            }
            Ok(Some(persisted)) => (
                default_engine(now),
                Some(AppError::Persistence(format!(
                    "unsupported state schema version {}",
                    persisted.schema_version
                ))),
            ),
            Ok(None) => (default_engine(now), None),
            Err(error) => (default_engine(now), Some(error)),
        }
    }

    pub fn save_engine(&self, engine: &ReminderEngine) -> Result<(), AppError> {
        let state = PersistedAppState {
            schema_version: SCHEMA_VERSION,
            settings: engine.settings().clone(),
            reminder_state: engine.persisted_state().clone(),
        };
        let bytes = serde_json::to_vec_pretty(&state)
            .map_err(|error| AppError::Persistence(error.to_string()))?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| AppError::Persistence(error.to_string()))?;
        }
        let temporary = self.path.with_extension("json.tmp");
        let mut file = fs::File::create(&temporary)
            .map_err(|error| AppError::Persistence(error.to_string()))?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| AppError::Persistence(error.to_string()))?;
        fs::rename(&temporary, &self.path).map_err(|error| AppError::Persistence(error.to_string()))
    }

    fn load(&self) -> Result<Option<PersistedAppState>, AppError> {
        match fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|error| AppError::Persistence(error.to_string())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(AppError::Persistence(error.to_string())),
        }
    }
}

fn default_engine(now: DateTime<Utc>) -> ReminderEngine {
    match ReminderEngine::new(now, AppSettings::default()) {
        Ok(engine) => engine,
        Err(_) => unreachable!("built-in default settings must remain valid"),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::domain::{ReminderKind, ReminderState};

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 1, 12, 0, 0).single().unwrap()
    }

    #[test]
    fn missing_state_uses_defaults() {
        let temporary = tempfile::tempdir().unwrap();
        let store = PersistenceStore::new(temporary.path().join("state.json"));
        let (engine, warning) = store.load_engine(now());
        assert!(warning.is_none());
        assert!(matches!(
            engine.snapshot(now()).water.state,
            ReminderState::Scheduled { .. }
        ));
    }

    #[test]
    fn corrupted_state_falls_back_without_panicking() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary.path().join("state.json");
        fs::write(&path, b"not json").unwrap();
        let store = PersistenceStore::new(path);
        let (engine, warning) = store.load_engine(now());
        assert!(warning.is_some());
        assert!(engine.settings().water_enabled);
    }

    #[test]
    fn persisted_state_round_trips() {
        let temporary = tempfile::tempdir().unwrap();
        let store = PersistenceStore::new(temporary.path().join("state.json"));
        let mut engine = ReminderEngine::new(now(), AppSettings::default()).unwrap();
        engine.complete(ReminderKind::Water, now());
        store.save_engine(&engine).unwrap();

        let (restored, warning) = store.load_engine(now());
        assert!(warning.is_none());
        assert_eq!(
            restored.snapshot(now()).water.last_completed_at,
            Some(now())
        );
    }
}
