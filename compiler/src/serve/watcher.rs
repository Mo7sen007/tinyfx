//! # serve::watcher
//!
//! File watcher for the TinyFX development server.
//!
//! ## Responsibilities
//! - Watch the source tree for meaningful changes
//! - Ignore generated files that would trigger rebuild loops
//! - Trigger rebuilds and browser reload notifications
//!
//! ## What this module does NOT do
//! - It does not serve HTTP responses
//! - It does not handle WebSocket connections
//! - It does not contain build phase logic

use std::path::PathBuf;
use std::time::Duration;

use crate::build;
use crate::config;
use anyhow::Result;
use notify::{recommended_watcher, Event, RecursiveMode, Watcher};
use tokio::sync::broadcast;

/// Watch the TinyFX source tree for file-system changes.
///
/// On first change a 100 ms debounce window is opened to coalesce rapid
/// bursts such as formatters or generator output. After the burst settles,
/// the current project is rebuilt and a reload signal is broadcast to all
/// connected WebSocket clients.
///
/// # Errors
/// Returns an error when the watch root cannot be resolved, when the notify
/// watcher cannot be created, or when a triggered rebuild fails.
pub async fn watch(config: &config::Config, tx: broadcast::Sender<()>) -> Result<()> {
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel::<()>(32);

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let src_dir = PathBuf::from(config.entry.trim_end_matches('/'));
    let watch_dir = std::fs::canonicalize(&src_dir)
        .unwrap_or_else(|_| cwd.join(&src_dir));

    // Build absolute ignore paths so they match what notify actually emits.
    //
    // 1. The generated glue file lives inside src/ but must not retrigger builds.
    let gen_file = watch_dir.join("tinyfx.gen.ts");

    // 2. The output directory (dist/) is outside src/ but the build also
    //    touches files there; resolve it absolutely to be sure.
    let out_dir = std::fs::canonicalize(&config.out_dir)
        .unwrap_or_else(|_| cwd.join(&config.out_dir));

    let mut watcher = recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(ev) = res {
            // Ignore events that are just Access (Reads) to avoid unnecessary triggers
            if ev.kind.is_access() {
                return;
            }

            let mut ignore = false;
            for p in &ev.paths {
                // p from notify is usually absolute
                if p == &gen_file {
                    ignore = true;
                    break;
                }
                if p.starts_with(&out_dir) {
                    ignore = true;
                    break;
                }
            }

            if !ignore {
                let _ = notify_tx.blocking_send(());
            }
        }
    })?;

    watcher.watch(watch_dir.as_path(), RecursiveMode::Recursive)?;
    println!("[tinyfx] Watching {:?} for changes ...", watch_dir);

    while notify_rx.recv().await.is_some() {
        // Debounce: wait 100 ms then drain any further events in the queue.
        tokio::time::sleep(Duration::from_millis(100)).await;
        while notify_rx.try_recv().is_ok() {}

        println!("[tinyfx] Change detected — reloading browsers ...");
        build::run_build(config)?;
        let _ = tx.send(());
    }

    Ok(())
}
