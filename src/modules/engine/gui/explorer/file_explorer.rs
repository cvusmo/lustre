// src/modules/engine/gui/explorer/file_explorer.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, FileChooserAction, FileChooserDialog, Label, ResponseType};
use std::sync::{Arc, Mutex};
use gtk4::glib;

// open file explorer
pub fn open_file_dialog(state: Arc<Mutex<AppState>>, parent: Arc<ApplicationWindow>) {
    let dialog = FileChooserDialog::builder()
        .title("Select a Text File")
        .transient_for(parent.as_ref())
        .action(FileChooserAction::Open)
        .modal(true)
        .build();

    // Add a filter to allow only text files
    let filter = gtk4::FileFilter::new();
    filter.add_pattern("*.txt");
    filter.set_name(Some("Text Files"));
    dialog.add_filter(&filter);

    // Add response buttons
    dialog.add_button("_Cancel", ResponseType::Cancel);
    dialog.add_button("_Open", ResponseType::Accept);

    let state_clone = Arc::clone(&state);
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                let file_path = file.path().expect("Failed to get file path");

                // Read the file content
                match std::fs::read_to_string(&file_path) {
                    Ok(content) => {
                        // Ensure UI updates run in the main context
                        glib::MainContext::default().spawn_local({
                            let state_clone = Arc::clone(&state_clone);
                            async move {
                                let state = state_clone.lock().unwrap();

                                // If project_area exists, update it with new content
                                if let Some(ref project_area) = state.project_area {
                                    // Clear the previous content
                                    while let Some(child) = project_area.first_child() {
                                        project_area.remove(&child);
                                    }

                                    // Display the new content
                                    let content_label = Label::new(Some(&content));
                                    project_area.append(&content_label);
                                    content_label.show(); // Make sure new label is visible
                                    project_area.show();  // Ensure project area is updated
                                }

                                // Log information
                                log_info(&state_clone, &format!("Opened file: {}", file_path.display()));
                            }
                        });
                    }
                    Err(err) => {
                        log_error(&state_clone, &format!("Failed to read file content: {}", err));
                    }
                }
            }
        }
        dialog.close();
    });

    dialog.show();
}

