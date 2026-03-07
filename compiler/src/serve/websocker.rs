use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Bind a WebSocket server on `127.0.0.1:3001`.
///
/// Each connected browser subscribes to the broadcast channel.  When the
/// file watcher sends a `()` reload signal every client receives a `"reload"`
/// text message causing it to call `location.reload()`.
pub async fn run(tx: broadcast::Sender<()>) -> Result<()> {
    let addr = "127.0.0.1:3001";
    let listener = TcpListener::bind(addr).await?;
    println!("[tinyfx] WebSocket HMR at ws://{addr}");

    while let Ok((stream, _)) = listener.accept().await {
        // Give each connection its own receiver from the shared sender.
        let rx = tx.subscribe();
        tokio::spawn(handle_client(stream, rx));
    }

    Ok(())
}

async fn handle_client(stream: tokio::net::TcpStream, mut rx: broadcast::Receiver<()>) {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[tinyfx] ws handshake error: {e}");
            return;
        }
    };

    let (mut sink, _stream) = ws.split();

    loop {
        match rx.recv().await {
            Ok(()) => {
                if sink.send(Message::Text("reload".to_string().into())).await.is_err() {
                    break; // client disconnected — stop the task
                }
            }
            // Sender was dropped or we lagged — either way, stop.
            Err(_) => break,
        }
    }
}
