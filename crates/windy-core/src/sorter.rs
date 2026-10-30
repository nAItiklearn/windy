use std::cmp::Reverse;

use crate::settings::{PinTo, SortBy};
use crate::types::Item;

fn pin_rank(c: char) -> (u8, char) {
    // Letters sort before symbols.
    (u8::from(!c.is_alphanumeric()), c)
}

/// Indices of `items` in display order: pinned items (user's pin order, then letters before symbols)
/// above or below the unpinned items sorted by `sort_by`, all descending.
pub fn sort_items(
    items: &[Item],
    sort_by: SortBy,
    pin_to: PinTo,
    pin_order: &[char],
) -> Vec<usize> {
    let mut pinned: Vec<usize> = (0..items.len())
        .filter(|&i| items[i].pin.is_some())
        .collect();
    let mut rest: Vec<usize> = (0..items.len())
        .filter(|&i| items[i].pin.is_none())
        .collect();

    pinned.sort_by_key(|&i| {
        let c = items[i].pin.unwrap_or(' ');
        match pin_order.iter().position(|p| *p == c) {
            Some(pos) => (0, pos, (0, ' ')),
            None => (1, 0, pin_rank(c)),
        }
    });

    rest.sort_by_key(|&i| {
        let it = &items[i];
        let key = match sort_by {
            SortBy::LastCopiedAt => it.last_copied_at,
            SortBy::FirstCopiedAt => it.first_copied_at,
            SortBy::NumberOfCopies => i64::from(it.number_of_copies),
        };
        (Reverse(key), Reverse(it.id))
    });

    match pin_to {
        PinTo::Top => pinned.into_iter().chain(rest).collect(),
        PinTo::Bottom => rest.into_iter().chain(pinned).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Content;

    fn item(id: i64, first: i64, last: i64, copies: u32, pin: Option<char>) -> Item {
        Item {
            id,
            application: None,
            first_copied_at: first,
            last_copied_at: last,
            number_of_copies: copies,
            pin,
            title: String::new(),
            contents: vec![Content::text(&id.to_string())],
        }
    }

    fn ids(items: &[Item], order: Vec<usize>) -> Vec<i64> {
        order.into_iter().map(|i| items[i].id).collect()
    }

    fn sample() -> Vec<Item> {
        vec![
            item(1, 10, 30, 1, None),
            item(2, 20, 10, 5, None),
            item(3, 30, 20, 3, None),
        ]
    }

    #[test]
    fn by_last_copied() {
        let s = sample();
        let o = sort_items(&s, SortBy::LastCopiedAt, PinTo::Top, &[]);
        assert_eq!(ids(&s, o), vec![1, 3, 2]);
    }

    #[test]
    fn by_first_copied() {
        let s = sample();
        let o = sort_items(&s, SortBy::FirstCopiedAt, PinTo::Top, &[]);
        assert_eq!(ids(&s, o), vec![3, 2, 1]);
    }

    #[test]
    fn by_number_of_copies() {
        let s = sample();
        let o = sort_items(&s, SortBy::NumberOfCopies, PinTo::Top, &[]);
        assert_eq!(ids(&s, o), vec![2, 3, 1]);
    }

    #[test]
    fn pins_top_and_bottom() {
        let mut s = sample();
        s[1].pin = Some('b');
        let top = sort_items(&s, SortBy::LastCopiedAt, PinTo::Top, &[]);
        assert_eq!(ids(&s, top), vec![2, 1, 3]);
        let bottom = sort_items(&s, SortBy::LastCopiedAt, PinTo::Bottom, &[]);
        assert_eq!(ids(&s, bottom), vec![1, 3, 2]);
    }

    #[test]
    fn letters_before_symbols_when_no_pin_order() {
        let s = vec![
            item(1, 1, 1, 1, Some(';')),
            item(2, 1, 1, 1, Some('c')),
            item(3, 1, 1, 1, Some('b')),
        ];
        let o = sort_items(&s, SortBy::LastCopiedAt, PinTo::Top, &[]);
        assert_eq!(ids(&s, o), vec![3, 2, 1]);
    }

    #[test]
    fn user_pin_order_wins() {
        let s = vec![
            item(1, 1, 1, 1, Some('b')),
            item(2, 1, 1, 1, Some('c')),
            item(3, 1, 1, 1, Some('d')),
        ];
        let o = sort_items(&s, SortBy::LastCopiedAt, PinTo::Top, &['d', 'b']);
        assert_eq!(ids(&s, o), vec![3, 1, 2]);
    }

    #[test]
    fn several_unpinned_items_are_all_listed() {
        let s = sample();
        assert_eq!(
            sort_items(&s, SortBy::LastCopiedAt, PinTo::Top, &['x']).len(),
            3
        );
    }
}
