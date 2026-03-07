mod coordinator;
mod server;
mod watcher;
mod websocker;

use std::path::PathBuf;

use anyhow::Result;

/// Start the development server.
///
/// Serves static files from `dir` on `http://localhost:3000`, watches the
/// directory for changes, and notifies connected browsers via a WebSocket
/// on `ws://localhost:3001` to live-reload.
pub fn run_serve(dir: PathBuf) -> Result<()> {
    coordinator::run(dir)
}