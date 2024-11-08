// src/modules/engine/gui/menu_bar.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};
use crate::modules::engine::gui::new_project::open_new_project_dialog;

use gtk4::prelude::*;
use gtk4::{Align, ApplicationWindow, Button, Box as GtkBox, 
    FileChooserAction, FileChooserDialog, Label, MenuButton, Orientation, Popover, ResponseType};
use std::sync::{Arc, Mutex};

pub fn create_menu_bar(state: &Arc<Mutex<AppState>>, parent: &Arc<ApplicationWindow>) -> GtkBox {
    log_info(state, "Creating menu bar...");

    // Create horizontal menu bar container
    let menu_bar = GtkBox::new(Orientation::Horizontal, 5);
    menu_bar.set_halign(Align::Start);
    menu_bar.add_css_class("menu-bar");

    // File Button
    let file_button = MenuButton::builder()
        .label("File")
        .build();
    file_button.add_css_class("menu-button");
    let file_popover = Popover::new();
    let file_box = GtkBox::new(Orientation::Vertical, 5);

    // Create new project
    let new_button = Button::with_label("New");
    file_box.append(&new_button);

    // Connect New project to dialog function
    let state_clone = Arc::clone(state);
    let parent_clone = Arc::clone(parent);
    new_button.connect_clicked(move |_| {
        open_new_project_dialog(&state_clone, &parent_clone);
    });

    // TODO: Add Open saved file
    let open_button = Button::with_label("Open");
    file_box.append(&open_button);
    let state_clone_open = Arc::clone(state);
    let parent_clone_open = Arc::clone(parent);
    open_button.connect_clicked(move |_| {
        let dialog = FileChooserDialog::builder()
            .title("Select a Text File")
            .transient_for(parent_clone_open.as_ref())
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

        let state_clone_dialog = Arc::clone(&state_clone_open);
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    let file_path = file.path().expect("Failed to get file path");

                    // Read the file content
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        let mut state = state_clone_dialog.lock().unwrap();

                        // If project_area exists, update it with new content
                        if let Some(ref project_area) = state.project_area {
                            // Clear the previous content
                            while let Some(child) = project_area.first_child() {
                                project_area.remove(&child);
                            }

                            // Display the new content
                            let content_label = Label::new(Some(&content));
                            project_area.append(&content_label);

                            project_area.show();
                        }

                        // Log information
                        log_info(&state_clone_dialog, &format!("Opened file: {}", file_path.display()));
                    } else {
                        log_error(&state_clone_dialog, "Failed to read file content");
                    }
                }
            }
            dialog.close();
        });

        dialog.show();
    });

    // TODO: Add Exit with a prompt to check if saved
    file_box.append(&Label::new(Some("Exit")));
    file_popover.set_child(Some(&file_box));
    file_button.set_popover(Some(&file_popover));

    // Edit button
    let edit_button = MenuButton::builder()
        .label("Edit")
        .build();
    edit_button.add_css_class("menu-button");
    let edit_popover = Popover::new();
    let edit_box = GtkBox::new(Orientation::Vertical, 5);
    edit_box.append(&Label::new(Some("Undo")));
    edit_box.append(&Label::new(Some("Redo")));
    edit_box.append(&Label::new(Some("Preferences")));
    edit_popover.set_child(Some(&edit_box));
    edit_button.set_popover(Some(&edit_popover));

    // Add buttons to menu bar
    menu_bar.append(&file_button);
    menu_bar.append(&edit_button);

    log_info(state, "Menu bar created successfully.");
    menu_bar
}
