use anyhow::Result;
use clap::{Parser, Subcommand};
use ls_core::{
    classifier, dedup,
    models::FileEntry,
    organizer::{self, ConflictStrategy, OrganizerConfig},
    scanner::{self, ScanOptions},
};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "lifesort", about = "LifeSort — AI File Organizer CLI")]
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
            println!("Gescannt: {} Dateien", count);
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
            let groups = dedup::find_duplicate_groups(&entries);
            if groups.is_empty() {
                println!("Keine Duplikate gefunden.");
            } else {
                let total_wasted: u64 = groups.iter().map(|g| g.total_wasted_bytes).sum();
                println!("{} Duplikatgruppen, {:.1} MB verschwendet", groups.len(), total_wasted as f64 / 1_048_576.0);
                for g in &groups {
                    println!("  Hash: {} ({} Kopien, {:.1} KB)", &g.hash[..8], g.file_ids.len(), g.size as f64 / 1024.0);
                }
            }
        }
        Cmd::Organize { path, target, execute } => {
            let session_id = Uuid::new_v4().to_string();
            let opts = ScanOptions::default();
            let mut entries: Vec<FileEntry> = vec![];
            scanner::scan_directory(&path, &session_id, &opts, |e| entries.push(e))?;
            // Rule-based classification only (no async AI in CLI for now)
            for entry in &mut entries {
                entry.classification = Some(ls_core::classifier::download::classify(entry));
            }
            let config = OrganizerConfig {
                target_root: target,
                dry_run: !execute,
                on_conflict: ConflictStrategy::Rename,
            };
            let mut actions = organizer::propose_actions(&entries, &config);
            println!("{} Vorschläge{}:", actions.len(), if execute { "" } else { " (dry-run)" });
            for action in &mut actions {
                println!("  {} → {}", action.source_path, action.target_path.as_deref().unwrap_or("-"));
                if execute {
                    if let Err(e) = organizer::execute_action(action) {
                        eprintln!("    Fehler: {e}");
                    }
                }
            }
        }
    }
    Ok(())
}
