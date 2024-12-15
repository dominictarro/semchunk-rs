// MIT License
//
// Copyright (c) 2024 Dominic Tarro
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! # High performance text chunking in Rust
//! 
//! A port of [umarbutler/semchunk](https://github.com/umarbutler/semchunk) into Rust for splitting text into semantically meaningful chunks.
//! 
//! # Example
//! 
//! ```
//! use semchunk_rs::Chunker;
//! 
//! let chunker = Chunker::new(4, Box::new(|s: &str| s.len() - s.replace(" ", "").len() + 1));
//! let text = "The quick brown fox jumps over the lazy dog.";
//! let chunks = chunker.chunk(text);
//! assert_eq!(chunks, vec!["The quick brown fox", "jumps over the lazy", "dog."]);
//! ```
//! 
//! With `rust_tokenizers`:
//! 
//! ```
//! use rust_tokenizers::tokenizer::{RobertaTokenizer, Tokenizer};
//! use semchunk_rs::Chunker;
//! 
//! let tokenizer = RobertaTokenizer::from_file(
//!    "data/roberta-base-vocab.json",
//!    "data/roberta-base-merges.txt",
//!    false,
//!    false,
//! ).expect("Error loading tokenizer");
//! let token_counter = Box::new(move |s: &str| tokenizer.tokenize(s).len());
//! let chunker = Chunker::new(4, token_counter);
//! let text = "The quick brown fox jumps over the lazy dog.";
//! let chunks = chunker.chunk(text);
//! assert_eq!(chunks, vec!["The quick brown fox", "jumps over the", "lazy dog."]);
//! ```

pub mod chunker;
pub mod splitter;

pub use chunker::Chunker;
pub use splitter::Splitter;
