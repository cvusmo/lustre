// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/engine/ui/file_explorer.rs

use crate::engine::ui::lua_editor::create_lua_editor;
use crate::engine::ui::utils::load_project_area;
use crate::state::{log_error, log_info, AppState};
use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Box as GtkBox, FileChooserAction, FileChooserDialog, FileFilter,
    ResponseType,
};
use std::fs;
use std::sync::{Arc, Mutex};

// Opens a file and loads it into the project area
pub fn open_file(state: Arc<Mutex<AppState>>, parent: &ApplicationWindow, project_area: &GtkBox) {
    let dialog = FileChooserDialog::builder()
        .title("Select a Project File")
        .transient_for(parent)
        .modal(true)
        .action(FileChooserAction::Open)
        .build();

    let filter = FileFilter::new();
    filter.add_pattern("*.lua");
    filter.add_pattern("*.txt");
    filter.set_name(Some("Project Files"));
    dialog.add_filter(&filter);

    dialog.add_button("_Cancel", ResponseType::Cancel);
    dialog.add_button("_Open", ResponseType::Accept);

    let state_clone = Arc::clone(&state);
    let project_area_clone = project_area.clone();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            if let Some(file) = dialog.file() {
                let file_path = file.path().expect("Failed to get file path");
                match fs::read_to_string(&file_path) {
                    Ok(content) => {
                        let mut state_lock = state_clone.lock().unwrap();
                        state_lock.project_path = Some(file_path.clone());
                        drop(state_lock);
                        load_project_area(
                            &state_clone,
                            &content,
                            &project_area_clone,
                            create_lua_editor,
                        );
                        log_info(&format!("Project loaded: {}", file_path.display()));
                    }
                    Err(err) => log_error(&format!("Failed to read file: {}", err)),
                }
            }
        }
        dialog.close();
    });

    dialog.show();
}
