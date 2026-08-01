pub mod application;
pub mod domain;
pub mod error;
pub mod infrastructure;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    application::run();
}
