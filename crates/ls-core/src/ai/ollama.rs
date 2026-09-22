use super::AiBackend;
use crate::models::{Category, Classification, ClassifierKind};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

pub struct OllamaBackend {
    pub base_url: String,
    pub text_model: String,
    pub vision_model: String,
    client: reqwest::Client,
    /// Models that rejected `format: "json"`. Ollama's MLX engine answers it
    /// with 501 "structured output is unavailable"; such models are asked
    /// without the flag and the JSON is read out of the plain answer.
    plain_output: Mutex<HashSet<String>>,
}

/// What the app can tell the user before a long classification run.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AiStatus {
    Ready,
    Unreachable,
    MissingModels { models: Vec<String> },
}

impl OllamaBackend {
    pub fn new(base_url: String, text_model: String, vision_model: String) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(3))
            // A vision model loading into memory for the first time can take
            // a minute on an 8 GB Mac; after that an answer takes seconds.
            .timeout(Duration::from_secs(300))
            .build()
            .expect("static client configuration");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            text_model,
            vision_model,
            client,
            plain_output: Mutex::default(),
        }
    }

    /// Reachable, and are both configured models installed? Without this a
    /// missing model made every request fail with 404, and the app quietly
    /// fell back to rules for the whole run.
    pub async fn status(&self) -> AiStatus {
        let installed = match self.installed_models().await {
            Ok(m) => m,
            Err(_) => return AiStatus::Unreachable,
        };
        let mut missing: Vec<String> = [&self.text_model, &self.vision_model]
            .into_iter()
            .filter(|want| !installed.iter().any(|have| model_matches(want, have)))
            .cloned()
            .collect();
        missing.dedup();
        if missing.is_empty() { AiStatus::Ready } else { AiStatus::MissingModels { models: missing } }
    }

    async fn installed_models(&self) -> Result<Vec<String>> {
        #[derive(Deserialize)]
        struct Tags { models: Vec<Model> }
        #[derive(Deserialize)]
        struct Model { name: String }
        let tags: Tags = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(5))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    async fn generate(&self, model: &str, prompt: String, images: Option<Vec<String>>) -> Result<String> {
        let plain = self.plain_output.lock().map(|s| s.contains(model)).unwrap_or(false);
        if !plain {
            let req = GenerateRequest { model, prompt: &prompt, images: images.as_deref(), stream: false, format: Some("json") };
            let resp = self.client.post(format!("{}/api/generate", self.base_url)).json(&req).send().await?;
            if resp.status() != reqwest::StatusCode::NOT_IMPLEMENTED {
                return read_reply(resp, model).await;
            }
            if let Ok(mut set) = self.plain_output.lock() {
                set.insert(model.to_string());
            }
        }
        let req = GenerateRequest { model, prompt: &prompt, images: images.as_deref(), stream: false, format: None };
        let resp = self.client.post(format!("{}/api/generate", self.base_url)).json(&req).send().await?;
        read_reply(resp, model).await
    }
}

async fn read_reply(resp: reqwest::Response, model: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct Reply { response: String }
    let reply: Reply = resp
        .error_for_status()
        .with_context(|| format!("Ollama rejected the request for model {model}"))?
        .json()
        .await?;
    Ok(reply.response)
}

/// The JSON object inside an answer. Without the JSON format flag, models
/// wrap it in prose or a Markdown code fence.
fn json_object(text: &str) -> Result<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(text.trim()) {
        return Ok(v);
    }
    let start = text.find('{').context("no JSON object in the answer")?;
    let end = text.rfind('}').context("no JSON object in the answer")?;
    anyhow::ensure!(end > start, "no JSON object in the answer");
    Ok(serde_json::from_str(&text[start..=end])?)
}

/// "llava" is stored by Ollama as "llava:latest".
fn model_matches(wanted: &str, installed: &str) -> bool {
    installed == wanted || (!wanted.contains(':') && installed == format!("{wanted}:latest"))
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<&'a [String]>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'a str>,
}

#[async_trait]
impl AiBackend for OllamaBackend {
    async fn classify_text(&self, text: &str, context: &str) -> Result<Classification> {
        let raw = self.generate(&self.text_model, format!("{context}\n\nText:\n{text}"), None).await?;
        parse_document(&raw)
    }

    async fn classify_image(&self, image_b64: &str) -> Result<Classification> {
        let prompt = super::prompts::PHOTO_CLASSIFY.to_string();
        let raw = self.generate(&self.vision_model, prompt, Some(vec![image_b64.to_string()])).await?;
        parse_photo(&raw)
    }

    async fn is_available(&self) -> bool {
        self.status().await == AiStatus::Ready
    }
}

fn confidence(v: &Value, default: f64) -> f32 {
    v["confidence"].as_f64().unwrap_or(default).clamp(0.0, 1.0) as f32
}

fn string_list(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| a.iter().filter_map(|t| t.as_str().map(|s| s.trim().to_lowercase())).filter(|s| !s.is_empty()).take(5).collect())
        .unwrap_or_default()
}

pub fn parse_document(answer: &str) -> Result<Classification> {
    let v = json_object(answer)?;
    let category = match v["category"].as_str().unwrap_or("unknown") {
        "invoice" => Category::Invoice,
        "contract" => Category::Contract,
        "guarantee" => Category::Guarantee,
        "tax" => Category::TaxDocument,
        "letter" => Category::Letter,
        "certificate" => Category::Certificate,
        "report" => Category::Report,
        _ => Category::Unknown,
    };
    Ok(Classification {
        category,
        subcategory: v["subcategory"].as_str().map(String::from),
        confidence: confidence(&v, 0.5),
        tags: string_list(&v["tags"]),
        extracted_date: v["extracted_date"].as_str().and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
        extracted_amount: v["extracted_amount"].as_f64(),
        extracted_sender: v["extracted_sender"].as_str().map(String::from).filter(|s| !s.is_empty()),
        ai_summary: v["summary"].as_str().map(String::from).filter(|s| !s.is_empty()),
        classified_by: ClassifierKind::Ai,
    })
}

pub fn parse_photo(answer: &str) -> Result<Classification> {
    let v = json_object(answer)?;
    let category = if v["is_screenshot"].as_bool().unwrap_or(false) {
        Category::PhotoScreenshot
    } else {
        match v["category"].as_str().unwrap_or("other") {
            "person" => Category::PhotoPerson,
            "landscape" => Category::PhotoLandscape,
            "event" => Category::PhotoEvent,
            "screenshot" | "screen" => Category::PhotoScreenshot,
            "meme" => Category::PhotoMeme,
            "document" => Category::PhotoDocument,
            // Used to become "landscape", so a blurry pocket shot landed in
            // Places.
            _ => Category::PhotoOther,
        }
    };
    Ok(Classification {
        ai_summary: v["scene_description"].as_str().map(String::from).filter(|s| !s.is_empty()),
        tags: string_list(&v["tags"]),
        ..Classification::simple(category, confidence(&v, 0.7), &[], ClassifierKind::Ai)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_names_with_and_without_tag() {
        assert!(model_matches("llava", "llava:latest"));
        assert!(model_matches("llava:13b", "llava:13b"));
        assert!(!model_matches("llava", "llava:13b"));
        assert!(!model_matches("llama3", "llama3.2:latest"));
    }

    #[test]
    fn photo_other_is_not_a_landscape() {
        assert_eq!(parse_photo(r#"{"category":"other"}"#).unwrap().category, Category::PhotoOther);
        assert_eq!(parse_photo(r#"{"category":"person","is_screenshot":true}"#).unwrap().category, Category::PhotoScreenshot);
    }

    #[test]
    fn garbage_is_an_error_not_a_category() {
        assert!(parse_photo("not json").is_err());
    }

    #[test]
    fn json_is_found_inside_prose_and_code_fences() {
        let fenced = "Here you go:\n```json\n{\"category\": \"meme\"}\n```";
        assert_eq!(parse_photo(fenced).unwrap().category, Category::PhotoMeme);
    }
}
