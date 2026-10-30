//! Dev tool (not part of the app): writes a synthetic history.sqlite + settings.json.
//! Every string is generated here; nothing is read from the clipboard or the user's files.
//! Usage: windy-seed <dir> <count>   (count up to 999, the most the app keeps)

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use windy_core::clipboard::NewItem;
use windy_core::history::History;
use windy_core::settings::Settings;
use windy_core::storage::Storage;
use windy_core::types::Content;

const WORDS: [&str; 24] = [
    "quarterly",
    "invoice",
    "meeting",
    "notes",
    "draft",
    "release",
    "checklist",
    "budget",
    "roadmap",
    "feedback",
    "summary",
    "review",
    "backlog",
    "sprint",
    "design",
    "launch",
    "customer",
    "archive",
    "template",
    "agenda",
    "estimate",
    "report",
    "update",
    "proposal",
];
const APPS: [&str; 5] = [
    "chrome.exe",
    "Code.exe",
    "WindowsTerminal.exe",
    "notepad.exe",
    "OUTLOOK.EXE",
];

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = PathBuf::from(args.next().expect("usage: windy-seed <dir> <count>"));
    let count: usize = args
        .next()
        .expect("count")
        .parse()
        .expect("count is a number");
    std::fs::create_dir_all(&dir).expect("create dir");

    let mut settings = Settings::default();
    settings.history_size = 999;
    settings.clamp();

    let mut storage = Storage::open(&dir.join("history.sqlite")).expect("open storage");
    let mut history = History::new();

    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_millis() as i64;
    let step_ms: i64 = 17 * 60 * 1000;

    let mut ids = Vec::with_capacity(count);
    for i in 0..count {
        let w1 = WORDS[(next() % WORDS.len() as u64) as usize];
        let w2 = WORDS[(next() % WORDS.len() as u64) as usize];
        let text = match i % 8 {
            0 => format!("{w1} {w2} #{i}"),
            1 => format!("https://example.com/docs/{w1}/{w2}?ref=demo-{i}"),
            2 => format!(
                "fn {w1}_{i}(input: &str) -> usize {{\n    input.split_whitespace().count() + {i}\n}}"
            ),
            3 => format!("demo.user{i}@example.com"),
            4 => format!(
                "Sample paragraph {i}: the {w1} {w2} is attached for review. Please confirm the \
                 numbers before the next sync and flag anything that looks off. (synthetic text)"
            ),
            5 => format!("C:\\Users\\demo\\Documents\\{w1}-{w2}-{i}.pdf"),
            6 => format!("INV-2026-{i:05}"),
            _ => format!("{{\"id\": {i}, \"name\": \"{w1}\", \"tags\": [\"{w2}\", \"demo\"]}}"),
        };
        let app = APPS[(next() % APPS.len() as u64) as usize];
        let ts = now - (count - i) as i64 * step_ms;
        let id = history.add(
            NewItem {
                application: Some(app.to_owned()),
                contents: vec![Content::text(&text)],
                from_windy: false,
            },
            ts,
            &settings,
        );
        ids.push(id);
    }

    // Pin three items so the pinned section shows up.
    for k in [7usize, 15, 23] {
        if let Some(&id) = ids.get(k) {
            history.pin(id, next(), &[]);
        }
    }

    storage.apply(&mut history).expect("write history");
    let json = serde_json::to_string_pretty(&settings).expect("encode settings");
    std::fs::write(dir.join("settings.json"), json).expect("write settings");
    println!(
        "seeded {} items into {}",
        history.items().len(),
        dir.display()
    );
}
