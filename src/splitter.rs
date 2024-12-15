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

use regex::Regex;

const NON_WHITESPACE_SEMANTIC_SEPARATORS: [&str; 25] = [
    ".", "?", "!", "*", // Sentence terminators
    ";", ",", "(", ")", "[", "]", "“", "”", "‘", "’", "'", "\"", "`", // Clause separators.
    ":", "—", "…", // Sentence interrupters.
    "/", "\\", "–", "&", "-", // Word joiners.
];

/// A struct for splitting texts into segments based on the most desirable separator found.
/// 
/// # Examples
/// 
/// ```
/// use semchunk_rs::Splitter;
/// let splitter = Splitter::default();
/// let text = "Hello World\nGoodbye World";
/// let (separator, is_whitespace, segments) = splitter.split_text(text);
/// assert_eq!(separator, "\n");
/// assert!(is_whitespace);
/// assert_eq!(segments, vec!["Hello World", "Goodbye World"]);
/// ```
#[derive(Debug)]
pub struct Splitter {
    line_carriage: Regex,
    tab: Regex,
    space: Regex,
}

impl Default for Splitter {
    fn default() -> Self {
        Splitter {
            line_carriage: Regex::new(r"[\n\r]+").unwrap(),
            tab: Regex::new(r"\t").unwrap(),
            space: Regex::new(r"\s").unwrap(),
        }
    }
}

impl Splitter {
    /// Splits the given text into segments based on the most desirable separator found.
    ///
    /// The method prioritizes separators in the following order:
    /// 1. The largest sequence of newlines and/or carriage returns.
    /// 2. The largest sequence of tabs.
    /// 3. The largest sequence of whitespace characters.
    /// 4. A semantically meaningful non-whitespace separator.
    ///
    /// If no semantically meaningful separator is found, the text is split into individual characters.
    ///
    /// # Arguments
    ///
    /// * `text` - A string slice that holds the text to be split.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * The separator used for splitting the text.
    /// * A boolean indicating whether the separator is whitespace.
    /// * A vector of string slices representing the segments of the split text.
    ///
    /// # Examples
    ///
    /// ```
    /// use semchunk_rs::Splitter;
    /// let splitter = Splitter::default();
    /// let text = "Hello World\nGoodbye World";
    /// let (separator, is_whitespace, segments) = splitter.split_text(text);
    /// assert_eq!(separator, "\n");
    /// assert!(is_whitespace);
    /// assert_eq!(segments, vec!["Hello World", "Goodbye World"]);
    /// ```
    pub fn split_text<'a>(&self, text: &'a str) -> (&'a str, bool, Vec<&'a str>) {
        let mut separator_is_whitespace = true;
        let mut separator_search_pattern: Option<&Regex> = Option::None;
        let separator: &str;

        // Try splitting at, in order of most desirable to least desirable:
        // - The largest sequence of newlines and/or carriage returns;
        // - The largest sequence of tabs;
        // - The largest sequence of whitespace characters; and
        // - A semantically meaningful non-whitespace separator.
        if text.contains("\n") || text.contains("\r") {
            separator_search_pattern = Option::Some(&self.line_carriage);
            // Find longest line break
        } else if text.contains("\t") {
            separator_search_pattern = Option::Some(&self.tab);
        } else if self.space.is_match(text) {
            separator_search_pattern = Option::Some(&self.space);
        }

        match separator_search_pattern {
            Some(pattern) => {
                separator = pattern
                    .find_iter(text)
                    .map(|m| text.get(m.start()..m.end()).unwrap())
                    .max_by_key(|&s| s.len())
                    .unwrap();
            }
            None => {
                // Identify the most desirable semantically meaningful non-whitespace separator present in the text.
                match NON_WHITESPACE_SEMANTIC_SEPARATORS
                    .iter()
                    .find(|&&c| text.contains(c))
                    .copied()
                {
                    Some(c) => {
                        separator = c;
                        separator_is_whitespace = false;
                    }
                    None => {
                        // If no semantically meaningful separator is present in the text, return an empty string as the separator and the text as a list of characters.
                        // text.split("") does this obnoxious thing where it includes an empty string at the start and end of the list, so removing that.
                        return (
                            "",
                            true,
                            text.split("")
                                .collect::<Vec<&str>>()
                                .get(1..text.len() + 1)
                                .unwrap()
                                .to_vec(),
                        );
                    }
                }
            }
        }
        // Return the separator and the split text
        (
            separator,
            separator_is_whitespace,
            text.split(separator).collect::<Vec<&str>>().clone(),
        )
    }
}

#[cfg(test)]
mod splitter_tests {
    use super::*;

    #[test]
    fn test_whitespace_split() {
        let splitter = Splitter::default();
        let text = "Hello, World!";
        let (separator, separator_is_whitespace, split_text) = splitter.split_text(text);
        assert_eq!(separator, " ");
        assert!(separator_is_whitespace);
        assert_eq!(split_text, ["Hello,", "World!"]);

        let text = "Hello, World!\tGoodbye, World!";
        let (separator, separator_is_whitespace, split_text) = splitter.split_text(text);
        assert_eq!(separator, "\t");
        assert!(separator_is_whitespace);
        assert_eq!(split_text, ["Hello, World!", "Goodbye, World!"]);

        let text = "Hello, World!\nGoodbye, World!";
        let (separator, separator_is_whitespace, split_text) = splitter.split_text(text);
        assert_eq!(separator, "\n");
        assert!(separator_is_whitespace);
        assert_eq!(split_text, ["Hello, World!", "Goodbye, World!"]);

        // Prioritize \n\n over \n
        let text = "Hello, World!\n\nGoodbye, World!\n<EOF>";
        let (separator, separator_is_whitespace, split_text) = splitter.split_text(text);
        assert_eq!(separator, "\n\n");
        assert!(separator_is_whitespace);
        assert_eq!(split_text, ["Hello, World!", "Goodbye, World!\n<EOF>"]);
    }

    #[test]
    fn test_simple_semantic_chars_split() {
        // Prioritize ! over ,
        let splitter = Splitter::default();
        let text = "Hello,World!";
        let (separator, separator_is_whitespace, split_text) = splitter.split_text(text);
        assert_eq!(separator, "!");
        assert!(!separator_is_whitespace);
        assert_eq!(split_text, ["Hello,World", ""]);

        // Test with multiple separators
        let text = "Hello,_World!_Goodbye,_World!";
        let (separator, separator_is_whitespace, split_text) = splitter.split_text(text);
        assert_eq!(separator, "!");
        assert!(!separator_is_whitespace);
        assert_eq!(split_text, ["Hello,_World", "_Goodbye,_World", ""]);
    }

    #[test]
    fn test_no_match_split() {
        let splitter = Splitter::default();
        let text = "Hello_World";
        let (separator, separator_is_whitespace, split_text) = splitter.split_text(text);
        assert_eq!(separator, "");
        assert!(separator_is_whitespace);
        assert_eq!(
            split_text,
            ["H", "e", "l", "l", "o", "_", "W", "o", "r", "l", "d"]
        );
    }
}
