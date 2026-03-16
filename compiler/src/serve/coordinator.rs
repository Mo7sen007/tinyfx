//! # serve::coordinator
//!
//! Async coordinator for the TinyFX development server.
//!
//! ## Responsibilities
//! - Run an initial build before serving
//! - Start the HTTP server and WebSocket reload server
//! - Drive the file watcher loop
//!
//! ## What this module does NOT do
//! - It does not implement HTTP request handling
//! - It does not implement WebSocket protocol details
//! - It does not compile individual build phases itself

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::broadcast;

use crate::build;
use crate::config;

use super::server;
use super::watcher;
use super::websocket;

/// Top-level async entry point.  Spawns the HTTP and WebSocket servers,
/// then drives the file watcher on the current thread (it runs until the
/// process is killed).
async fn run_async(config: &config::Config) -> Result<()> {
    build::run_build(config)?;

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
        if let Err(e) = websocket::run(ws_tx).await {
            eprintln!("[tinyfx] websocket error: {e}");
        }
    });

    // ── File watcher (runs until process exits) ──────────────────────────
    watcher::watch(config, tx).await?;

    Ok(())
}

/// Synchronous wrapper used by the CLI.  Creates a Tokio runtime and blocks
/// until the server is shut down.
///
/// # Errors
/// Returns an error when the Tokio runtime cannot be created or when startup
/// of the build, watcher, HTTP server, or WebSocket server fails.
pub fn run(config: &config::Config) -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(run_async(&config))
}
