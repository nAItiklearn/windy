# Test matrix: Maccy's tests and their Windy equivalents

Source list: `docs/MACCY-SPEC.md` section 12. "mac-only" items have no meaning off macOS.

| Maccy test | Windy test |
|---|---|
| Clipboard: ignore whitespace-only text | `clipboard::ignores_whitespace_only_text` |
| Clipboard: ignore newline-only text | `clipboard::ignores_newline_only_text` |
| Clipboard: RTF and HTML not ignored | `clipboard::rtf_and_html_are_not_ignored_when_text_is_blank` |
| Clipboard: ignoreEvents / ignore only next | `clipboard::ignore_events_blocks_everything`, `ignore_only_next_resets_after_one_copy` |
| Clipboard: ignore application / all except | `clipboard::ignores_listed_application`, `ignore_all_apps_except_listed` |
| Clipboard: ignore transient / custom / unknown types | `clipboard::ignores_transient_concealed_and_auto_generated`, `ignores_custom_listed_types`, `ignores_unknown_only_types` |
| Clipboard: ignore regexp | `clipboard::ignore_regexp_skips_matching_text`, `invalid_ignore_regexp_never_blocks` |
| Clipboard: remove disabled / dynamic types | `clipboard::disabled_types_are_removed`, `dynamic_types_are_removed`, `ole_source_types_are_dropped` |
| Clipboard: Safari web archive not stored | `clipboard::safari_web_archive_is_not_stored` |
| Clipboard: merge multiple items | `clipboard::multiple_clipboard_items_are_merged` |
| Clipboard: items without data | `clipboard::items_without_data_are_ignored` |
| Clipboard: frontmost app id | `clipboard::frontmost_app_is_recorded_and_windy_flag_passed_on` |
| Clipboard: copy, copy string, copy without formatting | `action::full_truth_table`, `copy_without_formatting_*` |
| HistoryItem: title for string / RTF / HTML / file / image | `title::title_for_*` |
| HistoryItem: empty item | `title::title_for_empty_item_keeps_previous` |
| HistoryItem: letters before symbols pin order | `sorter::letters_before_symbols_when_no_pin_order` |
| HistoryItem: several items with empty pin | `sorter::several_unpinned_items_are_all_listed` |
| Shortened string, unsafe title scalars | `title::title_is_shortened`, `unsafe_scalars_are_removed` |
| Search: simple / fuzzy / regexp | `search::simple_*`, `fuzzy_*`, `regexp_*`, `mixed_*` |
| Sorter: last copied / first copied / copies | `sorter::by_last_copied`, `by_first_copied`, `by_number_of_copies` |
| History: dedupe, limit, pins, clear | `history::*` |
| HistoryItemDecorator: highlight, thumbnails, deallocation | UI: `popup.spec.js` (search highlight, virtualised list). Image thumbnails need the real app (manual) |
| XCUITest popup suite | `tests/e2e/popup.spec.js` (demo mode) |
| Storage persistence | `storage::*` |

Not ported: Universal Clipboard title variants, OCR title for images, HEIC colour-image tests (all macOS-only);
the modified-type shortcut (`x.nspasteboard.ModifiedType`).
