//! Checklist domain model — mirrors the shared JSON contract of the original TUI.
//!
//! A checklist file is EITHER a bare array of item objects, OR `{ "name"?, "items": [...] }`.
//! Item shape: `{ "label", "done", "detail"?, "notes"? }`. `detail`/`notes` are markdown.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One checklist item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub done: bool,
    // Always written (even ""), to stay byte-compatible with the Python writer.
    #[serde(default)]
    pub detail: String,
    // Kept only when a non-empty markdown string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Item {
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), done: false, detail: String::new(), notes: None }
    }

    /// Build tolerantly from a raw JSON value; non-objects are rejected (None = skip the entry).
    pub fn from_value(v: &Value) -> Option<Item> {
        let o = v.as_object()?;
        Some(Item {
            label: o.get("label").and_then(Value::as_str).unwrap_or("").to_string(),
            done: o.get("done").and_then(Value::as_bool).unwrap_or(false),
            detail: o.get("detail").and_then(Value::as_str).unwrap_or("").to_string(),
            // notes kept only when present AND non-empty
            notes: o
                .get("notes")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
        })
    }
}

/// Done/total for a checklist, with a rounded percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
}

impl Progress {
    pub fn percent(&self) -> u8 {
        if self.total == 0 {
            0
        } else {
            ((self.done as f64 / self.total as f64) * 100.0).round() as u8
        }
    }
    pub fn is_complete(&self) -> bool {
        self.total > 0 && self.done == self.total
    }
}

/// A whole checklist: its id (filename stem), optional display name, and items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checklist {
    pub id: String,
    pub name: Option<String>,
    pub items: Vec<Item>,
}

impl Checklist {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), name: None, items: Vec::new() }
    }

    /// Display name: the custom `name` if set, otherwise the filename stem (`id`).
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.id)
    }

    pub fn progress(&self) -> Progress {
        Progress {
            done: self.items.iter().filter(|i| i.done).count(),
            total: self.items.len(),
        }
    }

    /// Parse a checklist file's text (bare array, or `{name, items}`), tolerant of bad JSON
    /// (an unreadable/half-written file yields an empty checklist, matching the TUI).
    pub fn from_json(id: impl Into<String>, text: &str) -> Checklist {
        let id = id.into();
        let value: Value = serde_json::from_str(text).unwrap_or(Value::Null);
        let (name, items_val) = match &value {
            Value::Object(o) => (
                o.get("name")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string),
                o.get("items").cloned().unwrap_or_else(|| Value::Array(Vec::new())),
            ),
            Value::Array(_) => (None, value.clone()),
            _ => (None, Value::Array(Vec::new())),
        };
        let items = items_val
            .as_array()
            .map(|a| a.iter().filter_map(Item::from_value).collect())
            .unwrap_or_default();
        Checklist { id, name, items }
    }

    /// Serialize to file text: an object when a name is set, else a bare array; trailing newline.
    pub fn to_json(&self) -> String {
        let body = match &self.name {
            Some(name) => serde_json::to_string_pretty(&Named { name, items: &self.items }),
            None => serde_json::to_string_pretty(&self.items),
        };
        let mut s = body.unwrap_or_else(|_| "[]".to_string());
        s.push('\n');
        s
    }
}

/// Internal: the `{name, items}` object form used only when a name is set.
#[derive(Serialize)]
struct Named<'a> {
    name: &'a str,
    items: &'a [Item],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_array() {
        let cl = Checklist::from_json("t", r#"[{"label":"a","done":true}]"#);
        assert_eq!(cl.name, None);
        assert_eq!(cl.items.len(), 1);
        assert!(cl.items[0].done);
        assert_eq!(cl.display_name(), "t");
    }

    #[test]
    fn parses_named_object() {
        let cl = Checklist::from_json("t", r#"{"name":"Nice","items":[{"label":"a","done":false}]}"#);
        assert_eq!(cl.name.as_deref(), Some("Nice"));
        assert_eq!(cl.display_name(), "Nice");
    }

    #[test]
    fn invalid_json_is_empty() {
        assert!(Checklist::from_json("t", "{ broken").items.is_empty());
    }

    #[test]
    fn empty_notes_dropped() {
        let cl = Checklist::from_json("t", r#"[{"label":"a","done":false,"notes":""}]"#);
        assert_eq!(cl.items[0].notes, None);
    }

    #[test]
    fn progress_percent_rounds() {
        let mut cl = Checklist::new("t");
        cl.items = vec![Item::new("a"), Item::new("b"), Item::new("c")];
        cl.items[0].done = true;
        assert_eq!(cl.progress(), Progress { done: 1, total: 3 });
        assert_eq!(cl.progress().percent(), 33);
        assert!(!cl.progress().is_complete());
    }
}
