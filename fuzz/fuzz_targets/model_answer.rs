//! Whatever a model answers, prose, broken JSON or JSON inside a code fence,
//! parsing it must return a result or an error, never panic.
#![no_main]
use libfuzzer_sys::fuzz_target;
use ls_core::ai::ollama::{parse_document, parse_photo};

fuzz_target!(|answer: &str| {
    let _ = parse_photo(answer);
    let _ = parse_document(answer);
});
