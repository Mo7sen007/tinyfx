mod coordinator;
mod server;
mod watcher;
mod websocker;

use anyhow::Result;

use crate::config;

/// Start the development server.
///
/// Serves static files from `dir` on `http://localhost:3000`, watches the
/// directory for changes, and notifies connected browsers via a WebSocket
/// on `ws://localhost:3001` to live-reload.
pub fn run_serve(config: &config::Config) -> Result<()> {
    coordinator::run(config)
}