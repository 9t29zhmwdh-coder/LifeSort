//! PhotoKit calls. Everything here blocks and must run off the main thread.

use crate::{Access, AssetKind, PhotoAsset};
use anyhow::{bail, Context, Result};
use block2::RcBlock;
use chrono::{DateTime, TimeZone, Utc};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{sel, Message};
use objc2_app_kit::NSImage;
use objc2_core_foundation::CGSize;
use objc2_foundation::{ns_string, NSArray, NSDictionary, NSNumber, NSObjectNSKeyValueCoding, NSObjectProtocol, NSString};
use objc2_photos::{
    PHAccessLevel, PHAsset, PHAssetCollection, PHAssetCollectionChangeRequest, PHAssetCollectionSubtype,
    PHAssetCollectionType, PHAssetMediaSubtype, PHAssetMediaType, PHAssetResource, PHAuthorizationStatus,
    PHFetchOptions, PHImageContentMode, PHImageManager, PHImageRequestOptions, PHImageRequestOptionsDeliveryMode,
    PHPhotoLibrary,
};
use std::sync::{mpsc, Arc, Mutex};

fn map_status(s: PHAuthorizationStatus) -> Access {
    match s {
        PHAuthorizationStatus::Authorized => Access::Authorized,
        PHAuthorizationStatus::Limited => Access::Limited,
        PHAuthorizationStatus::Denied => Access::Denied,
        PHAuthorizationStatus::NotDetermined => Access::NotDetermined,
        _ => Access::Restricted,
    }
}

pub fn access() -> Access {
    map_status(unsafe { PHPhotoLibrary::authorizationStatusForAccessLevel(PHAccessLevel::ReadWrite) })
}

/// Shows the system prompt the first time and waits for the answer.
pub fn request_access() -> Access {
    let (tx, rx) = mpsc::channel();
    let handler = RcBlock::new(move |status: PHAuthorizationStatus| {
        let _ = tx.send(status);
    });
    unsafe { PHPhotoLibrary::requestAuthorizationForAccessLevel_handler(PHAccessLevel::ReadWrite, &handler) };
    rx.recv().map(map_status).unwrap_or(Access::Denied)
}

/// PHFetchResult has no Rust iterator; this walks it by index.
fn fetched<T: Message>(result: &objc2_photos::PHFetchResult<T>) -> Vec<Retained<T>> {
    (0..unsafe { result.count() }).map(|i| unsafe { result.objectAtIndex(i) }).collect()
}

fn to_utc(date: Option<Retained<objc2_foundation::NSDate>>) -> Option<DateTime<Utc>> {
    let secs = date?.timeIntervalSince1970();
    Utc.timestamp_opt(secs as i64, 0).single()
}

/// Bytes of all stored resources of an asset. PHAssetResource has no public
/// size property; the private `fileSize` getter is what every photo cleaner
/// uses. Checked with respondsToSelector first, so a future macOS without it
/// yields 0 instead of an exception.
fn resource_bytes(asset: &PHAsset) -> u64 {
    let resources = unsafe { PHAssetResource::assetResourcesForAsset(asset) };
    resources
        .iter()
        .map(|r| {
            if !r.respondsToSelector(sel!(fileSize)) {
                return 0;
            }
            // Through key-value coding, which boxes the value in an NSNumber
            // whatever integer type the private getter has.
            unsafe { r.valueForKey(ns_string!("fileSize")) }
                .and_then(|v| v.downcast::<NSNumber>().ok())
                .map(|n| n.unsignedLongLongValue())
                .unwrap_or(0)
        })
        .sum()
}

fn original_filename(asset: &PHAsset) -> String {
    let resources = unsafe { PHAssetResource::assetResourcesForAsset(asset) };
    resources.iter().next().map(|r| unsafe { r.originalFilename() }.to_string()).unwrap_or_default()
}

/// Every photo and video, burst frames included, hidden ones left out.
pub fn list_assets(mut on_progress: impl FnMut(usize, usize)) -> Result<Vec<PhotoAsset>> {
    match access() {
        Access::Authorized | Access::Limited => {}
        other => bail!("no access to the Photos library ({other:?})"),
    }
    let options = unsafe { PHFetchOptions::new() };
    unsafe {
        options.setIncludeAllBurstAssets(true);
        options.setIncludeHiddenAssets(false);
    }
    let result = unsafe { PHAsset::fetchAssetsWithOptions(Some(&options)) };
    let total = unsafe { result.count() };
    let mut out = Vec::with_capacity(total);
    for i in 0..total {
        let asset = unsafe { result.objectAtIndex(i) };
        let kind = match unsafe { asset.mediaType() } {
            PHAssetMediaType::Image => AssetKind::Image,
            PHAssetMediaType::Video => AssetKind::Video,
            _ => AssetKind::Other,
        };
        let burst = unsafe { asset.burstIdentifier() }.is_some();
        out.push(PhotoAsset {
            id: unsafe { asset.localIdentifier() }.to_string(),
            kind,
            screenshot: unsafe { asset.mediaSubtypes() }.contains(PHAssetMediaSubtype::PhotoScreenshot),
            burst_extra: burst && !unsafe { asset.representsBurst() },
            width: unsafe { asset.pixelWidth() } as u32,
            height: unsafe { asset.pixelHeight() } as u32,
            duration_s: unsafe { asset.duration() },
            bytes: resource_bytes(&asset),
            filename: original_filename(&asset),
            created: to_utc(unsafe { asset.creationDate() }),
            favorite: unsafe { asset.isFavorite() },
        });
        if i % 100 == 0 || i + 1 == total {
            on_progress(i + 1, total);
        }
    }
    Ok(out)
}

fn fetch_by_ids(ids: &[String]) -> Retained<objc2_photos::PHFetchResult<PHAsset>> {
    let ns: Vec<Retained<NSString>> = ids.iter().map(|s| NSString::from_str(s)).collect();
    let array = NSArray::from_retained_slice(&ns);
    unsafe { PHAsset::fetchAssetsWithLocalIdentifiers_options(&array, None) }
}

/// A JPEG of at most `edge` pixels. Fetches a preview from iCloud when the
/// original is not on this Mac, never the full original.
pub fn thumbnail_jpeg(id: &str, edge: u32) -> Option<Vec<u8>> {
    let result = fetch_by_ids(&[id.to_string()]);
    let asset = unsafe { result.firstObject() }?;
    let options = unsafe { PHImageRequestOptions::new() };
    unsafe {
        options.setSynchronous(true);
        options.setNetworkAccessAllowed(true);
        options.setDeliveryMode(PHImageRequestOptionsDeliveryMode::HighQualityFormat);
    }
    let slot: Arc<Mutex<Option<Retained<NSImage>>>> = Arc::default();
    let sink = slot.clone();
    let handler = RcBlock::new(move |image: *mut NSImage, _info: *mut NSDictionary| {
        if let Some(img) = unsafe { Retained::retain(image) } {
            *sink.lock().unwrap() = Some(img);
        }
    });
    let size = CGSize { width: edge as f64, height: edge as f64 };
    unsafe {
        PHImageManager::defaultManager().requestImageForAsset_targetSize_contentMode_options_resultHandler(
            &asset,
            size,
            PHImageContentMode::AspectFit,
            Some(&options),
            &handler,
        );
    }
    let image = slot.lock().unwrap().take()?;
    let tiff = image.TIFFRepresentation()?.to_vec();
    let decoded = image::load_from_memory_with_format(&tiff, image::ImageFormat::Tiff).ok()?;
    let decoded = if decoded.width().max(decoded.height()) > edge { decoded.thumbnail(edge, edge) } else { decoded };
    let mut jpeg = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 85)
        .encode_image(&decoded.to_rgb8())
        .ok()?;
    Some(jpeg)
}

fn find_album(title: &str) -> Option<Retained<PHAssetCollection>> {
    let albums = unsafe {
        PHAssetCollection::fetchAssetCollectionsWithType_subtype_options(
            PHAssetCollectionType::Album,
            PHAssetCollectionSubtype::AlbumRegular,
            None,
        )
    };
    fetched(&albums)
        .into_iter()
        .find(|a| unsafe { a.localizedTitle() }.is_some_and(|t| t.to_string() == title))
}

/// Adds the assets to the album with this title, creating it when needed.
/// Returns how many assets were found to add. Adding to an album changes
/// nothing about the photos themselves.
pub fn add_to_album(title: &str, ids: &[String]) -> Result<usize> {
    let assets = fetched(&fetch_by_ids(ids));
    if assets.is_empty() {
        return Ok(0);
    }
    let found = assets.len();
    let array = NSArray::from_retained_slice(&assets);
    let existing = find_album(title);
    let title = NSString::from_str(title);
    let change = RcBlock::new(move || {
        let request = match &existing {
            Some(album) => unsafe { PHAssetCollectionChangeRequest::changeRequestForAssetCollection(album) },
            None => Some(unsafe { PHAssetCollectionChangeRequest::creationRequestForAssetCollectionWithTitle(&title) }),
        };
        if let Some(request) = request {
            unsafe { request.addAssets(ProtocolObject::from_ref(&*array)) };
        }
    });
    let library = unsafe { PHPhotoLibrary::sharedPhotoLibrary() };
    unsafe { library.performChangesAndWait_error(&*change as *const _ as *mut _) }
        .map_err(|e| anyhow::anyhow!(e.localizedDescription().to_string()))
        .context("Photos refused the album change")?;
    Ok(found)
}
