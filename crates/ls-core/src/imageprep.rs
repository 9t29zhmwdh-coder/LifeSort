//! Turns any photo LifeSort can read into a small JPEG a vision model accepts.
//!
//! Sending the original is wrong twice: iPhone photos are HEIC, which Ollama
//! cannot decode, and a 4 MB JPEG costs seconds of upload and tokenisation
//! for no gain, since the models look at a few hundred pixels anyway.

use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;
use std::path::Path;

/// Longest edge sent to the model. llava works at 336 px, newer vision models
/// at up to about 1000, so 1024 keeps detail without wasting time.
pub const MODEL_EDGE: u32 = 1024;

/// A JPEG of at most `MODEL_EDGE` pixels on the longer side, or `None` when
/// the file cannot be decoded on this platform.
pub fn prepare_for_model(path: &Path) -> Option<Vec<u8>> {
    let img = decode(path)?;
    // thumbnail() scales up as well as down.
    let img = if img.width().max(img.height()) > MODEL_EDGE {
        img.thumbnail(MODEL_EDGE, MODEL_EDGE)
    } else {
        img
    };
    encode_jpeg(&img)
}

/// Pixel size, including HEIC on macOS.
pub fn dimensions(path: &Path) -> Option<(u32, u32)> {
    image::image_dimensions(path).ok().or_else(|| heic::dimensions(path))
}

fn decode(path: &Path) -> Option<DynamicImage> {
    image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
        .or_else(|| heic::decode(path))
}

fn encode_jpeg(img: &DynamicImage) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, 85)
        .encode_image(&img.to_rgb8())
        .ok()?;
    Some(out)
}

/// HEIC through `sips`, which ships with every macOS. Rust has no HEIC
/// decoder without pulling in libheif, and Windows or Linux users rarely have
/// HEIC files that did not come off an iPhone via a Mac anyway.
#[cfg(target_os = "macos")]
mod heic {
    use super::*;
    use std::process::Command;

    pub fn decode(path: &Path) -> Option<DynamicImage> {
        let tmp = std::env::temp_dir().join(format!("lifesort-{}.jpg", uuid::Uuid::new_v4()));
        let status = Command::new("/usr/bin/sips")
            .args(["-s", "format", "jpeg", "-Z", &MODEL_EDGE.to_string()])
            .arg(path)
            .arg("--out")
            .arg(&tmp)
            .output()
            .ok()?
            .status;
        let img = status.success().then(|| image::open(&tmp).ok()).flatten();
        let _ = std::fs::remove_file(&tmp);
        img
    }

    pub fn dimensions(path: &Path) -> Option<(u32, u32)> {
        let out = Command::new("/usr/bin/sips")
            .args(["-g", "pixelWidth", "-g", "pixelHeight"])
            .arg(path)
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let value = |key: &str| {
            text.lines()
                .find_map(|l| l.trim().strip_prefix(key))
                .and_then(|v| v.trim().parse::<u32>().ok())
        };
        Some((value("pixelWidth:")?, value("pixelHeight:")?))
    }
}

#[cfg(not(target_os = "macos"))]
mod heic {
    use super::*;
    pub fn decode(_: &Path) -> Option<DynamicImage> {
        None
    }
    pub fn dimensions(_: &Path) -> Option<(u32, u32)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, RgbImage};

    fn write_png(w: u32, h: u32) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("ls-prep-{}.png", uuid::Uuid::new_v4()));
        RgbImage::from_pixel(w, h, image::Rgb([200, 30, 30])).save(&path).unwrap();
        path
    }

    /// The old code sent the first 512 KB of the file, which for any real
    /// photo is a cut-off JPEG. The model must get a complete, decodable image.
    #[test]
    fn large_image_becomes_small_complete_jpeg() {
        let path = write_png(4032, 3024);
        let jpeg = prepare_for_model(&path).unwrap();
        let back = image::load_from_memory_with_format(&jpeg, ImageFormat::Jpeg).unwrap();
        assert_eq!((back.width(), back.height()), (1024, 768));
    }

    #[test]
    fn small_image_is_not_enlarged() {
        let path = write_png(300, 200);
        let jpeg = prepare_for_model(&path).unwrap();
        let back = image::load_from_memory(&jpeg).unwrap();
        assert_eq!((back.width(), back.height()), (300, 200));
    }

    /// HEIC is what every iPhone writes. Converted with sips here, the same
    /// tool the code uses, so the test needs no fixture file in the repo.
    #[cfg(target_os = "macos")]
    #[test]
    fn heic_is_readable_on_macos() {
        let png = write_png(2000, 1500);
        let heic = png.with_extension("heic");
        let ok = std::process::Command::new("/usr/bin/sips")
            .args(["-s", "format", "heic"])
            .arg(&png)
            .arg("--out")
            .arg(&heic)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "sips could not write the HEIC fixture");
        assert!(image::image_dimensions(&heic).is_err(), "image crate learned HEIC, drop the sips path");
        assert_eq!(dimensions(&heic), Some((2000, 1500)));
        let jpeg = prepare_for_model(&heic).unwrap();
        let back = image::load_from_memory(&jpeg).unwrap();
        assert_eq!((back.width(), back.height()), (1024, 768));
    }
}
