// src/modules/engine/gui/new_project.rs
// github.com/cvusmo/gameengine

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Button, Dialog, FileChooserAction, 
    FileChooserDialog, ResponseType, Box as GtkBox, Label, Orientation};
use crate::modules::engine::configuration::logger::{log_info, AppState};
use crate::modules::engine::gui::window::update_project_area;
use std::sync::{Arc, Mutex};

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
            update_project_area(&state);
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
        });
    }

    dialog.show(); 
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

