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

use bisection::bisect_left;

use crate::splitter::Splitter;


/// A struct for chunking texts into segments based on a maximum number of tokens per chunk and a token counter function.
/// 
/// # Fields
/// 
/// * `chunk_size` - The maximum number of tokens that can be in a chunk.
/// * `token_counter` - A function that counts the number of tokens in a string.
/// * `splitter` - The Splitter instance used to split the text.
/// 
/// # Example
/// 
/// ```
/// use semchunk_rs::Chunker;
/// let chunker = Chunker::new(4, Box::new(|s: &str| s.len() - s.replace(" ", "").len() + 1));
/// let text = "The quick brown fox jumps over the lazy dog.";
/// let chunks = chunker.chunk(text);
/// assert_eq!(chunks, vec!["The quick brown fox", "jumps over the lazy", "dog."]);
/// ```
///
/// With `rust_tokenizers`:
///
/// ```
/// use rust_tokenizers::tokenizer::{RobertaTokenizer, Tokenizer};
/// use semchunk_rs::Chunker;
/// let tokenizer = RobertaTokenizer::from_file("data/roberta-base-vocab.json", "data/roberta-base-merges.txt", false, false)
///    .expect("Error loading tokenizer");
/// let token_counter = Box::new(move |s: &str| {
///    tokenizer.tokenize(s).len()
/// });
/// let chunker = Chunker::new(10, token_counter);
/// ```
pub struct Chunker {
    chunk_size: usize,
    token_counter: Box<dyn Fn(&str) -> usize>,
    splitter: Splitter,
}

impl Chunker {
    /// Creates a new Chunker instance. Uses the default Splitter instance. S
    ///
    /// # Arguments
    ///
    /// * `chunk_size` - The maximum number of tokens that can be in a chunk.
    /// * `token_counter` - A function that counts the number of tokens in a string.
    ///
    /// # Returns
    ///
    /// A new Chunker instance.
    pub fn new(chunk_size: usize, token_counter: Box<dyn Fn(&str) -> usize>) -> Self {
        Chunker {
            chunk_size,
            token_counter,
            splitter: Splitter::default(),
        }
    }

    /// Sets the splitter for the Chunker instance.
    pub fn splitter(mut self, splitter: Splitter) -> Self {
        self.splitter = splitter;
        self
    }

    /// Recursively chunks the given text into segments based on the maximum number of tokens per chunk.
    /// 
    /// # Arguments
    /// 
    /// * `text` - A string slice that holds the text to be chunked.
    /// * `recursion_depth` - The current recursion depth.
    /// 
    /// # Returns
    /// 
    /// A vector of string slices representing the chunks of the split text.
    pub fn _chunk(&self, text: &str, recursion_depth: usize) -> Vec<String> {
        let (separator, separator_is_whitespace, text_splits) = self.splitter.split_text(text);

        let mut chunks: Vec<String> = Vec::new();

        // Iterate through the splits
        let mut i = 0;
        while i < text_splits.len() {
            if (self.token_counter)(text_splits[i]) > self.chunk_size {
                // If the split is over the chunk size, recursively chunk it.
                let sub_chunks = self._chunk(text_splits[i], recursion_depth + 1);
                for sub_chunk in sub_chunks {
                    chunks.push(sub_chunk);
                }
                i += 1;
            } else {
                // If the split is equal to or under the chunk size, add it and any subsequent splits to a new chunk until the chunk size is reached.
                let (split_idx, merged_chunk) = self.merge_splits(&text_splits[i..], separator);
                chunks.push(merged_chunk);
                i += split_idx;
            }

            let n_chunks = chunks.len();
            // If the separator is not whitespace and the split is not the last split, add the separator to the end of the last chunk if doing so would not cause it to exceed the chunk size otherwise add the splitter as a new chunk.
            if !separator_is_whitespace && i < text_splits.len() {
                let last_chunk_with_separator = chunks[n_chunks - 1].clone() + separator;
                if (self.token_counter)(&last_chunk_with_separator) <= self.chunk_size {
                    chunks[n_chunks - 1] = last_chunk_with_separator;
                } else {
                    chunks.push(separator.to_string());
                }
            }
        }
        if recursion_depth > 0 {
            chunks = chunks
                .iter()
                .filter(|&c| !c.is_empty())
                .map(|c| c.to_string())
                .collect();
        }
        chunks
    }

    /// Merges first N splits into a chunk that has <= chunk_size tokens.
    ///
    /// # Arguments
    ///
    /// * `splits` - A vector of string slices representing the splits to merge.
    /// * `separator` - The separator used to split the text.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * The index merging stopped at (not inclusive).
    /// * The merged text.
    ///
    /// # Examples
    ///
    /// ```
    /// use semchunk_rs::Chunker;
    /// let chunker = Chunker::new(4, Box::new(|s: &str| s.len() - s.replace(" ", "").len() + 1));
    /// let splits = vec!["The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"];
    /// let separator = " ";
    /// let (split_idx, merged) = chunker.merge_splits(&splits, separator);
    /// assert_eq!(split_idx, 4);
    /// assert_eq!(merged, "The quick brown fox");
    /// ```
    pub fn merge_splits(&self, splits: &[&str], separator: &str) -> (usize, String) {
        let mut low = 0;
        let mut high = splits.len();

        let mut n_tokens: usize;
        let mut tokens_per_split = 5.0;
        let cumulative_split_char_counts = splits
            .iter()
            .scan(0, |acc, &s| {
                *acc += s.len() as u64;
                Some(*acc)
            })
            .collect::<Vec<u64>>();

        while low < high {
            // estimate number of splits to increment by using the number of tokens per split
            let increment_by = bisect_left(
                &cumulative_split_char_counts[low..high],
                &((self.chunk_size as f64 * tokens_per_split) as u64),
            );
            let est_midpoint = std::cmp::min(low + increment_by, high - 1);
            n_tokens =
                (self.token_counter)(splits.get(..est_midpoint).unwrap().join(separator).as_ref());

            match n_tokens.cmp(&self.chunk_size) {
                std::cmp::Ordering::Greater => high = est_midpoint,
                std::cmp::Ordering::Equal => {
                    low = est_midpoint;
                    break;
                }
                std::cmp::Ordering::Less => low = est_midpoint + 1,
            }

            if n_tokens > 0 && cumulative_split_char_counts[est_midpoint] > 0 {
                tokens_per_split =
                    n_tokens as f64 / cumulative_split_char_counts[est_midpoint] as f64;
            }
        }
        (low, splits.get(..low).unwrap().join(separator))
    }

    /// Chunks the given text into segments based on the maximum number of tokens per chunk.
    /// 
    /// # Arguments
    /// 
    /// * `text` - A string slice that holds the text to be chunked.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use semchunk_rs::Chunker;
    /// 
    /// let chunker = Chunker::new(4, Box::new(|s: &str| s.len() - s.replace(" ", "").len() + 1));
    /// let text = "The quick brown fox jumps over the lazy dog.";
    /// let chunks = chunker._chunk(text, 0);
    /// assert_eq!(chunks, vec!["The quick brown fox", "jumps over the lazy", "dog."]);
    /// ```
    pub fn chunk(&self, text: &str) -> Vec<String> {
        self._chunk(text, 0)
    }
}


#[cfg(test)]
mod chunker_tests {
    use super::*;
    use std::io::Read;
    use std::path::PathBuf;

    #[cfg(feature = "rust_tokenizers")]
    use rust_tokenizers::tokenizer::{RobertaTokenizer, Tokenizer};

    fn get_data_path() -> PathBuf {
        PathBuf::from(std::env::var("DATA_DIR").unwrap_or_else(|_| ".".to_string()))
    }

    fn get_roberta_vocab_path() -> PathBuf {
        get_data_path().join("roberta-base-vocab.json")
    }

    fn get_roberta_merges_path() -> PathBuf {
        get_data_path().join("roberta-base-merges.txt")
    }

    fn get_gutenberg_path() -> PathBuf {
        get_data_path().join("gutenberg")
    }

    fn get_gutenberg_corpus_path(corpus_filename: &str) -> PathBuf {
        get_gutenberg_path().join(corpus_filename)
    }

    fn read_gutenberg_corpus(corpus_filename: &str) -> String {
        let mut file = std::fs::File::open(get_gutenberg_corpus_path(corpus_filename))
            .expect("Error opening file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).expect("Error reading file");
        String::from_utf8_lossy(&buffer).to_string()
    }

    #[test]
    #[cfg(feature = "rust_tokenizers")]
    fn test_chunk_rust_tokenizers() {
        let tokenizer = RobertaTokenizer::from_file(
            get_roberta_vocab_path(),
            get_roberta_merges_path(),
            false,
            false,
        )
        .expect("Error loading tokenizer");

        let token_counter = Box::new(move |s: &str| tokenizer.tokenize(s).len());
        let chunker = Chunker::new(10, token_counter);
        let text = "The quick brown fox jumps over the lazy dog.\n\nThe subject is\n\t- \"The quick brown fox\"\n\t- \"jumps over\"\n\t- \"the lazy dog\"";
        println!("Text: {}", text);
        let chunks = chunker.chunk(text);
        assert_eq!(
            chunks,
            vec![
                "The quick brown fox jumps over the lazy dog.",
                "The subject is\n\t- \"The quick brown fox\"",
                "\t- \"jumps over\"\n\t- \"the lazy dog\"",
            ]
        )
    }

    #[test]
    #[cfg(feature = "rust_tokenizers")]
    fn test_chunk_rust_tokenizers_gutenberg_austen_emma() {
        let tokenizer = RobertaTokenizer::from_file(
            get_roberta_vocab_path(),
            get_roberta_merges_path(),
            false,
            false,
        )
        .expect("Error loading tokenizer");

        let token_counter = Box::new(move |s: &str| tokenizer.tokenize(s).len());
        let chunker = Chunker::new(10, token_counter);
        let text = read_gutenberg_corpus("austen-emma.txt");
        let chunks = chunker.chunk(&text);
        assert_eq!(chunks.len(), 606);
    }

    #[test]
    #[cfg(feature = "rust_tokenizers")]
    fn test_chunk_rust_tokenizers_gutenberg_milton_paradise() {
        let tokenizer = RobertaTokenizer::from_file(
            get_roberta_vocab_path(),
            get_roberta_merges_path(),
            false,
            false,
        )
        .expect("Error loading tokenizer");

        let token_counter = Box::new(move |s: &str| tokenizer.tokenize(s).len());
        let chunker = Chunker::new(10, token_counter);
        let text = read_gutenberg_corpus("milton-paradise.txt");
        let chunks = chunker.chunk(&text);
        assert_eq!(chunks.len(), 12196);
    }

    #[test]
    #[cfg(feature = "rust_tokenizers")]
    fn test_chunk_rust_tokenizers_gutenberg_shakespeare_hamlet() {
        let tokenizer = RobertaTokenizer::from_file(
            get_roberta_vocab_path(),
            get_roberta_merges_path(),
            false,
            false,
        )
        .expect("Error loading tokenizer");

        let token_counter = Box::new(move |s: &str| tokenizer.tokenize(s).len());
        let chunker = Chunker::new(10, token_counter);
        let text = read_gutenberg_corpus("shakespeare-hamlet.txt");
        let chunks = chunker.chunk(&text);
        assert_eq!(chunks.len(), 4474);
    }

    #[test]
    fn test_merge_splits_simple() {
        let chunker = Chunker::new(
            2,
            Box::new(|s: &str| s.len() - s.replace(" ", "").len() + 1),
        );
        let splits = vec!["Hello", "World", "Goodbye", "World"];
        let separator = " ";
        let (split_idx, merged) = chunker.merge_splits(&splits, separator);
        assert_eq!(split_idx, 2);
        assert_eq!(merged, "Hello World");

        let (split_idx, merged) = chunker.merge_splits(splits.get(split_idx..).unwrap(), separator);
        assert_eq!(split_idx, 2);
        assert_eq!(merged, "Goodbye World");
    }

    #[test]
    fn test_merge_splits_uneven() {
        let chunker = Chunker::new(
            4,
            Box::new(|s: &str| s.len() - s.replace(" ", "").len() + 1),
        );
        let splits = vec![
            "The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog",
        ]; // 9 tokens
        let separator = " ";
        let (split_idx, merged) = chunker.merge_splits(&splits, separator);
        assert_eq!(split_idx, 4);
        assert_eq!(merged, "The quick brown fox");

        let (split_idx_2, merged) =
            chunker.merge_splits(splits.get(split_idx..).unwrap(), separator);
        assert_eq!(split_idx_2, 4);
        assert_eq!(merged, "jumps over the lazy");

        let (split_idx_3, merged) =
            chunker.merge_splits(splits.get(split_idx + split_idx_2..).unwrap(), separator);
        assert_eq!(split_idx_3, 1);
        assert_eq!(merged, "dog");
    }
}
