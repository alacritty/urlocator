//! # URL Locator
//!
//! This library provides a streaming parser for locating URLs.
//!
//! Instead of returning the URL itself, this library will only return the length of the URL and
//! the offset from the current parsing position.
//!
//! The length and offset counts follow the example of Rust's standard library's [`char`] type and
//! are based on unicode scalar values instead of graphemes.
//!
//! # Usage
//!
//! This crate is available on [crates.io](https://crates.io/crates/urlocator) and can be used by
//! adding `urlocator` to your dependencies in your project's Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! urlocator = "0.1.4"
//! ```
//!
//! # Example: URL boundaries
//!
//! By keeping track of the current parser position, it is possible to locate the boundaries of a
//! URL in a character stream:
//!
//! ```rust
//! # use urlocator::{UrlLocator, UrlLocation};
//! // Boundaries:      10-v                 v-28
//! let input = "[example](https://example.org)";
//!
//! let mut locator = UrlLocator::new();
//!
//! let (mut start, mut end) = (0, 0);
//!
//! for (i, c) in input.chars().enumerate() {
//!     if let UrlLocation::Url(length, end_offset) = locator.advance(c) {
//!         start = 1 + i - length as usize;
//!         end = i - end_offset as usize;
//!     }
//! }
//!
//! assert_eq!(start, 10);
//! assert_eq!(end, 28);
//! ```
//!
//! # Examlpe: Counting URLs
//!
//! By checking for the return state of the parser, it is possible to determine exactly when a URL
//! has been broken. Using this, you can count the number of URLs in a stream:
//!
//! ```rust
//! # use urlocator::{UrlLocator, UrlLocation};
//! let input = "https://example.org/1 https://rust-lang.org/二 https://example.com/Ⅲ";
//!
//! let mut locator = UrlLocator::new();
//!
//! let mut url_count = 0;
//! let mut reset = true;
//!
//! for c in input.chars() {
//!     match locator.advance(c) {
//!         UrlLocation::Url(..) if reset => {
//!             url_count += 1;
//!             reset = false;
//!         },
//!         UrlLocation::Reset => reset = true,
//!         _ => (),
//!     }
//! }
//!
//! assert_eq!(url_count, 3);
//! ```

#![cfg_attr(all(test, feature = "nightly"), feature(test))]
#![cfg_attr(not(test), no_std)]

mod scheme;
#[cfg(test)]
mod tests;

use scheme::SchemeState;

/// Position of the URL parser.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UrlLocation {
    /// Current location is the end of a valid URL.
    Url(u16, u16),
    /// Current location is possibly a URL scheme.
    Scheme,
    /// Last advancement has reset the URL parser.
    Reset,
}

/// URL parser positional state.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum State {
    /// Parsing the URL scheme.
    Scheme(SchemeState),
    /// Parsing a valid URL.
    Url,
}

impl Default for State {
    #[inline]
    fn default() -> Self {
        State::Scheme(SchemeState::default())
    }
}

/// URL parser.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct UrlLocator {
    state: State,

    illegal_end_chars: u16,
    len: u16,

    open_parentheses: u8,
    open_brackets: u8,
}

impl UrlLocator {
    /// Create a new parser.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the parser by one char.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use urlocator::{UrlLocator, UrlLocation};
    /// let mut locator = UrlLocator::new();
    ///
    /// let location = locator.advance('h');
    ///
    /// assert_eq!(location, UrlLocation::Scheme);
    /// ```
    #[inline]
    pub fn advance(&mut self, c: char) -> UrlLocation {
        self.len += 1;

        match self.state {
            State::Scheme(state) => self.advance_scheme(state, c),
            State::Url => self.advance_url(c),
        }
    }

    #[inline]
    fn advance_scheme(&mut self, state: SchemeState, c: char) -> UrlLocation {
        self.state = match state.advance(c) {
            SchemeState::RESET => return self.reset(),
            SchemeState::COMPLETE => State::Url,
            state => State::Scheme(state),
        };

        UrlLocation::Scheme
    }

    #[inline]
    fn advance_url(&mut self, c: char) -> UrlLocation {
        if Self::is_illegal_at_end(c) {
            self.illegal_end_chars += 1;
        } else {
            self.illegal_end_chars = 0;
        }

        self.url(c)
    }

    #[inline]
    fn url(&mut self, c: char) -> UrlLocation {
        match c {
            '(' => self.open_parentheses += 1,
            '[' => self.open_brackets += 1,
            ')' => {
                if self.open_parentheses == 0 {
                    return self.reset();
                } else {
                    self.open_parentheses -= 1;
                }
            },
            ']' => {
                if self.open_brackets == 0 {
                    return self.reset();
                } else {
                    self.open_brackets -= 1;
                }
            },
            // Illegal URL characters
            '\u{00}'..='\u{1F}'
            | '\u{7F}'..='\u{9F}'
            | '<'
            | '>'
            | '"'
            | ' '
            | '{'..='}'
            | '\\'
            | '^'
            | '⟨'
            | '⟩'
            | '`' => return self.reset(),
            _ => (),
        }

        self.state = State::Url;

        UrlLocation::Url(self.len - self.illegal_end_chars, self.illegal_end_chars)
    }

    #[inline]
    fn is_illegal_at_end(c: char) -> bool {
        match c {
            '.' | ',' | ':' | ';' | '?' | '!' | '(' | '[' | '\'' => true,
            _ => false,
        }
    }

    #[inline]
    fn reset(&mut self) -> UrlLocation {
        *self = Self::default();
        UrlLocation::Reset
    }
}
