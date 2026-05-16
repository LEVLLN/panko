# panko

A small, zero-copy text tokenizer for Rust.

## Why "panko"?

*Panko* (パン粉) is the Japanese word for **breadcrumbs** — light, airy crumbs
made by breaking bread into small, uniform pieces. This crate does the same
thing to text: it crumbles a string into a flat stream of small, uniform
**tokens** — words, single-character symbols, and newlines — that you can sift,
sort, and reassemble.

Like real panko, the pieces are kept light: every token borrows directly from
the input slice, so tokenizing allocates only the output `Vec`.

## What it does

`panko` segments arbitrary text — including Unicode and mixed scripts
(Cyrillic, emoji, …) — into a `Vec<Token>` where every `Token` is one of:

- `Word(&str)` — a contiguous run of `is_alphanumeric()` characters. Words
  compare **case-insensitively** (via [`unicase`]), so `Word("Строка")` equals
  `Word("строкА")`.
- `Symbol(&str)` — any single non-alphanumeric character (punctuation, spaces,
  emoji, …). Symbols compare byte-exact.
- `Newline` — emitted between lines, and trailing if the input ends with `\n`.

On top of the tokenizer, `panko` ships a few small helpers:

| Function           | Purpose                                                                 |
| ------------------ | ----------------------------------------------------------------------- |
| `tokenize`         | `&str` → `Vec<Token<'_>>`                                               |
| `tokens_to_string` | Reassemble tokens back into a `String`                                  |
| `split_tokens`     | Split a token stream by any of N separator tokens (case-insensitive)    |
| `normalize_text`   | Lowercase + collapse Russian `ё` to `е`                                 |
| `fit_message`      | Slice a string into ≤ N-character, UTF-8-safe chunks (caller picks N)   |

[`unicase`]: https://crates.io/crates/unicase

## Quick start

### 1. Add the dependency

```toml
[dependencies]
panko = "0.1"
```

### 2. Tokenize some text

```rust
use panko::{tokenize, Token::*};

let tokens = tokenize("Hello, world!\n");

assert_eq!(
    tokens,
    vec![
        Word("Hello"),
        Symbol(","),
        Symbol(" "),
        Word("world"),
        Symbol("!"),
        Newline,
    ],
);
```

### 3. Case-insensitive word comparisons

```rust
use panko::Token::Word;

assert_eq!(Word("Rust"),   Word("RUST"));
assert_eq!(Word("Строка"), Word("строкА"));
```

### 4. Split a stream by multiple separators

`split_tokens` cuts a token stream every time it sees a token that matches one
of the separators (using `Token`'s case-insensitive equality), and concatenates
the rest into one `String` per chunk. A leading space after a separator is
dropped, so chunks read cleanly.

```rust
use panko::{split_tokens, Token::*};

let tokens = [Word("a"), Symbol(","), Word("b"), Word("or"), Word("c")];
let separators = [Symbol(","), Word("or")];

assert_eq!(split_tokens(&tokens, &separators), vec!["a", "b", "c"]);
```

### 5. Fit long messages into fixed-size chunks

`fit_message` slices a string into pieces of at most N characters each, where
N is supplied by the caller as a [`NonZeroUsize`] — so a zero chunk size is
impossible to construct at compile time. Useful for chat APIs with per-message
character limits (Telegram = 4096, Discord = 2000, etc.).

```rust
use std::num::NonZeroUsize;
use panko::fit_message;

let size = NonZeroUsize::new(4096).unwrap_or(NonZeroUsize::MIN);

let long = "x".repeat(10_000);
let chunks = fit_message(&long, size);

assert_eq!(chunks.len(), 3);
assert_eq!(chunks[0].chars().count(), 4096);
```

[`NonZeroUsize`]: https://doc.rust-lang.org/std/num/struct.NonZeroUsize.html

## Development

```sh
cargo build
cargo test                                  # unit + doctests + insta snapshots
cargo clippy --all-targets -- -D warnings   # strict lint set, matches Cargo.toml
cargo insta review                          # review snapshot diffs after changes
```

## License

[MIT](./LICENSE)
