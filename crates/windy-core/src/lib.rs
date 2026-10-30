//! Windy core: the clipboard history engine. No OS, UI or Tauri dependencies.

pub mod action;
pub mod clipboard;
pub mod history;
pub mod search;
pub mod settings;
pub mod sorter;
pub mod storage;
pub mod title;
pub mod types;

pub use types::{Content, Item};
