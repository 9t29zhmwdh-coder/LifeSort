use super::AiBackend;
use crate::models::{Category, Classification, ClassifierKind};
use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct OllamaBackend {
    pub base_url: String,
    pub text_model: String,
    pub vision_model: String,
    client: reqwest::Client,
}

impl OllamaBackend {
    pub fn new(base_url: String, text_model: String, vision_model: String) -> Self {
        Self {
            base_url,
            text_model,
            vision_model,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
    stream: bool,
    format: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[async_trait]
impl AiBackend for OllamaBackend {
    async fn classify_text(&self, text: &str, context: &str) -> Result<Classification> {
        let prompt = format!("{context}\n\nText:\n{text}");
        let req = OllamaRequest {
            model: self.text_model.clone(),
            prompt,
            images: None,
            stream: false,
            format: "json".into(),
        };
        let resp: OllamaResponse = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await?
            .json()
            .await?;
        parse_classification(&resp.response, ClassifierKind::Ai)
    }

    async fn classify_image(&self, image_b64: &str) -> Result<Classification> {
        let req = OllamaRequest {
            model: self.vision_model.clone(),
            prompt: super::prompts::PHOTO_CLASSIFY.to_string(),
            images: Some(vec![image_b64.to_string()]),
            stream: false,
            format: "json".into(),
        };
        let resp: OllamaResponse = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await?
            .json()
            .await?;
        parse_photo_classification(&resp.response)
    }

    async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

fn parse_classification(json_str: &str, kind: ClassifierKind) -> Result<Classification> {
    let v: Value = serde_json::from_str(json_str)?;
    let category = match v["category"].as_str().unwrap_or("unknown") {
        "invoice"     => Category::Invoice,
        "contract"    => Category::Contract,
        "guarantee"   => Category::Guarantee,
        "tax"         => Category::TaxDocument,
        "letter"      => Category::Letter,
        "certificate" => Category::Certificate,
        "report"      => Category::Report,
        _             => Category::Unknown,
    };
    let tags = v["tags"]
        .as_array()
        .map(|a| a.iter().filter_map(|t| t.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let extracted_date = v["extracted_date"]
        .as_str()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let extracted_amount = v["extracted_amount"].as_f64();
    let extracted_sender = v["extracted_sender"].as_str().map(String::from);
    let ai_summary = v["summary"].as_str().map(String::from);
    let confidence = v["confidence"].as_f64().unwrap_or(0.5) as f32;

    Ok(Classification {
        category,
        subcategory: v["subcategory"].as_str().map(String::from),
        confidence,
        tags,
        extracted_date,
        extracted_amount,
        extracted_sender,
        ai_summary,
        classified_by: kind,
    })
}

fn parse_photo_classification(json_str: &str) -> Result<Classification> {
    let v: Value = serde_json::from_str(json_str)?;
    let category = match v["category"].as_str().unwrap_or("other") {
        "person"    => Category::PhotoPerson,
        "landscape" => Category::PhotoLandscape,
        "event"     => Category::PhotoEvent,
        "screenshot" | "screen" => Category::PhotoScreenshot,
        "meme"      => Category::PhotoMeme,
        "document"  => Category::PhotoDocument,
        _           => Category::PhotoLandscape,
    };
    // Override with is_screenshot flag
    let is_screenshot = v["is_screenshot"].as_bool().unwrap_or(false);
    let category = if is_screenshot { Category::PhotoScreenshot } else { category };

    let tags = v["tags"]
        .as_array()
        .map(|a| a.iter().filter_map(|t| t.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let confidence = v["confidence"].as_f64().unwrap_or(0.7) as f32;
    let ai_summary = v["scene_description"].as_str().map(String::from);

    Ok(Classification {
        category,
        subcategory: None,
        confidence,
        tags,
        extracted_date: None,
        extracted_amount: None,
        extracted_sender: None,
        ai_summary,
        classified_by: ClassifierKind::Ai,
    })
}
