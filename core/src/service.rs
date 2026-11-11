//! Stdio JSON-lines service — one long-lived process the Electron main talks to.
//!
//! Protocol: one JSON object per line.
//!   request  -> {"id": <any>, "method": "list|get|toggle|set_done|config", "params": {..}}
//!   response -> {"id": <same>, "ok": true, "result": {..}}  |  {"id", "ok": false, "error": ".."}
//!   event    -> {"event": "changed"}   (pushed when the store changes on disk; no id)
//!
//! Requests arrive on stdin, responses + events go to stdout. On stdin EOF the process exits.

use std::io::{BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};

use anyhow::{anyhow, bail, Result};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::model::{Checklist, Item};
use crate::store::Store;

const POLL_MS: u64 = 500; // same cadence as the TUI's poll

#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// Run the service until stdin closes.
pub fn serve(store: Store) -> Result<()> {
    let out = Arc::new(Mutex::new(std::io::stdout()));

    // Watch thread: poll the store dir, push a `changed` event when files change.
    {
        let out = Arc::clone(&out);
        let dir = store.dir().to_path_buf();
        thread::spawn(move || {
            let watch = Store::new(dir);
            let mut last = poll_sig(&watch);
            loop {
                thread::sleep(Duration::from_millis(POLL_MS));
                let sig = poll_sig(&watch);
                if sig != last {
                    last = sig;
                    write_line(&out, &json!({ "event": "changed" }).to_string());
                }
            }
        });
    }

    // Main thread: read request lines, reply.
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let resp = handle_line(&store, &line);
        write_line(&out, &resp);
    }
    Ok(())
}

fn write_line(out: &Arc<Mutex<std::io::Stdout>>, s: &str) {
    if let Ok(mut o) = out.lock() {
        let _ = o.write_all(s.as_bytes());
        let _ = o.write_all(b"\n");
        let _ = o.flush();
    }
}

fn handle_line(store: &Store, line: &str) -> String {
    let line = line.trim_start_matches('\u{feff}'); // tolerate a leading UTF-8 BOM
    let req: Request = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => return json!({ "ok": false, "error": format!("bad request: {e}") }).to_string(),
    };
    let id = req.id.clone().unwrap_or(Value::Null);
    match dispatch(store, &req) {
        Ok(result) => json!({ "id": id, "ok": true, "result": result }).to_string(),
        Err(e) => json!({ "id": id, "ok": false, "error": e.to_string() }).to_string(),
    }
}

fn dispatch(store: &Store, req: &Request) -> Result<Value> {
    match req.method.as_str() {
        "list" => {
            let checklists: Vec<Value> = store.read_all().iter().map(summary).collect();
            Ok(json!({ "dir": store.dir().display().to_string(), "checklists": checklists }))
        }
        "get" => {
            let id = param_str(req, "id")?;
            Ok(full(&store.read(id)?))
        }
        "toggle" => {
            let id = param_str(req, "id")?;
            let index = param_usize(req, "index")?;
            let done = store.toggle_item(id, index)?;
            Ok(json!({ "id": id, "index": index, "done": done }))
        }
        "set_done" => {
            let id = param_str(req, "id")?;
            let index = param_usize(req, "index")?;
            let done = req.params.get("done").and_then(Value::as_bool)
                .ok_or_else(|| anyhow!("set_done: missing bool 'done'"))?;
            let mut cl = store.read(id)?;
            if index >= cl.items.len() {
                bail!("item index {index} out of range");
            }
            cl.items[index].done = done;
            store.write(&cl)?;
            Ok(json!({ "id": id, "index": index, "done": done }))
        }
        "rename" => {
            let id = param_str(req, "id")?;
            // empty/absent name clears it (falls back to the stem); matching the stem also clears
            let name = req.params.get("name").and_then(Value::as_str).map(str::trim)
                .filter(|s| !s.is_empty() && *s != id).map(str::to_string);
            let mut cl = store.read(id)?;
            cl.name = name;
            store.write(&cl)?;
            Ok(json!({ "id": id, "name": cl.name }))
        }
        "edit_item" => {
            let id = param_str(req, "id")?;
            let index = param_usize(req, "index")?;
            let label = req.params.get("label").and_then(Value::as_str).unwrap_or("").to_string();
            let detail = req.params.get("detail").and_then(Value::as_str).unwrap_or("").to_string();
            let mut cl = store.read(id)?;
            if index >= cl.items.len() { bail!("item index {index} out of range"); }
            cl.items[index].label = label;
            cl.items[index].detail = detail;
            store.write(&cl)?;
            Ok(full(&cl))
        }
        "add_item" => {
            let id = param_str(req, "id")?;
            let label = req.params.get("label").and_then(Value::as_str).unwrap_or("").to_string();
            let mut cl = store.read(id)?;
            cl.items.push(Item::new(label));
            store.write(&cl)?;
            Ok(full(&cl))
        }
        "delete_item" => {
            let id = param_str(req, "id")?;
            let index = param_usize(req, "index")?;
            let mut cl = store.read(id)?;
            if index >= cl.items.len() { bail!("item index {index} out of range"); }
            cl.items.remove(index);
            store.write(&cl)?;
            Ok(full(&cl))
        }
        "delete" => {
            let id = param_str(req, "id")?;
            store.delete(id)?;
            Ok(json!({ "id": id, "deleted": true }))
        }
        "create" => {
            let id = param_str(req, "id")?;
            if !safe_id(id) { bail!("invalid checklist id"); }
            if store.exists(id) { bail!("checklist '{id}' already exists"); }
            let name = req.params.get("name").and_then(Value::as_str).map(str::trim)
                .filter(|s| !s.is_empty()).map(str::to_string);
            let mut cl = Checklist::new(id);
            cl.name = name;
            store.write(&cl)?;
            Ok(summary(&cl))
        }
        "config" | "get_config" => {
            let cfg = crate::config::Config::load();
            Ok(json!({ "dir": store.dir().display().to_string(), "theme": cfg.theme, "chime": cfg.chime, "tick": cfg.tick }))
        }
        "set_config" => {
            let mut cfg = crate::config::Config::load();
            if let Some(t) = req.params.get("theme").and_then(Value::as_str) { cfg.theme = t.to_string(); }
            if let Some(c) = req.params.get("chime").and_then(Value::as_bool) { cfg.chime = c; }
            if let Some(t) = req.params.get("tick").and_then(Value::as_bool) { cfg.tick = t; }
            cfg.save()?;
            Ok(json!({ "theme": cfg.theme, "chime": cfg.chime, "tick": cfg.tick }))
        }
        other => bail!("unknown method: {other}"),
    }
}

fn param_str<'a>(req: &'a Request, key: &str) -> Result<&'a str> {
    req.params.get(key).and_then(Value::as_str).ok_or_else(|| anyhow!("missing string '{key}'"))
}

fn param_usize(req: &Request, key: &str) -> Result<usize> {
    req.params.get(key).and_then(Value::as_u64).map(|n| n as usize)
        .ok_or_else(|| anyhow!("missing number '{key}'"))
}

/// A `create` id becomes a filename, so reject anything that could escape the store dir.
fn safe_id(id: &str) -> bool {
    !id.is_empty() && !id.contains(['/', '\\', ':']) && !id.contains("..")
}

/// Lightweight row for the sidebar/list.
fn summary(cl: &Checklist) -> Value {
    let p = cl.progress();
    json!({
        "id": cl.id,
        "name": cl.name,
        "displayName": cl.display_name(),
        "done": p.done,
        "total": p.total,
        "percent": p.percent(),
        "complete": p.is_complete(),
    })
}

/// Full checklist incl. items (detail/notes markdown).
fn full(cl: &Checklist) -> Value {
    let p = cl.progress();
    json!({
        "id": cl.id,
        "name": cl.name,
        "displayName": cl.display_name(),
        "items": serde_json::to_value(&cl.items).unwrap_or_else(|_| json!([])),
        "done": p.done,
        "total": p.total,
        "percent": p.percent(),
        "complete": p.is_complete(),
    })
}

/// (id, mtime-nanos) per file — the change signature the watch thread compares.
fn poll_sig(store: &Store) -> Vec<(String, u128)> {
    store
        .list_ids()
        .iter()
        .map(|id| {
            let mtime = std::fs::metadata(store.path_for(id))
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            (id.clone(), mtime)
        })
        .collect()
}
