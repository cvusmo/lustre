// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/main.rs

use lustre::launcher::launcher;
use lustre::window::lustre_window;
// use lustre::state::initialize_state;

fn main() {
    //TODO: initialize_state
    // initialize_state("lustre.log", LevelFilter::Info).expect("Failed to initalize state");
    launcher();
    lustre_window();
}
