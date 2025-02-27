// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/engine/ui/utils.rs

use crate::state::{log_error, log_info, AppState};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, ButtonsType, FileChooserAction,
    FileChooserDialog, MessageDialog, MessageType, PolicyType::Automatic, ResponseType,
    ScrolledWindow, TextBuffer, TextView, WrapMode,
};
use std::fs;
use std::sync::{Arc, Mutex};

// Creates a text editor widget with the given content
pub fn create_text_editor(content: &str) -> (TextView, ScrolledWindow) {
    let buffer = TextBuffer::new(None);
    buffer.set_text(content);
    buffer.set_enable_undo(true);

    let text_view = TextView::with_buffer(&buffer);
    text_view.set_editable(true);
    text_view.set_focusable(true);
    text_view.set_wrap_mode(WrapMode::Word);
    text_view.set_visible(true);
    log_info("TextView created: editable and focusable");

    let scrolled_window = ScrolledWindow::new();
    scrolled_window.set_vexpand(true);
    scrolled_window.set_hexpand(true);
    scrolled_window.set_min_content_width(400);
    scrolled_window.set_min_content_height(300);
    scrolled_window.set_child(Some(&text_view));
    scrolled_window.set_policy(Automatic, Automatic);
    scrolled_window.set_focusable(true);
    scrolled_window.set_visible(true);
    log_info("ScrolledWindow created: visible and focusable");

    (text_view, scrolled_window)
}

// Clears all children from a GtkBox
pub fn clear_project_area(project_area: &GtkBox) {
    log_info("Clearing previous content...");
    while let Some(child) = project_area.first_child() {
        project_area.remove(&child);
    }
}

// Loads content into a project area by creating and appending a new editor
pub fn load_project_area<F>(
    _state: &Arc<Mutex<AppState>>,
    content: &str,
    project_area: &GtkBox,
    create_editor: F,
) -> TextView
where
    F: Fn(&str) -> (TextView, ScrolledWindow),
{
    clear_project_area(project_area);
    log_info("Creating new text editor...");
    let (text_view, scrolled_window) = create_editor(content);
    log_info("Appending editor to project area...");
    project_area.append(&scrolled_window);
    text_view.grab_focus();
    project_area.show();
    text_view
}

// Saves content from a TextView to an existing file path
pub fn save_file(text_view: &TextView, state: &Arc<Mutex<AppState>>) {
    log_info("Saving project...");

    let state_lock = state.lock().unwrap();
    let path = state_lock.project_path.clone();
    drop(state_lock);

    if let Some(path) = path {
        let buffer = text_view.buffer();
        let start = buffer.start_iter();
        let end = buffer.end_iter();
        let text = buffer.text(&start, &end, true).to_string();
        match fs::write(&path, &text) {
            Ok(_) => {
                log_info(&format!("File saved: {}", path.display()));
                let mut state_lock = state.lock().unwrap();
                state_lock.is_modified = false;
            }
            Err(err) => log_error(&format!("Failed to save file: {}", err)),
        }
    } else {
        log_error("No file path available. Use 'Save As...' to specify a location.");
    }
}

// Saves content from a TextView to a new file via a file chooser dialog
pub fn save_as_file(
    text_view: &TextView,
    state: Arc<Mutex<AppState>>,
    parent: &ApplicationWindow, // Changed from Arc<ApplicationWindow> to &ApplicationWindow
) {
    log_info("Saving project as a new file...");

    let dialog = FileChooserDialog::builder()
        .title("Save File As")
        .transient_for(parent) // Updated to use &ApplicationWindow
        .action(FileChooserAction::Save)
        .modal(true)
        .build();

    dialog.add_button("_Cancel", ResponseType::Cancel);
    dialog.add_button("_Save", ResponseType::Accept);

    let state_clone = Arc::clone(&state);
    let text_view_clone = text_view.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                let file_path = file.path().expect("Failed to get file path");
                let buffer = text_view_clone.buffer();
                let start = buffer.start_iter();
                let end = buffer.end_iter();
                let text = buffer.text(&start, &end, true);
                match fs::write(&file_path, text) {
                    Ok(_) => {
                        log_info(&format!("File saved as: {}", file_path.display()));
                        let mut state = state_clone.lock().unwrap();
                        state.project_path = Some(file_path);
                        state.is_modified = false;
                    }
                    Err(err) => log_error(&format!("Failed to save file: {}", err)),
                }
            }
        }
        dialog.close();
    });

    dialog.show();
}

// Prompts to save changes before exiting the application
pub fn handle_exit(text_view: &TextView, state: Arc<Mutex<AppState>>, app: &Application) {
    let state_lock = state.lock().unwrap();
    if !state_lock.is_modified {
        app.quit();
        return;
    }
    drop(state_lock);

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
    let text_view_clone = text_view.clone();
    dialog.connect_response(move |dialog, response| match response {
        ResponseType::Yes => {
            save_file(&text_view_clone, &state_clone);
            app_clone.quit();
        }
        ResponseType::No => app_clone.quit(),
        _ => dialog.close(),
    });

    dialog.show();
}
