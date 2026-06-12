use crate::ai::AiBackend;
use crate::models::{Classification, ClassifierKind, FileEntry};
use crate::ai::prompts::DOCUMENT_CLASSIFY;

pub async fn classify(entry: &FileEntry, ai: Option<&dyn AiBackend>) -> Classification {
    let text = extract_pdf_text(&entry.path);

    if let (Some(backend), Some(ref text)) = (ai, &text) {
        if !text.trim().is_empty() {
            let truncated = &text[..text.len().min(4000)];
            if let Ok(c) = backend.classify_text(truncated, DOCUMENT_CLASSIFY).await {
                return c;
            }
        }
    }

    // Rule-based fallback on extracted text
    if let Some(ref text) = text {
        return rule_classify_text(text);
    }

    Classification::unknown(ClassifierKind::Rules)
}

fn extract_pdf_text(path: &str) -> Option<String> {
    let doc = lopdf::Document::load(path).ok()?;
    let pages = doc.get_pages();
    let mut text = String::new();
    for (page_num, _) in pages.iter().take(5) {
        if let Ok(page_text) = doc.extract_text(&[*page_num]) {
            text.push_str(&page_text);
            text.push('\n');
        }
    }
    if text.trim().is_empty() { None } else { Some(text) }
}

pub fn rule_classify_text_pub(text: &str) -> Classification {
    rule_classify_text(text)
}

fn rule_classify_text(text: &str) -> Classification {
    use crate::models::{Category};
    use once_cell::sync::Lazy;
    use regex::Regex;

    static INVOICE:   Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(rechnung|invoice|betrag|total|eur|chf|usd|\bMwSt\b|\bVAT\b)").unwrap());
    static CONTRACT:  Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(vertrag|contract|vereinbarung|agreement|unterzeichnet|signiert)").unwrap());
    static GUARANTEE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(garantie|warranty|gewährleistung|garantieschein)").unwrap());
    static TAX:       Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(steuer|steuererklärung|finanzamt|deklaration|tax|steuerformular)").unwrap());

    let (category, confidence, tags) = if TAX.is_match(text) {
        (Category::TaxDocument, 0.65, vec!["steuer".into()])
    } else if GUARANTEE.is_match(text) {
        (Category::Guarantee, 0.65, vec!["garantie".into()])
    } else if CONTRACT.is_match(text) {
        (Category::Contract, 0.65, vec!["vertrag".into()])
    } else if INVOICE.is_match(text) {
        (Category::Invoice, 0.65, vec!["rechnung".into()])
    } else {
        (Category::Report, 0.3, vec!["pdf".into()])
    };

    // Extract date with simple regex
    let extracted_date = extract_date(text);

    Classification {
        category,
        subcategory: None,
        confidence,
        tags,
        extracted_date,
        extracted_amount: extract_amount(text),
        extracted_sender: None,
        ai_summary: None,
        classified_by: ClassifierKind::Rules,
    }
}

fn extract_date(text: &str) -> Option<chrono::NaiveDate> {
    use once_cell::sync::Lazy;
    use regex::Regex;
    static DATE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(\d{1,2})[.\-/](\d{1,2})[.\-/](\d{2,4})").unwrap()
    });
    let cap = DATE.captures(text)?;
    let d: u32 = cap[1].parse().ok()?;
    let m: u32 = cap[2].parse().ok()?;
    let y_raw: i32 = cap[3].parse().ok()?;
    let y = if y_raw < 100 { y_raw + 2000 } else { y_raw };
    chrono::NaiveDate::from_ymd_opt(y, m, d)
}

fn extract_amount(text: &str) -> Option<f64> {
    use once_cell::sync::Lazy;
    use regex::Regex;
    static AMOUNT: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?:CHF|EUR|USD|€|\$)\s*([\d'.,]+)").unwrap()
    });
    let cap = AMOUNT.captures(text)?;
    let s = cap[1].replace('\'', "").replace(',', ".");
    s.parse().ok()
}
