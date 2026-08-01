use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::{
    domain::{
        AppSettings, AppSnapshot, Clock, EvaluationAction, ReminderEngine, ReminderKind,
        ValidationIssue,
    },
    error::AppError,
    infrastructure::persistence::PersistenceStore,
};

pub struct AppService {
    engine: Mutex<ReminderEngine>,
    store: PersistenceStore,
    clock: Arc<dyn Clock>,
    timezone: Tz,
}

impl AppService {
    pub fn new(
        engine: ReminderEngine,
        store: PersistenceStore,
        clock: Arc<dyn Clock>,
        timezone: Tz,
    ) -> Self {
        Self {
            engine: Mutex::new(engine),
            store,
            clock,
            timezone,
        }
    }

    pub fn snapshot(&self) -> Result<AppSnapshot, AppError> {
        Ok(self.lock()?.snapshot(self.clock.now()))
    }

    pub fn validate_settings(settings: &AppSettings) -> Result<(), AppError> {
        settings.validate().map_err(AppError::InvalidSettings)
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<AppSnapshot, AppError> {
        Self::validate_settings(&settings)?;
        self.mutate(|engine, now| {
            engine
                .update_settings(settings, now)
                .map_err(AppError::InvalidSettings)
        })
    }

    pub fn complete(&self, kind: ReminderKind) -> Result<AppSnapshot, AppError> {
        self.mutate(|engine, now| {
            engine.complete(kind, now);
            Ok(())
        })
    }

    pub fn snooze(&self, kind: ReminderKind) -> Result<AppSnapshot, AppError> {
        self.mutate(|engine, now| {
            engine.snooze(kind, now);
            Ok(())
        })
    }

    pub fn skip(&self, kind: ReminderKind) -> Result<AppSnapshot, AppError> {
        self.mutate(|engine, now| {
            engine.skip(kind, now);
            Ok(())
        })
    }

    pub fn pause(&self, until: Option<DateTime<Utc>>) -> Result<AppSnapshot, AppError> {
        self.mutate(|engine, _| {
            engine.pause(until);
            Ok(())
        })
    }

    pub fn resume(&self) -> Result<AppSnapshot, AppError> {
        self.mutate(|engine, _| {
            engine.resume();
            Ok(())
        })
    }

    pub fn reset(&self) -> Result<AppSnapshot, AppError> {
        self.mutate(|engine, now| {
            engine.reset(now);
            Ok(())
        })
    }

    pub fn evaluate(&self) -> Result<(AppSnapshot, Vec<EvaluationAction>), AppError> {
        let now = self.clock.now();
        let mut engine = self.lock()?;
        let before = engine.clone();
        let actions = engine.evaluate(now, self.timezone);
        if *engine != before {
            self.store.save_engine(&engine)?;
        }
        Ok((engine.snapshot(now), actions))
    }

    fn mutate<F>(&self, operation: F) -> Result<AppSnapshot, AppError>
    where
        F: FnOnce(&mut ReminderEngine, DateTime<Utc>) -> Result<(), AppError>,
    {
        let now = self.clock.now();
        let mut engine = self.lock()?;
        operation(&mut engine, now)?;
        self.store.save_engine(&engine)?;
        Ok(engine.snapshot(now))
    }

    fn lock(&self) -> Result<MutexGuard<'_, ReminderEngine>, AppError> {
        self.engine.lock().map_err(|_| AppError::StateUnavailable)
    }
}

impl From<Vec<ValidationIssue>> for AppError {
    fn from(value: Vec<ValidationIssue>) -> Self {
        Self::InvalidSettings(value)
    }
}
