use clap::{Arg, Command};
use gtk::{glib, prelude::*, Application};
use gtk4 as gtk;
use mlua::prelude::*;
use std::fs;
use std::sync::{Arc, Mutex};

use gameengine::create_state;
use gameengine::modules::engine::configuration::config::Config;
use gameengine::modules::engine::configuration::logger::{log_error, log_info, log_warn, AppState};
use gameengine::modules::engine::gui;
use gameengine::modules::engine::gui::editor::lua_editor::register_lua_functions;

const APP_ID: &str = "org.cvusmo.gameengine";

fn main() -> glib::ExitCode {
    if let Err(e) = gtk::init() {
        eprintln!("Failed to initialize GTK: {}", e);
        return glib::ExitCode::FAILURE;
    }
    let lua = Lua::new();
    let state = create_state();

    // Register Lua functions from lua_editor
    if let Err(e) = register_lua_functions(&lua, Arc::clone(&state)) {
        eprintln!("Failed to register Lua functions: {}", e);
        return glib::ExitCode::FAILURE;
    }

    // Example Lua function registration (addition)
    if let Err(e) = lua.globals().set(
        "add",
        lua.create_function(|_, (a, b): (i32, i32)| Ok(a + b))
            .unwrap(),
    ) {
        eprintln!("Failed to set Lua global: {}", e);
        return glib::ExitCode::FAILURE;
    }

    // Command-line argument parsing
    let matches = Command::new("gameengine")
        .version("0.0.1")
        .about("gameengine - A voxel game engine")
        .arg(
            Arg::new("script")
                .help("Path to the Lua script to execute")
                .value_name("SCRIPT")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("gui")
                .short('g')
                .long("gui")
                .help("Launch the GUI")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Specifies a custom config file")
                .value_name("gameengine.conf")
                .num_args(1),
        )
        .get_matches();
    // Load and execute the Lua script
    let script_path = matches
        .get_one::<String>("script")
        .expect("Lua script path is required");
    match fs::read_to_string(script_path) {
        Ok(script_content) => match lua.load(&script_content).exec() {
            Ok(_) => println!("Lua script executed successfully."),
            Err(err) => eprintln!("Lua execution error: {}", err),
        },
        Err(err) => eprintln!("Failed to read Lua script: {}", err),
    }

    // Check if GUI
    if matches.get_flag("gui") {
        // Init GUI
        if let Err(e) = gtk::init() {
            eprintln!("Failed to initialize GTK: {}", e);
            return glib::ExitCode::FAILURE;
        }
    }

    // CLI-only mode
    if !matches.get_flag("gui") {
        match fs::read_to_string(script_path) {
            Ok(script_content) => {
                if let Err(err) = lua.load(&script_content).exec() {
                    eprintln!("Failed to execute Lua script: {}", err);
                    return glib::ExitCode::FAILURE;
                } else {
                    println!("Lua script executed successfully.");
                }
            }
            Err(err) => {
                eprintln!("Failed to read Lua script at '{}': {}", script_path, err);
                return glib::ExitCode::FAILURE;
            }
        }
        return glib::ExitCode::SUCCESS;
    }

    let config_file = matches.get_one::<String>("config").cloned();

    // Create application
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(move |app| run_main(app, &state, config_file.clone()));
    app.run()
}

fn run_main(app: &Application, state: &Arc<Mutex<AppState>>, config_file: Option<String>) {
    // Initialize config
    let config = match Config::check_config(config_file) {
        Ok(config) => config,
        Err(e) => {
            log_error(state, &format!("Failed to load config: {}", e));
            Config::new()
        }
    };

    // Initialize window explicitly
    let window = gui::window::build_ui(app, &config, state);
    window.present();
}
