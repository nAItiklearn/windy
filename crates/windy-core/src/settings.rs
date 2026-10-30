use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortBy {
    LastCopiedAt,
    FirstCopiedAt,
    NumberOfCopies,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PinTo {
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchMode {
    Exact,
    Fuzzy,
    Regexp,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PopupPosition {
    Cursor,
    StatusItem,
    Window,
    Center,
    LastPosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Highlight {
    Color,
    Bold,
    Italic,
    Underline,
}

/// Every default mirrors `docs/MACCY-SPEC.md`. Unknown or missing keys fall back to the default.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub history_size: u32,
    pub sort_by: SortBy,
    pub pin_to: PinTo,
    pub search_mode: SearchMode,
    pub paste_by_default: bool,
    pub remove_formatting_by_default: bool,
    pub popup_position: PopupPosition,
    pub popup_screen: u32,
    pub window_size: (u32, u32),
    pub window_position: (f32, f32),
    pub preview_width: u32,
    pub preview_delay_ms: u32,
    pub open_preview_automatically: bool,
    pub image_max_height: u32,
    pub highlight_match: Highlight,
    pub show_special_symbols: bool,
    pub show_search: bool,
    pub show_title: bool,
    pub show_footer: bool,
    pub show_application_icons: bool,
    pub show_hex_color_swatch: bool,
    pub show_in_status_bar: bool,
    pub show_recent_copy_in_menu_bar: bool,
    pub save_files: bool,
    pub save_images: bool,
    pub save_text: bool,
    pub ignored_pasteboard_types: Vec<String>,
    pub ignored_apps: Vec<String>,
    pub ignore_all_apps_except_listed: bool,
    pub ignore_regexp: Vec<String>,
    pub ignore_events: bool,
    pub clear_on_quit: bool,
    pub clear_system_clipboard: bool,
    pub suppress_clear_alert: bool,
    pub clipboard_check_interval_ms: u32,
    pub pin_order: Vec<char>,
    /// Global shortcut that toggles the popup, in Tauri accelerator syntax.
    pub popup_hotkey: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            history_size: 200,
            sort_by: SortBy::LastCopiedAt,
            pin_to: PinTo::Top,
            search_mode: SearchMode::Exact,
            paste_by_default: false,
            remove_formatting_by_default: false,
            popup_position: PopupPosition::Cursor,
            popup_screen: 0,
            window_size: (450, 800),
            window_position: (0.5, 0.8),
            preview_width: 400,
            preview_delay_ms: 1500,
            open_preview_automatically: true,
            image_max_height: 40,
            highlight_match: Highlight::Bold,
            show_special_symbols: true,
            show_search: true,
            show_title: true,
            show_footer: true,
            show_application_icons: false,
            show_hex_color_swatch: true,
            show_in_status_bar: true,
            show_recent_copy_in_menu_bar: false,
            save_files: true,
            save_images: true,
            save_text: true,
            ignored_pasteboard_types: [
                "Pasteboard generator type",
                "com.agilebits.onepassword",
                "com.typeit4me.clipping",
                "de.petermaurer.TransientPasteboardType",
                "net.antelle.keeweb",
            ]
            .map(String::from)
            .to_vec(),
            ignored_apps: Vec::new(),
            ignore_all_apps_except_listed: false,
            ignore_regexp: Vec::new(),
            ignore_events: false,
            clear_on_quit: false,
            clear_system_clipboard: false,
            suppress_clear_alert: false,
            clipboard_check_interval_ms: 500,
            pin_order: Vec::new(),
            popup_hotkey: "CommandOrControl+Alt+V".to_owned(),
        }
    }
}

impl Settings {
    /// Brings out-of-range values back into the ranges the settings UI allows.
    pub fn clamp(&mut self) {
        self.history_size = self.history_size.clamp(1, 999);
        self.image_max_height = self.image_max_height.clamp(1, 200);
        self.preview_delay_ms = self.preview_delay_ms.clamp(200, 100_000);
        self.clipboard_check_interval_ms = self.clipboard_check_interval_ms.clamp(100, 5_000);
    }

    pub fn from_json(s: &str) -> Self {
        let mut v: Settings = serde_json::from_str(s).unwrap_or_default();
        v.clamp();
        v
    }
}
