// src/modules/engine/gui/menu_bar.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, AppState};
use crate::modules::engine::gui::new_project::open_new_project_dialog;
use crate::modules::engine::gui::explorer::file_explorer::open_file_dialog;
use crate::modules::engine::gui::save_file::save_file;
use crate::modules::engine::gui::saveas_file::save_as_file;

use gtk4::prelude::*;
use gtk4::{Align, ApplicationWindow, Button, Box as GtkBox, Label, MenuButton, Orientation, Popover};
use std::sync::{Arc, Mutex};

pub fn create_menu_bar(state: &Arc<Mutex<AppState>>, parent: &Arc<ApplicationWindow>) -> GtkBox {
    log_info(state, "Creating menu bar...");

    // Create horizontal menu bar container
    let menu_bar = GtkBox::new(Orientation::Horizontal, 5);
    menu_bar.set_halign(Align::Start);
    menu_bar.add_css_class("menu-bar");

    // File Button
    let file_button = MenuButton::builder()
        .label("File")
        .build();
    file_button.add_css_class("menu-button");
    let file_popover = Popover::new();
    let file_box = GtkBox::new(Orientation::Vertical, 5);

    // Create new project
    let new_button = Button::with_label("New");
    file_box.append(&new_button);

    // Connect New project to dialog function
    let state_clone = Arc::clone(state);
    let parent_clone = Arc::clone(parent);
    new_button.connect_clicked(move |_| {
        open_new_project_dialog(&state_clone, &parent_clone);
    });

    // Open file button
    let open_button = Button::with_label("Open");
    file_box.append(&open_button);
    let state_clone_open = Arc::clone(state);
    let parent_clone_open = Arc::clone(parent);
    open_button.connect_clicked(move |_| {
        open_file_dialog(state_clone_open.clone(), parent_clone_open.clone());
    });

    // Save file button
    let save_button = Button::with_label("Save");
    file_box.append(&save_button);
    let state_clone_save = Arc::clone(state);
    save_button.connect_clicked(move |_| {
        save_file(&state_clone_save);
    });

    // Save As... button
    let save_as_button = Button::with_label("Save As");
    file_box.append(&save_as_button);
    let state_clone_save_as = Arc::clone(state);
    let parent_clone_save_as = Arc::clone(parent);
    save_as_button.connect_clicked(move |_| {
        save_as_file(state_clone_save_as.clone(), parent_clone_save_as.clone());
    });

    // TODO: Add Exit with a prompt to check if saved
    file_box.append(&Label::new(Some("Exit")));
    file_popover.set_child(Some(&file_box));
    file_button.set_popover(Some(&file_popover));
    
    // Edit button
    let edit_button = MenuButton::builder()
        .label("Edit")
        .build();
    edit_button.add_css_class("menu-button");
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
