// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/launcher.rs

use crate::gui::build_ui;
use crate::state::create_state;
use gtk4::prelude::*;
use gtk4::Application;
use std::sync::{Arc, Mutex};

pub fn launcher() {
    println!("Launching lustre...");

    // Create GTK app
    let app = Application::builder()
        .application_id("org.cvusmo.lustre")
        .build();

    // Create app state
    let state = create_state();

    // Activate signal
    app.connect_activate(move |app| {
        let window = build_ui(app, &state);
        window.present();
    });

    // Run the application
    app.run();
}
