// src/modules/engine/gui/utils.rs
// github.com/cvusmo/gameengine

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, ScrolledWindow, TextView};
use std::sync::{Arc, Mutex};
use crate::modules::engine::configuration::logger::{log_info, log_error, AppState};

/// Clears the contents of the given project area.
pub fn clear_project_area(project_area: &GtkBox) {
    while let Some(child) = project_area.first_child() {
        project_area.remove(&child);
    }
}

/// Loads content into the project area by creating a new TextView and appending it.
pub fn load_project_content(state: &Arc<Mutex<AppState>>, content: &str) {
    let mut state_lock = state.lock().unwrap();

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
    } else {
        log_error(state, "Project area not found to load content.");
    }
}

