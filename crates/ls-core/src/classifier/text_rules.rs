//! Keyword rules for documents, used when no language model is available.
//!
//! Each category scores one point per distinct keyword found as a whole word,
//! and the highest score wins. The old version took the first rule that
//! matched anywhere, so "tax" on an ordinary invoice made it a tax document
//! and "eur" inside "teuer" made any text an invoice.

use crate::models::{Category, Classification, ClassifierKind};
use chrono::NaiveDate;
use once_cell::sync::Lazy;
use regex::Regex;

const RULES: &[(Category, &[&str])] = &[
    (Category::Invoice, &[
        "rechnung", "rechnungsnummer", "rechnungsbetrag", "invoice", "facture", "zahlbar",
        "zahlungsfrist", "fällig", "mwst", "ust", "vat", "amount due", "total chf",
        "total eur", "qr-rechnung", "einzahlungsschein", "iban",
    ]),
    (Category::TaxDocument, &[
        "steuererklärung", "steuerverwaltung", "steueramt", "finanzamt", "lohnausweis",
        "veranlagung", "steuerjahr", "steuerperiode", "tax return", "steuerbescheid",
    ]),
    (Category::Contract, &[
        "vertrag", "vertragsparteien", "vereinbarung", "contract", "agreement",
        "kündigungsfrist", "laufzeit", "unterzeichnet", "unterschrift",
    ]),
    (Category::Guarantee, &["garantie", "garantieschein", "gewährleistung", "warranty"]),
    (Category::Certificate, &["zertifikat", "certificate", "zeugnis", "diplom", "bescheinigung"]),
    (Category::Letter, &[
        "sehr geehrte", "sehr geehrter", "freundliche grüsse", "freundlichen grüssen",
        "freundlichen grüßen", "dear", "sincerely", "kind regards",
    ]),
];

static MATCHERS: Lazy<Vec<(Category, Vec<Regex>)>> = Lazy::new(|| {
    RULES
        .iter()
        .map(|(cat, words)| {
            let res = words
                .iter()
                .map(|w| Regex::new(&format!(r"(?i)\b{}\b", regex::escape(w))).unwrap())
                .collect();
            (*cat, res)
        })
        .collect()
});

pub fn classify(text: &str) -> Classification {
    let best = MATCHERS
        .iter()
        .map(|(cat, res)| (*cat, res.iter().filter(|r| r.is_match(text)).count()))
        // max_by_key keeps the last of equal scores; reversed, the first rule wins ties.
        .rev()
        .max_by_key(|(_, score)| *score)
        .filter(|(_, score)| *score > 0);

    let (category, confidence, tag) = match best {
        Some((cat, score)) => (cat, (0.45 + 0.1 * score as f32).min(0.85), cat.key()),
        None => (Category::Unknown, 0.2, "document".to_string()),
    };
    Classification {
        extracted_date: extract_date(text),
        extracted_amount: extract_amount(text),
        ..Classification::simple(category, confidence, &[tag.as_str()], ClassifierKind::Rules)
    }
}

fn extract_date(text: &str) -> Option<NaiveDate> {
    static DMY: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d{1,2})[./](\d{1,2})[./](\d{4}|\d{2})\b").unwrap());
    static ISO: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d{4})-(\d{2})-(\d{2})\b").unwrap());
    let dmy = DMY.captures_iter(text).find_map(|c| {
        let y: i32 = c[3].parse().ok()?;
        let y = if y < 100 { y + 2000 } else { y };
        NaiveDate::from_ymd_opt(y, c[2].parse().ok()?, c[1].parse().ok()?)
    });
    dmy.or_else(|| {
        ISO.captures_iter(text).find_map(|c| {
            NaiveDate::from_ymd_opt(c[1].parse().ok()?, c[2].parse().ok()?, c[3].parse().ok()?)
        })
    })
}

fn extract_amount(text: &str) -> Option<f64> {
    static BEFORE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?:CHF|EUR|USD|Fr\.|€|\$)\s*(\d[\d'’ .,]*\d|\d)").unwrap());
    static AFTER: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(\d[\d'’ .,]*\d|\d)\s*(?:CHF|EUR|USD|€)").unwrap());
    BEFORE
        .captures(text)
        .or_else(|| AFTER.captures(text))
        .and_then(|c| parse_amount(&c[1]))
}

/// Understands 1'234.50 (CH), 1.234,50 (DE) and 1,234.50 (EN).
fn parse_amount(raw: &str) -> Option<f64> {
    let s: String = raw.chars().filter(|c| !matches!(c, '\'' | '’' | ' ')).collect();
    let decimal = match (s.rfind('.'), s.rfind(',')) {
        (Some(d), Some(c)) => Some(if d > c { '.' } else { ',' }),
        (None, Some(c)) if s.len() - c == 3 => Some(','),
        (Some(d), None) if s.len() - d == 3 => Some('.'),
        _ => None,
    };
    let normalized: String = s
        .chars()
        .filter_map(|ch| match ch {
            '.' | ',' if Some(ch) == decimal => Some('.'),
            '.' | ',' => None,
            other => Some(other),
        })
        .collect();
    normalized.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_invoice_with_tax_line_stays_an_invoice() {
        let c = classify("INVOICE No. 4711\nSubtotal 100.00\nTax 7.70\nAmount due CHF 107.70");
        assert_eq!(c.category, Category::Invoice);
        assert_eq!(c.extracted_amount, Some(107.70));
    }

    #[test]
    fn keyword_inside_a_word_does_not_count() {
        // "teuer" contains "eur", "Syntax" contains "tax".
        let c = classify("Das war teuer. Die Syntax stimmt.");
        assert_eq!(c.category, Category::Unknown);
    }

    #[test]
    fn swiss_tax_return_is_a_tax_document() {
        let c = classify("Steuererklärung 2024\nKantonale Steuerverwaltung Aargau\nLohnausweis beigelegt");
        assert_eq!(c.category, Category::TaxDocument);
    }

    #[test]
    fn letter_with_signature() {
        let c = classify("Sehr geehrte Frau Muster\n\nwir freuen uns.\n\nFreundliche Grüsse");
        assert_eq!(c.category, Category::Letter);
    }

    #[test]
    fn amounts_in_three_notations() {
        assert_eq!(parse_amount("1'234.50"), Some(1234.50));
        assert_eq!(parse_amount("1.234,50"), Some(1234.50));
        assert_eq!(parse_amount("1,234.50"), Some(1234.50));
        assert_eq!(parse_amount("45"), Some(45.0));
        assert_eq!(extract_amount("Total 89,90 €"), Some(89.90));
    }

    #[test]
    fn dates_swiss_and_iso() {
        assert_eq!(extract_date("Datum: 31.12.2024"), NaiveDate::from_ymd_opt(2024, 12, 31));
        assert_eq!(extract_date("issued 2025-02-03"), NaiveDate::from_ymd_opt(2025, 2, 3));
        assert_eq!(extract_date("Version 99.99.99"), None);
    }
}
