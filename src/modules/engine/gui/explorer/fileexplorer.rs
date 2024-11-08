// src/modules/engine/gui/explorer/fileexplorer.rs
// github.com/cvusmo/gameengine

use std::process::Command;

pub fn open_file_explorer(path: &str) {
    let file_manager = "dolphin";
    Command::new(file_manager)
        .arg(path)
        .spawn()
        .expect("Failed to open file manager");

    // TESTING
}
