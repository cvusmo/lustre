// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/engine/ui/menu_bar.rs

use crate::engine::ui::file_explorer::open_file;
use crate::engine::ui::lua_editor::{create_lua_editor, run_lua_script};
use crate::engine::ui::utils::{handle_exit, load_project_area, save_as_file, save_file};
use crate::launcher::close_launcher;
use crate::state::{log_info, AppState};
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, MenuButton, Orientation,
    Popover, TextView,
};
use std::sync::{Arc, Mutex};

pub fn create_menu_bar(
    state: &Arc<Mutex<AppState>>,
    parent: &Arc<ApplicationWindow>,
    app: &Application,
    project_area: &GtkBox,
) -> (GtkBox, TextView) {
    log_info("Creating menu bar...");

    let menu_bar = GtkBox::new(Orientation::Horizontal, 5);
    menu_bar.set_halign(Align::Start);
    menu_bar.add_css_class("menu-bar");

    let file_button = MenuButton::builder().label("File").build();
    file_button.add_css_class("menu-button");
    let file_popover = Popover::new();
    let file_box = GtkBox::new(Orientation::Vertical, 5);

    let new_button = Button::with_label("New");
    file_box.append(&new_button);
    let state_clone = Arc::clone(state);
    let project_area_clone = project_area.clone();
    new_button.connect_clicked({
        let content = String::from(""); // Define content here for closure
        move |_| {
            log_info("Creating New Project...");
            load_project_area(
                &state_clone,
                &content,
                &project_area_clone,
                create_lua_editor,
            );
        }
    });

    let open_button = Button::with_label("Open");
    file_box.append(&open_button);
    let state_clone_open = Arc::clone(state);
    let parent_clone_open = Arc::clone(parent);
    let project_area_clone_open = project_area.clone();
    open_button.connect_clicked(move |_| {
        log_info("Open file button clicked.");
        open_file(
            state_clone_open.clone(),
            parent_clone_open.as_ref(),
            &project_area_clone_open,
        );
    });

    let save_button = Button::with_label("Save");
    file_box.append(&save_button);
    let state_clone_save = Arc::clone(state);
    let text_view_save = load_project_area(state, "", project_area, create_lua_editor); // Initial load
    save_button.connect_clicked({
        let state_clone_save = Arc::clone(&state_clone_save);
        let text_view_save = text_view_save.clone();
        move |_| {
            log_info("Save file button clicked.");
            save_file(&text_view_save, &state_clone_save);
            log_info("File saved operation finished.");
        }
    });

    let save_as_button = Button::with_label("Save As");
    file_box.append(&save_as_button);
    let state_clone_save_as = Arc::clone(state);
    let parent_clone_save_as = Arc::clone(parent);
    let text_view_save_as = text_view_save.clone();
    save_as_button.connect_clicked({
        let state_clone_save_as = Arc::clone(&state_clone_save_as);
        let parent_clone_save_as = parent_clone_save_as.clone();
        let text_view_save_as = text_view_save_as.clone();
        move |_| {
            log_info("Save As button clicked.");
            save_as_file(
                &text_view_save_as,
                state_clone_save_as.clone(),
                parent_clone_save_as.clone(),
            );
            log_info("Save As operation finished.");
        }
    });

    let exit_button = Button::with_label("Exit");
    file_box.append(&exit_button);
    let state_clone_exit = Arc::clone(state);
    let app_clone = app.clone();
    let text_view_exit = text_view_save.clone();
    exit_button.connect_clicked({
        let state_clone_exit = Arc::clone(&state_clone_exit);
        let app_clone = app_clone.clone();
        let text_view_exit = text_view_exit.clone();
        move |_| {
            log_info("Exit button clicked.");
            handle_exit(&text_view_exit, state_clone_exit.clone(), &app_clone);
        }
    });

    file_popover.set_child(Some(&file_box));
    file_button.set_popover(Some(&file_popover));

    let edit_button = MenuButton::builder().label("Edit").build();
    edit_button.add_css_class("menu-button");
    let edit_popover = Popover::new();
    let edit_box = GtkBox::new(Orientation::Vertical, 5);
    edit_box.append(&Label::new(Some("Undo")));
    edit_box.append(&Label::new(Some("Redo")));
    edit_box.append(&Label::new(Some("Preferences")));
    edit_popover.set_child(Some(&edit_box));
    edit_button.set_popover(Some(&edit_popover));

    let project_button = MenuButton::builder().label("Project").build();
    project_button.add_css_class("menu-button");
    let project_popover = Popover::new();
    let project_box = GtkBox::new(Orientation::Vertical, 5);

    let compile_button = Button::with_label("Compile");
    project_box.append(&compile_button);
    let state_clone_compile = Arc::clone(state);
    compile_button.connect_clicked(move |_| {
        log_info("Begin compiling project...");
        run_lua_script(&state_clone_compile);
        log_info("Project compiled.");
    });

    let play_button = Button::with_label("Play");
    project_box.append(&play_button);
    let state_clone_play = Arc::clone(state);
    let app_clone_play = app.clone();
    play_button.connect_clicked(move |_| {
        log_info("Play button clicked, closing launcher...");
        close_launcher(app_clone_play.clone(), state_clone_play.clone());
    });

    project_popover.set_child(Some(&project_box));
    project_button.set_popover(Some(&project_popover));

    menu_bar.append(&file_button);
    menu_bar.append(&edit_button);
    menu_bar.append(&project_button);

    let content = String::from(""); // Define content separately here
    let text_view = load_project_area(state, &content, project_area, create_lua_editor);

    log_info("Menu bar created successfully.");
    (menu_bar, text_view)
} 
  
