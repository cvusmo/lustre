// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/main.rs

use crate::launcher::launcher;
use crate::state::{create_state, initialize_state};
use crate::window::lustre_window;
use std::sync::{Arc, Mutex};

mod engine;
mod launcher;
mod shaders;
mod state;
mod window;

fn main() {
    let log_file_path = "lustre.log";
    let log_level = log::LevelFilter::Info;

    initialize_state(log_file_path, log_level).expect("Failed to initialize logger");
    let state = create_state();

    // Run the launcher; if it returns true, launch the game
    if launcher(state.clone()) {
        lustre_window(state);
    }
}

