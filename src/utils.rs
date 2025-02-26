// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre

// src/modules/engine/gui/utils.rs
// github.com/cvusmo/gameengine

use crate::state::{log_error, log_info, AppState};
use glib::source::timeout_add_local;
use gtk4::prelude::*;
use gtk4::WrapMode::Word;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, ButtonsType, FileChooserAction,
    FileChooserDialog, MessageDialog, MessageType, PolicyType::Automatic, ResponseType,
    ScrolledWindow, TextBuffer, TextView,
};
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Create text editor
pub fn create_text_editor(content: &str, state: &Arc<Mutex<AppState>>) -> ScrolledWindow {
    // Create TextBuffer
    let text_buffer = TextBuffer::new(None);
    text_buffer.set_text(content);
    text_buffer.set_enable_undo(true);

    // Create TextView with TextBuffer
    let text_view = TextView::with_buffer(&text_buffer);
    text_view.set_editable(true);
    text_view.set_focusable(true);
    text_view.set_wrap_mode(Word);
    text_view.set_visible(true);
    text_view.show();

    log_info("TextView created and set to be editable and focusable.");

    // Signal to track changes in buffer and debounce
    {
        let state_clone = Arc::clone(state);
        let debounce_interval = Duration::from_millis(100);
        text_buffer.connect_changed(move |_| {
            // Debounce change signal
            let state_clone_inner = Arc::clone(&state_clone);
            timeout_add_local(debounce_interval, move || {
                // Lock
                {
                    let mut state_lock = state_clone_inner.lock().unwrap();
                    state_lock.is_modified = true;
                }

                // Log the change
                log_info("TextView content changed.");

                // Use ControlFlow::Break to stop further invocation of the closure
                glib::ControlFlow::Break
            });
        });
    }

    // Store TextView in AppState
    {
        let mut state_lock = state.lock().unwrap();
        state_lock.text_view = Some(text_view.clone());
    }

    // Create ScrolledWindow and add TextView
    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);
    scrolled_window.set_min_content_width(400);
    scrolled_window.set_min_content_height(300);
    scrolled_window.set_child(Some(&text_view));
    scrolled_window.set_policy(Automatic, Automatic);
    scrolled_window.set_focusable(true);
    scrolled_window.set_visible(true);
    scrolled_window.show();

    log_info("ScrolledWindow created, set to visible, and focusable.");

    scrolled_window
}

// Clears the contents of the given project area.
pub fn clear_project_area(project_area: &GtkBox) {
    while let Some(child) = project_area.first_child() {
        project_area.remove(&child);
    }
}

// Loads content into the project area by creating a new TextView and appending it.
pub fn load_project_area<F>(state: &Arc<Mutex<AppState>>, content: &str, create_editor: F)
where
    F: Fn(&str, &Arc<Mutex<AppState>>) -> ScrolledWindow,
{
    let state_lock = state.lock().unwrap();

    // Clone project_area and release lock
    if let Some(ref project_area) = state_lock.project_area {
        let project_area = project_area.clone();
        drop(state_lock);

        // Clear previous content
        log_info("Clearing previous content...");
        clear_project_area(&project_area);

        // Create new editor
        log_info("Creating new text editor...");
        let editor = create_editor(content, state);

        // Append scrolled window
        log_info("Appending editor to project area...");
        project_area.append(&editor);

        // Request focus for text view
        let text_view = editor.child().unwrap().downcast::<TextView>().unwrap();
        text_view.grab_focus();
        project_area.show();

        // Re-lock state and update the text_view
        if let Ok(mut state_lock) = state.lock() {
            state_lock.text_view = Some(editor.child().unwrap().downcast::<TextView>().unwrap());
            state_lock.is_modified = false;
        }
    } else {
        log_error("Project area not found to load content.");
    }
}

/// Function to save the current project to an existing file path.
pub fn save_file(state: &Arc<Mutex<AppState>>) {
    log_info("Saving project...");

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
                log_error("No text view found in the current project.");
                (None, String::new())
            }
        } else {
            log_error("No file path available. Use 'Save As...' to specify a location.");
            (None, String::new())
        }
    };

    if let Some(path) = path {
        match fs::write(&path, text) {
            Ok(_) => {
                log_info(&format!("File saved: {}", path.display()));
                // Mark as not modified after a successful save
                if let Ok(mut state_lock) = state.lock() {
                    state_lock.is_modified = false;
                }
            }
            Err(err) => {
                log_error(&format!("Failed to save file: {}", err));
            }
        }
    }
}

/// Function to save the current project as a new file.
pub fn save_as_file(state: Arc<Mutex<AppState>>, parent: Arc<ApplicationWindow>) {
    log_info("Saving project as a new file...");

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
                                log_info(&format!("File saved as: {}", file_path.display()));
                                state.project_path = Some(file_path);
                                state.is_modified = false; // Mark as not modified after saving
                            }
                            Err(err) => {
                                log_error(&format!("Failed to save file: {}", err));
                            }
                        }
                    } else {
                        log_error("No text view found in the current project.");
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
        Ok(_) => log_info("Lua script compiled and executed successfuly."),
        Err(err) => log_error(&format!("Failed to execute Lua script: {:?}", err)),
    }
}
