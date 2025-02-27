// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/main.rs

use lustre::launcher::launcher;
use lustre::state::{create_state, initialize_state, log_error, log_info};
use lustre::window::lustre_window;
use std::sync::mpsc::channel;

fn main() {
    let log_file_path = "lustre.log";
    let log_level = log::LevelFilter::Debug;

    initialize_state(log_file_path, log_level).expect("Failed to initialize logger");
    let state = create_state();

    // Create a channel for signaling the "Play" action
    let (tx, rx) = channel::<()>();

    // Launch the GTK launcher on the main thread
    log_info("Launching GTK launcher...");
    launcher(state.clone(), tx.clone());

    log_info("Waiting for launcher to signal game launch...");
    if rx.recv().is_ok() {
        log_info("Launching game window...");
        lustre_window(state);
    } else {
        log_error("Failed to receive launch signal from launcher");
    }
}

