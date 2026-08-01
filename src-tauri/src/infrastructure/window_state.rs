use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{PhysicalPosition, WebviewWindow};

use crate::error::AppError;

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
    use super::rectangles_intersect;

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
