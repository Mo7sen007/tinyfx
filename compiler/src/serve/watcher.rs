use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use notify::{Event, RecursiveMode, Watcher, recommended_watcher};
use tokio::sync::broadcast;

/// Watch `dir` recursively for any file-system change.
///
/// On first change a 100 ms debounce window is opened to coalesce rapid
/// bursts (e.g. a formatter touching many files at once).  After the burst
/// settles a `()` is sent on `tx` so that connected WebSocket clients know
/// to reload.
pub async fn watch(dir: Arc<PathBuf>, tx: broadcast::Sender<()>) -> Result<()> {
    // Bridge: notify runs on its own thread pool, so we use a sync-safe
    // mpsc channel and then handle events in async code.
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel::<()>(32);

    let mut watcher = recommended_watcher(move |res: notify::Result<Event>| {
        if res.is_ok() {
            // `blocking_send` is safe here because we're on a notify
            // background thread, not inside a Tokio async context.
            let _ = notify_tx.blocking_send(());
        }
    })?;

    watcher.watch(dir.as_ref(), RecursiveMode::Recursive)?;
    println!("[tinyfx] Watching {:?} for changes ...", dir);

    while notify_rx.recv().await.is_some() {
        // Debounce: wait 100 ms then drain any further events in the queue.
        tokio::time::sleep(Duration::from_millis(100)).await;
        while notify_rx.try_recv().is_ok() {}

        println!("[tinyfx] Change detected — reloading browsers ...");
        let _ = tx.send(());
    }

    Ok(())
}
