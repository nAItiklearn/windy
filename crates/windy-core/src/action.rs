use crate::types::{kind, Content};

/// `Primary` is Cmd on macOS and Ctrl on Windows/Linux.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modifier {
    None,
    Primary,
    Alt,
    AltShift,
    PrimaryShift,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Copy,
    CopyWithoutFormatting,
    Paste,
    PasteWithoutFormatting,
    /// Unknown combination: ignored.
    Unknown,
}

/// Truth table from `HistoryItemAction.init` (MACCY-SPEC section 5).
pub fn resolve(paste_by_default: bool, remove_formatting_by_default: bool, m: Modifier) -> Action {
    use Action::*;
    use Modifier as M;
    match (paste_by_default, remove_formatting_by_default, m) {
        (false, false, M::None | M::Primary) => Copy,
        (false, false, M::Alt) => Paste,
        (false, false, M::AltShift) => PasteWithoutFormatting,
        (false, true, M::None) => CopyWithoutFormatting,
        (false, true, M::Primary) => Copy,
        (false, true, M::Alt) => PasteWithoutFormatting,
        (false, true, M::AltShift) => Paste,
        (true, false, M::None | M::Primary) => Paste,
        (true, false, M::Alt) => Copy,
        (true, false, M::PrimaryShift) => PasteWithoutFormatting,
        (true, true, M::None | M::Primary) => PasteWithoutFormatting,
        (true, true, M::Alt) => Copy,
        (true, true, M::PrimaryShift) => Paste,
        _ => Unknown,
    }
}

/// Keeps only plain text (and file URLs). Items without plain text are returned unchanged.
pub fn strip_formatting(contents: &[Content]) -> Vec<Content> {
    if !contents.iter().any(|c| c.kind == kind::TEXT) {
        return contents.to_vec();
    }
    contents
        .iter()
        .filter(|c| c.kind == kind::TEXT || c.kind == kind::FILES)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Action::*;
    use Modifier as M;

    #[test]
    fn full_truth_table() {
        let mods = [M::None, M::Primary, M::Alt, M::AltShift, M::PrimaryShift];
        let table = [
            (
                (false, false),
                [Copy, Copy, Paste, PasteWithoutFormatting, Unknown],
            ),
            (
                (false, true),
                [
                    CopyWithoutFormatting,
                    Copy,
                    PasteWithoutFormatting,
                    Paste,
                    Unknown,
                ],
            ),
            (
                (true, false),
                [Paste, Paste, Copy, Unknown, PasteWithoutFormatting],
            ),
            (
                (true, true),
                [
                    PasteWithoutFormatting,
                    PasteWithoutFormatting,
                    Copy,
                    Unknown,
                    Paste,
                ],
            ),
        ];
        for ((p, r), row) in table {
            for (m, want) in mods.iter().zip(row) {
                assert_eq!(resolve(p, r, *m), want, "paste={p} remove={r} mod={m:?}");
            }
        }
    }

    #[test]
    fn copy_without_formatting_keeps_text_and_files() {
        let c = vec![
            Content::new(kind::HTML, "<b>x</b>"),
            Content::text("x"),
            Content::new(kind::FILES, "file:///a"),
            Content::new(kind::PNG, vec![1]),
        ];
        let out = strip_formatting(&c);
        assert_eq!(out.len(), 2);
        assert!(out
            .iter()
            .all(|c| c.kind == kind::TEXT || c.kind == kind::FILES));
    }

    #[test]
    fn copy_without_formatting_on_item_without_text_is_unchanged() {
        let c = vec![
            Content::new(kind::PNG, vec![1]),
            Content::new(kind::HTML, "<b>x</b>"),
        ];
        assert_eq!(strip_formatting(&c), c);
    }
}
