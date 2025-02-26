// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
//src/lua_editor.rs

use crate::state::{log_error, log_info, AppState};
use crate::utils::{create_text_editor, execute_lua_script};
use crate::window::lustre_window;
use gtk4::prelude::*;
use gtk4::ScrolledWindow;
use mlua::prelude::*;
use std::fs;
use std::sync::{Arc, Mutex};

/// Registers basic Lua functions (e.g. print_message) with the provided Lua context.
pub fn register_lua_functions(lua: &Lua, _state: Arc<Mutex<AppState>>) -> LuaResult<()> {
    let print_message = lua.create_function(move |_, message: String| {
        log_info(&message);
        Ok(())
    })?;
    lua.globals().set("print_message", print_message)?;
    Ok(())
}

/// Registers the Vulkan render function so that Lua can trigger it.
fn register_render_functions(lua: &Lua) -> LuaResult<()> {
    let launch_fn = lua.create_function(|_, ()| {
        launch_vulkan_render();
        Ok(())
    })?;
    lua.globals().set("launch_render", launch_fn)?;
    Ok(())
}

/// Combined registration function that calls both register_lua_functions and register_render_functions.
pub fn register_all(lua: &Lua, state: Arc<Mutex<AppState>>) -> LuaResult<()> {
    // Register basic functions.
    register_lua_functions(lua, state.clone())?;
    // Register the Vulkan render trigger.
    register_render_functions(lua)?;
    Ok(())
}

/// Loads additional Lua modules from the "./mods" directory.
pub fn load_mods(lua: &Lua, _state: &Arc<Mutex<AppState>>) {
    let mod_path = "./mods";
    if let Ok(paths) = fs::read_dir(mod_path) {
        for entry in paths.flatten() {
            let script_path = entry.path();
            if script_path.extension().and_then(|s| s.to_str()) == Some("lua") {
                if let Ok(script_content) = fs::read_to_string(&script_path) {
                    if let Err(e) = lua.load(&script_content).exec() {
                        log_error(&format!(
                            "Failed to load Lua script {:?}: {}",
                            script_path, e
                        ));
                    }
                }
            }
        }
    } else {
        log_error("Failed to read mods directory.");
    }
}

/// Creates a Lua editor widget.
pub fn create_lua_editor(content: &str, state: &Arc<Mutex<AppState>>) -> ScrolledWindow {
    create_text_editor(content, state)
}

/// Runs a Lua script that is currently in the editor.
pub fn run_lua_from_editor(state: &Arc<Mutex<AppState>>) {
    let script_content = {
        // Lock state, read text view, then drop lock.
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

    // Execute the Lua script.
    execute_lua_script(state, &script_content);
}

/// Runs a Lua script from a file specified in the AppState.
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

    let script_content = match fs::read_to_string(&script_path) {
        Ok(content) => content,
        Err(err) => {
            log_error(&format!("Failed to read lua script from file: {}", err));
            return;
        }
    };

    execute_lua_script(state, &script_content);
}

/// Launches the Vulkan render window.
pub fn launch_vulkan_render() {
    lustre_window();
}
