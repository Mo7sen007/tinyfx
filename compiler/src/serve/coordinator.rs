use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::broadcast;

use super::server;
use super::watcher;
use super::websocker;

/// Top-level async entry point.  Spawns the HTTP and WebSocket servers,
/// then drives the file watcher on the current thread (it runs until the
/// process is killed).
async fn run_async(dir: PathBuf) -> Result<()> {
    let dir = Arc::new(dir);

    // Broadcast channel: watcher sends `()`, WebSocket clients receive it.
    let (tx, _) = broadcast::channel::<()>(16);

    // ── HTTP file server ────────────────────────────────────────────────
    let srv_dir = Arc::clone(&dir);
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
    watcher::watch(Arc::clone(&dir), tx).await?;

    Ok(())
}

/// Synchronous wrapper used by the CLI.  Creates a Tokio runtime and blocks
/// until the server is shut down.
pub fn run(dir: PathBuf) -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(run_async(dir))
}
