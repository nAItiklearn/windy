//! The per-copy decision pipeline (MACCY-SPEC sections 1 and 2). Pure: the platform layer supplies
//! a `CopyEvent` and stores whatever this returns.

use regex::Regex;

use crate::settings::Settings;
use crate::title::{strip_html, strip_rtf};
use crate::types::{kind, Content};

/// One clipboard change as seen by the platform layer. A clipboard may hold several items
/// (some editors write two); they are merged into one record.
#[derive(Clone, Debug, Default)]
pub struct CopyEvent {
    pub items: Vec<Vec<Content>>,
    /// Frontmost application identifier at the time of the change.
    pub frontmost_app: Option<String>,
    /// The copy was written by Windy itself.
    pub from_windy: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewItem {
    pub application: Option<String>,
    pub contents: Vec<Content>,
    pub from_windy: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    IgnoreEvents,
    IgnoreNext,
    NoEnabledType,
    IgnoredType,
    IgnoredApp,
    NothingLeft,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Store(NewItem),
    Ignored(Reason),
}

/// Holds the two runtime ignore switches.
#[derive(Clone, Debug, Default)]
pub struct Monitor {
    pub ignore_events: bool,
    pub ignore_only_next: bool,
}

const BUILTIN_IGNORED: [&str; 3] = [kind::CONCEALED, kind::TRANSIENT, kind::AUTO_GENERATED];

fn is_enabled(s: &Settings, k: &str) -> bool {
    (s.save_files && k == kind::FILES)
        || (s.save_images && kind::IMAGES.contains(&k))
        || (s.save_text && kind::TEXTS.contains(&k))
}

fn is_unpersistable(k: &str) -> bool {
    k == kind::WEB_ARCHIVE || k.starts_with("dyn.") || k.starts_with("com.microsoft.ole.source.")
}

/// Invalid patterns never block a copy.
fn matches_any(patterns: &[String], text: &str) -> bool {
    patterns
        .iter()
        .any(|p| Regex::new(p).map(|r| r.is_match(text)).unwrap_or(false))
}

fn has_rich_text(item: &[Content]) -> bool {
    item.iter().any(|c| match (c.kind.as_str(), c.as_str()) {
        (kind::RTF, Some(s)) => !strip_rtf(s).trim().is_empty(),
        (kind::HTML, Some(s)) => !strip_html(s).trim().is_empty(),
        _ => false,
    })
}

impl Monitor {
    pub fn new(s: &Settings) -> Self {
        Self {
            ignore_events: s.ignore_events,
            ignore_only_next: false,
        }
    }

    /// Option+Shift-click: ignore on and armed for one copy only.
    pub fn arm_ignore_next(&mut self) {
        self.ignore_events = true;
        self.ignore_only_next = true;
    }

    pub fn process(&mut self, ev: &CopyEvent, s: &Settings) -> Outcome {
        if self.ignore_events {
            let reason = if self.ignore_only_next {
                Reason::IgnoreNext
            } else {
                Reason::IgnoreEvents
            };
            if self.ignore_only_next {
                self.ignore_events = false;
                self.ignore_only_next = false;
            }
            return Outcome::Ignored(reason);
        }

        let all_kinds = || ev.items.iter().flatten().map(|c| c.kind.as_str());
        if !all_kinds().any(|k| is_enabled(s, k)) {
            return Outcome::Ignored(Reason::NoEnabledType);
        }
        if all_kinds().any(|k| {
            BUILTIN_IGNORED.contains(&k) || s.ignored_pasteboard_types.iter().any(|t| t == k)
        }) {
            return Outcome::Ignored(Reason::IgnoredType);
        }

        let listed = ev
            .frontmost_app
            .as_ref()
            .is_some_and(|a| s.ignored_apps.contains(a));
        let app_ignored = if s.ignore_all_apps_except_listed {
            !listed
        } else {
            listed
        };
        if app_ignored {
            return Outcome::Ignored(Reason::IgnoredApp);
        }

        let mut merged: Vec<Content> = Vec::new();
        for item in &ev.items {
            let text = item
                .iter()
                .find(|c| c.kind == kind::TEXT)
                .and_then(Content::as_str);
            if let Some(t) = text {
                if t.trim().is_empty() && !has_rich_text(item) {
                    continue;
                }
                if matches_any(&s.ignore_regexp, t) {
                    continue;
                }
            }
            for c in item {
                if is_enabled(s, &c.kind) && !is_unpersistable(&c.kind) && !merged.contains(c) {
                    merged.push(c.clone());
                }
            }
        }
        if merged.is_empty() {
            return Outcome::Ignored(Reason::NothingLeft);
        }
        Outcome::Store(NewItem {
            application: ev.frontmost_app.clone(),
            contents: merged,
            from_windy: ev.from_windy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(items: Vec<Vec<Content>>) -> CopyEvent {
        CopyEvent {
            items,
            frontmost_app: Some("app.one".into()),
            from_windy: false,
        }
    }

    fn text_ev(t: &str) -> CopyEvent {
        ev(vec![vec![Content::text(t)]])
    }

    fn run(e: &CopyEvent, s: &Settings) -> Outcome {
        Monitor::new(s).process(e, s)
    }

    fn stored(o: Outcome) -> NewItem {
        match o {
            Outcome::Store(n) => n,
            other => panic!("expected Store, got {other:?}"),
        }
    }

    #[test]
    fn stores_plain_text_with_source_app() {
        let n = stored(run(&text_ev("hello"), &Settings::default()));
        assert_eq!(n.contents, vec![Content::text("hello")]);
        assert_eq!(n.application.as_deref(), Some("app.one"));
    }

    #[test]
    fn ignores_whitespace_only_text() {
        assert_eq!(
            run(&text_ev("   \t "), &Settings::default()),
            Outcome::Ignored(Reason::NothingLeft)
        );
    }

    #[test]
    fn ignores_newline_only_text() {
        assert_eq!(
            run(&text_ev("\n\n"), &Settings::default()),
            Outcome::Ignored(Reason::NothingLeft)
        );
    }

    #[test]
    fn rtf_and_html_are_not_ignored_when_text_is_blank() {
        let rtf = ev(vec![vec![
            Content::text(" "),
            Content::new(kind::RTF, r"{\rtf1 hi}"),
        ]]);
        assert!(matches!(run(&rtf, &Settings::default()), Outcome::Store(_)));
        let html = ev(vec![vec![
            Content::text(" "),
            Content::new(kind::HTML, "<b>hi</b>"),
        ]]);
        assert!(matches!(
            run(&html, &Settings::default()),
            Outcome::Store(_)
        ));
    }

    #[test]
    fn ignore_events_blocks_everything() {
        let s = Settings {
            ignore_events: true,
            ..Settings::default()
        };
        let mut m = Monitor::new(&s);
        assert_eq!(
            m.process(&text_ev("a"), &s),
            Outcome::Ignored(Reason::IgnoreEvents)
        );
        assert_eq!(
            m.process(&text_ev("b"), &s),
            Outcome::Ignored(Reason::IgnoreEvents)
        );
    }

    #[test]
    fn ignore_only_next_resets_after_one_copy() {
        let s = Settings::default();
        let mut m = Monitor::new(&s);
        m.arm_ignore_next();
        assert_eq!(
            m.process(&text_ev("a"), &s),
            Outcome::Ignored(Reason::IgnoreNext)
        );
        assert!(matches!(m.process(&text_ev("b"), &s), Outcome::Store(_)));
    }

    #[test]
    fn ignores_listed_application() {
        let s = Settings {
            ignored_apps: vec!["app.one".into()],
            ..Settings::default()
        };
        assert_eq!(run(&text_ev("a"), &s), Outcome::Ignored(Reason::IgnoredApp));
    }

    #[test]
    fn ignore_all_apps_except_listed() {
        let mut s = Settings {
            ignore_all_apps_except_listed: true,
            ..Settings::default()
        };
        assert_eq!(run(&text_ev("a"), &s), Outcome::Ignored(Reason::IgnoredApp));
        s.ignored_apps = vec!["app.one".into()];
        assert!(matches!(run(&text_ev("a"), &s), Outcome::Store(_)));
    }

    #[test]
    fn ignores_transient_concealed_and_auto_generated() {
        for k in BUILTIN_IGNORED {
            let e = ev(vec![vec![Content::text("secret"), Content::new(k, "1")]]);
            assert_eq!(
                run(&e, &Settings::default()),
                Outcome::Ignored(Reason::IgnoredType),
                "{k}"
            );
        }
    }

    #[test]
    fn ignores_custom_listed_types() {
        let e = ev(vec![vec![
            Content::text("pw"),
            Content::new("com.agilebits.onepassword", "x"),
        ]]);
        assert_eq!(
            run(&e, &Settings::default()),
            Outcome::Ignored(Reason::IgnoredType)
        );
    }

    #[test]
    fn ignores_unknown_only_types() {
        let e = ev(vec![vec![Content::new("application/x-unknown", "x")]]);
        assert_eq!(
            run(&e, &Settings::default()),
            Outcome::Ignored(Reason::NoEnabledType)
        );
    }

    #[test]
    fn ignore_regexp_skips_matching_text() {
        let s = Settings {
            ignore_regexp: vec![r"^\d{16}$".into()],
            ..Settings::default()
        };
        assert!(matches!(
            run(&text_ev("4111111111111111"), &s),
            Outcome::Ignored(_)
        ));
        assert!(matches!(run(&text_ev("not a card"), &s), Outcome::Store(_)));
    }

    #[test]
    fn invalid_ignore_regexp_never_blocks() {
        let s = Settings {
            ignore_regexp: vec!["(".into()],
            ..Settings::default()
        };
        assert!(matches!(run(&text_ev("x"), &s), Outcome::Store(_)));
    }

    #[test]
    fn disabled_types_are_removed() {
        let s = Settings {
            save_images: false,
            ..Settings::default()
        };
        let e = ev(vec![vec![
            Content::text("a"),
            Content::new(kind::PNG, vec![1, 2]),
        ]]);
        assert_eq!(stored(run(&e, &s)).contents, vec![Content::text("a")]);
        let only_img = ev(vec![vec![Content::new(kind::PNG, vec![1])]]);
        assert_eq!(run(&only_img, &s), Outcome::Ignored(Reason::NoEnabledType));
    }

    #[test]
    fn dynamic_types_are_removed() {
        let e = ev(vec![vec![
            Content::text("a"),
            Content::new("dyn.ah62d4rv4gu8y", "x"),
        ]]);
        assert_eq!(
            stored(run(&e, &Settings::default())).contents,
            vec![Content::text("a")]
        );
    }

    #[test]
    fn safari_web_archive_is_not_stored() {
        let e = ev(vec![vec![
            Content::text("a"),
            Content::new(kind::WEB_ARCHIVE, vec![0; 64]),
        ]]);
        assert_eq!(
            stored(run(&e, &Settings::default())).contents,
            vec![Content::text("a")]
        );
    }

    #[test]
    fn word_link_types_and_pdf_are_dropped() {
        let e = ev(vec![vec![
            Content::text("a"),
            Content::new(kind::WORD_LINK_SOURCE, "x"),
            Content::new(kind::WORD_OBJECT_LINK, "x"),
            Content::new(kind::PDF, vec![1]),
        ]]);
        assert_eq!(
            stored(run(&e, &Settings::default())).contents,
            vec![Content::text("a")]
        );
    }

    #[test]
    fn ole_source_types_are_dropped() {
        let e = ev(vec![vec![
            Content::text("a"),
            Content::new("com.microsoft.ole.source.foo", "x"),
        ]]);
        assert_eq!(stored(run(&e, &Settings::default())).contents.len(), 1);
    }

    #[test]
    fn multiple_clipboard_items_are_merged() {
        let e = ev(vec![
            vec![Content::text("a")],
            vec![Content::new(kind::HTML, "<b>a</b>")],
            vec![Content::text("a")],
        ]);
        let n = stored(run(&e, &Settings::default()));
        assert_eq!(
            n.contents,
            vec![Content::text("a"), Content::new(kind::HTML, "<b>a</b>")]
        );
    }

    #[test]
    fn items_without_data_are_ignored() {
        let e = ev(vec![]);
        assert!(matches!(run(&e, &Settings::default()), Outcome::Ignored(_)));
    }

    #[test]
    fn frontmost_app_is_recorded_and_windy_flag_passed_on() {
        let mut e = text_ev("a");
        e.from_windy = true;
        e.frontmost_app = Some("x.y".into());
        let n = stored(run(&e, &Settings::default()));
        assert!(n.from_windy);
        assert_eq!(n.application.as_deref(), Some("x.y"));
    }

    #[test]
    fn files_are_stored_when_enabled_only() {
        let e = ev(vec![vec![Content::new(kind::FILES, "file:///a")]]);
        assert!(matches!(run(&e, &Settings::default()), Outcome::Store(_)));
        let s = Settings {
            save_files: false,
            ..Settings::default()
        };
        assert!(matches!(run(&e, &s), Outcome::Ignored(_)));
    }
}
