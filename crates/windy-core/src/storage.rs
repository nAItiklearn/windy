//! SQLite persistence (WAL). The history keeps a change log; `apply` writes it.

use std::path::Path;

use rusqlite::{params, Connection};

use crate::history::{Change, History};
use crate::types::{Content, Item};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY,
    application TEXT,
    first_copied_at INTEGER NOT NULL,
    last_copied_at INTEGER NOT NULL,
    number_of_copies INTEGER NOT NULL,
    pin TEXT,
    title TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS contents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    data BLOB NOT NULL
);
CREATE INDEX IF NOT EXISTS contents_item ON contents(item_id);
";

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        restrict_permissions(path);
        conn.query_row("PRAGMA journal_mode=WAL", [], |_| Ok(()))?;
        Self::init(conn)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn load(&self) -> Result<Vec<Item>> {
        let mut items_q = self.conn.prepare(
            "SELECT id, application, first_copied_at, last_copied_at, number_of_copies, pin, title
             FROM items ORDER BY id",
        )?;
        let mut contents_q = self
            .conn
            .prepare("SELECT kind, data FROM contents WHERE item_id = ?1 ORDER BY id")?;
        let rows = items_q.query_map([], |r| {
            let pin: Option<String> = r.get(5)?;
            Ok(Item {
                id: r.get(0)?,
                application: r.get(1)?,
                first_copied_at: r.get(2)?,
                last_copied_at: r.get(3)?,
                number_of_copies: r.get(4)?,
                pin: pin.and_then(|p| p.chars().next()),
                title: r.get(6)?,
                contents: Vec::new(),
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            let mut item = row?;
            item.contents = contents_q
                .query_map([item.id], |r| {
                    Ok(Content {
                        kind: r.get(0)?,
                        data: r.get(1)?,
                    })
                })?
                .collect::<std::result::Result<_, _>>()?;
            out.push(item);
        }
        Ok(out)
    }

    /// Writes everything the history changed since the last call, in one transaction.
    pub fn apply(&mut self, history: &mut History) -> Result<()> {
        let changes = history.drain_changes();
        if changes.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction()?;
        for change in changes {
            match change {
                Change::Delete(id) => {
                    tx.execute("DELETE FROM items WHERE id = ?1", [id])?;
                }
                Change::Upsert(id) => {
                    let Some(item) = history.get(id) else {
                        continue;
                    };
                    tx.execute("DELETE FROM items WHERE id = ?1", [id])?;
                    tx.execute(
                        "INSERT INTO items (id, application, first_copied_at, last_copied_at,
                                            number_of_copies, pin, title)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![
                            item.id,
                            item.application,
                            item.first_copied_at,
                            item.last_copied_at,
                            item.number_of_copies,
                            item.pin.map(String::from),
                            item.title
                        ],
                    )?;
                    for c in &item.contents {
                        tx.execute(
                            "INSERT INTO contents (item_id, kind, data) VALUES (?1, ?2, ?3)",
                            params![item.id, c.kind, c.data],
                        )?;
                    }
                }
            }
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

/// On Windows the file lives in a per-user directory under `%LOCALAPPDATA%`; the platform layer
/// tightens the ACL.
#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clipboard::NewItem;
    use crate::settings::Settings;
    use crate::types::kind;

    fn new_item(t: &str) -> NewItem {
        NewItem {
            application: Some("app".into()),
            contents: vec![Content::text(t)],
            from_windy: false,
        }
    }

    #[test]
    fn round_trip() {
        let s = Settings::default();
        let mut st = Storage::open_in_memory().unwrap();
        let mut h = History::new();
        let a = h.add(new_item("a"), 1, &s);
        h.add(
            NewItem {
                application: None,
                contents: vec![Content::new(kind::PNG, vec![0, 1, 2, 255])],
                from_windy: false,
            },
            2,
            &s,
        );
        h.pin(a, 3, &[]);
        st.apply(&mut h).unwrap();
        assert_eq!(st.load().unwrap(), h.items().to_vec());
    }

    #[test]
    fn deletes_and_dedupe_are_persisted() {
        let s = Settings::default();
        let mut st = Storage::open_in_memory().unwrap();
        let mut h = History::new();
        let a = h.add(new_item("a"), 1, &s);
        h.add(new_item("b"), 2, &s);
        h.add(new_item("a"), 3, &s);
        let b = h.items().iter().find(|i| i.title == "b").unwrap().id;
        h.delete(b);
        st.apply(&mut h).unwrap();
        let loaded = st.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_ne!(loaded[0].id, a);
        assert_eq!(loaded[0].number_of_copies, 2);
    }

    #[test]
    fn cascade_removes_contents() {
        let s = Settings::default();
        let mut st = Storage::open_in_memory().unwrap();
        let mut h = History::new();
        let a = h.add(new_item("a"), 1, &s);
        st.apply(&mut h).unwrap();
        h.delete(a);
        st.apply(&mut h).unwrap();
        let n: i64 = st
            .conn
            .query_row("SELECT COUNT(*) FROM contents", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn history_reloads_and_continues() {
        let s = Settings::default();
        let dir = std::env::temp_dir().join(format!("windy-test-{}", std::process::id()));
        let path = dir.join("history.sqlite");
        {
            let mut st = Storage::open(&path).unwrap();
            let mut h = History::new();
            h.add(new_item("a"), 1, &s);
            st.apply(&mut h).unwrap();
        }
        let st = Storage::open(&path).unwrap();
        let mut h = History::from_items(st.load().unwrap());
        assert_eq!(h.items().len(), 1);
        h.add(new_item("b"), 2, &s);
        assert_eq!(h.items().len(), 2);
        drop(st);
        let _ = std::fs::remove_dir_all(dir);
    }
}
