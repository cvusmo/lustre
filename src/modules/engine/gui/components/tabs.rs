// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/modules/engine/gui/components/tabs.rs

// Create window tabs
pub fn create_tabs() -> Notebook {
    let notebook = Notebook::new();
    let window_a_label = Label::new(Some("WindowA Content"));
    let window_b_label = Label::new(Some("WindowB Content"));

    notebook.append_page(&window_a_label, Some(&Label::new(Some("WindowA"))));
    notebook.append_page(&window_b_label, Some(&Label::new(Some("WindowB"))));

    notebook
}
