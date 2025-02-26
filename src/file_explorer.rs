// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/file_explorer.rs

use crate::lua_editor::create_lua_editor;
use crate::state::AppState;
use crate::state::*;
use crate::utils::load_project_area;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{FileChooserAction, FileChooserDialog, ResponseType};
use std::fs;
use std::sync::{Arc, Mutex};

// Unified function to open a file and load it into the project area
pub fn open_file(state: Arc<Mutex<AppState>>, parent: &impl IsA<gtk4::Window>) {
    let dialog = FileChooserDialog::builder()
        .title("Select a Project File")
        .transient_for(parent)
        .modal(true)
        .action(FileChooserAction::Open)
        .build();

    // Ensure the dialog remains in front until it is closed
    dialog.set_modal(true);
    dialog.set_transient_for(Some(parent));

    // Add a filter to allow only text files
    let filter = gtk4::FileFilter::new();
    filter.add_pattern("*.lua");
    filter.add_pattern("*.txt");
    filter.set_name(Some("Project Files"));
    dialog.add_filter(&filter);

    // Add response buttons
    dialog.add_button("_Cancel", ResponseType::Cancel);
    dialog.add_button("_Open", ResponseType::Accept);

    let state_clone = Arc::clone(&state);
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                let file_path = file.path().expect("Failed to get file path");

                // Clone the state to use in async operation
                let state_clone_inner = Arc::clone(&state_clone);

                glib::MainContext::default().spawn_local(async move {
                    // Read the file content asynchronously
                    match fs::read_to_string(&file_path) {
                        Ok(content) => {
                            {
                                let mut state_lock = state_clone_inner.lock().unwrap();
                                state_lock.project_path = Some(file_path.clone());
                            }

                            // Load project area
                            load_project_area(&state_clone_inner, &content, create_lua_editor);

                            // Log information
                            log_info(&format!("Project path set to: {}", file_path.display()));
                        }
                        Err(err) => {
                            log_error(&format!("Failed to read file content: {}", err));
                        }
                    }
                });
            }
        }
        dialog.close();
    });

    dialog.show();
}
