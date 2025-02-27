// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/state.rs

use fern::Dispatch;
use gtk4::Box as GtkBox;
use gtk4::{DrawingArea, TextView};
use mlua::prelude::*;
use once_cell::sync::OnceCell;
use std::time::Instant;
use std::{
    error::Error,
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;

static STATE_INITIALIZED: OnceCell<bool> = OnceCell::new();

/// Represents the application's state.
pub struct AppState {
    pub project_path: Option<PathBuf>,
    pub project_area: Option<GtkBox>,
    pub vulkan_area: Option<DrawingArea>,
    pub vulkan_instance: Option<Arc<Instance>>,
    pub vulkan_surface: Option<Arc<Surface>>,
    pub text_view: Option<TextView>,
    pub lua: Arc<Mutex<Lua>>,
    pub is_modified: bool,
    pub start_time: Instant,
    pub voxel_grid: Vec<Vec<Vec<bool>>>,
}

impl AppState {
    // Create new AppState with initialized voxel grid
    // TODO: World Generation
    pub fn new() -> Self {
        let grid_width = 64;
        let grid_height = 1;
        let grid_depth = 64;
        let mut voxel_grid = vec![vec![vec![false; grid_depth]; grid_height]; grid_width];
        // Init one voxel at (0, 0, 0) for testing
        voxel_grid[0][0][0] = true;

        Self {
            project_path: None,
            project_area: None,
            vulkan_area: None,
            vulkan_instance: None,
            vulkan_surface: None,
            lua: Arc::new(Mutex::new(Lua::new())),
            is_modified: false,
            text_view: None,
            start_time: Instant::now(),
            voxel_grid,
        }
    }

    pub fn toggle_voxel(&mut self, x: usize, y: usize, z: usize) -> bool {
        if x < self.voxel_grid.len()
            && y < self.voxel_grid[0].len()
            && z < self.voxel_grid[0][0].len()
        {
            self.voxel_grid[x][y][z] = !self.voxel_grid[x][y][z];
            self.is_modified = true;
            return true;
        }
        false
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Initializes the application state along with logging.
pub fn initialize_state(
    log_file_path: &str,
    log_level: log::LevelFilter,
) -> Result<(), Box<dyn Error>> {
    let log_file = File::create(log_file_path)?;

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
        .chain(log_file)
        .apply()?;

    println!("Logger successfully initialized...");

    STATE_INITIALIZED.set(true).unwrap();
    Ok(())
}

/// Creates the initial application state.
pub fn create_state() -> Arc<Mutex<AppState>> {
    Arc::new(Mutex::new(AppState::default()))
}

/// Logs an informational message.
pub fn log_info(message: &str) {
    log::info!("{}", message);
}

/// Logs a debug message.
pub fn log_debug(message: &str) {
    log::debug!("{}", message);
}

/// Logs a warning message.
pub fn log_warn(message: &str) {
    log::warn!("{}", message);
}

/// Logs an error message.
pub fn log_error(message: &str) {
    log::error!("{}", message);
}
