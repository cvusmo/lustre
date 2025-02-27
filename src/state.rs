// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/state.rs

use fern::Dispatch;
use once_cell::sync::OnceCell;
use rand::prelude::*;
use std::time::Instant;
use std::{
    error::Error,
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;

// Global flag to enable/disable debug logs (optional for production)
static ENABLE_DEBUG: bool = cfg!(debug_assertions); // Enable in debug builds, disable in release
static STATE_INITIALIZED: OnceCell<bool> = OnceCell::new();

pub struct AppState {
    pub project_path: Option<PathBuf>,
    pub vulkan_instance: Option<Arc<Instance>>,
    pub vulkan_surface: Option<Arc<Surface>>,
    pub is_modified: bool,
    pub start_time: Instant,
    pub voxel_grid: Vec<Vec<Vec<bool>>>,
}

impl AppState {
    pub fn new() -> Self {
        let grid_width = 64;
        let grid_height = 64;
        let grid_depth = 64;
        let mut voxel_grid = vec![vec![vec![false; grid_depth]; grid_height]; grid_width];
        let mut rng = rand::rng();

        for x in 0..grid_width {
            for y in 0..grid_height {
                for z in 0..grid_depth {
                    if rng.random_bool(0.01) {
                        voxel_grid[x][y][z] = true;
                    }
                }
            }
        }

        Self {
            project_path: None,
            vulkan_instance: None,
            vulkan_surface: None,
            is_modified: false,
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

pub fn create_state() -> Arc<Mutex<AppState>> {
    Arc::new(Mutex::new(AppState::default()))
}

// Improved logging functions with dynamic formatting and conditional debug
pub fn log_info(message: &str) {
    log::info!("{}", message);
}

pub fn log_debug(args: std::fmt::Arguments<'_>) {
    if ENABLE_DEBUG {
        log::debug!("{}", args);
    }
}

pub fn log_debug_fmt<F: Fn() -> String>(message: F) {
    if ENABLE_DEBUG {
        log::debug!("{}", message());
    }
}

pub fn log_warn(message: &str) {
    log::warn!("{}", message);
}

pub fn log_error(message: &str) {
    log::error!("{}", message);
} 
    
