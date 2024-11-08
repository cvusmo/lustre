// src/modules/engine/gui/save_file.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};
use gtk4::prelude::*;
use std::fs;
use std::sync::{Arc, Mutex};

// Function to save the current project to an existing file path
pub fn save_file(state: &Arc<Mutex<AppState>>) {
    let state = state.lock().unwrap();
    if let Some(ref path) = state.project_path {
        // Save the content of the project area to the file
        if let Some(ref project_area) = state.project_area {
            if let Some(text_view) = project_area.first_child().and_then(|widget| widget.downcast::<gtk4::TextView>().ok()) {
                if let Some(buffer) = text_view.buffer() {
                    if let Some(text) = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true) {
                        if let Err(err) = fs::write(path, text) {
                            log_error(&state, &format!("Failed to save file: {}", err));
                        } else {
                            log_info(&state, &format!("File saved: {}", path.display()));
                        }
                    }
                }
            }
        }
    } else {
        log_error(&state, "No file path available. Use 'Save As...' to specify a location.");
    }
}

