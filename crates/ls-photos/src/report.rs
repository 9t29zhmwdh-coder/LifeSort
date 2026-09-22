//! Groups of assets that are worth a look when space runs out.

use crate::{AssetKind, PhotoAsset};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum GroupKey {
    /// All videos, largest first: usually most of the space.
    Videos,
    Screenshots,
    /// Burst frames that were never picked.
    BurstExtras,
    /// Memes, greeting images and similar, as recognised by the model.
    Memes,
    /// Photos of receipts, letters and screens, as recognised by the model.
    PhotographedDocuments,
}

#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub key: GroupKey,
    pub count: usize,
    pub bytes: u64,
    /// Largest first, so an album shows the big wins at the top.
    pub ids: Vec<String>,
}

/// Groups the assets. `ai` maps asset ids to a group the model assigned.
/// Favourites never land in a group: nobody wants those suggested for
/// deletion. An asset can be in more than one group, for example a screen
/// recording is a video and a screenshot-like image is also a meme.
pub fn group_assets(assets: &[PhotoAsset], ai: &HashMap<String, GroupKey>) -> Vec<Group> {
    let mut buckets: HashMap<GroupKey, Vec<&PhotoAsset>> = HashMap::new();
    for a in assets.iter().filter(|a| !a.favorite) {
        let mut put = |k| buckets.entry(k).or_default().push(a);
        if a.kind == AssetKind::Video {
            put(GroupKey::Videos);
        }
        if a.screenshot {
            put(GroupKey::Screenshots);
        }
        if a.burst_extra {
            put(GroupKey::BurstExtras);
        }
        if let Some(k) = ai.get(&a.id) {
            if !(a.screenshot && *k == GroupKey::PhotographedDocuments) {
                put(*k);
            }
        }
    }
    let mut groups: Vec<Group> = buckets
        .into_iter()
        .map(|(key, mut members)| {
            members.sort_by_key(|a| std::cmp::Reverse(a.bytes));
            Group {
                key,
                count: members.len(),
                bytes: members.iter().map(|a| a.bytes).sum(),
                ids: members.iter().map(|a| a.id.clone()).collect(),
            }
        })
        .collect();
    groups.sort_by_key(|g| std::cmp::Reverse(g.bytes));
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(id: &str, kind: AssetKind, bytes: u64) -> PhotoAsset {
        PhotoAsset {
            id: id.into(),
            kind,
            screenshot: false,
            burst_extra: false,
            width: 100,
            height: 100,
            duration_s: 0.0,
            bytes,
            filename: format!("{id}.jpg"),
            created: None,
            favorite: false,
        }
    }

    #[test]
    fn groups_are_sorted_by_space_and_members_by_size() {
        let mut shot = asset("s", AssetKind::Image, 2_000_000);
        shot.screenshot = true;
        let assets = vec![
            asset("v1", AssetKind::Video, 50_000_000),
            asset("v2", AssetKind::Video, 900_000_000),
            shot,
            asset("p", AssetKind::Image, 3_000_000),
        ];
        let groups = group_assets(&assets, &HashMap::new());
        assert_eq!(groups[0].key, GroupKey::Videos);
        assert_eq!(groups[0].ids, vec!["v2", "v1"]);
        assert_eq!(groups[0].bytes, 950_000_000);
        assert_eq!(groups[1].key, GroupKey::Screenshots);
        assert_eq!(groups.len(), 2, "an ordinary photo is in no group");
    }

    #[test]
    fn favourites_are_never_suggested() {
        let mut fav = asset("fav", AssetKind::Video, 1_000_000_000);
        fav.favorite = true;
        let ai = HashMap::from([("fav".to_string(), GroupKey::Memes)]);
        assert!(group_assets(&[fav], &ai).is_empty());
    }

    #[test]
    fn model_groups_and_burst_extras() {
        let mut extra = asset("b", AssetKind::Image, 4_000_000);
        extra.burst_extra = true;
        let ai = HashMap::from([
            ("m".to_string(), GroupKey::Memes),
            ("d".to_string(), GroupKey::PhotographedDocuments),
        ]);
        let groups = group_assets(
            &[extra, asset("m", AssetKind::Image, 100_000), asset("d", AssetKind::Image, 900_000)],
            &ai,
        );
        let keys: Vec<GroupKey> = groups.iter().map(|g| g.key).collect();
        assert_eq!(keys, vec![GroupKey::BurstExtras, GroupKey::PhotographedDocuments, GroupKey::Memes]);
    }
}
