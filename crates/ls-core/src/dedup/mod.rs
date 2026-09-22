pub mod hasher;

use crate::models::{DuplicateGroup, FileEntry};
use rayon::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Hashes only files that share their size with another file. Different
/// sizes cannot be duplicates, and skipping them saves reading most of a
/// photo library from disk.
pub fn compute_hashes(entries: &mut [FileEntry]) {
    let mut per_size: HashMap<u64, usize> = HashMap::new();
    for e in entries.iter() {
        *per_size.entry(e.size).or_default() += 1;
    }
    entries
        .par_iter_mut()
        .filter(|e| e.size > 0 && per_size[&e.size] > 1)
        .for_each(|e| e.hash = hasher::hash_file(&e.path));
}

/// Groups of at least two files with identical content, and marks each
/// member with its group id.
pub fn find_duplicate_groups(entries: &mut [FileEntry]) -> Vec<DuplicateGroup> {
    let mut by_hash: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, entry) in entries.iter().enumerate() {
        if let Some(ref hash) = entry.hash {
            by_hash.entry(hash.clone()).or_default().push(i);
        }
    }
    for e in entries.iter_mut() {
        e.duplicate_group_id = None;
    }

    let mut groups: Vec<DuplicateGroup> = by_hash
        .into_iter()
        .filter(|(_, idx)| idx.len() > 1)
        .map(|(hash, idx)| {
            let id = Uuid::new_v4().to_string();
            for &i in &idx {
                entries[i].duplicate_group_id = Some(id.clone());
            }
            let size = entries[idx[0]].size;
            DuplicateGroup {
                id,
                hash,
                size,
                file_ids: idx.iter().map(|&i| entries[i].id.clone()).collect(),
                keep_id: None,
                total_wasted_bytes: size * (idx.len() as u64 - 1),
            }
        })
        .collect();
    groups.sort_by_key(|g| std::cmp::Reverse(g.total_wasted_bytes));
    groups
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{scan_directory, ScanOptions};

    #[test]
    fn finds_copies_and_ignores_same_size_different_content() {
        let dir = std::env::temp_dir().join(format!("ls-dup-{}", Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.jpg"), b"same bytes").unwrap();
        std::fs::write(dir.join("sub/a copy.jpg"), b"same bytes").unwrap();
        std::fs::write(dir.join("b.jpg"), b"diff bytes").unwrap();
        std::fs::write(dir.join("unique.txt"), b"only one of this length").unwrap();

        let mut entries = vec![];
        scan_directory(&dir, "s", &ScanOptions::default(), |e| entries.push(e)).unwrap();
        compute_hashes(&mut entries);
        let groups = find_duplicate_groups(&mut entries);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].file_ids.len(), 2);
        assert_eq!(groups[0].total_wasted_bytes, 10);
        let unique = entries.iter().find(|e| e.name == "unique.txt").unwrap();
        assert!(unique.hash.is_none(), "a file with a unique size must not be read");
        assert_eq!(entries.iter().filter(|e| e.duplicate_group_id.is_some()).count(), 2);
    }
}
