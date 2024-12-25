// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/modules/engine/gui/components/tabs.rs

use crate::modules::engine::configuration::logger::AppState;
use crate::modules::engine::gui::editor::lua_editor::create_lua_editor;
use crate::modules::engine::gui::utils::create_text_editor;
use gtk4::prelude::*;
use gtk4::{Label, Notebook, ScrolledWindow};
use std::sync::{Arc, Mutex};

/// Create window tabs
pub fn create_tabs(state: &Arc<Mutex<AppState>>) -> Notebook {
    let notebook = Notebook::new();

    // Function to add a new tab
    let add_tab = |notebook: &Notebook, content: ScrolledWindow, tab_name: &str| {
        let tab_label = Label::new(Some(tab_name));
        notebook.append_page(&content, Some(&tab_label));
        notebook.set_tab_detachable(&content, true);
        notebook.set_tab_reorderable(&content, true);
        content.show();
    };

    // Add "Project Area" Tab
    let project_area = create_text_editor("Welcome to the Project Area", state);
    add_tab(&notebook, project_area, "Project Area");

    // Add "Lua Editor" Tab
    let lua_editor = create_lua_editor("print(\"Hello, World!\")", state);
    add_tab(&notebook, lua_editor, "Lua Editor");

    notebook
}
