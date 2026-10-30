use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use regex::RegexBuilder;

use crate::settings::SearchMode;

pub const MAX_FUZZY_INPUT_CHARS: usize = 5000;

/// Returns indices into `titles`, in display order. An empty query matches everything.
pub fn search(query: &str, mode: SearchMode, titles: &[&str]) -> Vec<usize> {
    if query.is_empty() {
        return (0..titles.len()).collect();
    }
    match mode {
        SearchMode::Exact => exact(query, titles),
        SearchMode::Regexp => regexp(query, titles),
        SearchMode::Fuzzy => fuzzy(query, titles),
        SearchMode::Mixed => {
            let r = exact(query, titles);
            if !r.is_empty() {
                return r;
            }
            let r = regexp(query, titles);
            if !r.is_empty() {
                return r;
            }
            fuzzy(query, titles)
        }
    }
}

fn exact(query: &str, titles: &[&str]) -> Vec<usize> {
    let q = query.to_lowercase();
    titles
        .iter()
        .enumerate()
        .filter(|(_, t)| t.to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}

/// An invalid pattern matches nothing.
fn regexp(query: &str, titles: &[&str]) -> Vec<usize> {
    let Ok(re) = RegexBuilder::new(query)
        .case_insensitive(true)
        .size_limit(1 << 20)
        .build()
    else {
        return Vec::new();
    };
    titles
        .iter()
        .enumerate()
        .filter(|(_, t)| re.is_match(t))
        .map(|(i, _)| i)
        .collect()
}

/// Best score first; equal scores keep list order.
fn fuzzy(query: &str, titles: &[&str]) -> Vec<usize> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let mut buf = Vec::new();
    let mut scored: Vec<(usize, u32)> = titles
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            let t: String = t.chars().take(MAX_FUZZY_INPUT_CHARS).collect();
            pattern
                .score(Utf32Str::new(&t, &mut buf), &mut matcher)
                .map(|s| (i, s))
        })
        .collect();
    scored.sort_by_key(|a| std::cmp::Reverse(a.1));
    scored.into_iter().map(|(i, _)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: [&str; 4] = ["foo bar", "Foobar baz", "something else", "f00"];

    #[test]
    fn simple_is_case_insensitive_substring() {
        assert_eq!(search("FOO", SearchMode::Exact, &T), vec![0, 1]);
        assert_eq!(search("zzz", SearchMode::Exact, &T), Vec::<usize>::new());
    }

    #[test]
    fn empty_query_matches_all() {
        assert_eq!(search("", SearchMode::Exact, &T), vec![0, 1, 2, 3]);
    }

    #[test]
    fn fuzzy_matches_subsequence() {
        let r = search("fb", SearchMode::Fuzzy, &T);
        assert!(r.contains(&0) && r.contains(&1));
        assert!(!r.contains(&2));
    }

    #[test]
    fn fuzzy_tolerates_gaps_exact_does_not() {
        assert!(search("sthg", SearchMode::Exact, &T).is_empty());
        assert!(search("sthg", SearchMode::Fuzzy, &T).contains(&2));
    }

    #[test]
    fn regexp_matches_unanchored() {
        assert_eq!(search("^f.*r", SearchMode::Regexp, &T), vec![0, 1]);
        assert_eq!(search("o{2}", SearchMode::Regexp, &T), vec![0, 1]);
    }

    #[test]
    fn invalid_regexp_matches_nothing() {
        assert!(search("(", SearchMode::Regexp, &T).is_empty());
    }

    #[test]
    fn mixed_prefers_exact_then_regexp_then_fuzzy() {
        assert_eq!(search("baz", SearchMode::Mixed, &T), vec![1]);
        assert_eq!(search("^some", SearchMode::Mixed, &T), vec![2]);
        assert!(search("sthg", SearchMode::Mixed, &T).contains(&2));
    }

    #[test]
    fn fuzzy_input_is_capped() {
        let long = format!("{}needle", "a".repeat(MAX_FUZZY_INPUT_CHARS + 10));
        assert!(search("needle", SearchMode::Fuzzy, &[long.as_str()]).is_empty());
    }
}
