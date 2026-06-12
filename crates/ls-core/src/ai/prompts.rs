pub const DOCUMENT_CLASSIFY: &str = r#"You are a document classifier. Analyze the following text and return ONLY a JSON object with these fields:
{
  "category": one of ["invoice","contract","guarantee","tax","letter","certificate","report","unknown"],
  "subcategory": string or null,
  "confidence": number 0.0-1.0,
  "tags": array of strings (max 5),
  "extracted_date": "YYYY-MM-DD" or null,
  "extracted_amount": number or null,
  "extracted_sender": string or null,
  "summary": one sentence in the document's language or null
}
Return ONLY valid JSON, no explanation."#;

pub const PHOTO_CLASSIFY: &str = r#"Analyze this image and return ONLY a JSON object:
{
  "category": one of ["person","landscape","event","screenshot","meme","document","other"],
  "tags": array of strings (max 5, e.g. "outdoor","group","receipt","text"),
  "confidence": number 0.0-1.0,
  "has_text": boolean,
  "is_screenshot": boolean,
  "scene_description": one sentence or null
}
Return ONLY valid JSON."#;

pub const DOWNLOAD_CLASSIFY: &str = r#"Classify this file based on its name and context. Return ONLY JSON:
{
  "category": one of ["installer","archive","asset","junk","media","unknown"],
  "confidence": number 0.0-1.0,
  "tags": array of strings (max 3)
}
Return ONLY valid JSON."#;
