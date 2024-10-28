// src/modules/engine/gui/menu_bar.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, AppState};
use gtk4::prelude::*;
use gtk4::{MenuButton, Popover, Label, Box as GtkBox, Align, Orientation};
use std::sync::{Arc, Mutex};

pub fn create_menu_bar(state: &Arc<Mutex<AppState>>) -> GtkBox {
    log_info(state, "Creating menu bar...");

    // Create horizontal menu bar container
    let menu_bar = GtkBox::new(Orientation::Horizontal, 5);
    menu_bar.set_halign(Align::Start);

    // File Button
    let file_button = MenuButton::builder()
        .label("File")
        .build();
    let file_popover = Popover::new();
    let file_box = GtkBox::new(Orientation::Vertical, 5);
    file_box.append(&Label::new(Some("New")));
    file_box.append(&Label::new(Some("Open")));
    file_box.append(&Label::new(Some("Exit")));
    file_popover.set_child(Some(&file_box));
    file_button.set_popover(Some(&file_popover));

    // Edit button
    let edit_button = MenuButton::builder()
        .label("Edit")
        .build();
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
