// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre

// src/configuration/logger.rs
// github.com/cvusmo/gameengine

use fern::Dispatch;
use gtk::Label;
use gtk4 as gtk;
use mlua::prelude::*;
use once_cell::sync::OnceCell;
use std::{
    env,
    error::Error,
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
};

static LOGGER_INITIALIZED: OnceCell<bool> = OnceCell::new();

// AppState
pub struct AppState {
    pub log_label: Label,
    pub project_area: Option<gtk::Box>,
    pub project_path: Option<PathBuf>,
    pub text_view: Option<gtk4::TextView>,
    pub is_modified: bool,
    pub lua: Arc<Mutex<Lua>>,
}

// Init logger
fn initialize_logger(
    _state: &Arc<Mutex<AppState>>,
    log_file_path: &str,
    log_level: log::LevelFilter,
) -> Result<(), Box<dyn Error>> {
    let log_file_result = File::create(log_file_path)?;

    Dispatch::new()
        .format(|out, message, record| {
            let module = record.target().split("::").last().unwrap_or("unknown");
            let line = record
                .line()
                .map_or("unknown".to_string(), |l| l.to_string());
            out.finish(format_args!(
                "[{}] {}, {}:{}",
                record.level(),
                message,
                module,
                line
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .chain(log_file_result)
        .apply()?;

    //log_info(state, &format!("Logger successfully created: {}", log_file_path));
    LOGGER_INITIALIZED.set(true).unwrap();
    Ok(())
}

// Normal mode
pub fn normal_mode(state: &Arc<Mutex<AppState>>) -> Result<(), Box<dyn Error>> {
    if LOGGER_INITIALIZED.get().is_none() {
        log_info(state, "Creating log file...");

        let user_dir = env::var("HOME").unwrap_or_else(|_| "/home/default".to_string());
        let log_file_path = format!("{}/.config/gameengine/gameengine.log", user_dir);

        initialize_logger(state, &log_file_path, log::LevelFilter::Info)?;

        log_info(state, "Logger initialized in normal mode.");
        log_warn(state, "This is a warning message.");
        log_error(state, "This is an error message.");
    } else {
        log_info(state, "Logger is already initialized.");
    }
    Ok(())
}

// DEBUG MODE
pub fn debug_mode(state: &Arc<Mutex<AppState>>) -> Result<(), Box<dyn Error>> {
    if LOGGER_INITIALIZED.get().is_none() {
        let user_dir = env::var("HOME").unwrap_or_else(|_| "/home/default".to_string());
        let log_file_path = format!("{}/.config/gameengine/gameengine-debug.log", user_dir);

        initialize_logger(state, &log_file_path, log::LevelFilter::Debug)?;
    } else {
        log_info(state, "Logger is already initialized.");
    }
    Ok(())
}

// Setup logging
pub fn setup_logging(state: &Arc<Mutex<AppState>>, debug: bool) -> Result<(), Box<dyn Error>> {
    log_info(state, "Setting up logging...");
    if debug {
        debug_mode(state)
    } else {
        normal_mode(state)
    }
}

// Create states
pub fn create_state() -> Arc<Mutex<AppState>> {
    let log_label = Label::new(None);
    let lua = Lua::new();
    Arc::new(Mutex::new(AppState {
        log_label,
        project_area: None,
        project_path: None,
        text_view: None,
        is_modified: false,
        lua: Arc::new(Mutex::new(lua)),
    }))
}

// Update Log Label
pub fn update_log_label(state: &Arc<Mutex<AppState>>, message: &str) {
    let state = state.lock().unwrap();
    state.log_label.set_label(message);
}

// Log Info
pub fn log_info(state: &Arc<Mutex<AppState>>, message: &str) {
    log::info!("{}", message);
    update_log_label(state, message);
}

// Log Debug
pub fn log_debug(state: &Arc<Mutex<AppState>>, message: &str) {
    log::debug!("{}", message);
    update_log_label(state, message);
}

// Log Warn
pub fn log_warn(state: &Arc<Mutex<AppState>>, message: &str) {
    log::warn!("{}", message);
    update_log_label(state, message);
}

// Log Error
pub fn log_error(state: &Arc<Mutex<AppState>>, message: &str) {
    log::error!("{}", message);
    update_log_label(state, message);
}
