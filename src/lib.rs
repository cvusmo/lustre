// src/lib.rs
// github.com/cvusmo/gameengine

pub mod modules;

// Re-export necessary components
pub use modules::engine::configuration::config::Config;
pub use modules::engine::configuration::logger::{create_state, setup_logging};
