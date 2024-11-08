// src/modules/engine/gui/explorer/file_explorer.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, FileChooserAction, FileChooserDialog, TextView, ScrolledWindow, ResponseType};
use std::sync::{Arc, Mutex};
use std::fs;

// Unified function to open a file and load it into the project area
pub fn open_file(state: Arc<Mutex<AppState>>, parent: &impl IsA<gtk4::Window>) {
    let dialog = FileChooserDialog::builder()
        .title("Select a Text File")
        .transient_for(parent) 
        .modal(true)
        .action(FileChooserAction::Open)
        .build();

    // Ensure the dialog remains in front until it is closed
    dialog.set_modal(true);
    dialog.set_transient_for(Some(parent));

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
                if let Ok(content) = fs::read_to_string(&file_path) {
                    let mut state_lock = state_clone.lock().unwrap();

                    // Update the project path
                    state_lock.project_path = Some(file_path.clone());

                    // Clone the `project_area` and release the lock on state before making mutable changes
                    let project_area = state_lock.project_area.clone();
                    drop(state_lock); 

                    // Update the project area with the file content
                    if let Some(ref project_area) = project_area {
                        // Clear the previous content
                        while let Some(child) = project_area.first_child() {
                            project_area.remove(&child);
                        }

                        // Add a TextView with the loaded content
                        let text_view = TextView::new();
                        text_view.set_editable(true);
                        text_view.set_wrap_mode(gtk4::WrapMode::Word);
                        text_view.buffer().set_text(&content);

                        let scrolled_window = ScrolledWindow::new();
                        scrolled_window.set_vexpand(true);
                        scrolled_window.set_hexpand(true);
                        scrolled_window.set_min_content_width(400);
                        scrolled_window.set_min_content_height(300);
                        scrolled_window.set_child(Some(&text_view));
                        scrolled_window.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);

                        // Append scrolled window
                        project_area.append(&scrolled_window);
                        project_area.show();

                        // Re-lock the state to update `text_view`
                        let mut state_lock = state_clone.lock().unwrap();
                        state_lock.text_view = Some(text_view);

                        // Log information
                        log_info(&state_clone, &format!("Project path set to: {}", file_path.display()));
                    }
                } else {
                    log_error(&state_clone, &format!("Failed to read file content from: {}", file_path.display()));
                }
            }
        }
        dialog.close();
    });

    dialog.show();
}

