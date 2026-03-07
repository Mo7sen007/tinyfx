use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::broadcast;

use crate::config;

use super::server;
use super::watcher;
use super::websocker;

/// Top-level async entry point.  Spawns the HTTP and WebSocket servers,
/// then drives the file watcher on the current thread (it runs until the
/// process is killed).
async fn run_async(config: &config::Config) -> Result<()> {
    // The HTTP server serves from the *output* directory, not the source tree.
    let out_dir = Arc::new(PathBuf::from(&config.out_dir));
    // Broadcast channel: watcher sends `()`, WebSocket clients receive it.
    let (tx, _) = broadcast::channel::<()>(16);

    // ── HTTP file server ────────────────────────────────────────────────
    let srv_dir = Arc::clone(&out_dir);
    tokio::spawn(async move {
        if let Err(e) = server::run(srv_dir).await {
            eprintln!("[tinyfx] server error: {e}");
        }
    });

    // ── WebSocket live-reload server ─────────────────────────────────────
    let ws_tx = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = websocker::run(ws_tx).await {
            eprintln!("[tinyfx] websocket error: {e}");
        }
    });

    // ── File watcher (runs until process exits) ──────────────────────────
    watcher::watch(config, tx).await?;

    Ok(())
}

/// Synchronous wrapper used by the CLI.  Creates a Tokio runtime and blocks
/// until the server is shut down.
pub fn run(config: &config::Config) -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(run_async(&config))
}
