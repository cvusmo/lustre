// src/modules/engine/gui/new_project.rs
// github.com/cvusmo/gameengine

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Button, Dialog, FileChooserAction, 
    FileChooserDialog, ResponseType, Box as GtkBox, Label, Orientation, TextView, ScrolledWindow};
use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};
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

    // Set the default save path
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/home/user"));
    let default_project_dir = PathBuf::from(format!("{}/gamengine/projects", home_dir));
    let default_project_file = default_project_dir.join("new_project.txt");

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
            log_error(state, &format!("Failed to create default project file: {}", e));
            return;
        }
    }

    // Clone the `project_area` and release the lock on state before making mutable changes
    let project_area = state_lock.project_area.clone();

    drop(state_lock); // Release the lock to prevent borrowing conflicts

    if let Some(ref project_area) = project_area {
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

        // Re-lock the state to update `text_view`
        let mut state_lock = state.lock().unwrap();
        state_lock.text_view = Some(text_view);
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

                // Read the file content
                if let Ok(content) = fs::read_to_string(&file_path) {
                    let mut state_lock = state_clone.lock().unwrap();

                    // Update the project path
                    state_lock.project_path = Some(file_path.clone());

                    // Clone the `project_area` and release the lock on state before making mutable changes
                    let project_area = state_lock.project_area.clone();
                    drop(state_lock); // Release the lock to prevent borrowing conflicts

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

