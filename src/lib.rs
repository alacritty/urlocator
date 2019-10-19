#![cfg_attr(all(test, feature = "bench"), feature(test))]

use std::num::NonZeroU16;

mod scheme;
#[cfg(test)]
mod tests;

use scheme::SchemeState;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UrlLocation {
    /// Current location is the end of a valid URL.
    Url(u16, u16),
    /// Current location is possibly a URL scheme.
    Scheme,
    /// Last advancement has reset the URL parser.
    Reset,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum State {
    Scheme(SchemeState),
    Separators(u8),
    Url,
}

impl Default for State {
    #[inline]
    fn default() -> Self {
        Self::Scheme(SchemeState::default())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct UrlLocator {
    open_parentheses: u8,
    open_brackets: u8,

    len_without_quote: Option<NonZeroU16>,
    illegal_end_chars: u16,
    len: u16,

    state: State,
}

impl UrlLocator {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn advance(&mut self, c: char) -> UrlLocation {
        self.len += 1;

        match self.state {
            State::Scheme(state) => self.advance_scheme(state, c),
            State::Separators(count) => self.advance_separators(count, c),
            State::Url => self.advance_url(c),
        }
    }

    #[inline]
    fn advance_scheme(&mut self, state: SchemeState, c: char) -> UrlLocation {
        self.state = match state.advance(c) {
            SchemeState::NONE => return self.reset(),
            SchemeState::COMPLETE => State::Separators(0),
            state => State::Scheme(state),
        };

        UrlLocation::Scheme
    }

    #[inline]
    fn advance_separators(&mut self, count: u8, c: char) -> UrlLocation {
        match (c, count) {
            ('/', 0) => {
                self.state = State::Separators(1);
                UrlLocation::Scheme
            },
            ('/', 1) => {
                self.state = State::Separators(2);
                UrlLocation::Scheme
            },
            // Reset if there are more or less than two separators
            ('/', 2) | (_, 1) => self.reset(),
            _ => self.url(c),
        }
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
            '\'' => {
                self.len_without_quote = match self.len_without_quote {
                    Some(_) => None,
                    None => NonZeroU16::new(self.len - self.illegal_end_chars - 1),
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
        let len = self
            .len_without_quote
            .map(NonZeroU16::get)
            .unwrap_or(self.len - self.illegal_end_chars);
        UrlLocation::Url(len, self.illegal_end_chars)
    }

    #[inline]
    fn is_illegal_at_end(c: char) -> bool {
        match c {
            '.' | ',' | ':' | ';' | '?' | '!' | '(' | '[' => true,
            _ => false,
        }
    }

    #[inline]
    fn reset(&mut self) -> UrlLocation {
        *self = Self::default();
        UrlLocation::Reset
    }
}
