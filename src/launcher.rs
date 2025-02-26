// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/launcher.rs

use crate::engine::gui::build_ui;
use crate::state::{log_info, AppState};
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
