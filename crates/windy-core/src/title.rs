use crate::types::{kind, Content};

pub const MAX_TITLE_CHARS: usize = 1000;

pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let hi = (b[i + 1] as char).to_digit(16);
            let lo = (b[i + 2] as char).to_digit(16);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                out.push((hi * 16 + lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

pub fn strip_rtf(s: &str) -> String {
    const SKIPPED: [&str; 5] = ["\\*", "\\fonttbl", "\\colortbl", "\\stylesheet", "\\info"];
    let b: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let mut depth = 0usize;
    let mut skip_at: Option<usize> = None;
    while i < b.len() {
        match b[i] {
            '{' => {
                depth += 1;
                if skip_at.is_none() {
                    let rest: String = b[i + 1..].iter().take(12).collect();
                    if SKIPPED.iter().any(|d| rest.starts_with(d)) {
                        skip_at = Some(depth);
                    }
                }
                i += 1;
            }
            '}' => {
                if skip_at == Some(depth) {
                    skip_at = None;
                }
                depth = depth.saturating_sub(1);
                i += 1;
            }
            '\\' => {
                i += 1;
                let Some(&n) = b.get(i) else { break };
                if n.is_ascii_alphabetic() {
                    let start = i;
                    while i < b.len() && b[i].is_ascii_alphabetic() {
                        i += 1;
                    }
                    let word: String = b[start..i].iter().collect();
                    let pstart = i;
                    if i < b.len() && (b[i] == '-' || b[i].is_ascii_digit()) {
                        i += 1;
                        while i < b.len() && b[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                    let param: String = b[pstart..i].iter().collect();
                    if i < b.len() && b[i] == ' ' {
                        i += 1;
                    }
                    if skip_at.is_none() {
                        match word.as_str() {
                            "par" | "line" => out.push('\n'),
                            "tab" => out.push('\t'),
                            "u" => {
                                if let Ok(n) = param.parse::<i32>() {
                                    if let Some(ch) = char::from_u32(u32::from(n as i16 as u16)) {
                                        out.push(ch);
                                    }
                                    // skip the single fallback character
                                    if i < b.len() {
                                        i += 1;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                } else if n == '\'' {
                    let hex: String = b.iter().skip(i + 1).take(2).collect();
                    if let (Ok(v), true) = (u8::from_str_radix(&hex, 16), skip_at.is_none()) {
                        out.push(v as char);
                    }
                    i += 3;
                } else {
                    if skip_at.is_none() && matches!(n, '\\' | '{' | '}') {
                        out.push(n);
                    }
                    i += 1;
                }
            }
            '\r' | '\n' => i += 1,
            c => {
                if skip_at.is_none() {
                    out.push(c);
                }
                i += 1;
            }
        }
    }
    out
}

/// Characters that break text layout engines: controls (except tab/newline), bidi overrides, zero-width marks.
fn is_unsafe(c: char) -> bool {
    (c.is_control() && c != '\n' && c != '\t')
        || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2064}' | '\u{FEFF}')
}

fn special_symbols(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let lead = chars.iter().take_while(|c| **c == ' ').count();
    let trail = chars[lead..]
        .iter()
        .rev()
        .take_while(|c| **c == ' ')
        .count();
    let end = chars.len() - trail;
    chars
        .iter()
        .enumerate()
        .map(|(i, c)| match c {
            ' ' if i < lead || i >= end => '·',
            '\n' => '⏎',
            '\t' => '⇥',
            c => *c,
        })
        .collect()
}

fn text_of(contents: &[Content], k: &str) -> Option<String> {
    contents
        .iter()
        .find(|c| c.kind == k)
        .and_then(|c| c.as_str())
        .map(str::to_owned)
}

/// Preview text priority: files, plain text, RTF, HTML, then the previous title.
/// Image-only items get an empty title.
pub fn generate_title(contents: &[Content], previous: &str, show_special_symbols: bool) -> String {
    let files = text_of(contents, kind::FILES)
        .map(|s| s.lines().map(percent_decode).collect::<Vec<_>>().join("\n"));
    let raw = files
        .or_else(|| text_of(contents, kind::TEXT))
        .or_else(|| text_of(contents, kind::RTF).map(|s| strip_rtf(&s)))
        .or_else(|| text_of(contents, kind::HTML).map(|s| strip_html(&s)))
        .unwrap_or_else(|| {
            if contents
                .iter()
                .any(|c| kind::IMAGES.contains(&c.kind.as_str()))
            {
                String::new()
            } else {
                previous.to_owned()
            }
        });
    let cleaned: String = raw
        .chars()
        .take(MAX_TITLE_CHARS)
        .filter(|c| !is_unsafe(*c))
        .collect();
    if show_special_symbols {
        special_symbols(&cleaned)
    } else {
        cleaned.trim().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn title(c: Vec<Content>) -> String {
        generate_title(&c, "", true)
    }

    #[test]
    fn title_for_string() {
        assert_eq!(title(vec![Content::text("foo")]), "foo");
    }

    #[test]
    fn title_for_rtf() {
        let rtf = r"{\rtf1\ansi{\fonttbl\f0 Arial;}\f0 Hello world\par}";
        assert_eq!(title(vec![Content::new(kind::RTF, rtf)]), "Hello world⏎");
    }

    #[test]
    fn title_for_rtf_unicode_and_hex() {
        let rtf = r"{\rtf1\ansi caf\'e9 \u8364?}";
        assert_eq!(title(vec![Content::new(kind::RTF, rtf)]), "café €");
    }

    #[test]
    fn title_for_html() {
        let html = "<p>Hello <b>world</b> &amp; co</p>";
        assert_eq!(
            title(vec![Content::new(kind::HTML, html)]),
            "Hello world & co"
        );
    }

    #[test]
    fn title_for_file_decodes_percent_and_joins_lines() {
        let f = "file:///C:/My%20Docs/%C3%A9.txt\nfile:///C:/b.txt";
        assert_eq!(
            title(vec![Content::new(kind::FILES, f)]),
            "file:///C:/My Docs/é.txt⏎file:///C:/b.txt"
        );
    }

    #[test]
    fn title_for_image_is_empty() {
        assert_eq!(title(vec![Content::new(kind::PNG, vec![1, 2, 3])]), "");
    }

    #[test]
    fn title_for_empty_item_keeps_previous() {
        assert_eq!(generate_title(&[], "old", true), "old");
    }

    #[test]
    fn plain_text_wins_over_rtf_and_html() {
        let t = title(vec![
            Content::new(kind::HTML, "<b>html</b>"),
            Content::new(kind::RTF, r"{\rtf1 rtf}"),
            Content::text("plain"),
        ]);
        assert_eq!(t, "plain");
    }

    #[test]
    fn special_symbols_on() {
        assert_eq!(title(vec![Content::text("  a\tb\nc  ")]), "··a⇥b⏎c··");
    }

    #[test]
    fn special_symbols_off_trims() {
        let c = vec![Content::text("  a\tb  ")];
        assert_eq!(generate_title(&c, "", false), "a\tb");
    }

    #[test]
    fn whitespace_only_text() {
        assert_eq!(title(vec![Content::text("   ")]), "···");
    }

    #[test]
    fn title_is_shortened() {
        let long = "x".repeat(5000);
        assert_eq!(
            title(vec![Content::text(&long)]).chars().count(),
            MAX_TITLE_CHARS
        );
    }

    #[test]
    fn unsafe_scalars_are_removed() {
        let s = "a\u{0}b\u{200B}c\u{202E}d\u{FEFF}e";
        assert_eq!(title(vec![Content::text(s)]), "abcde");
    }

    #[test]
    fn percent_decode_is_lenient() {
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz%41x"), "%zzAx");
        assert_eq!(percent_decode("é%4"), "é%4");
    }
}
