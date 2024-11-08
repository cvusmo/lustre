// src/modules/engine/gui/save_file.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};
use gtk4::prelude::*;
use std::fs;
use std::sync::{Arc, Mutex};

// Function to save the current project to an existing file path
pub fn save_file(state: &Arc<Mutex<AppState>>) {
    let state_clone = Arc::clone(state); // Cloning the original Arc to pass to logging functions
    let state = state.lock().unwrap();
    if let Some(ref path) = state.project_path {
        // Save the content of the project area to the file
        if let Some(ref project_area) = state.project_area {
            if let Some(text_view) = project_area.first_child().and_then(|widget| widget.downcast::<gtk4::TextView>().ok()) {
                if let Some(buffer) = text_view.buffer() {
                    let start = buffer.start_iter();
                    let end = buffer.end_iter();
                    let text = buffer.text(&start, &end, true);
                    if let Err(err) = fs::write(path, text) {
                        log_error(&state_clone, &format!("Failed to save file: {}", err));
                    } else {
                        log_info(&state_clone, &format!("File saved: {}", path.display()));
                    }
                }
            }
        }
    } else {
        log_error(&state_clone, "No file path available. Use 'Save As...' to specify a location.");
    }
}

