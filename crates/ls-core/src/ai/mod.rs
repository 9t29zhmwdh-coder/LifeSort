pub mod ollama;
pub mod prompts;

use crate::models::Classification;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait AiBackend: Send + Sync {
    async fn classify_text(&self, text: &str, context: &str) -> Result<Classification>;
    async fn classify_image(&self, image_b64: &str) -> Result<Classification>;
    async fn is_available(&self) -> bool;
}
