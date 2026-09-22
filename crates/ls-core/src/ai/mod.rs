pub mod ollama;
pub mod prompts;

/// One model for photos and documents, chosen by measurement (see the
/// README): the best result per gigabyte of memory, and small enough for an
/// 8 GB Mac. Apple silicon gets the MLX build, which Ollama runs on Apple's
/// own engine; everything else the GGUF build of the same model.
pub const DEFAULT_MODEL: &str = if cfg!(target_os = "macos") { "qwen3.5:4b-mlx" } else { "qwen3.5:4b" };

use crate::models::Classification;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait AiBackend: Send + Sync {
    async fn classify_text(&self, text: &str, context: &str) -> Result<Classification>;
    async fn classify_image(&self, image_b64: &str) -> Result<Classification>;
    async fn is_available(&self) -> bool;
}
