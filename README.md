# semchunk-rs

A port of [umarbutler/semchunk](https://github.com/umarbutler/semchunk) into Rust for splitting text into semantically meaningful chunks.

## Usage

```rust
use semchunk::Chunker;
use rust_tokenizers::tokenizer::{RobertaTokenizer, Tokenizer};

const CHUNK_SIZE: usize = 4;

fn main() {
    let tokenizer = RobertaTokenizer
        ::from_file("data/roberta-base-vocab.json", "data/roberta-base-merges.txt", false, false)
        .unwrap();
    let token_counter = Box::new(move |s: &str| tokenizer.tokenize(s).len());
    let chunker = Chunker::new(4, token_counter);

    let text = "The quick brown fox jumps over the lazy dog.";
    let chunks = chunker.chunk(text);

    for (i, chunk) in chunks.iter().enumerate() {
        println!("{}) {}", i + 1, chunk);
    }
}
```

```text
1) The quick brown fox
2) jumps over the
3) lazy dog.
```

## Benchmarks 📊

**Environment:**

| Component | Version |
| --- | --- |
| Rust Version | 1.80.1 |
| Computer | Apple 2022 Macbook Pro |
| Processor | Apple M2 |
| Memory | 24 GB |
| Operating System | Sequoia 15.1.1 |

### gutenberg

Benchmarking against the 18 texts of the Gutenberg corpus which contains 3,001,260 tokens. Code [here](benches/gutenberg.rs).

| Parameter | Value |
| --- | --- |
| Iterations | 100 |
| Chunk Size | 512 |
| Tokenizer | RoBERTa (base) |
| Tokenizer Library | `rust_tokenizers` (8.1.1) |

| Metric | semchunk-rs |
| --- | --- |
| Mean | 6.2223 s |
| Min | 6.2040 s |
| Max | 6.2431 s |
