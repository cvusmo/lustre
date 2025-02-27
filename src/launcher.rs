// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/launcher.rs

use crate::engine::gui::build_ui;
use crate::state::{log_info, AppState};
use gtk4::prelude::*;
use gtk4::Application;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// Static flag to signal game launch
static LAUNCH_GAME: AtomicBool = AtomicBool::new(false);

pub fn launcher(state: Arc<Mutex<AppState>>) -> bool {
    log_info("Launching lustre...");

    let launcher_app = Application::builder()
        .application_id("org.cvusmo.lustre")
        .build();

    launcher_app.connect_activate(move |launcher_app| {
        let window = build_ui(launcher_app, &state);
        window.present();
    });

    // Run the application and return whether to launch the game
    launcher_app.run();
    LAUNCH_GAME.load(Ordering::SeqCst)
}

pub fn close_launcher(launcher_app: Application, _state: Arc<Mutex<AppState>>) {
    log_info("Closing launcher and signaling game launch...");
    LAUNCH_GAME.store(true, Ordering::SeqCst);
    launcher_app.quit();
}

