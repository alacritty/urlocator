use std::collections::HashMap;

use crate::{SchemeState, UrlLocation, UrlLocator};

#[test]
fn advance_schemes() {
    let state = SchemeState::RESET;

    let state = state.advance('h');
    assert_eq!(state, SchemeState::H);
    let state = state.advance('x');
    assert_eq!(state, SchemeState::INVALID);
    let state = state.advance(' ');
    assert_eq!(state, SchemeState::RESET);

    let state = state.advance('h');
    assert_eq!(state, SchemeState::H);
    let state = state.advance('t');
    assert_eq!(state, SchemeState::HT);
    let state = state.advance('T');
    assert_eq!(state, SchemeState::HTT);
    let state = state.advance('p');
    assert_eq!(state, SchemeState::HTTP);
    let state = state.advance('S');
    assert_eq!(state, SchemeState::HTTPS);
    let state = state.advance(':');
    assert_eq!(state, SchemeState::COMPLETE);
}

#[test]
fn boundaries() {
    assert_eq!(max_len("before https://example.org after"), Some(19));

    assert_eq!(position("before https://example.org after"), (7, 6));
    assert_eq!(position("before https://example.org"), (7, 0));
    assert_eq!(position("https://example.org after"), (0, 6));
    assert_eq!(position("https://example.org/test'ing;"), (0, 1));
}

#[test]
fn exclude_end() {
    assert_eq!(max_len("https://example.org/test\u{00}ing"), Some(24));
    assert_eq!(max_len("https://example.org/test\u{1F}ing"), Some(24));
    assert_eq!(max_len("https://example.org/test\u{7F}ing"), Some(24));
    assert_eq!(max_len("https://example.org/test\u{9F}ing"), Some(24));
    assert_eq!(max_len("https://example.org/test\ting"), Some(24));
    assert_eq!(max_len("https://example.org/test ing"), Some(24));
    assert_eq!(max_len("https://example.org/test?ing"), Some(28));
    assert_eq!(max_len("https://example.org/.,;:(!?"), Some(20));
    assert_eq!(max_len("https://example.org/"), Some(20));
}

#[test]
fn exclude_start() {
    assert_eq!(max_len("complicated:https://example.org"), Some(19));
    assert_eq!(max_len("\u{2502}https://example.org"), Some(19));
    assert_eq!(max_len("test.https://example.org"), Some(19));
    assert_eq!(max_len("https://sub.example.org"), Some(23));
    assert_eq!(max_len(",https://example.org"), Some(19));
}

#[test]
fn url_unicode() {
    assert_eq!(max_len("https://xn--example-2b07f.org"), Some(29));
    assert_eq!(max_len("https://example.org/\u{2008A}"), Some(21));
    assert_eq!(max_len("https://example.org/\u{f17c}"), Some(21));
    assert_eq!(max_len("https://üñîçøðé.com/ä"), Some(21));
}

#[test]
fn url_schemes() {
    assert_eq!(max_len("invalidscheme://example.org"), None);
    assert_eq!(max_len("makefile://example.org"), None);
    assert_eq!(max_len("mailto://example.org"), Some(20));
    assert_eq!(max_len("gemini://example.org"), Some(20));
    assert_eq!(max_len("gopher://example.org"), Some(20));
    assert_eq!(max_len("https://example.org"), Some(19));
    assert_eq!(max_len("http://example.org"), Some(18));
    assert_eq!(max_len("news://example.org"), Some(18));
    assert_eq!(max_len("file://example.org"), Some(18));
    assert_eq!(max_len("git://example.org"), Some(17));
    assert_eq!(max_len("ssh://example.org"), Some(17));
    assert_eq!(max_len("ftp://example.org"), Some(17));
}

#[test]
fn url_matching_chars() {
    assert_eq!(max_len("(https://example.org/test(ing)/?)"), Some(30));
    assert_eq!(max_len("(https://example.org/test(ing))"), Some(29));
    assert_eq!(max_len("https://example.org/test(ing)"), Some(29));
    assert_eq!(max_len("((https://example.org))"), Some(19));
    assert_eq!(max_len(")https://example.org("), Some(19));
    assert_eq!(max_len("https://example.org)"), Some(19));
    assert_eq!(max_len("https://example.org("), Some(19));

    assert_eq!(max_len("https://[2001:db8:a0b:12f0::1]:80"), Some(33));
    assert_eq!(max_len("([(https://example.org/test(ing))])"), Some(29));
    assert_eq!(max_len("https://example.org/]()"), Some(20));
    assert_eq!(max_len("[https://example.org]"), Some(19));

    assert_eq!(max_len("https://example.org/tester's_dream"), Some(34));
    assert_eq!(max_len("'https://example.org/test'ing'/'"), Some(30));
    assert_eq!(max_len("https://example.org/test'ing'/"), Some(30));
    assert_eq!(max_len("'https://example.org'"), Some(19));

    assert_eq!(max_len("\"https://example.org\""), Some(19));
    assert_eq!(max_len("\"https://example.org"), Some(19));

    assert_eq!(max_len("⟨https://example.org⟩"), Some(19));
    assert_eq!(max_len("⟩https://example.org⟨"), Some(19));
}

#[test]
fn markdown() {
    let input = "[test](https://example.org)";
    let mut result_map = HashMap::new();
    result_map.insert(25, UrlLocation::Url(19, 0));
    result_map.insert(26, UrlLocation::Reset);
    exact_url_match(input, result_map);

    let input = "[https://example.org](test)";
    let mut result_map = HashMap::new();
    result_map.insert(19, UrlLocation::Url(19, 0));
    result_map.insert(20, UrlLocation::Reset);
    exact_url_match(input, result_map);

    let input = "[https://example.org](https://example.org/longer)";
    let mut result_map = HashMap::new();
    result_map.insert(19, UrlLocation::Url(19, 0));
    result_map.insert(20, UrlLocation::Reset);
    result_map.insert(47, UrlLocation::Url(26, 0));
    result_map.insert(48, UrlLocation::Reset);
    exact_url_match(input, result_map);
}

#[test]
fn file() {
    assert_eq!(max_len("file:///test.rs:13:9"), Some(20));
    assert_eq!(max_len("file:///test"), Some(12));
    assert_eq!(max_len("file://test"), Some(11));
    assert_eq!(max_len("file:/test"), Some(10));
    assert_eq!(max_len("file:test"), Some(9));
}

#[test]
fn multiple_urls() {
    let input = "https://example.org https://example.com/test";
    let mut result_map = HashMap::new();
    result_map.insert(18, UrlLocation::Url(19, 0));
    result_map.insert(19, UrlLocation::Reset);
    result_map.insert(43, UrlLocation::Url(24, 0));
    exact_url_match(input, result_map);
}

#[test]
fn parser_states() {
    let input = "   https://example.org test ;";
    let mut result_map = HashMap::new();
    result_map.insert(0, UrlLocation::Reset);
    result_map.insert(3, UrlLocation::Scheme);
    result_map.insert(8, UrlLocation::Scheme);
    result_map.insert(9, UrlLocation::Url(7, 0));
    result_map.insert(21, UrlLocation::Url(19, 0));
    result_map.insert(22, UrlLocation::Reset);
    result_map.insert(24, UrlLocation::Scheme);
    result_map.insert(27, UrlLocation::Reset);
    exact_url_match(input, result_map);
}

fn exact_url_match(input: &str, result_map: HashMap<usize, UrlLocation>) {
    let mut locator = UrlLocator::new();

    for (i, c) in input.chars().enumerate() {
        let result = locator.advance(c);

        if let Some(expected) = result_map.get(&i) {
            assert_eq!(&result, expected);
        }
    }
}

fn max_len(input: &str) -> Option<u16> {
    let mut locator = UrlLocator::new();
    let mut url_len = None;

    for c in input.chars() {
        if let UrlLocation::Url(len, _end_offset) = locator.advance(c) {
            url_len = Some(len);
        }
    }

    url_len
}

fn position(input: &str) -> (usize, usize) {
    let mut locator = UrlLocator::new();
    let mut url = None;

    for (i, c) in input.chars().enumerate() {
        if let UrlLocation::Url(len, end_offset) = locator.advance(c) {
            url = Some((i + 1 - end_offset as usize, len as usize));
        }
    }

    url.map(|(end, len)| (end - len, input.len() - end)).unwrap()
}

#[cfg(all(test, feature = "nightly"))]
mod bench {
    extern crate test;

    use crate::{UrlLocation, UrlLocator};

    #[bench]
    fn library(b: &mut test::Bencher) {
        let mut input = String::new();
        for i in 0..10_000 {
            if i % 1_000 == 0 {
                input.push_str("https://example.org");
            } else {
                input.push_str(" test ");
            }
        }

        b.iter(|| {
            let mut locator = UrlLocator::new();
            for c in input.chars() {
                if let UrlLocation::Url(len, end_offset) = locator.advance(c) {
                    test::black_box((len, end_offset));
                }
            }
        });
    }

    #[bench]
    fn lower_bound(b: &mut test::Bencher) {
        let mut input = String::new();
        for i in 0..10_000 {
            if i % 1_000 == 0 {
                input.push_str("https://example.org");
            } else {
                input.push_str(" test ");
            }
        }

        b.iter(|| {
            for c in input.chars().rev() {
                test::black_box(c);
            }
        });
    }
}
