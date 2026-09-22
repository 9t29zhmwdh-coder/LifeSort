//! Measures how well local vision models sort photos, using exactly the code
//! path of the app: same image preparation, same prompt, same parsing.
//!
//!     cargo run --release --example bench_vision -- <image-dir> <model> [<model> …]
//!
//! `<image-dir>` holds the images and a `manifest.tsv` with the columns
//! `file` and `label`, where label is one of person, landscape, event,
//! screenshot, meme, document, other. The images used for the README table
//! are listed with source and licence in `docs/benchmark/manifest.tsv`.
//!
//! Per model it reports the memory Ollama reserves, the load time, the mean
//! time per photo, and accuracy twice: the model alone, and the full pipeline
//! in which the screenshot rules answer before the model is asked.

use base64::Engine;
use ls_core::ai::ollama::{AiStatus, OllamaBackend};
use ls_core::ai::AiBackend;
use ls_core::classifier::classify_entry;
use ls_core::models::{Category, FileEntry};
use ls_core::scanner::{scan_directory, ScanOptions};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

const OLLAMA: &str = "http://localhost:11434";

fn expected(label: &str) -> Category {
    match label {
        "person" => Category::PhotoPerson,
        "landscape" => Category::PhotoLandscape,
        "event" => Category::PhotoEvent,
        "screenshot" => Category::PhotoScreenshot,
        "meme" => Category::PhotoMeme,
        "document" => Category::PhotoDocument,
        _ => Category::PhotoOther,
    }
}

fn manifest(dir: &Path) -> BTreeMap<String, String> {
    std::fs::read_to_string(dir.join("manifest.tsv"))
        .expect("manifest.tsv missing")
        .lines()
        .skip(1)
        .filter_map(|l| {
            let mut cols = l.split('\t');
            Some((cols.next()?.to_string(), cols.next()?.to_string()))
        })
        .collect()
}

/// Asks Ollama to drop a model from memory, so the next one is measured alone.
async fn unload(client: &reqwest::Client, model: &str) {
    let body = serde_json::json!({ "model": model, "keep_alive": 0 });
    let _ = client.post(format!("{OLLAMA}/api/generate")).json(&body).send().await;
}

/// Memory Ollama reports for the loaded model, in GB.
async fn loaded_size(client: &reqwest::Client, model: &str) -> f64 {
    let ps: serde_json::Value = match client.get(format!("{OLLAMA}/api/ps")).send().await {
        Ok(r) => r.json().await.unwrap_or_default(),
        Err(_) => return 0.0,
    };
    ps["models"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|m| m["name"].as_str().is_some_and(|n| n == model || n == format!("{model}:latest")))
        .and_then(|m| m["size"].as_f64())
        .map(|b| b / 1e9)
        .unwrap_or(0.0)
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let dir = PathBuf::from(args.next().expect("usage: bench_vision <image-dir> <model>…"));
    let models: Vec<String> = args.collect();
    let labels = manifest(&dir);

    let mut entries: Vec<FileEntry> = vec![];
    scan_directory(&dir, "bench", &ScanOptions::default(), |e| {
        if labels.contains_key(&e.name) {
            entries.push(e)
        }
    })
    .unwrap();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    let jpegs: Vec<String> = entries
        .iter()
        .map(|e| {
            let jpeg = ls_core::imageprep::prepare_for_model(Path::new(&e.path)).expect("undecodable image");
            base64::engine::general_purpose::STANDARD.encode(jpeg)
        })
        .collect();
    eprintln!("{} images", entries.len());

    let client = reqwest::Client::new();
    println!("model\tgb\tload_s\ts_per_image\tmodel_acc\tpipeline_acc\terrors\tper_label");
    for model in &models {
        let backend = OllamaBackend::new(OLLAMA.into(), model.clone(), model.clone());
        if backend.status().await != AiStatus::Ready {
            println!("{model}\tnot installed");
            continue;
        }

        // First request loads the model; timed separately.
        let t = Instant::now();
        let _ = backend.classify_image(&jpegs[0]).await;
        let load_s = t.elapsed().as_secs_f64();
        let gb = loaded_size(&client, model).await;

        let (mut hits, mut errors, mut secs) = (0usize, 0usize, 0f64);
        let mut per_label: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        let mut log = vec![];
        for (entry, b64) in entries.iter().zip(&jpegs) {
            let label = &labels[&entry.name];
            let t = Instant::now();
            let result = backend.classify_image(b64).await;
            secs += t.elapsed().as_secs_f64();
            let slot = per_label.entry(label.clone()).or_default();
            slot.1 += 1;
            match result {
                Ok(c) if c.category == expected(label) => {
                    hits += 1;
                    slot.0 += 1;
                    log.push(format!("{}\t{label}\tok", entry.name));
                }
                Ok(c) => log.push(format!("{}\t{label}\t{}", entry.name, c.category.key())),
                Err(e) => {
                    errors += 1;
                    log.push(format!("{}\t{label}\terror: {e}", entry.name));
                }
            }
        }

        let mut pipeline_hits = 0;
        for entry in &entries {
            if classify_entry(entry, Some(&backend)).await.category == expected(&labels[&entry.name]) {
                pipeline_hits += 1;
            }
        }

        let n = entries.len() as f64;
        let per: Vec<String> = per_label.iter().map(|(l, (h, t))| format!("{l} {h}/{t}")).collect();
        println!(
            "{model}\t{gb:.1}\t{load_s:.1}\t{:.2}\t{:.0}%\t{:.0}%\t{errors}\t{}",
            secs / n,
            100.0 * hits as f64 / n,
            100.0 * pipeline_hits as f64 / n,
            per.join(", ")
        );
        let safe = model.replace([':', '/'], "_");
        std::fs::write(format!("predictions_{safe}.tsv"), log.join("\n")).unwrap();
        unload(&client, model).await;
    }
}
