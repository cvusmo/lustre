// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/modules/engine/gui/components/sidebar.rs

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Orientation, TextView};

// Create a sidebar with functional sections
pub fn create_sidebar() -> GtkBox {
    let sidebar = GtkBox::new(Orientation::Vertical, 5);
    sidebar.set_hexpand(false);
    sidebar.set_vexpand(true);

    // Top section of the sidebar - TextView for logs or information display
    let textview = TextView::new();
    textview.set_vexpand(true);
    textview.set_editable(false); // Make it a display-only area
    textview.set_cursor_visible(false);
    textview
        .buffer()
        .set_text("Logs/Information will appear here.");
    sidebar.append(&textview);

    // Bottom section of the sidebar - Buttons for actions
    let button1 = Button::with_label("Action 1");
    button1.connect_clicked(|_| {
        println!("Action 1 clicked");
    });
    sidebar.append(&button1);

    let button2 = Button::with_label("Action 2");
    button2.connect_clicked(|_| {
        println!("Action 2 clicked");
    });
    sidebar.append(&button2);

    let button3 = Button::with_label("Action 3");
    button3.connect_clicked(|_| {
        println!("Action 3 clicked");
    });
    sidebar.append(&button3);

    let button4 = Button::with_label("Action 4");
    button4.connect_clicked(|_| {
        println!("Action 4 clicked");
    });
    sidebar.append(&button4);

    let button5 = Button::with_label("Action 5");
    button5.connect_clicked(|_| {
        println!("Action 5 clicked");
    });
    sidebar.append(&button5);

    sidebar
}
