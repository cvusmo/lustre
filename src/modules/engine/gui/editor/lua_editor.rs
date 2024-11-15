//src/modules/engine/gui/editor/lua_editor.rs

use crate::modules::engine::configuration::logger::{log_error, log_info, AppState};
use crate::modules::engine::gui::utils::{create_text_editor, execute_lua_script};
use gtk4::prelude::*;
use gtk4::ScrolledWindow;
use std::fs;
use std::sync::{Arc, Mutex};

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
            log_error(state, "No text view found in current project.");
            return;
        }
    };

    // Use helper function to execute script
    execute_lua_script(state, &script_content);
}

// Runs lua script
pub fn run_lua_script(state: &Arc<Mutex<AppState>>) {
    log_info(state, "Running lua script...");

    let script_path = {
        let state_lock = state.lock().unwrap();
        if let Some(ref path) = state_lock.project_path {
            path.clone()
        } else {
            log_error(
                state,
                "No project file is open. Please open or create a new project.",
            );
            return;
        }
    };

    // Read Lua script from file
    let script_content = match fs::read_to_string(&script_path) {
        Ok(content) => content,
        Err(err) => {
            log_error(
                state,
                &format!("Failed to read lua script from file: {}", err),
            );
            return;
        }
    };

    // Use helper function to execute script
    execute_lua_script(state, &script_content);
}
