// src/main.rs
// github.com/cvusmo/gameengine

mod modules;
mod debug;

use crate::modules::engine::configuration::{
    config::Config,
    logger::{create_state, log_error, log_info, log_warn, setup_logging, AppState},
};

use crate::modules::engine::gui;
use crate::debug::debug::enable_debug_mode;

use clap::{Arg, Command};
use gtk::{glib, prelude::*, Application};
use gtk4 as gtk;
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

const APP_ID: &str = "org.cvusmo.gameengine";

fn main() -> glib::ExitCode {
    let _gtkinit = gtk::init();
    let lua = Lua::new();

    let add = lua.create_function(|_, (a, b): (i32, i32)| {
    Ok(a + b)
})?;

    lua.globals().set("add", add)?;

    lua.load(r#"
        local result = add(3, 4)
        print("Result of addition:", result)
    "#).exec()?;

    let matches = Command::new("gameengine")
        .version("0.0.1")
        .about("gamengine - A voxel game engine")
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .help("Enables debug mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Specifies a custom config file")
                .value_name("FILE")
                .num_args(1),
        )
        .get_matches();

    // --debug flag
    let debug_mode = *matches.get_one::<bool>("debug").unwrap_or(&false);
    if debug_mode {
        enable_debug_mode();
    }

    // Create log
    let state = create_state();
    if let Err(e) = setup_logging(&state, debug_mode) {
        log_error(&state, &format!("Failed to setup logging: {}", e));
    }

    // Handle config file
    let config_file = matches.get_one::<String>("config").cloned(); 
    if let Some(file) = &config_file {
        log_info(&state, &format!("Using config file: {}", file));
    }

    // Create application
    let app = Application::builder().application_id(APP_ID).build();
    
    // Pass the config_file to run_main
    app.connect_activate(move |app| run_main(app, &state, config_file.clone()));
    app.run()
}

fn run_main(app: &Application, state: &Arc<Mutex<AppState>>, config_file: Option<String>) {
    // Initialize config
    let config = match Config::check_config(config_file) { 
        // Pass config_file to check_config
        Ok(config) => config,
        Err(e) => {
            log_error(state, &format!("Failed to load config: {}", e));
            log_warn(state, "Using default configuration due to error.");
            log_info(state, &format!("Logging info check: {}", e));
            Config::new() 
        }
    };

    // Initialize window
    let window = gui::window::build_ui(app, &config, state);
    window.present();
}
