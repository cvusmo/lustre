// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/main.rs

use log::LevelFilter;
use lustre::engine::ui::lua_editor::register_all;
use lustre::launcher::launcher;
use lustre::state::{create_state, initialize_state};
use mlua::Lua;

fn main() {
    // Initialize state
    initialize_state("lustre.log", LevelFilter::Info).expect("Failed to initalize state");
    let state = create_state();
    {
        let mut state = state.lock().unwrap();
        state.is_modified = true;
    }

    // Init and register Lua
    let lua = Lua::new();
    register_all(&lua, state.clone()).expect("failed to register Lua functions.");

    // Launch the launcher
    launcher(state);
}
