// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/modules/engine/gui/menu_bar.rs

use crate::modules::engine::configuration::logger::AppState;
use crate::modules::engine::configuration::logger::*;
use crate::modules::engine::gui::editor::lua_editor::{create_lua_editor, run_lua_script};
use crate::modules::engine::gui::explorer::file_explorer::*;
use crate::modules::engine::gui::utils::{handle_exit, load_project_area, save_as_file, save_file};
//use crate::modules::render::vulkan::wayland::core::VulkanContext;

use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, MenuButton, Orientation,
    Popover,
};
use std::sync::{Arc, Mutex};

pub fn create_menu_bar(
    state: &Arc<Mutex<AppState>>,
    parent: &Arc<ApplicationWindow>,
    app: &Application,
) -> GtkBox {
    log_info(state, "Creating menu bar...");

    // Create horizontal menu bar container
    let menu_bar = GtkBox::new(Orientation::Horizontal, 5);
    menu_bar.set_halign(Align::Start);
    menu_bar.add_css_class("menu-bar");

    // File Button
    let file_button = MenuButton::builder().label("File").build();
    file_button.add_css_class("menu-button");
    let file_popover = Popover::new();
    let file_box = GtkBox::new(Orientation::Vertical, 5);

    // Create new project
    let new_button = Button::with_label("New");
    file_box.append(&new_button);

    // Connect New Project to dialog function
    let state_clone = Arc::clone(state);
    let content = String::from("");
    new_button.connect_clicked(move |_| {
        log_info(&state_clone, "Creating New Project...");
        load_project_area(&state_clone, &content, create_lua_editor);
    });

    // Open file button
    let open_button = Button::with_label("Open");
    file_box.append(&open_button);
    let state_clone_open = Arc::clone(state);
    let parent_clone_open = Arc::clone(parent);
    open_button.connect_clicked(move |_| {
        log_info(&state_clone_open, "Open file button clicked.");
        open_file(state_clone_open.clone(), parent_clone_open.as_ref());
    });

    // Save file button
    let save_button = Button::with_label("Save");
    file_box.append(&save_button);
    let state_clone_save = Arc::clone(state);
    save_button.connect_clicked(move |_| {
        log_info(&state_clone_save, "Save file button clicked.");
        save_file(&state_clone_save);
        log_info(&state_clone_save, "File saved operation finished.");
    });

    // Save As... button
    let save_as_button = Button::with_label("Save As");
    file_box.append(&save_as_button);
    let state_clone_save_as = Arc::clone(state);
    let parent_clone_save_as = Arc::clone(parent);
    save_as_button.connect_clicked(move |_| {
        log_info(&state_clone_save_as, "Save As button clicked.");
        save_as_file(state_clone_save_as.clone(), parent_clone_save_as.clone());
        log_info(&state_clone_save_as, "Save As operation finished.");
    });

    // Exit button
    let exit_button = Button::with_label("Exit");
    file_box.append(&exit_button);
    let state_clone_exit = Arc::clone(state);
    let app_clone = app.clone();
    exit_button.connect_clicked(move |_| {
        log_info(&state_clone_exit, "Exit button clicked.");
        handle_exit(state_clone_exit.clone(), &app_clone);
    });

    file_popover.set_child(Some(&file_box));
    file_button.set_popover(Some(&file_popover));

    // Edit button
    let edit_button = MenuButton::builder().label("Edit").build();
    edit_button.add_css_class("menu-button");
    let edit_popover = Popover::new();
    let edit_box = GtkBox::new(Orientation::Vertical, 5);
    edit_box.append(&Label::new(Some("Undo")));
    edit_box.append(&Label::new(Some("Redo")));
    edit_box.append(&Label::new(Some("Preferences")));
    edit_popover.set_child(Some(&edit_box));
    edit_button.set_popover(Some(&edit_popover));

    // Project Button
    let project_button = MenuButton::builder().label("Project").build();
    project_button.add_css_class("menu-button");
    let project_popover = Popover::new();
    let project_box = GtkBox::new(Orientation::Vertical, 5);

    // Compile project button
    let compile_button = Button::with_label("Compile");
    project_box.append(&compile_button);
    let state_clone_compile = Arc::clone(state);
    compile_button.connect_clicked(move |_| {
        log_info(&state_clone_compile, "Begin compiling project...");
        run_lua_script(&state_clone_compile);
        // run_lua_from_editor(&state_clone_compile);
        log_info(&state_clone_compile, "Project compiled.");
    });

    // Render Project Button

    project_popover.set_child(Some(&project_box));
    project_button.set_popover(Some(&project_popover));

    // Add buttons to menu bar
    menu_bar.append(&file_button);
    menu_bar.append(&edit_button);
    menu_bar.append(&project_button);

    log_info(state, "Menu bar created successfully.");
    menu_bar
}
