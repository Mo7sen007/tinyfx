//! # serve::websocket
//!
//! WebSocket reload broadcaster for the TinyFX development server.
//!
//! ## Responsibilities
//! - Accept browser WebSocket connections
//! - Subscribe each client to the reload broadcast channel
//! - Push `reload` messages after rebuilds
//!
//! ## What this module does NOT do
//! - It does not watch the filesystem
//! - It does not serve HTTP files
//! - It does not run build phases directly

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Bind a WebSocket server on `127.0.0.1:3001`.
///
/// Each connected browser subscribes to the broadcast channel. When the file
/// watcher sends a reload signal, every client receives a `reload` text
/// message and refreshes the page.
///
/// # Errors
/// Returns an error when the TCP listener cannot bind or when accepting a new
/// socket connection fails.
pub async fn run(tx: broadcast::Sender<()>) -> Result<()> {
    let addr = "127.0.0.1:3001";
    let listener = TcpListener::bind(addr).await?;
    println!("[tinyfx] WebSocket HMR at ws://{addr}");

    while let Ok((stream, _)) = listener.accept().await {
        let receiver = tx.subscribe();
        tokio::spawn(handle_client(stream, receiver));
    }

    Ok(())
}

async fn handle_client(stream: tokio::net::TcpStream, mut receiver: broadcast::Receiver<()>) {
    let websocket = match accept_async(stream).await {
        Ok(websocket) => websocket,
        Err(error) => {
            eprintln!("[tinyfx] ws handshake error: {error}");
            return;
        }
    };

    let (mut sink, _stream) = websocket.split();

    loop {
        match receiver.recv().await {
            Ok(()) => {
                if sink
                    .send(Message::Text("reload".to_string().into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
