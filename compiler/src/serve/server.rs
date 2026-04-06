//! # serve::server
//!
//! HTTP file server for built TinyFX output.
//!
//! ## Responsibilities
//! - Serve files from the build output directory
//! - Inject the live-reload client into HTML responses
//! - Return content types for built assets
//!
//! ## What this module does NOT do
//! - It does not watch the source tree
//! - It does not broadcast reload events
//! - It does not run build phases directly

use std::convert::Infallible;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use flate2::write::GzEncoder;
use flate2::Compression;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use mime_guess::from_path;
use tokio::fs;
use tokio::net::TcpListener;

type ResponseBody = Full<Bytes>;

/// Injected into every HTML response to connect back to the WebSocket
/// reload server running on port 3001.
const LIVE_RELOAD_SCRIPT: &str = r#"<script>
(function () {
  const ws = new WebSocket("ws://localhost:3001");
  ws.onmessage = (e) => { if (e.data === "reload") location.reload(); };
  // Retry after disconnect so the page reconnects after a rebuild.
  ws.onclose = () => setTimeout(() => location.reload(), 1000);
})();
</script>"#;

/// Bind an HTTP server on `127.0.0.1:3000` and serve files from `dir`.
///
/// HTML responses have the live-reload script injected just before `</body>`
/// so connected browsers can listen for reload notifications.
///
/// # Errors
/// Returns an error when the TCP listener cannot bind or when accepting a new
/// connection fails.
pub async fn run(dir: Arc<PathBuf>) -> Result<()> {
    let addr = "127.0.0.1:3000";
    let listener = TcpListener::bind(addr).await?;

    println!("[tinyfx] Serving  http://{addr}");
    println!("[tinyfx] Press Ctrl+C to stop.\n");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let dir = Arc::clone(&dir);

        tokio::spawn(async move {
            // Capture `dir` per-connection by moving it into the closure.
            let service = service_fn(move |req| {
                let dir = Arc::clone(&dir);
                async move { serve_file(req, dir).await }
            });

            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                eprintln!("[tinyfx] connection error: {e}");
            }
        });
    }
}

async fn serve_file(
    req: Request<Incoming>,
    dir: Arc<PathBuf>,
) -> Result<Response<ResponseBody>, Infallible> {
    let req_path_raw = req.uri().path();

    // Basic path sanitation: reject attempts at directory traversal.
    if req_path_raw.contains("..") {
        let mut response = Response::new(Full::new(Bytes::from("404 Not Found")));
        *response.status_mut() = StatusCode::NOT_FOUND;
        response.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            hyper::header::HeaderValue::from_static("text/plain"),
        );
        return Ok(response);
    }

    let rel = req_path_raw.trim_start_matches('/');

    // Resolve candidate file paths in order:
    // 1) exact file at `dist/<rel>`
    // 2) if `dist/<rel>` is a directory, try `dist/<rel>/index.html`
    // 3) if request is empty, serve `dist/index.html`
    let candidates: Vec<PathBuf> = if rel.is_empty() {
        vec![dir.join("index.html")]
    } else {
        let direct = dir.join(rel);
        vec![direct.clone(), direct.join("index.html")]
    };

    // Find the first existing regular file among candidates.
    let mut found: Option<PathBuf> = None;
    for candidate in &candidates {
        if let Ok(meta) = fs::metadata(candidate).await {
            if meta.is_file() {
                found = Some(candidate.clone());
                break;
            }
        }
    }

    if let Some(full_path) = found {
        match fs::read(&full_path).await {
            Ok(raw) => {
                let mime = from_path(&full_path).first_or_octet_stream();

                // Inject live-reload snippet into HTML files only.
                let body = if mime.essence_str() == "text/html" {
                    let mut html = String::from_utf8_lossy(&raw).into_owned();
                    if let Some(pos) = html.rfind("</body>") {
                        html.insert_str(pos, LIVE_RELOAD_SCRIPT);
                    } else {
                        html.push_str(LIVE_RELOAD_SCRIPT);
                    }
                    Bytes::from(html.into_bytes())
                } else {
                    Bytes::from(raw)
                };

                // Compress if client accepts gzip and content type is text-based.
                let compressible = is_compressible_mime(&mime);
                let accepts_gzip = header_contains(&req, "accept-encoding", "gzip");
                let (body, content_encoding) = if compressible && accepts_gzip {
                    let compressed = gzip_compress(&body);
                    (Bytes::from(compressed), Some("gzip"))
                } else {
                    (body, None)
                };

                let mut response = Response::new(Full::new(body));
                *response.status_mut() = StatusCode::OK;
                response.headers_mut().insert(
                    hyper::header::CONTENT_TYPE,
                    hyper::header::HeaderValue::from_str(mime.as_ref()).unwrap_or(
                        hyper::header::HeaderValue::from_static("application/octet-stream"),
                    ),
                );
                if let Some(enc) = content_encoding {
                    response.headers_mut().insert(
                        hyper::header::CONTENT_ENCODING,
                        hyper::header::HeaderValue::from_static(enc),
                    );
                }
                Ok(response)
            }
            Err(_) => {
                let mut response = Response::new(Full::new(Bytes::from("404 Not Found")));
                *response.status_mut() = StatusCode::NOT_FOUND;
                response.headers_mut().insert(
                    hyper::header::CONTENT_TYPE,
                    hyper::header::HeaderValue::from_static("text/plain"),
                );
                Ok(response)
            }
        }
    } else {
        let mut response = Response::new(Full::new(Bytes::from("404 Not Found")));
        *response.status_mut() = StatusCode::NOT_FOUND;
        response.headers_mut().insert(
            hyper::header::CONTENT_TYPE,
            hyper::header::HeaderValue::from_static("text/plain"),
        );
        Ok(response)
    }
}

/// Returns true if the MIME type is text-based and benefits from compression.
fn is_compressible_mime(mime: &mime_guess::Mime) -> bool {
    let essence = mime.essence_str();
    matches!(
        essence,
        "text/html"
            | "text/css"
            | "text/javascript"
            | "application/javascript"
            | "application/json"
            | "text/xml"
            | "application/xml"
            | "image/svg+xml"
    )
}

/// Check if a request header contains a given substring (case-insensitive).
fn header_contains<K: AsRef<str>>(req: &Request<Incoming>, key: K, value: &str) -> bool {
    let Some(header) = req.headers().get(key.as_ref()) else {
        return false;
    };
    let Ok(val) = header.to_str() else {
        return false;
    };
    val.to_ascii_lowercase().contains(value)
}

/// Compress bytes with gzip at default compression level.
fn gzip_compress(data: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).expect("gzip write failed");
    encoder.finish().expect("gzip finish failed")
}
