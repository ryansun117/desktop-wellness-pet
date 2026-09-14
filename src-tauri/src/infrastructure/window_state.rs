use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{LogicalSize, PhysicalPosition, WebviewWindow};

use crate::error::AppError;

pub fn expand_settings(window: &WebviewWindow) -> Result<(), AppError> {
    window.set_resizable(true).map_err(window_error)?;
    if let Some(monitor) = window.current_monitor().map_err(window_error)? {
        let area = monitor.work_area();
        let scale = monitor.scale_factor();
        let width = 420.0_f64.min(f64::from(area.size.width) / scale);
        let height = 620.0_f64.min(f64::from(area.size.height) / scale);
        window
            .set_size(LogicalSize::new(width, height))
            .map_err(window_error)?;
        let position = window.outer_position().map_err(window_error)?;
        // The native resize is queued, so outer_size may still report compact
        // dimensions here. Clamp against the requested borderless window size.
        let size = LogicalSize::new(width, height).to_physical::<u32>(scale);
        window
            .set_position(PhysicalPosition::new(
                clamp_axis(position.x, area.position.x, area.size.width, size.width),
                clamp_axis(position.y, area.position.y, area.size.height, size.height),
            ))
            .map_err(window_error)?;
    } else {
        window
            .set_size(LogicalSize::new(420.0, 620.0))
            .map_err(window_error)?;
        window.center().map_err(window_error)?;
    }
    Ok(())
}

fn clamp_axis(position: i32, origin: i32, available: u32, size: u32) -> i32 {
    position.clamp(origin, origin + available.saturating_sub(size) as i32)
}

#[derive(Debug, Clone)]
pub struct WindowStateStore {
    path: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct SavedWindowPosition {
    x: i32,
    y: i32,
}

impl WindowStateStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn save(&self, position: PhysicalPosition<i32>) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(persistence_error)?;
        }
        let value = SavedWindowPosition {
            x: position.x,
            y: position.y,
        };
        let bytes = serde_json::to_vec(&value).map_err(persistence_error)?;
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, bytes).map_err(persistence_error)?;
        fs::rename(temporary, &self.path).map_err(persistence_error)
    }

    pub fn restore(window: &WebviewWindow, path: &PathBuf) -> Result<(), AppError> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(persistence_error(error)),
        };
        let saved: SavedWindowPosition =
            serde_json::from_slice(&bytes).map_err(persistence_error)?;
        let position = PhysicalPosition::new(saved.x, saved.y);
        let monitors = window.available_monitors().map_err(window_error)?;
        let is_visible = monitors.iter().any(|monitor| {
            let origin = monitor.position();
            let size = monitor.size();
            rectangles_intersect(
                (position.x, position.y, 240, 260),
                (origin.x, origin.y, size.width as i32, size.height as i32),
            )
        });
        if is_visible {
            window.set_position(position).map_err(window_error)?;
        } else {
            window.center().map_err(window_error)?;
        }
        Ok(())
    }
}

fn rectangles_intersect(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

fn persistence_error(error: impl std::fmt::Display) -> AppError {
    AppError::Persistence(error.to_string())
}

fn window_error(error: tauri::Error) -> AppError {
    AppError::Window(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{clamp_axis, rectangles_intersect};

    #[test]
    fn expanded_window_stays_inside_monitor_work_area() {
        assert_eq!(clamp_axis(1300, 0, 1440, 420), 1020);
        assert_eq!(clamp_axis(700, 25, 850, 620), 255);
        assert_eq!(clamp_axis(-1300, -1440, 1440, 420), -1300);
        assert_eq!(clamp_axis(-1500, -1440, 1440, 420), -1440);
        assert_eq!(clamp_axis(100, 0, 300, 420), 0);
    }

    #[test]
    fn detects_visible_and_offscreen_windows() {
        assert!(rectangles_intersect(
            (100, 100, 240, 260),
            (0, 0, 1440, 900)
        ));
        assert!(!rectangles_intersect(
            (2000, 100, 240, 260),
            (0, 0, 1440, 900)
        ));
    }
}
