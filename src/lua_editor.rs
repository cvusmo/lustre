// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre

//src/modules/engine/gui/editor/lua_editor.rs

use crate::state::{log_error, log_info, AppState};
use crate::utils::{create_text_editor, execute_lua_script};
use gtk4::prelude::*;
use gtk4::ScrolledWindow;
use mlua::prelude::*;
use std::fs;
use std::sync::{Arc, Mutex};

// Function to expose Lua
pub fn register_lua_functions(lua: &Lua, _state: Arc<Mutex<AppState>>) -> LuaResult<()> {
    let print_message = lua.create_function(move |_, message: String| {
        log_info(&message);
        Ok(())
    })?;

    // Register the function in the global Lua environment
    lua.globals().set("print_message", print_message)?;
    Ok(())
}

// Function to define Lua bindings
pub fn load_mods(lua: &Lua, _state: &Arc<Mutex<AppState>>) {
    let mod_path = "./mods";
    let paths = std::fs::read_dir(mod_path).unwrap();

    for path in paths {
        let script_path = path.unwrap().path();
        if script_path.extension().and_then(std::ffi::OsStr::to_str) == Some("lua") {
            let script_content = std::fs::read_to_string(&script_path).unwrap();
            if let Err(e) = lua.load(&script_content).exec() {
                log_error(&format!(
                    "Failed to load Lua script {:?}: {}",
                    script_path, e
                ));
            }
        }
    }
}

// Create lua editor
pub fn create_lua_editor(content: &str, state: &Arc<Mutex<AppState>>) -> ScrolledWindow {
    create_text_editor(content, state)
}

// Run lua script from editor
pub fn run_lua_from_editor(state: &Arc<Mutex<AppState>>) {
    let script_content = {
        // Lock state, read text view, then drop lock
        let state_lock = state.lock().unwrap();
        if let Some(ref text_view) = state_lock.text_view {
            let buffer = text_view.buffer();
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            buffer.text(&start, &end, true).to_string()
        } else {
            log_error("No text view found in current project.");
            return;
        }
    };

    // Use helper function to execute script
    execute_lua_script(state, &script_content);
}

// Runs lua script
pub fn run_lua_script(state: &Arc<Mutex<AppState>>) {
    log_info("Running lua script...");

    let script_path = {
        let state_lock = state.lock().unwrap();
        if let Some(ref path) = state_lock.project_path {
            path.clone()
        } else {
            log_error("No project file is open. Please open or create a new project.");
            return;
        }
    };

    // Read Lua script from file
    let script_content = match fs::read_to_string(&script_path) {
        Ok(content) => content,
        Err(err) => {
            log_error(&format!("Failed to read lua script from file: {}", err));
            return;
        }
    };

    // Use helper function to execute script
    execute_lua_script(state, &script_content);
}
