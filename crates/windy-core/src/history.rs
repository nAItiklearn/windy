//! In-memory history: dedupe, size limit, pins, clear (MACCY-SPEC section 3).
//! Mutations are recorded as `Change`s that `storage::Storage::apply` persists.

use crate::clipboard::NewItem;
use crate::settings::Settings;
use crate::sorter::sort_items;
use crate::title::generate_title;
use crate::types::Item;

/// Characters a pin may use (MACCY-SPEC section 3). Callers pass keys reserved by shortcuts.
pub const PIN_CHARS: &str = "bcdefghijklmnoprstuxy[];'./-=\\";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Upsert(i64),
    Delete(i64),
}

#[derive(Debug, Default)]
pub struct History {
    items: Vec<Item>,
    next_id: i64,
    changes: Vec<Change>,
}

impl History {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
            changes: Vec::new(),
        }
    }

    pub fn from_items(items: Vec<Item>) -> Self {
        let next_id = items.iter().map(|i| i.id).max().unwrap_or(0) + 1;
        Self {
            items,
            next_id,
            changes: Vec::new(),
        }
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn get(&self, id: i64) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn drain_changes(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }

    /// Items in display order for the current settings.
    pub fn ordered(&self, s: &Settings) -> Vec<&Item> {
        sort_items(&self.items, s.sort_by, s.pin_to, &s.pin_order)
            .into_iter()
            .map(|i| &self.items[i])
            .collect()
    }

    /// Adds a copy. A stored item that already contains everything in the new one is replaced:
    /// the new item moves to the top and inherits its counts, pin and title.
    pub fn add(&mut self, n: NewItem, now: i64, s: &Settings) -> i64 {
        let id = self.next_id.max(1);
        self.next_id = id + 1;
        let mut item = Item {
            id,
            application: n.application,
            first_copied_at: now,
            last_copied_at: now,
            number_of_copies: 1,
            pin: None,
            title: String::new(),
            contents: n.contents,
        };

        let similar = self.items.iter().position(|e| e.supersedes(&item));
        if let Some(pos) = similar {
            let old = self.items.remove(pos);
            self.changes.push(Change::Delete(old.id));
            item.first_copied_at = old.first_copied_at;
            item.number_of_copies += old.number_of_copies;
            item.pin = old.pin;
            item.title = old.title;
            if n.from_windy {
                item.application = old.application;
            }
        } else {
            item.title = generate_title(&item.contents, "", s.show_special_symbols);
        }

        self.items.push(item);
        self.changes.push(Change::Upsert(id));
        self.enforce_limit(s);
        id
    }

    /// Deletes unpinned items beyond `history_size`, lowest in the current sort first.
    pub fn enforce_limit(&mut self, s: &Settings) {
        let order = sort_items(&self.items, s.sort_by, s.pin_to, &s.pin_order);
        let doomed: Vec<i64> = order
            .into_iter()
            .map(|i| &self.items[i])
            .filter(|it| it.pin.is_none())
            .skip(s.history_size as usize)
            .map(|it| it.id)
            .collect();
        for id in doomed {
            self.delete(id);
        }
    }

    pub fn delete(&mut self, id: i64) -> bool {
        let before = self.items.len();
        self.items.retain(|i| i.id != id);
        let removed = self.items.len() != before;
        if removed {
            self.changes.push(Change::Delete(id));
        }
        removed
    }

    /// Gives the item a free pin character chosen with `rand`. Returns it, or `None` when
    /// the item is unknown, already pinned, or no character is free.
    pub fn pin(&mut self, id: i64, rand: u64, reserved: &[char]) -> Option<char> {
        let used: Vec<char> = self.items.iter().filter_map(|i| i.pin).collect();
        let free: Vec<char> = PIN_CHARS
            .chars()
            .filter(|c| !used.contains(c) && !reserved.contains(c))
            .collect();
        if free.is_empty() {
            return None;
        }
        let ch = free[(rand % free.len() as u64) as usize];
        let item = self
            .items
            .iter_mut()
            .find(|i| i.id == id && i.pin.is_none())?;
        item.pin = Some(ch);
        self.changes.push(Change::Upsert(id));
        Some(ch)
    }

    pub fn unpin(&mut self, id: i64) -> bool {
        match self
            .items
            .iter_mut()
            .find(|i| i.id == id && i.pin.is_some())
        {
            Some(item) => {
                item.pin = None;
                self.changes.push(Change::Upsert(id));
                true
            }
            None => false,
        }
    }

    /// "Clear": deletes all unpinned items.
    pub fn clear(&mut self) {
        let ids: Vec<i64> = self
            .items
            .iter()
            .filter(|i| i.pin.is_none())
            .map(|i| i.id)
            .collect();
        for id in ids {
            self.delete(id);
        }
    }

    /// "Clear all": deletes everything including pins.
    pub fn clear_all(&mut self) {
        let ids: Vec<i64> = self.items.iter().map(|i| i.id).collect();
        for id in ids {
            self.delete(id);
        }
    }
}

/// After deleting `deleted` from the visible `order`, the item to select: the next one, else the previous.
pub fn selection_after_delete(order: &[i64], deleted: i64) -> Option<i64> {
    let pos = order.iter().position(|id| *id == deleted)?;
    order
        .get(pos + 1)
        .or_else(|| pos.checked_sub(1).and_then(|p| order.get(p)))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::SortBy;
    use crate::types::{kind, Content};

    fn new_item(text: &str) -> NewItem {
        NewItem {
            application: Some("app".into()),
            contents: vec![Content::text(text)],
            from_windy: false,
        }
    }

    fn texts(h: &History, s: &Settings) -> Vec<String> {
        h.ordered(s).iter().map(|i| i.title.clone()).collect()
    }

    #[test]
    fn newest_copy_is_first() {
        let s = Settings::default();
        let mut h = History::new();
        h.add(new_item("a"), 1, &s);
        h.add(new_item("b"), 2, &s);
        assert_eq!(texts(&h, &s), vec!["b", "a"]);
    }

    #[test]
    fn duplicate_moves_to_top_and_counts() {
        let s = Settings::default();
        let mut h = History::new();
        h.add(new_item("a"), 1, &s);
        h.add(new_item("b"), 2, &s);
        h.add(new_item("a"), 3, &s);
        assert_eq!(h.items().len(), 2);
        let top = h.ordered(&s)[0];
        assert_eq!(top.title, "a");
        assert_eq!(top.number_of_copies, 2);
        assert_eq!(top.first_copied_at, 1);
        assert_eq!(top.last_copied_at, 3);
    }

    #[test]
    fn duplicate_keeps_pin() {
        let s = Settings::default();
        let mut h = History::new();
        let id = h.add(new_item("a"), 1, &s);
        let ch = h.pin(id, 0, &[]).unwrap();
        h.add(new_item("a"), 2, &s);
        assert_eq!(h.items().len(), 1);
        assert_eq!(h.items()[0].pin, Some(ch));
    }

    #[test]
    fn subset_copy_is_a_duplicate_of_richer_item() {
        let s = Settings::default();
        let mut h = History::new();
        h.add(
            NewItem {
                application: None,
                contents: vec![Content::text("a"), Content::new(kind::HTML, "<b>a</b>")],
                from_windy: false,
            },
            1,
            &s,
        );
        h.add(new_item("a"), 2, &s);
        assert_eq!(h.items().len(), 1);
    }

    #[test]
    fn noise_types_do_not_prevent_dedupe() {
        let s = Settings::default();
        let mut h = History::new();
        h.add(new_item("a"), 1, &s);
        h.add(
            NewItem {
                application: None,
                contents: vec![Content::text("a"), Content::new(kind::MARKER, "1")],
                from_windy: true,
            },
            2,
            &s,
        );
        assert_eq!(h.items().len(), 1);
    }

    #[test]
    fn different_bytes_are_not_duplicates() {
        let s = Settings::default();
        let mut h = History::new();
        h.add(new_item("a"), 1, &s);
        h.add(new_item("A"), 2, &s);
        assert_eq!(h.items().len(), 2);
    }

    #[test]
    fn windy_copy_keeps_original_application() {
        let s = Settings::default();
        let mut h = History::new();
        h.add(new_item("a"), 1, &s);
        h.add(
            NewItem {
                application: Some("windy".into()),
                contents: vec![Content::text("a")],
                from_windy: true,
            },
            2,
            &s,
        );
        assert_eq!(h.items()[0].application.as_deref(), Some("app"));
    }

    #[test]
    fn foreign_copy_updates_application() {
        let s = Settings::default();
        let mut h = History::new();
        h.add(new_item("a"), 1, &s);
        h.add(
            NewItem {
                application: Some("other".into()),
                contents: vec![Content::text("a")],
                from_windy: false,
            },
            2,
            &s,
        );
        assert_eq!(h.items()[0].application.as_deref(), Some("other"));
    }

    #[test]
    fn size_limit_removes_oldest_unpinned() {
        let s = Settings {
            history_size: 3,
            ..Settings::default()
        };
        let mut h = History::new();
        for (i, t) in ["a", "b", "c", "d"].iter().enumerate() {
            h.add(new_item(t), i as i64, &s);
        }
        assert_eq!(texts(&h, &s), vec!["d", "c", "b"]);
    }

    #[test]
    fn pinned_items_do_not_count_against_limit() {
        let s = Settings {
            history_size: 2,
            ..Settings::default()
        };
        let mut h = History::new();
        let a = h.add(new_item("a"), 1, &s);
        h.pin(a, 0, &[]);
        for (i, t) in ["b", "c", "d"].iter().enumerate() {
            h.add(new_item(t), 2 + i as i64, &s);
        }
        assert_eq!(texts(&h, &s), vec!["a", "d", "c"]);
    }

    #[test]
    fn limit_respects_sort_by_number_of_copies() {
        let s = Settings {
            history_size: 2,
            sort_by: SortBy::NumberOfCopies,
            ..Settings::default()
        };
        let mut h = History::new();
        h.add(new_item("a"), 1, &s);
        h.add(new_item("a"), 2, &s);
        h.add(new_item("b"), 3, &s);
        h.add(new_item("c"), 4, &s);
        let titles = texts(&h, &s);
        assert_eq!(titles.len(), 2);
        assert!(titles.contains(&"a".to_string()));
    }

    #[test]
    fn pin_chars_are_unique_and_skip_reserved() {
        let s = Settings {
            history_size: 999,
            ..Settings::default()
        };
        let mut h = History::new();
        let mut seen = Vec::new();
        for i in 0..PIN_CHARS.chars().count() - 1 {
            let id = h.add(new_item(&format!("t{i}")), i as i64, &s);
            let c = h.pin(id, i as u64 * 7, &['p']).expect("a free pin char");
            assert_ne!(c, 'p');
            assert!(!seen.contains(&c));
            seen.push(c);
        }
        let id = h.add(new_item("one more"), 999, &s);
        assert_eq!(h.pin(id, 0, &['p']), None);
    }

    #[test]
    fn reserved_keys_are_never_pin_chars() {
        for c in ['a', 'q', 'v', 'w', 'z', ',', '`'] {
            assert!(!PIN_CHARS.contains(c), "{c}");
        }
    }

    #[test]
    fn unpin_and_double_pin() {
        let s = Settings::default();
        let mut h = History::new();
        let id = h.add(new_item("a"), 1, &s);
        assert!(h.pin(id, 0, &[]).is_some());
        assert!(h.pin(id, 1, &[]).is_none());
        assert!(h.unpin(id));
        assert!(!h.unpin(id));
    }

    #[test]
    fn clear_keeps_pins_clear_all_removes_everything() {
        let s = Settings::default();
        let mut h = History::new();
        let a = h.add(new_item("a"), 1, &s);
        h.add(new_item("b"), 2, &s);
        h.pin(a, 0, &[]);
        h.clear();
        assert_eq!(texts(&h, &s), vec!["a"]);
        h.clear_all();
        assert!(h.items().is_empty());
    }

    #[test]
    fn delete_unknown_is_false() {
        let mut h = History::new();
        assert!(!h.delete(42));
    }

    #[test]
    fn changes_are_recorded_for_storage() {
        let s = Settings::default();
        let mut h = History::new();
        let a = h.add(new_item("a"), 1, &s);
        assert_eq!(h.drain_changes(), vec![Change::Upsert(a)]);
        h.delete(a);
        assert_eq!(h.drain_changes(), vec![Change::Delete(a)]);
        assert!(h.drain_changes().is_empty());
    }

    #[test]
    fn selection_moves_to_neighbour_after_delete() {
        assert_eq!(selection_after_delete(&[1, 2, 3], 2), Some(3));
        assert_eq!(selection_after_delete(&[1, 2, 3], 3), Some(2));
        assert_eq!(selection_after_delete(&[1], 1), None);
        assert_eq!(selection_after_delete(&[1], 9), None);
    }

    #[test]
    fn image_items_get_empty_title() {
        let s = Settings::default();
        let mut h = History::new();
        h.add(
            NewItem {
                application: None,
                contents: vec![Content::new(kind::PNG, vec![1])],
                from_windy: false,
            },
            1,
            &s,
        );
        assert_eq!(h.items()[0].title, "");
    }

    #[test]
    fn from_items_continues_ids() {
        let s = Settings::default();
        let mut h = History::new();
        h.add(new_item("a"), 1, &s);
        let mut h2 = History::from_items(h.items().to_vec());
        let id = h2.add(new_item("b"), 2, &s);
        assert!(id > h.items()[0].id);
    }
}
