//! Filesystem store: one `*.json` file per checklist in a shared directory.
//!
//! The directory is injected (from config), never hardcoded — the same store the TUI, the GUI,
//! and agents all point at.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::model::Checklist;

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Create the store directory if it does not exist.
    pub fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.dir)
            .with_context(|| format!("creating store dir {}", self.dir.display()))
    }

    pub fn path_for(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    pub fn exists(&self, id: &str) -> bool {
        self.path_for(id).is_file()
    }

    /// Sorted checklist ids (filename stems of `*.json`), matching the TUI's sidebar order.
    pub fn list_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = match fs::read_dir(&self.dir) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_file() && p.extension().map_or(false, |x| x == "json"))
                .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .collect(),
            Err(_) => Vec::new(),
        };
        ids.sort();
        ids
    }

    /// Read one checklist. Tolerant of mid-write/invalid JSON (yields empty items then).
    pub fn read(&self, id: &str) -> Result<Checklist> {
        let text = fs::read_to_string(self.path_for(id))
            .with_context(|| format!("reading checklist {id}"))?;
        Ok(Checklist::from_json(id, &text))
    }

    /// Read every checklist, in sidebar order. Unreadable files are skipped.
    pub fn read_all(&self) -> Vec<Checklist> {
        self.list_ids().iter().filter_map(|id| self.read(id).ok()).collect()
    }

    /// Write a checklist atomically (temp file + rename) so readers never see a partial file.
    pub fn write(&self, cl: &Checklist) -> Result<()> {
        self.ensure_dir()?;
        let path = self.path_for(&cl.id);
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, cl.to_json()).with_context(|| format!("writing {}", tmp.display()))?;
        fs::rename(&tmp, &path).with_context(|| format!("replacing {}", path.display()))?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let path = self.path_for(id);
        if path.exists() {
            fs::remove_file(&path).with_context(|| format!("deleting {}", path.display()))?;
        }
        Ok(())
    }

    /// Toggle one item's done state (preserving the name). Returns the new state.
    pub fn toggle_item(&self, id: &str, index: usize) -> Result<bool> {
        let mut cl = self.read(id)?;
        anyhow::ensure!(index < cl.items.len(), "item index {index} out of range");
        cl.items[index].done = !cl.items[index].done;
        let now = cl.items[index].done;
        self.write(&cl)?;
        Ok(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Item;
    use std::sync::atomic::{AtomicU32, Ordering};

    // Minimal std-only temp dir, removed on drop — avoids a dev-dependency.
    struct TmpDir {
        path: std::path::PathBuf,
    }
    impl TmpDir {
        fn new() -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let mut path = std::env::temp_dir();
            path.push(format!("checklists-test-{}-{}", std::process::id(), n));
            std::fs::create_dir_all(&path).unwrap();
            TmpDir { path }
        }
        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }
    impl Drop for TmpDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn store() -> (TmpDir, Store) {
        let dir = TmpDir::new();
        let store = Store::new(dir.path());
        (dir, store)
    }

    #[test]
    fn bare_array_round_trips() {
        let (_d, s) = store();
        let cl = Checklist {
            id: "t".into(),
            name: None,
            items: vec![
                Item { label: "a".into(), done: true, detail: String::new(), notes: None },
                Item { label: "b".into(), done: false, detail: "do it".into(), notes: Some("n".into()) },
            ],
        };
        s.write(&cl).unwrap();
        assert_eq!(s.read("t").unwrap(), cl);
        let text = fs::read_to_string(s.path_for("t")).unwrap();
        assert!(text.trim_start().starts_with('['), "no-name form writes a bare array");
    }

    #[test]
    fn named_object_preserves_name() {
        let (_d, s) = store();
        s.write(&Checklist { id: "t".into(), name: Some("Nice".into()), items: Vec::new() }).unwrap();
        let text = fs::read_to_string(s.path_for("t")).unwrap();
        assert!(text.contains("\"name\": \"Nice\""));
        assert_eq!(s.read("t").unwrap().display_name(), "Nice");
    }

    #[test]
    fn list_ids_sorted() {
        let (_d, s) = store();
        for id in ["b", "a", "c"] {
            s.write(&Checklist::new(id)).unwrap();
        }
        assert_eq!(s.list_ids(), vec!["a", "b", "c"]);
    }

    #[test]
    fn toggle_updates_progress_and_keeps_name() {
        let (_d, s) = store();
        let cl = Checklist {
            id: "t".into(),
            name: Some("Keep".into()),
            items: vec![Item::new("a"), Item::new("b")],
        };
        s.write(&cl).unwrap();
        assert_eq!(s.read("t").unwrap().progress().percent(), 0);
        assert!(s.toggle_item("t", 0).unwrap());
        let back = s.read("t").unwrap();
        assert_eq!(back.progress().percent(), 50);
        assert_eq!(back.display_name(), "Keep");
    }

    #[test]
    fn delete_removes_file() {
        let (_d, s) = store();
        s.write(&Checklist::new("t")).unwrap();
        assert!(s.exists("t"));
        s.delete("t").unwrap();
        assert!(!s.exists("t"));
    }
}
