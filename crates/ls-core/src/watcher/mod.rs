#![allow(unused_imports)]
use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct FileWatcher {
    _debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
}

impl FileWatcher {
    pub fn start(
        path: &Path,
        tx: mpsc::Sender<Vec<String>>,
    ) -> Result<Self> {
        let (debouncer_tx, debouncer_rx) = std::sync::mpsc::channel::<DebounceEventResult>();
        let tx_clone = tx.clone();

        std::thread::spawn(move || {
            for result in debouncer_rx {
                if let Ok(events) = result {
                    let paths: Vec<String> = events
                        .iter()
                        .flat_map(|e| e.event.paths.iter())
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect();
                    if !paths.is_empty() {
                        let _ = tx_clone.blocking_send(paths);
                    }
                }
            }
        });

        let mut debouncer = new_debouncer(Duration::from_secs(2), None, debouncer_tx)?;
        debouncer.watcher().watch(path, RecursiveMode::Recursive)?;

        Ok(Self { _debouncer: debouncer })
    }
}
