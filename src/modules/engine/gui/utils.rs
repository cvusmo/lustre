// src/modules/engine/gui/utils.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_error, log_info, AppState};
use crate::modules::engine::gui::editor::lua_editor::create_lua_editor;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, ButtonsType, Dialog, FileChooserAction,
    FileChooserDialog, Label, MessageDialog, MessageType, Orientation, ResponseType, TextView,
};
use std::sync::{Arc, Mutex};
use std::{env, fs, path::PathBuf};

// Clears the contents of the given project area.
pub fn clear_project_area(project_area: &GtkBox) {
    while let Some(child) = project_area.first_child() {
        project_area.remove(&child);
    }
}

// Loads content into the project area by creating a new TextView and appending it.
pub fn load_project_area(state: &Arc<Mutex<AppState>>, content: &str) {
    let state_lock = state.lock().unwrap();

    // Clone the `project_area` and release the lock on state before making mutable changes
    if let Some(ref project_area) = state_lock.project_area {
        let project_area = project_area.clone();
        drop(state_lock);

        // Clear the previous content
        clear_project_area(&project_area);

        // Create a new lua editor
        let lua_editor = create_lua_editor(content, Arc::clone(state));

        // Append scrolled window to the project area
        project_area.append(&lua_editor);
        project_area.show();

        // Re-lock the state to update `text_view`
        if let Ok(mut state_lock) = state.lock() {
            state_lock.text_view =
                Some(lua_editor.child().unwrap().downcast::<TextView>().unwrap());
            state_lock.is_modified = false;
        }
    } else {
        log_error(state, "Project area not found to load content.");
    }
}

// Function update new project
pub fn update_new_project(state: &Arc<Mutex<AppState>>) {
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

    // Release the lock to prevent borrowing conflicts
    drop(state_lock);

    // use load_project_area to load empty content into project area
    load_project_area(state, "");
}

// Function to open the "New Project" dialog
pub fn open_new_project(state: &Arc<Mutex<AppState>>, parent: &Arc<ApplicationWindow>) {
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
    // dialog.set_current_name("new_project.lua");

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

// Helper function to execute lua script
pub fn execute_lua_script(state: &Arc<Mutex<AppState>>, script_content: &str) {
    // Get reference to Lua instance
    let lua = {
        let state_lock = state.lock().unwrap();
        state_lock.lua.clone()
    };

    let lua_lock = lua.lock().unwrap();

    // Load and execute Lua script
    match lua_lock.load(script_content).exec() {
        Ok(_) => log_info(state, "Lua script compiled and executed successfuly."),
        Err(err) => log_error(state, &format!("Failed to execute Lua script: {:?}", err)),
    }
}
