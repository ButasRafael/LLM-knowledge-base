
use once_cell::sync::Lazy;
use tiktoken_rs::{o200k_base, CoreBPE};

static BPE: Lazy<CoreBPE> = Lazy::new(|| {
    o200k_base().unwrap_or_else(|_| {
        tiktoken_rs::cl100k_base().expect("No tokenizer data found!")
    })
});


pub fn count(text: &str) -> i32 {
    BPE.encode_with_special_tokens(text).len() as i32
}
