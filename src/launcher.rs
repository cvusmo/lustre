// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/launcher.rs

use crate::engine::gui::build_ui;
use crate::state::{log_info, AppState};
use crate::window::lustre_window;
use glib::MainContext;
use gtk4::prelude::*;
use gtk4::Application;
use std::sync::{Arc, Mutex};

pub fn launcher(state: Arc<Mutex<AppState>>) {
    log_info("Launching lustre...");

    // Create GTK app
    let launcher_app = Application::builder()
        .application_id("org.cvusmo.lustre")
        .build();

    // Activate signal using the provided state
    launcher_app.connect_activate(move |launcher_app| {
        let window = build_ui(launcher_app, &state);
        window.present();
    });

    // Run the application
    launcher_app.run();
}

pub fn close_launcher(launcher_app: &gtk4::Application, state: Arc<Mutex<AppState>>) {
    launcher_app.connect_shutdown(move |_| {
        log_info("Launcher shutdown confirmed, launching game window...");
        lustre_window(Arc::clone(&state));
    });
    // Schedule the quit to be run on the main thread.
    MainContext::default().invoke_local({
        let launcher_app = launcher_app.clone();
        move || {
            launcher_app.quit();
        }
    });
}
