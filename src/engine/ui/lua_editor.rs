// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/engine/ui/lua_editor.rs

use crate::engine::ui::utils::create_text_editor;
use crate::state::{log_error, log_info, AppState};
use gtk4::prelude::*;
use gtk4::{ScrolledWindow, TextView};
use mlua::prelude::*;
use std::fs;
use std::sync::{Arc, Mutex};

// Registers basic Lua functions (e.g., print_message)
fn register_lua_functions(lua: &Lua) -> LuaResult<()> {
    let print_message = lua.create_function(|_, message: String| {
        log_info(&message);
        Ok(())
    })?;
    lua.globals().set("print_message", print_message)?;
    Ok(())
}

// Registers Vulkan render trigger function
fn register_render_functions(lua: &Lua) -> LuaResult<()> {
    let launch_fn = lua.create_function(|_, ()| {
        launch_vulkan_render();
        Ok(())
    })?;
    lua.globals().set("launch_render", launch_fn)?;
    Ok(())
}

// Registers all Lua functions (basic and render)
pub fn register_all(lua: &Lua) -> LuaResult<()> {
    register_lua_functions(lua)?;
    register_render_functions(lua)?;
    Ok(())
}

// Loads Lua scripts from the "./mods" directory
pub fn load_mods(lua: &Lua) {
    let mod_path = "./mods";
    match fs::read_dir(mod_path) {
        Ok(paths) => {
            for entry in paths.flatten() {
                let script_path = entry.path();
                if script_path.extension().and_then(|s| s.to_str()) == Some("lua") {
                    match fs::read_to_string(&script_path) {
                        Ok(script_content) => {
                            if let Err(e) = lua.load(&script_content).exec() {
                                log_error(&format!(
                                    "Failed to load Lua script {:?}: {}",
                                    script_path, e
                                ));
                            }
                        }
                        Err(e) => log_error(&format!(
                            "Failed to read Lua script {:?}: {}",
                            script_path, e
                        )),
                    }
                }
            }
        }
        Err(_) => log_error("Failed to read mods directory"),
    }
}

// Creates a Lua editor widget
pub fn create_lua_editor(content: &str) -> (TextView, ScrolledWindow) {
    create_text_editor(content)
}

// Executes a Lua script
pub fn execute_lua_script(script_content: &str) {
    let lua = Lua::new();
    match lua.load(script_content).exec() {
        Ok(_) => log_info("Lua script executed successfully"),
        Err(err) => log_error(&format!("Failed to execute Lua script: {:?}", err)),
    }
}

// Runs the Lua script from the current editor content
pub fn run_lua_from_editor(text_view: &TextView) {
    let buffer = text_view.buffer();
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let script_content = buffer.text(&start, &end, true).to_string();
    execute_lua_script(&script_content);
}

// Runs a Lua script from the file specified in AppState
pub fn run_lua_script(state: &Arc<Mutex<AppState>>) {
    log_info("Running Lua script...");
    let state_lock = state.lock().unwrap();
    if let Some(script_path) = &state_lock.project_path {
        match fs::read_to_string(script_path) {
            Ok(script_content) => {
                drop(state_lock);
                execute_lua_script(&script_content);
            }
            Err(err) => log_error(&format!("Failed to read Lua script from file: {}", err)),
        }
    } else {
        log_error("No project file open. Please open or create a new project");
    }
}

// Placeholder for launching the Vulkan render window
fn launch_vulkan_render() {
    log_info("Vulkan render launch placeholder called");
}

