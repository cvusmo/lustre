// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/modules/engine/gui/window.rs

use crate::modules::engine::configuration::config::Config;
use crate::modules::engine::configuration::logger::{log_debug, log_info, AppState};
use crate::modules::engine::gui::components::grid::create_grid;
use crate::modules::engine::gui::components::sidebar::create_sidebar;
use crate::modules::engine::gui::menu_bar::create_menu_bar;
use crate::modules::engine::gui::utils::{create_text_editor, load_project_area};

use gtk::{
    gdk::Display, prelude::*, Application, ApplicationWindow, CssProvider, DrawingArea, Label,
};
use gtk4 as gtk;
use std::{
    env,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

// Function that builds components for window
pub fn build_ui(
    app: &Application,
    config: &Config,
    state: &Arc<Mutex<AppState>>,
) -> Arc<ApplicationWindow> {
    log_info(state, "Begin building ui && loading config...");

    // Loading and applying theme and CSS
    let (background_color, font_color, font_size) = load_theme(config, state);
    let _config_path = load_configuration_path(state);
    let css = generate_css(&font_color, font_size, &background_color);
    apply_css(&css, state);

    // Create the main window
    log_info(state, "Creating window UI...");
    let window = create_window(app);

    // Wrap window in Arc
    let window = Arc::new(window);

    // Create main layout grid
    log_info(state, "Creating grid...");
    let grid = create_grid();
    window.set_child(Some(&grid));

    // Add Sidebar
    log_info(state, "Creating sidebar...");
    let sidebar = create_sidebar();
    grid.attach(&sidebar, 0, 1, 1, 2);

    // Add project area and set it in AppState
    log_info(state, "Creating project area...");
    let project_area = create_project_area();
    project_area.add_css_class("project-area");
    grid.attach(&project_area, 0, 1, 2, 1);

    {
        let mut state = state.lock().unwrap();
        state.project_area = Some(project_area.clone());
    }

    // Load an initial empty project
    log_info(state, "Loading initial project area...");
    load_project_area(state, "", create_text_editor);

    // Add Tabs

    // Create a drawing area for Vulkan
    log_info(state, "Creating Vulkan drawing area...");
    let vulkan_area = DrawingArea::new();
    vulkan_area.set_vexpand(true);
    vulkan_area.set_hexpand(true);
    vulkan_area.set_size_request(800, 600);
    vulkan_area.set_valign(gtk::Align::Fill);
    vulkan_area.set_halign(gtk::Align::Fill);
    grid.attach(&vulkan_area, 1, 2, 1, 1); // Place it in a different row/column than project_area

    // Add menu bar
    log_info(state, "Creating menu bar...");
    let menu_bar = create_menu_bar(state, &window, app); //, vulkan_context);
    menu_bar.add_css_class("menu-bar");
    grid.attach(&menu_bar, 0, 0, 2, 1);

    log_info(state, "Build UI successfully.");

    window
}

// Create project area
fn create_project_area() -> gtk::Box {
    let project_area = gtk::Box::new(gtk::Orientation::Vertical, 5);
    project_area.set_vexpand(true);
    project_area.set_hexpand(true);

    let label = Label::new(Some("Project Area"));
    project_area.append(&label);
    project_area
}

// Loads theme configuration
fn load_theme(config: &Config, state: &Arc<Mutex<AppState>>) -> (String, String, f32) {
    let background_color = config.theme.background_color.clone();
    log_info(state, &format!("Background color: {}", background_color));

    let font_color = config.theme.font_color.clone();
    log_info(state, &format!("Font color: {}", font_color));

    let font_size = config.theme.font_size as f32;
    log_info(state, &format!("Font size: {}", font_size));

    (background_color, font_color, font_size)
}

// Loads the configuration
fn load_configuration_path(state: &Arc<Mutex<AppState>>) -> PathBuf {
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/home/$USER"));
    let config_file = format!("{}/.config/lustre/lustre.conf", home_dir);
    let config_path = Path::new(&config_file);
    log_info(
        state,
        &format!("Configuration file path: {}", config_path.display()),
    );
    config_path.to_path_buf()
}

// Generates CSS
fn generate_css(font_color: &str, font_size: f32, background_color: &str) -> String {
    format!(
        "
        .menu-bar {{
            background-color: #454240;
        }}
        .menu-button {{
            background-color: #595653;
            color: #F4E3C1;
            border: none;
            padding: 10px;
            border-radius: 5px;
        }}
        .menu-button:hover {{
            background-color: #;
        }}
        .clock {{
            color: {};
            font-size: {}px;
        }}
        .window {{
            background-color: {};
        }}
        .menu-bar {{
            background-color: #1C1B1A;
        }}
        .project-area {{
            background-color: #333333; /* Charcoal gray */
        }}
        ",
        font_color, font_size, background_color
    )
}

// Applies CSS
fn apply_css(css: &str, state: &Arc<Mutex<AppState>>) {
    let provider = CssProvider::new();
    provider.load_from_data(css);
    log_debug(state, "CSS loaded successfully.");

    gtk::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    log_debug(
        state,
        &format!(
            "Generated CSS:
{}",
            css
        ),
    );
}

// Creates the main window
fn create_window(app: &Application) -> ApplicationWindow {
    ApplicationWindow::builder()
        .application(app)
        .title("lustre")
        .css_classes(vec!["window".to_string()])
        .build()
}
