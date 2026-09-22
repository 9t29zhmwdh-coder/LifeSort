//! Measures how well local language models sort documents, through the same
//! prompt and parsing as the app, and compares them with the keyword rules.
//!
//!     cargo run --release --example bench_text -- <doc-dir> <model> [<model> …]
//!
//! `<doc-dir>` holds UTF-8 text files and a `manifest.tsv` with the columns
//! `file` and `label` (invoice, tax, contract, guarantee, letter,
//! certificate, report).

use ls_core::ai::ollama::{AiStatus, OllamaBackend};
use ls_core::ai::{prompts, AiBackend};
use ls_core::classifier::{text_rules, truncate_utf8, MAX_TEXT_BYTES};
use ls_core::models::Category;
use std::path::PathBuf;
use std::time::Instant;

fn expected(label: &str) -> Category {
    match label {
        "invoice" => Category::Invoice,
        "tax" => Category::TaxDocument,
        "contract" => Category::Contract,
        "guarantee" => Category::Guarantee,
        "letter" => Category::Letter,
        "certificate" => Category::Certificate,
        "report" => Category::Report,
        _ => Category::Unknown,
    }
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let dir = PathBuf::from(args.next().expect("usage: bench_text <doc-dir> <model>…"));
    let docs: Vec<(String, String, String)> = std::fs::read_to_string(dir.join("manifest.tsv"))
        .expect("manifest.tsv missing")
        .lines()
        .skip(1)
        .filter_map(|l| {
            let (file, label) = l.split_once('\t')?;
            let text = std::fs::read_to_string(dir.join(file)).ok()?;
            Some((file.to_string(), label.to_string(), text))
        })
        .collect();

    let rules = docs.iter().filter(|(_, l, t)| text_rules::classify(t).category == expected(l)).count();
    println!("model\ts_per_doc\taccuracy\terrors\tmisses");
    println!("rules only\t0.00\t{:.0}%\t0\t", 100.0 * rules as f64 / docs.len() as f64);

    for model in args {
        let backend = OllamaBackend::new("http://localhost:11434".into(), model.clone(), model.clone());
        if backend.status().await != AiStatus::Ready {
            println!("{model}\tnot installed");
            continue;
        }
        // Load the model before timing.
        let _ = backend.classify_text("warm up", prompts::DOCUMENT_CLASSIFY).await;
        let (mut hits, mut errors, mut secs, mut misses) = (0, 0, 0f64, vec![]);
        for (file, label, text) in &docs {
            let t = Instant::now();
            let result = backend.classify_text(truncate_utf8(text, MAX_TEXT_BYTES), prompts::DOCUMENT_CLASSIFY).await;
            secs += t.elapsed().as_secs_f64();
            match result {
                Ok(c) if c.category == expected(label) => hits += 1,
                Ok(c) => misses.push(format!("{file}→{}", c.category.key())),
                Err(_) => errors += 1,
            }
        }
        let n = docs.len() as f64;
        println!("{model}\t{:.2}\t{:.0}%\t{errors}\t{}", secs / n, 100.0 * hits as f64 / n, misses.join(", "));
        let unload = serde_json::json!({ "model": model, "keep_alive": 0 });
        let _ = reqwest::Client::new().post("http://localhost:11434/api/generate").json(&unload).send().await;
    }
}
