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
    let app = Application::builder()
        .application_id("org.cvusmo.lustre")
        .build();

    // Activate signal using the provided state
    app.connect_activate(move |app| {
        let window = build_ui(app, &state);
        window.present();
    });

    // Run the application
    app.run();
}

pub fn close_launcher(app: &gtk4::Application, state: Arc<Mutex<AppState>>) {
    app.connect_shutdown(move |_| {
        log_info("Launcher shutdown confirmed, launching game window...");
        lustre_window(Arc::clone(&state));
    });
    // Schedule the quit to be run on the main thread.
    MainContext::default().invoke_local({
        let app = app.clone();
        move || {
            app.quit();
        }
    });
}
