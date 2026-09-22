use anyhow::Result;
use clap::{Parser, Subcommand};
use ls_core::{
    ai::{ollama::{AiStatus, OllamaBackend}, AiBackend},
    classifier, dedup,
    models::{FileEntry, FolderLang},
    organizer::{self, OrganizerConfig},
    scanner::{self, ScanOptions},
};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "lifesort", about = "LifeSort: sorts files by what is inside them")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Scan a directory and print file summary
    Scan {
        path: PathBuf,
        #[arg(long, default_value = "false")]
        hidden: bool,
    },
    /// Find duplicate files
    Dedup {
        path: PathBuf,
    },
    /// Propose organization actions (dry-run)
    Organize {
        path: PathBuf,
        #[arg(long)]
        target: PathBuf,
        #[arg(long, default_value = "false")]
        execute: bool,
        /// Folder names in German instead of English
        #[arg(long, default_value = "false")]
        german: bool,
        /// Classify with a local Ollama instead of rules only
        #[arg(long, default_value = "false")]
        ai: bool,
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,
        #[arg(long, default_value = "llama3")]
        text_model: String,
        #[arg(long, default_value = "llava")]
        vision_model: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Scan { path, hidden } => {
            let session_id = Uuid::new_v4().to_string();
            let opts = ScanOptions { skip_hidden: !hidden, ..Default::default() };
            let mut entries: Vec<FileEntry> = vec![];
            let count = scanner::scan_directory(&path, &session_id, &opts, |e| entries.push(e))?;
            println!("Scanned: {} files", count);
            let mut kinds = std::collections::HashMap::new();
            for e in &entries {
                *kinds.entry(format!("{:?}", e.kind)).or_insert(0u32) += 1;
            }
            for (k, v) in &kinds {
                println!("  {:>12}: {v}", k);
            }
        }
        Cmd::Dedup { path } => {
            let session_id = Uuid::new_v4().to_string();
            let opts = ScanOptions::default();
            let mut entries: Vec<FileEntry> = vec![];
            scanner::scan_directory(&path, &session_id, &opts, |e| entries.push(e))?;
            dedup::compute_hashes(&mut entries);
            let groups = dedup::find_duplicate_groups(&mut entries);
            if groups.is_empty() {
                println!("No duplicates found.");
            } else {
                let total_wasted: u64 = groups.iter().map(|g| g.total_wasted_bytes).sum();
                println!("{} duplicate groups, {:.1} MB wasted", groups.len(), total_wasted as f64 / 1_048_576.0);
                for g in &groups {
                    println!("  hash {} ({} copies, {:.1} KB each)", &g.hash[..8], g.file_ids.len(), g.size as f64 / 1024.0);
                }
            }
        }
        Cmd::Organize { path, target, execute, german, ai, ollama_url, text_model, vision_model } => {
            let session_id = Uuid::new_v4().to_string();
            let mut entries: Vec<FileEntry> = vec![];
            scanner::scan_directory(&path, &session_id, &ScanOptions::default(), |e| entries.push(e))?;

            let backend = OllamaBackend::new(ollama_url, text_model, vision_model);
            let use_ai = ai && match backend.status().await {
                AiStatus::Ready => true,
                other => {
                    eprintln!("Ollama not usable ({other:?}), classifying by rules only");
                    false
                }
            };
            let total = entries.len();
            for (i, entry) in entries.iter_mut().enumerate() {
                let ai_ref: Option<&dyn AiBackend> = if use_ai { Some(&backend) } else { None };
                entry.classification = Some(classifier::classify_entry(entry, ai_ref).await);
                if use_ai {
                    eprint!("\r{}/{total}", i + 1);
                }
            }
            if use_ai {
                eprintln!();
            }

            let config = OrganizerConfig {
                target_root: target,
                folder_lang: if german { FolderLang::De } else { FolderLang::En },
            };
            let mut actions = organizer::propose_actions(&entries, &config);
            println!("{} proposals{}:", actions.len(), if execute { "" } else { " (dry run)" });
            for action in &mut actions {
                if execute {
                    if let Err(e) = organizer::execute_action(action) {
                        eprintln!("  error: {} ({e})", action.source_path);
                        continue;
                    }
                }
                println!("  {} -> {}", action.source_path, action.target_path.as_deref().unwrap_or("-"));
            }
        }
    }
    Ok(())
}
