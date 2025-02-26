// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/lib.rs

use crate::menu_bar::create_menu_bar;
use crate::state::{log_info, AppState};
use crate::utils::{create_text_editor, load_project_area};

use gtk::{prelude::*, Application, ApplicationWindow, DrawingArea, Grid, Label};
use gtk4 as gtk;
use std::sync::{Arc, Mutex};

// Function that builds components for window
pub fn build_ui(app: &Application, state: &Arc<Mutex<AppState>>) -> Arc<ApplicationWindow> {
    log_info("Begin building ui && loading config...");

    // Create the main window
    log_info("Creating window UI...");
    let window = create_window(app);

    // Wrap window in Arc
    let window = Arc::new(window);

    // Create main layout grid
    log_info("Creating grid...");
    let grid = create_grid();
    window.set_child(Some(&grid));

    // Add project area and set it in AppState
    log_info("Creating project area...");
    let project_area = create_project_area();
    project_area.add_css_class("project-area");
    grid.attach(&project_area, 0, 1, 2, 1);

    {
        let mut state = state.lock().unwrap();
        state.project_area = Some(project_area.clone());
    }

    // Load an initial empty project
    log_info("Loading initial project area...");
    load_project_area(state, "", create_text_editor);

    // Create a drawing area for Vulkan
    log_info("Creating Vulkan drawing area...");
    let vulkan_area = DrawingArea::new();
    vulkan_area.set_vexpand(true);
    vulkan_area.set_hexpand(true);
    vulkan_area.set_size_request(800, 600);
    vulkan_area.set_valign(gtk::Align::Fill);
    vulkan_area.set_halign(gtk::Align::Fill);
    grid.attach(&vulkan_area, 1, 2, 1, 1); // Place it in a different row/column than project_area

    {
        let mut state_lock = state.lock().unwrap();
        state_lock.vulkan_area = Some(vulkan_area.clone());
    }

    // Add menu bar
    log_info("Creating menu bar...");
    let menu_bar = create_menu_bar(state, &window, app);
    menu_bar.add_css_class("menu-bar");
    grid.attach(&menu_bar, 0, 0, 2, 1);

    log_info("Build UI successfully.");

    window
}

// Create content area
fn create_project_area() -> gtk::Box {
    let project_area = gtk::Box::new(gtk::Orientation::Vertical, 5);
    project_area.set_vexpand(true);
    project_area.set_hexpand(true);

    let label = Label::new(Some("Project Area"));
    project_area.append(&label);
    project_area
}

// Creates the main window
fn create_window(app: &Application) -> ApplicationWindow {
    ApplicationWindow::builder()
        .application(app)
        .title("lustre ge")
        .css_classes(vec!["window".to_string()])
        .build()
}

// Creates a grid layout
fn create_grid() -> Grid {
    let grid = Grid::builder().row_spacing(10).column_spacing(10).build();

    // Set grid to expand
    grid.set_vexpand(true);
    grid.set_hexpand(true);

    // Grid alignment
    grid.set_halign(gtk::Align::Fill);
    grid.set_valign(gtk::Align::Fill);

    grid
}
