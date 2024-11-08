// src/modules/engine/gui/saveas_file.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, FileChooserAction, FileChooserDialog, ResponseType};
use std::fs;
use std::sync::{Arc, Mutex};

// Function to save the current project as a new file
pub fn save_as_file(state: Arc<Mutex<AppState>>, parent: Arc<ApplicationWindow>) {
    let dialog = FileChooserDialog::builder()
        .title("Save File As")
        .transient_for(parent.as_ref())
        .action(FileChooserAction::Save)
        .modal(true)
        .build();

    // Add response buttons
    dialog.add_button("_Cancel", ResponseType::Cancel);
    dialog.add_button("_Save", ResponseType::Accept);

    let state_clone = Arc::clone(&state);
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                let file_path = file.path().expect("Failed to get file path");

                // Save the content of the project area to the file
                let state_clone_inner = Arc::clone(&state_clone);
                let state = state_clone.lock().unwrap();
                
                if let Some(ref text_view) = state.text_view {
                    let buffer = text_view.buffer(); // Directly get the buffer (it's not an Option)
                    let start = buffer.start_iter();
                    let end = buffer.end_iter();
                    let text = buffer.text(&start, &end, true);
                    
                    if let Err(err) = fs::write(&file_path, text) {
                        log_error(&state_clone_inner, &format!("Failed to save file: {}", err));
                    } else {
                        log_info(&state_clone_inner, &format!("File saved as: {}", file_path.display()));

                        // Update the state with the new project path
                        drop(state); // Unlock before acquiring mutable lock
                        let mut state = state_clone_inner.lock().unwrap();
                        state.project_path = Some(file_path);
                    }
                }
            }
        }
        dialog.close();
    });

    dialog.show();
}

