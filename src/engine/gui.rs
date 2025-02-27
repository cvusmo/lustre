// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/engine/gui.rs

use crate::engine::ui::menu_bar::create_menu_bar;
use crate::state::{log_info, AppState};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, DrawingArea, Grid};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

// Builds the GTK-based launcher UI
pub fn build_ui(
    app: &Application,
    state: &Arc<Mutex<AppState>>,
    tx: Sender<()>,
) -> ApplicationWindow {
    // Changed from Arc<ApplicationWindow> to ApplicationWindow
    log_info("Begin building UI and loading config...");

    log_info("Creating window UI...");
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Lustre GE")
        .default_width(1025)
        .default_height(1024)
        .css_classes(vec!["window".to_string()])
        .build();
    log_info("Window created, preparing to return..."); // Debug log

    log_info("Creating grid...");
    let grid = Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
        .vexpand(true)
        .hexpand(true)
        .halign(gtk4::Align::Fill)
        .valign(gtk4::Align::Fill)
        .build();
    window.set_child(Some(&grid));

    log_info("Creating project area...");
    let project_area = GtkBox::new(gtk4::Orientation::Vertical, 5);
    project_area.set_vexpand(true);
    project_area.set_hexpand(true);
    project_area.add_css_class("project-area");
    grid.attach(&project_area, 0, 1, 2, 1);

    log_info("Creating menu bar...");
    let (menu_bar, _text_view) = create_menu_bar(state, &window, app, &project_area, tx.clone());
    log_info("Menu bar created, attaching to grid..."); // Debug log
    menu_bar.add_css_class("menu-bar");
    grid.attach(&menu_bar, 0, 0, 2, 1);

    log_info("Creating Vulkan drawing area...");
    let vulkan_area = DrawingArea::new();
    vulkan_area.set_vexpand(true);
    vulkan_area.set_hexpand(true);
    vulkan_area.set_size_request(800, 600);
    vulkan_area.set_valign(gtk4::Align::Fill);
    vulkan_area.set_halign(gtk4::Align::Fill);
    grid.attach(&vulkan_area, 1, 2, 1, 1);

    log_info("UI built successfully.");
    window // Return the window directly, not wrapped in Arc
}

