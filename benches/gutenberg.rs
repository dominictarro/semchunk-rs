use std::fs::File;
use std::io::Read;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glob::glob;
use rust_tokenizers::tokenizer::{RobertaTokenizer, Tokenizer};
use semchunk_rs::Chunker;

const CHUNK_SIZE: usize = 512;

fn chunk_texts(texts: Vec<&str>, chunker: &Chunker) {
    for text in texts {
        let _ = chunker.chunk(text);
    }
}

fn read_gutenberg_texts() -> Vec<String> {
    let mut texts = Vec::new();
    for fp in glob("data/gutenberg/*.txt").unwrap() {
        let path = fp.unwrap();
        let mut file = File::open(&path).expect("Error opening file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Error reading file");
        let text = String::from_utf8_lossy(&buffer).to_string();
        texts.push(text);
    }
    texts
}

fn benchmark_chunk_texts(c: &mut Criterion) {
    let tokenizer = RobertaTokenizer::from_file(
        "data/roberta-base-vocab.json",
        "data/roberta-base-merges.txt",
        false,
        false,
    )
    .expect("Error loading tokenizer");
    let token_counter = Box::new(move |s: &str| tokenizer.tokenize(s).len());
    let chunker = Chunker::new(CHUNK_SIZE, token_counter);
    let texts = read_gutenberg_texts();
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    c.bench_function("gutenberg", |b| {
        b.iter(|| chunk_texts(black_box(text_refs.clone()), black_box(&chunker)));
    });
}

criterion_group!(benches, benchmark_chunk_texts);
criterion_main!(benches);
