// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/launcher.rs

use crate::engine::gui::build_ui;
use crate::state::{log_debug, log_info, AppState};
use gtk4::prelude::*;
use gtk4::Application;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

// Launches the GTK launcher and returns a receiver to wait for the "Play" signal
pub fn launcher(state: Arc<Mutex<AppState>>, tx: Sender<()>) -> Receiver<()> {
    log_info("Launching lustre...");
    log_debug(format_args!(
        "Starting GTK launcher setup with state and signal channel..."
    ));

    let launcher_app = Application::builder()
        .application_id("org.cvusmo.lustre")
        .build();

    let (launch_tx, rx) = channel::<()>(); // Channel to signal when to launch the game
    log_debug(format_args!(
        "Created channel for launch signal: tx={:?}, rx={:?}",
        launch_tx, rx
    ));

    launcher_app.connect_activate(move |launcher_app| {
        log_debug(format_args!(
            "Activating launcher UI with application: {:?}",
            launcher_app
        ));
        let window = build_ui(launcher_app, &state, launch_tx.clone());
        window.present();
        log_debug(format_args!(
            "Launcher UI presented, window created: {:?}",
            window
        ));
    });

    // Clone the sender for the shutdown handler
    let tx_clone = tx.clone();
    launcher_app.connect_shutdown(move |app| {
        log_debug(format_args!(
            "Shutdown signal received for application: {:?}",
            app
        ));
        log_info("Launcher shutdown confirmed, signaling game launch...");
        log_debug(format_args!(
            "Sending launch signal through channel: {:?}",
            tx_clone
        ));
        let _ = tx_clone.send(());
        log_debug(format_args!(
            "Launch signal sent, waiting for channel to complete..."
        ));
    });

    // Run the application directly on the main thread
    log_debug(format_args!("Starting GTK application run loop..."));
    launcher_app.run();
    log_debug(format_args!(
        "GTK application run loop completed, returning receiver..."
    ));
    rx // Return the receiver to wait for the signal
}

pub fn close_launcher(launcher_app: Application, tx: Sender<()>) {
    log_debug(format_args!(
        "Initiating launcher closure with application: {:?}",
        launcher_app
    ));
    log_info("Closing launcher and signaling game launch...");
    log_debug(format_args!(
        "Preparing to send launch signal through channel: {:?}",
        tx
    ));
    let _ = tx.send(());
    log_debug(format_args!(
        "Launch signal sent, attempting to quit GTK application..."
    ));
    log_info("Quitting GTK application..."); // Debug log
    launcher_app.quit();
    log_debug(format_args!(
        "GTK quit command issued, awaiting shutdown..."
    ));
}

