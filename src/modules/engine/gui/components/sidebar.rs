// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/modules/engine/gui/components/sidebar.rs

// Create sidebar
pub fn create_sidebar() -> GtkBox {
    let sidebar = GtkBox::new(Orientation::Vertical, 5);
    sidebar.set_hexpand(false);
    sidebar.set_vexpand(true);

    // Top section of the sidebar - TextView
    let textview = TextView::new();
    textview.set_vexpand(true);
    sidebar.append(&textview);

    // Bottom section of the sidebar - Button4
    let button4 = Button::with_label("BUTTON4");
    button4.set_vexpand(true);
    sidebar.append(&button4);

    sidebar
}
