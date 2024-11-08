// src/modules/engine/gui/save_file.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};
use gtk4::prelude::*;
use std::fs;
use std::sync::{Arc, Mutex};
use gtk4::glib;

// Function to save the current project to an existing file path
pub fn save_file(state: &Arc<Mutex<AppState>>) {
    // Clone the state to ensure we can use it in the async context
    let state_clone = Arc::clone(state);

    // Spawn the save operation in the main context to avoid blocking the UI
    glib::MainContext::default().spawn_local(async move {
        // Lock the state to get the information needed for saving
        let state = state_clone.lock().unwrap();

        // Clone the path and text from the state to release the lock early
        if let Some(ref path) = state.project_path {
            if let Some(ref text_view) = state.text_view {
                let buffer = text_view.buffer();
                let start = buffer.start_iter();
                let end = buffer.end_iter();
                let text = buffer.text(&start, &end, true).to_string(); // Clone the text

                let path_clone = path.clone(); // Clone the path

                // Release the lock before performing file I/O to avoid blocking the UI
                drop(state);

                // Perform the file write operation
                match fs::write(&path_clone, text) {
                    Ok(_) => {
                        log_info(&state_clone, &format!("File saved: {}", path_clone.display()));
                    }
                    Err(err) => {
                        log_error(&state_clone, &format!("Failed to save file: {}", err));
                    }
                }
            } else {
                log_error(&state_clone, "No text view found in the current project.");
            }
        } else {
            log_error(&state_clone, "No file path available. Use 'Save As...' to specify a location.");
        }
    });
}

