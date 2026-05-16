use std::num::NonZeroUsize;

use itertools::Itertools;
use unicase::UniCase;

use Token::{Newline, Symbol, Word};

/// Token represents a single lexical item.
///
/// Word tokens compare case-insensitively, symbols compare by exact value.
///
/// Examples:
/// ```
/// use panko::Token::*;
/// assert_eq!(Word("строкА"), Word("Строка"));
/// assert_eq!(Word("string"), Word("STRING"));
/// assert_ne!(Word("a"), Word("b"));
/// assert_eq!(Symbol("?"), Symbol("?"));
/// ```
#[derive(Copy, Clone, Debug)]
pub enum Token<'a> {
    Newline,
    Word(&'a str),
    Symbol(&'a str),
}

impl PartialEq for Token<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Word(a), &Word(b)) => UniCase::new(a) == UniCase::new(b),
            (&Symbol(a), &Symbol(b)) => a == b,
            (&Newline, &Newline) => true,
            _ => false,
        }
    }
}

/// Join tokens into a string.
///
/// Examples:
/// ```
/// use panko::{tokens_to_string, Token::*};
/// assert_eq!(tokens_to_string(&[Word("some"), Symbol("_"), Word("str")]), "some_str");
/// ```
#[must_use]
pub fn tokens_to_string<'a>(tokens: &'a [Token<'a>]) -> String {
    tokens
        .iter()
        .map(|token| match token {
            Newline => "\n",
            Word(x) | Symbol(x) => x,
        })
        .collect()
}

#[must_use]
pub fn normalize_text(text: String) -> String {
    text.to_lowercase().replace('ё', "е")
}

/// Split tokens by any of the provided separator tokens. Each separator starts
/// a new chunk; surrounding tokens are concatenated.
///
/// Examples:
/// ```
/// use panko::{split_tokens, Token::*};
/// let tokens = [Word("a"), Symbol(","), Word("b"), Word("or"), Word("c")];
/// let separators = [Symbol(","), Word("or")];
/// assert_eq!(split_tokens(&tokens, &separators), vec!["a", "b", "c"]);
/// ```
#[must_use]
pub fn split_tokens<'a>(tokens: &'a [Token<'a>], separators: &[Token<'a>]) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    tokens.iter().for_each(|x| match x {
        Newline => {
            if result.is_empty() {
                result.push(String::from("\n"));
            } else if let Some(last) = result.last_mut() {
                last.push('\n');
            }
        }
        t if separators.contains(t) => result.push(String::new()),
        Symbol(x) | Word(x) if result.is_empty() => {
            result.push(x.to_string());
        }
        Symbol(x) | Word(x) => {
            if let Some(last) = result.last_mut() {
                if !last.is_empty() || *x != " " {
                    last.push_str(x);
                }
            }
        }
    });
    result
}

#[must_use]
#[allow(unstable_name_collisions)]
pub fn tokenize(text: &str) -> Vec<Token<'_>> {
    use Token::Newline;
    if text.is_empty() {
        return vec![];
    }

    fn tokenize_line(line: &str) -> Vec<Token<'_>> {
        use Token::{Symbol, Word};
        let mut out = vec![];
        let mut word_start: Option<usize> = None;
        for (i, ch) in line.char_indices() {
            if ch.is_alphanumeric() {
                if word_start.is_none() {
                    word_start = Some(i);
                }
            } else {
                if let Some(start) = word_start.take() {
                    out.push(Word(&line[start..i]));
                }
                let end = i + ch.len_utf8();
                out.push(Symbol(&line[i..end]));
            }
        }
        if let Some(start) = word_start {
            out.push(Word(&line[start..]));
        }
        out
    }

    let mut tokens = text
        .lines()
        .map(tokenize_line)
        .intersperse(vec![Newline])
        .flatten()
        .collect_vec();

    if text.ends_with('\n') {
        tokens.push(Newline);
    }

    tokens
}

/// Split text into chunks of at most `chunk_size` UTF-8 characters.
///
/// Examples:
/// ```
/// use std::num::NonZeroUsize;
/// use panko::fit_message;
///
/// let size = NonZeroUsize::new(4096).unwrap_or(NonZeroUsize::MIN);
/// let s = "a".repeat(4100);
/// let chunks = fit_message(&s, size);
/// assert_eq!(chunks.len(), 2);
/// assert_eq!(chunks[0].chars().count(), 4096);
/// assert_eq!(chunks[1], "aaaa");
/// ```
#[must_use]
pub fn fit_message(text: &str, chunk_size: NonZeroUsize) -> Vec<&str> {
    if text.is_empty() {
        return vec![];
    }

    let chunk_size = chunk_size.get();
    let mut result = Vec::with_capacity(text.len().div_ceil(chunk_size));
    let mut start_byte_index: usize = 0;
    let mut current_char_count: usize = 0;

    for (byte_index, ch) in text.char_indices() {
        current_char_count += 1;
        if current_char_count == chunk_size {
            let end = byte_index + ch.len_utf8();
            result.push(&text[start_byte_index..end]);
            start_byte_index = end;
            current_char_count = 0;
        }
    }

    if start_byte_index < text.len() {
        result.push(&text[start_byte_index..]);
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::tokenize;
    use insta::assert_debug_snapshot;
    use rstest::rstest;

    #[rstest]
    #[case("empty", "")]
    #[case("single_alpha", "a")]
    #[case("single_space", " ")]
    #[case("underscore_split", "some_string")]
    #[case("space_split", "some str")]
    #[case("double_symbol_tail", "some_str»»")]
    #[case("emoji_both_sides", "😀some_str😀")]
    #[case("carriage_return", "some_str\rsome_another_str")]
    #[case("multiline_complex", "\nsome_str\n \n some_another_str")]
    #[case("cyrillic_with_qmark_newline", "хлеб кто булочка?\n")]
    fn tokenize_cases(#[case] name: &str, #[case] input: &str) {
        let tokens = tokenize(input);
        assert_debug_snapshot!(name, tokens);
    }
}
