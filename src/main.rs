// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/main.rs

use log::LevelFilter;
use lustre::launcher::launcher;
use lustre::lua_editor::register_all;
use lustre::state::{create_state, initialize_state};
use lustre::window::lustre_window;
use mlua::Lua;

fn main() {
    //TODO: initialize_state
    initialize_state("lustre.log", LevelFilter::Info).expect("Failed to initalize state");

    let state = create_state();

    let lua = Lua::new();

    register_all(&lua, state.clone()).expect("failed to register Lua functions.");

    launcher();
    // lustre_window();
}
