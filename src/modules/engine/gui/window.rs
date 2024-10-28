// src/modules/engine/gui/window.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_debug, log_info, AppState};
use crate::modules::engine::configuration::config::Config;
use crate::modules::engine::gui::menu_bar::create_menu_bar;
use glib::ControlFlow::Continue;
use gtk4 as gtk;
use gtk::{ gdk::Display, prelude::*, Application, 
    ApplicationWindow, CssProvider, Grid, Label};
use std::{env, path::{Path, PathBuf}, sync::{Arc, Mutex}};

pub fn build_ui(
    app: &Application,
    config: &Config,
    state: &Arc<Mutex<AppState>>,
) -> ApplicationWindow {
    log_info(state, "Loading config...");
    
    let (background_color, font_color, font_size) = load_theme(config, state);
    let _config_path = load_configuration_path(state); 
    let css = generate_css(&font_color, font_size, &background_color);
    
    apply_css(&css, state);
    
    log_info(state, "Building window...");
    let window = create_window(app);
    
    // Create the main layout
    let grid = create_grid();
    window.set_child(Some(&grid));

    let menu_bar = create_menu_bar(state);
    grid.attach(&menu_bar, 0, 0, 1, 1);
    
    // Create a section for main content 
    let content_area = create_content_area();
    grid.attach(&content_area, 0, 2, 2, 1); // Main content area
    
    log_info(state, "Window built successfully.");
    window
}

// Create content area
fn create_content_area() -> gtk::Box {
    let content_area = gtk::Box::new(gtk::Orientation::Vertical, 5);
    // TODO: add content elements here (drawing area, buttons, etc.)
    let label = Label::new(Some("Main Content Area"));
    content_area.append(&label);
    content_area
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
    let config_file = format!("{}/.config/gameengine/gameengine.conf", home_dir);
    let config_path = Path::new(&config_file);
    log_info(state, &format!("Configuration file path: {}", config_path.display()));
    config_path.to_path_buf() 
}

// Generates the CSS string 
fn generate_css(font_color: &str, font_size: f32, background_color: &str) -> String {
    format!(
        "
        .clock {{
            color: {};
            font-size: {}px;
            width: 100%;
            height: 100%;
            text-align: center;
        }}
        .window {{
            background-color: {};
        }}
        ",
        font_color, font_size, background_color
    )
}

// Applies the generated CSS to the application.
fn apply_css(css: &str, state: &Arc<Mutex<AppState>>) {
    let provider = CssProvider::new();
    provider.load_from_data(css); 
    log_debug(state, "CSS loaded successfully.");

    gtk::style_context_add_provider_for_display(
        &Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    log_debug(state, &format!("Generated CSS:\n{}", css));
}

// Creates the main window
fn create_window(app: &Application) -> ApplicationWindow {
    ApplicationWindow::builder()
        .application(app)
        .title("gameengine")  
        .css_classes(vec!["window".to_string()])
        .build()
}

// Creates the clock label 
fn create_clock_label() -> Label {
    Label::builder()
        .label(get_current_time())
        .css_classes(vec!["clock".to_string()])
        .build()
}

// Creates a grid layout
fn create_grid() -> Grid {
    let grid = Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
        .build();

    grid.set_halign(gtk::Align::Start); 
    grid.set_valign(gtk::Align::Start); 

    grid
}

// Starts a timer that updates
fn start_clock_update(clock_label: Arc<Label>) {
    glib::timeout_add_seconds_local(1, move || {
        let current_time = get_current_time();
        clock_label.set_label(&current_time);
        Continue
    });
}

// Returns the current time
fn get_current_time() -> String {
    use chrono::{DateTime, Local};

    let now: DateTime<Local> = Local::now();
    now.format("%H:%M:%S").to_string()
}
