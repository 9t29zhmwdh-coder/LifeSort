//! Any text a PDF or text file can contain goes through the keyword rules
//! and the cut for the model. Neither may panic; the cut must stay within
//! the byte limit and on a character boundary.
#![no_main]
use libfuzzer_sys::fuzz_target;
use ls_core::classifier::{text_rules, truncate_utf8, MAX_TEXT_BYTES};

fuzz_target!(|text: &str| {
    let _ = text_rules::classify(text);
    let cut = truncate_utf8(text, MAX_TEXT_BYTES);
    assert!(cut.len() <= MAX_TEXT_BYTES);
    assert!(text.starts_with(cut));
});
