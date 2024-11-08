// src/modules/engine/gui/new_project.rs
// github.com/cvusmo/gameengine

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Button, Dialog, FileChooserAction, FileChooserDialog, ResponseType, Box as GtkBox, Label, Orientation, TextView, ScrolledWindow};
use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};
use std::sync::{Arc, Mutex};
use std::env;
use std::fs;
use std::path::PathBuf;

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

    // "Select Saved Project" button
    let open_button = Button::with_label("Select Saved Project");
    dialog_box.append(&open_button);

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

    // "Select Saved Project" button action
    {
        let state = Arc::clone(state);
        let dialog_clone = dialog.clone();
        open_button.connect_clicked(move |_| {
            log_info(&state, "Opening file chooser to select saved project...");
            open_file_chooser(state.clone(), &dialog_clone);
            dialog_clone.close();
        });
    }

    dialog.show(); 
}

// Function to update the project area with a new text project
fn update_new_project(state: &Arc<Mutex<AppState>>) {
    let state_clone = Arc::clone(state);
    let mut state = state.lock().unwrap();

    // Set the default save path
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/home/user"));
    let default_project_dir = PathBuf::from(format!("{}/gamengine/projects", home_dir));
    let default_project_file = default_project_dir.join("new_project.txt");

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&default_project_dir) {
        log_error(&state_clone, &format!("Failed to create default directory: {}", e));
        return;
    }

    // Set default project path in AppState
    state.project_path = Some(default_project_file.clone());

    // Create default file if it doesn't exist
    if !default_project_file.exists() {
        if let Err(e) = fs::write(&default_project_file, "") {
            log_error(&state_clone, &format!("Failed to create default project file: {}", e));
            return;
        }
    }

    // Retrieve existing project_area
    if let Some(ref project_area) = state.project_area {
        // Clear previous content
        while let Some(child) = project_area.first_child() {
            project_area.remove(&child);
        }

        // Add a TextView for new project content
        let text_view = TextView::new();
        text_view.set_editable(true);
        text_view.set_wrap_mode(gtk4::WrapMode::Word);

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
    }
}

// Function to open file chooser dialog for selecting a saved project
fn open_file_chooser(state: Arc<Mutex<AppState>>, parent: &impl IsA<gtk4::Window>) {
    let dialog = FileChooserDialog::builder()
        .title("Select Project File")
        .transient_for(parent) 
        .action(FileChooserAction::Open)
        .modal(true)
        .build();

    let state_clone = Arc::clone(&state); 

    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                let file_path = file.path().expect("Failed to get file path");

                // Lock the state and update the project path
                let mut state = state_clone.lock().unwrap();
                state.project_path = Some(file_path.clone());

                // Log information
                log_info(&state_clone, &format!("Project path set to: {}", file_path.display()));
            }
        }
        dialog.close();
    });

    dialog.show();
}

