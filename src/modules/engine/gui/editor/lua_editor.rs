//src/modules/engine/gui/editor/lua_editor.rs

use crate::modules::engine::configuration::logger::{log_error, log_info, AppState};
use crate::modules::engine::gui::utils::execute_lua_script;
use gtk4::prelude::*;
use gtk4::PolicyType::Automatic;
use gtk4::WrapMode::Word;
use gtk4::{ScrolledWindow, TextBuffer, TextView};
use std::fs;
use std::sync::{Arc, Mutex};

// Create lua editor
pub fn create_lua_editor(content: &str, state: Arc<Mutex<AppState>>) -> ScrolledWindow {
    // Create TextBuffer
    let text_buffer = TextBuffer::new(None);
    text_buffer.set_text(content);
    text_buffer.set_enable_undo(true);

    // Keybinding for saving (ctrl + s)
    // TODO: add keybinding functionality

    // Create Textview with TextBuffer
    let text_view = TextView::with_buffer(&text_buffer);
    text_view.set_editable(true);
    text_view.set_wrap_mode(Word);

    // Signal to track changes
    {
        let state_clone = Arc::clone(&state);
        text_buffer.connect_changed(move |_| {
            let mut state_lock = state_clone.lock().unwrap();
            state_lock.is_modified = true;
        });
    }

    // Store TextView in AppState
    {
        let mut state_lock = state.lock().unwrap();
        state_lock.text_view = Some(text_view.clone());
    }

    // Create ScrolledWindow
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);
    scrolled_window.set_min_content_width(400);
    scrolled_window.set_min_content_height(300);
    scrolled_window.set_child(Some(&text_view));
    scrolled_window.set_policy(Automatic, Automatic);

    scrolled_window
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
