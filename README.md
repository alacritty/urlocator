# URL Locator

This library provides a streaming parser for locating URLs.

Instead of returning the URL itself, this library will only return the length of the URL and
the offset from the current parsing position.

The length and offset counts follow the example of Rust's standard library's `char` type and are
based on unicode scalar values instead of graphemes.

### Usage

This crate is available on [crates.io](https://crates.io/crates/urlocator) and can be used by
adding `urlocator` to your dependencies in your project's Cargo.toml:

```toml
[dependencies]
urlocator = "0.1.3"
```

### Example: URL boundaries

By keeping track of the current parser position, it is possible to locate the boundaries of a
URL in a character stream:

```rust
use urlocator::{UrlLocator, UrlLocation};

// Boundaries:      10-v                 v-28
let input = "[example](https://example.org)";

let mut locator = UrlLocator::new();

let (mut start, mut end) = (0, 0);

for (i, c) in input.chars().enumerate() {
    if let UrlLocation::Url(length, end_offset) = locator.advance(c) {
        start = 1 + i - length as usize;
        end = i - end_offset as usize;
    }
}

assert_eq!(start, 10);
assert_eq!(end, 28);
```

### Examlpe: Counting URLs

By checking for the return state of the parser, it is possible to determine exactly when a URL
has been broken. Using this, you can count the number of URLs in a stream:

```rust
use urlocator::{UrlLocator, UrlLocation};

let input = "https://example.org/1 https://rust-lang.org/二 https://example.com/Ⅲ";

let mut locator = UrlLocator::new();

let mut url_count = 0;
let mut reset = true;

for c in input.chars() {
    match locator.advance(c) {
        UrlLocation::Url(_, _) if reset => {
            url_count += 1;
            reset = false;
        }
        UrlLocation::Reset => reset = true,
        _ => (),
    }
}

assert_eq!(url_count, 3);
```
