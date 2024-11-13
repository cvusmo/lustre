// src/modules/engine/gui/new_project.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_error, log_info, AppState};
use crate::modules::engine::gui::utils::{clear_project_area, load_project_content};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Button, Dialog, Label, Orientation};
use std::sync::{Arc, Mutex};
use std::{env, fs, path::PathBuf};

// Function to open the "New Project" dialog
pub fn open_new_project_dialog(state: &Arc<Mutex<AppState>>, parent: &Arc<ApplicationWindow>) {
    log_info(state, "Opening new project dialog...");

    // Create the dialog for "New Project"
    let dialog = Dialog::builder()
        .transient_for(parent.as_ref())
        .modal(true)
        .title("New Project")
        .build();

    // Set up content area in dialog
    let content_area = dialog.content_area();
    let dialog_box = GtkBox::new(Orientation::Vertical, 10);
    content_area.append(&dialog_box);

    // Add label to dialog
    let label = Label::new(Some("Choose an option to start a new project:"));
    dialog_box.append(&label);

    // "Create New Project" button
    let create_button = Button::with_label("Create New Project");
    dialog_box.append(&create_button);

    // "Create New Project" button action
    {
        let state = Arc::clone(state);
        let dialog_clone = dialog.clone();
        create_button.connect_clicked(move |_| {
            update_new_project(&state);
            log_info(&state, "New project created.");
            dialog_clone.close();
        });
    }

    dialog.show();
}

// Function to update the project area with a new text project
fn update_new_project(state: &Arc<Mutex<AppState>>) {
    let mut state_lock = state.lock().unwrap();

    // Set the default save path using the HOME environment variable
    let home_dir = env::var("HOME").expect("Failed to read HOME environment variable");
    let default_project_dir = PathBuf::from(format!("{}/gameengine/projects", home_dir));
    let default_project_file = default_project_dir.join("new_project.lua");

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&default_project_dir) {
        log_error(state, &format!("Failed to create default directory: {}", e));
        return;
    }

    // Set default project path in AppState
    state_lock.project_path = Some(default_project_file.clone());

    // Create default file if it doesn't exist
    if !default_project_file.exists() {
        if let Err(e) = fs::write(&default_project_file, "") {
            log_error(
                state,
                &format!("Failed to create default project file: {}", e),
            );
            return;
        }
    }

    // Clone the `project_area` and release the lock on state before making mutable changes
    let project_area = state_lock.project_area.clone();
    drop(state_lock); // Release the lock to prevent borrowing conflicts

    if let Some(ref project_area) = project_area {
        // Use the utility to clear the project area
        clear_project_area(&project_area);

        // Load empty content into the project area using the utility function
        load_project_content(state, "");
    }
}
