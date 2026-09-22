//! The Apple Photos library, read through PhotoKit.
//!
//! LifeSort never moves or deletes anything here. It reads what is in the
//! library, groups what takes space, and collects groups in albums. Deleting
//! stays in the Photos app, where it syncs to every device through iCloud
//! and can be undone for 30 days under "Recently Deleted".

use chrono::{DateTime, Utc};
use serde::Serialize;

mod report;
pub use report::{group_assets, Group, GroupKey};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{access, add_to_album, list_assets, request_access, thumbnail_jpeg};

/// Stand-ins for platforms without PhotoKit.
#[cfg(not(target_os = "macos"))]
mod unsupported {
    use super::*;
    pub fn access() -> Access {
        Access::Unsupported
    }
    pub fn request_access() -> Access {
        Access::Unsupported
    }
    pub fn list_assets(_: impl FnMut(usize, usize)) -> anyhow::Result<Vec<PhotoAsset>> {
        anyhow::bail!("the Photos library exists only on macOS")
    }
    pub fn thumbnail_jpeg(_: &str, _: u32) -> Option<Vec<u8>> {
        None
    }
    pub fn add_to_album(_: &str, _: &[String]) -> anyhow::Result<usize> {
        anyhow::bail!("the Photos library exists only on macOS")
    }
}
#[cfg(not(target_os = "macos"))]
pub use unsupported::*;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    Authorized,
    /// The user shared only selected photos; LifeSort sees just those.
    Limited,
    Denied,
    NotDetermined,
    Restricted,
    Unsupported,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Image,
    Video,
    Other,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhotoAsset {
    /// PhotoKit's local identifier; stable while the library exists.
    pub id: String,
    pub kind: AssetKind,
    pub screenshot: bool,
    /// A burst frame the camera kept but nobody picked.
    pub burst_extra: bool,
    pub width: u32,
    pub height: u32,
    pub duration_s: f64,
    /// Sum of the stored resources: photo, Live Photo video, edits.
    pub bytes: u64,
    pub filename: String,
    pub created: Option<DateTime<Utc>>,
    pub favorite: bool,
}
