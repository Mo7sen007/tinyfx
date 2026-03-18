//! # serve
//!
//! Development server entry points for `tinyfx serve`.
//!
//! ## Responsibilities
//! - Expose a single command entry for the serve subsystem
//! - Delegate to the async coordinator
//!
//! ## What this module does NOT do
//! - It does not implement HTTP serving directly
//! - It does not implement WebSocket reload handling directly
//! - It does not contain build pipeline logic

mod coordinator;
mod server;
mod watcher;
mod websocket;

use anyhow::Result;

use crate::config;

/// Start the TinyFX development server.
///
/// This runs an initial build using `config`, serves the configured output
/// directory on `http://127.0.0.1:3000`, starts the live-reload WebSocket on
/// `ws://127.0.0.1:3001`, and watches the project tree for rebuilds.
///
/// # Errors
/// Returns an error when the initial build fails, when the async runtime
/// cannot be created, or when the HTTP or WebSocket server cannot start.
pub fn run_serve(config: &config::Config) -> Result<()> {
    coordinator::run(config)
}
