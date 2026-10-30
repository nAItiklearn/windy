use serde::{Deserialize, Serialize};

/// Content type identifiers. Platform layers map native formats to these.
pub mod kind {
    pub const TEXT: &str = "text/plain";
    pub const HTML: &str = "text/html";
    pub const RTF: &str = "text/rtf";
    pub const PNG: &str = "image/png";
    pub const TIFF: &str = "image/tiff";
    pub const JPEG: &str = "image/jpeg";
    pub const HEIC: &str = "image/heic";
    /// Newline-separated, percent-encoded file URLs.
    pub const FILES: &str = "text/uri-list";
    pub const PDF: &str = "application/pdf";
    pub const WEB_ARCHIVE: &str = "application/x-webarchive";

    /// Marker added to everything Windy writes to the clipboard itself.
    pub const MARKER: &str = "application/x-windy";
    pub const SOURCE: &str = "application/x-clipboard-source";

    /// Clipboard hints that mean "do not record this copy".
    pub const CONCEALED: &str = "application/x-concealed";
    pub const TRANSIENT: &str = "application/x-transient";
    pub const AUTO_GENERATED: &str = "application/x-auto-generated";

    pub const WORD_LINK_SOURCE: &str = "com.microsoft.Link-Source";
    pub const WORD_OBJECT_LINK: &str = "com.microsoft.ObjectLink";

    pub const IMAGES: [&str; 4] = [PNG, TIFF, JPEG, HEIC];
    pub const TEXTS: [&str; 3] = [HTML, RTF, TEXT];

    /// Types that never take part in duplicate detection.
    pub const NOISE: [&str; 6] = [
        MARKER,
        SOURCE,
        "application/x-modified-type",
        "application/x-link-presentation",
        "application/x-chromium-custom-data",
        "application/x-webkit-custom-data",
    ];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Content {
    pub kind: String,
    pub data: Vec<u8>,
}

impl Content {
    pub fn new(kind: &str, data: impl Into<Vec<u8>>) -> Self {
        Self {
            kind: kind.to_owned(),
            data: data.into(),
        }
    }

    pub fn text(s: &str) -> Self {
        Self::new(kind::TEXT, s.as_bytes())
    }

    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.data).ok()
    }

    pub fn is_noise(&self) -> bool {
        kind::NOISE.contains(&self.kind.as_str())
    }
}

/// Timestamps are milliseconds since the Unix epoch and are always passed in by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: i64,
    pub application: Option<String>,
    pub first_copied_at: i64,
    pub last_copied_at: i64,
    pub number_of_copies: u32,
    pub pin: Option<char>,
    pub title: String,
    pub contents: Vec<Content>,
}

impl Item {
    pub fn content(&self, kind: &str) -> Option<&Content> {
        self.contents.iter().find(|c| c.kind == kind)
    }

    /// True when every non-noise content of `other` is present here with identical bytes.
    pub fn supersedes(&self, other: &Item) -> bool {
        let mut any = false;
        for c in other.contents.iter().filter(|c| !c.is_noise()) {
            any = true;
            if !self.contents.iter().any(|m| m == c) {
                return false;
            }
        }
        any
    }
}
