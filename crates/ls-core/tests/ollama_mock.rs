//! Checks what LifeSort actually sends to Ollama, against a stand-in server.
//!
//! The unit tests prove the pieces; this proves the wire: that a large photo
//! arrives as a complete, small JPEG, that the configured model is used, and
//! that a missing model is reported instead of silently falling back.

use base64::Engine;
use ls_core::ai::ollama::{AiStatus, OllamaBackend};
use ls_core::classifier::classify_entry;
use ls_core::models::{Category, ClassifierKind};
use ls_core::scanner::{scan_directory, ScanOptions};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

type Seen = Arc<Mutex<Vec<(String, Value)>>>;

/// Minimal HTTP/1.1 server: answers /api/tags with the given models and
/// /api/generate with the given JSON, and records every request.
async fn fake_ollama(models: &'static [&'static str], answer: &'static str) -> (String, Seen) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen: Seen = Arc::default();
    let log = seen.clone();
    tokio::spawn(async move {
        loop {
            let (mut sock, _) = listener.accept().await.unwrap();
            let log = log.clone();
            tokio::spawn(async move {
                let (path, body) = read_request(&mut sock).await;
                let reply = if path == "/api/tags" {
                    let list: Vec<Value> = models.iter().map(|m| serde_json::json!({ "name": m })).collect();
                    serde_json::json!({ "models": list }).to_string()
                } else {
                    serde_json::json!({ "response": answer }).to_string()
                };
                log.lock().unwrap().push((path, serde_json::from_slice(&body).unwrap_or(Value::Null)));
                let head = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    reply.len()
                );
                sock.write_all(head.as_bytes()).await.unwrap();
                sock.write_all(reply.as_bytes()).await.unwrap();
            });
        }
    });
    (url, seen)
}

async fn read_request(sock: &mut tokio::net::TcpStream) -> (String, Vec<u8>) {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 65536];
    let header_end = loop {
        let n = sock.read(&mut chunk).await.unwrap();
        buf.extend_from_slice(&chunk[..n]);
        if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break p + 4;
        }
    };
    let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
    let path = head.split_whitespace().nth(1).unwrap_or("").to_string();
    let len: usize = head
        .lines()
        .find_map(|l| l.to_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse().unwrap()))
        .unwrap_or(0);
    while buf.len() < header_end + len {
        let n = sock.read(&mut chunk).await.unwrap();
        buf.extend_from_slice(&chunk[..n]);
    }
    (path, buf[header_end..header_end + len].to_vec())
}

fn photo_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ls-mock-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    // 4000 × 3000 of noise: far above the old 512 KB cut-off once encoded.
    let img = image::RgbImage::from_fn(4000, 3000, |x, y| image::Rgb([(x * 7 % 251) as u8, (y * 13 % 241) as u8, ((x ^ y) % 255) as u8]));
    img.save(dir.join("IMG_4471.jpg")).unwrap();
    dir
}

#[tokio::test]
async fn photo_reaches_the_vision_model_as_a_complete_small_jpeg() {
    let (url, seen) = fake_ollama(
        &["llama3:latest", "qwen2.5vl:7b"],
        r#"{"category":"person","tags":["group","outdoor"],"confidence":0.9,"is_screenshot":false,"scene_description":"Two people at a lake"}"#,
    )
    .await;
    let backend = OllamaBackend::new(url, "llama3".into(), "qwen2.5vl:7b".into());
    assert_eq!(backend.status().await, AiStatus::Ready);

    let dir = photo_dir();
    let mut entries = vec![];
    scan_directory(&dir, "s", &ScanOptions::default(), |e| entries.push(e)).unwrap();
    assert!(entries[0].size > 512_000, "fixture must exceed the old cut-off");

    let c = classify_entry(&entries[0], Some(&backend)).await;
    assert_eq!(c.category, Category::PhotoPerson);
    assert_eq!(c.classified_by, ClassifierKind::Ai);
    assert_eq!(c.ai_summary.as_deref(), Some("Two people at a lake"));

    let requests = seen.lock().unwrap().clone();
    let (_, body) = requests.iter().find(|(p, _)| p == "/api/generate").expect("no generate call");
    assert_eq!(body["model"], "qwen2.5vl:7b", "settings model must be used, not a hard-coded one");
    let b64 = body["images"][0].as_str().unwrap();
    let jpeg = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
    let decoded = image::load_from_memory(&jpeg).expect("model must receive a decodable image");
    assert_eq!((decoded.width(), decoded.height()), (1024, 768));
}

/// Ollama's MLX engine answers `format: "json"` with 501. The same server
/// without the flag returns the JSON wrapped in a code fence.
async fn fake_mlx_ollama() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        loop {
            let (mut sock, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let (_, body) = read_request(&mut sock).await;
                let req: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
                let (status, reply) = if req.get("format").is_some() {
                    ("501 Not Implemented", r#"{"error":"structured output is unavailable"}"#.to_string())
                } else {
                    let text = "```json\n{\"category\":\"meme\",\"is_screenshot\":false}\n```";
                    ("200 OK", serde_json::json!({ "response": text }).to_string())
                };
                let head = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                    reply.len()
                );
                sock.write_all(head.as_bytes()).await.unwrap();
                sock.write_all(reply.as_bytes()).await.unwrap();
            });
        }
    });
    url
}

#[tokio::test]
async fn mlx_models_without_structured_output_still_work() {
    use ls_core::ai::AiBackend;
    let backend = OllamaBackend::new(fake_mlx_ollama().await, "t".into(), "qwen3.5:4b-mlx".into());
    let tiny = base64::engine::general_purpose::STANDARD.encode(b"not needed by the fake");
    let c = backend.classify_image(&tiny).await.expect("fallback without format must succeed");
    assert_eq!(c.category, Category::PhotoMeme);
    // Second call goes straight to the plain request.
    assert!(backend.classify_image(&tiny).await.is_ok());
}

#[tokio::test]
async fn missing_model_is_reported() {
    let (url, _) = fake_ollama(&["llama3:latest"], "{}").await;
    let backend = OllamaBackend::new(url, "llama3".into(), "llava".into());
    assert_eq!(backend.status().await, AiStatus::MissingModels { models: vec!["llava".into()] });
}

#[tokio::test]
async fn unreachable_server_is_reported() {
    let backend = OllamaBackend::new("http://127.0.0.1:9".into(), "a".into(), "b".into());
    assert_eq!(backend.status().await, AiStatus::Unreachable);
}
