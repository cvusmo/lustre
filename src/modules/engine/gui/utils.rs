// src/modules/engine/gui/utils.rs
// github.com/cvusmo/gameengine

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, ScrolledWindow, TextView, MessageDialog, ButtonsType, MessageType, ResponseType, Application};
use std::sync::{Arc, Mutex};
use crate::modules::engine::configuration::logger::{log_error, AppState};
use crate::modules::engine::gui::save_file::save_file;

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
        let mut state_lock = state.lock().unwrap();
        state_lock.text_view = Some(text_view);
        state_lock.is_modified = false; // Since we are loading the content, mark it as not modified
    } else {
        log_error(state, "Project area not found to load content.");
    }
}

/// Function to handle exit with save prompt.
pub fn handle_exit(state: Arc<Mutex<AppState>>, app: &Application) {
    let state_lock = state.lock().unwrap();

    // Check if project has unsaved changes
    if state_lock.is_modified {
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
    } else {
        // No unsaved changes, exit directly
        app.quit();
    }
}

