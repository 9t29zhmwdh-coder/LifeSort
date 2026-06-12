pub mod hasher;

use crate::models::{DuplicateGroup, FileEntry};
use dashmap::DashMap;
use rayon::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Hash all entries in parallel, mutating their hash field.
pub fn compute_hashes(entries: &mut Vec<FileEntry>) {
    let hashes: Vec<_> = entries
        .par_iter()
        .map(|e| (e.id.clone(), hasher::hash_file(&e.path)))
        .collect();
    let map: HashMap<String, Option<String>> = hashes.into_iter().collect();
    for entry in entries.iter_mut() {
        entry.hash = map.get(&entry.id).and_then(|h| h.clone());
    }
}

/// Group entries by hash → returns duplicate groups (≥2 files with same hash).
pub fn find_duplicate_groups(entries: &[FileEntry]) -> Vec<DuplicateGroup> {
    let mut by_hash: HashMap<String, Vec<&FileEntry>> = HashMap::new();
    for entry in entries {
        if let Some(ref hash) = entry.hash {
            by_hash.entry(hash.clone()).or_default().push(entry);
        }
    }

    by_hash
        .into_iter()
        .filter(|(_, group)| group.len() > 1)
        .map(|(hash, group)| {
            let size = group[0].size;
            let wasted = size * (group.len() as u64 - 1);
            DuplicateGroup {
                id: Uuid::new_v4().to_string(),
                hash,
                size,
                file_ids: group.iter().map(|e| e.id.clone()).collect(),
                keep_id: None,
                total_wasted_bytes: wasted,
            }
        })
        .collect()
}
