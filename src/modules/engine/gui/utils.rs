// src/modules/engine/gui/utils.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_error, log_info, AppState};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, ButtonsType, FileChooserAction,
    FileChooserDialog, MessageDialog, MessageType, ResponseType, ScrolledWindow, TextView,
};
use std::fs;
use std::sync::{Arc, Mutex};

/// Clears the contents of the given project area.
pub fn clear_project_area(project_area: &GtkBox) {
    while let Some(child) = project_area.first_child() {
        project_area.remove(&child);
    }
}

/// Loads content into the project area by creating a new TextView and appending it.
pub fn load_project_content(state: &Arc<Mutex<AppState>>, content: &str) {
    let state_lock = state.lock().unwrap();

    // Clone the `project_area` and release the lock on state before making mutable changes
    if let Some(ref project_area) = state_lock.project_area {
        let project_area = project_area.clone();
        drop(state_lock);

        // Clear the previous content
        clear_project_area(&project_area);

        // Create a new TextView with the provided content
        let text_view = TextView::new();
        text_view.set_editable(true);
        text_view.set_wrap_mode(gtk4::WrapMode::Word);
        text_view.buffer().set_text(content);

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_vexpand(true);
        scrolled_window.set_hexpand(true);
        scrolled_window.set_min_content_width(400);
        scrolled_window.set_min_content_height(300);
        scrolled_window.set_child(Some(&text_view));
        scrolled_window.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);

        // Append scrolled window to the project area
        project_area.append(&scrolled_window);
        project_area.show();

        // Re-lock the state to update `text_view`
        if let Ok(mut state_lock) = state.lock() {
            state_lock.text_view = Some(text_view);
            state_lock.is_modified = false; // Since we are loading the content, mark it as not modified
        }
    } else {
        log_error(state, "Project area not found to load content.");
    }
}

/// Function to save the current project to an existing file path.
pub fn save_file(state: &Arc<Mutex<AppState>>) {
    log_info(state, "Saving project...");

    let (path, text) = {
        // Lock state, read the necessary information, then drop the lock.
        let state_lock = state.lock().unwrap();
        if let Some(ref path) = state_lock.project_path {
            if let Some(ref text_view) = state_lock.text_view {
                let buffer = text_view.buffer();
                let start = buffer.start_iter();
                let end = buffer.end_iter();
                let text = buffer.text(&start, &end, true).to_string();
                (Some(path.clone()), text)
            } else {
                log_error(state, "No text view found in the current project.");
                (None, String::new())
            }
        } else {
            log_error(
                state,
                "No file path available. Use 'Save As...' to specify a location.",
            );
            (None, String::new())
        }
    };

    if let Some(path) = path {
        match fs::write(&path, text) {
            Ok(_) => {
                log_info(state, &format!("File saved: {}", path.display()));
                // Mark as not modified after a successful save
                if let Ok(mut state_lock) = state.lock() {
                    state_lock.is_modified = false;
                }
            }
            Err(err) => {
                log_error(state, &format!("Failed to save file: {}", err));
            }
        }
    }
}

/// Function to save the current project as a new file.
pub fn save_as_file(state: Arc<Mutex<AppState>>, parent: Arc<ApplicationWindow>) {
    log_info(&state, "Saving project as a new file...");

    let dialog = FileChooserDialog::builder()
        .title("Save File As")
        .transient_for(parent.as_ref())
        .action(FileChooserAction::Save)
        .modal(true)
        .build();

    dialog.add_button("_Cancel", ResponseType::Cancel);
    dialog.add_button("_Save", ResponseType::Accept);

    let state_clone = Arc::clone(&state);
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                let file_path = file.path().expect("Failed to get file path");

                {
                    // Lock the state and perform the save operation
                    let mut state = state_clone.lock().unwrap();
                    if let Some(ref text_view) = state.text_view {
                        let buffer = text_view.buffer();
                        let start = buffer.start_iter();
                        let end = buffer.end_iter();
                        let text = buffer.text(&start, &end, true);

                        match fs::write(&file_path, text) {
                            Ok(_) => {
                                log_info(
                                    &state_clone,
                                    &format!("File saved as: {}", file_path.display()),
                                );
                                state.project_path = Some(file_path);
                                state.is_modified = false; // Mark as not modified after saving
                            }
                            Err(err) => {
                                log_error(&state_clone, &format!("Failed to save file: {}", err));
                            }
                        }
                    } else {
                        log_error(&state_clone, "No text view found in the current project.");
                    }
                }
            }
        }
        dialog.close();
    });

    dialog.show();
}

/// Function to handle exit with save prompt.
pub fn handle_exit(state: Arc<Mutex<AppState>>, app: &Application) {
    // let state_lock = state.lock().unwrap();

    // Check if project has unsaved changes
    //if state_lock.is_modified {
    // Show message dialog prompting to save
    let dialog = MessageDialog::builder()
        .transient_for(app.active_window().as_ref().unwrap())
        .modal(true)
        .buttons(ButtonsType::YesNo)
        .text("Unsaved changes detected")
        .secondary_text("Do you want to save changes before exiting?")
        .message_type(MessageType::Warning)
        .build();

    let state_clone = Arc::clone(&state);
    let app_clone = app.clone();
    dialog.connect_response(move |dialog, response| {
        match response {
            ResponseType::Yes => {
                // Save file before exiting
                save_file(&state_clone);

                // Once saved, exit application
                app_clone.quit();
            }
            ResponseType::No => {
                // Exit without saving
                app_clone.quit();
            }
            _ => {
                // Do nothing, just close prompt dialog
                dialog.close();
            }
        }
    });

    dialog.show();
    //} else {
    // No unsaved changes, exit directly
    //app.quit();
    //}
}
