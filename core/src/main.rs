//! checklists — a thin CLI over the core crate.
//!
//! It backs the Electron + Vue GUI (the GUI's main process can shell out to it, or we later swap
//! it for a stdio/JSON-RPC service), and is handy standalone for inspecting the shared store.

use anyhow::{anyhow, bail, Result};

use checklists_app::config::{config_path, Config};
use checklists_app::store::Store;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cfg = Config::load();
    let store = Store::new(cfg.checklist_dir.clone());

    match args.first().map(String::as_str) {
        None | Some("list") => cmd_list(&store),
        Some("show") => cmd_show(&store, args.get(1)),
        Some("toggle") => cmd_toggle(&store, args.get(1), args.get(2)),
        Some("serve") => checklists_app::service::serve(store),
        Some("config") => cmd_config(&cfg),
        Some("init-config") => {
            cfg.save()?;
            println!("wrote {}", config_path().display());
            Ok(())
        }
        Some("help") | Some("-h") | Some("--help") => {
            print_usage();
            Ok(())
        }
        Some(other) => {
            bail!("unknown command: {other}\n\n{}", usage_text());
        }
    }
}

fn cmd_list(store: &Store) -> Result<()> {
    let checklists = store.read_all();
    if checklists.is_empty() {
        println!("(no checklists in {})", store.dir().display());
        return Ok(());
    }
    for cl in checklists {
        let p = cl.progress();
        let mark = if p.is_complete() { "done" } else { "" };
        println!("{:<28} {:>3}%  {}/{}  {}", cl.display_name(), p.percent(), p.done, p.total, mark);
    }
    Ok(())
}

fn cmd_show(store: &Store, id: Option<&String>) -> Result<()> {
    let id = id.ok_or_else(|| anyhow!("usage: show <id>"))?;
    let cl = store.read(id)?;
    println!("{}  ({}%)", cl.display_name(), cl.progress().percent());
    for (i, it) in cl.items.iter().enumerate() {
        println!("  {:>2} [{}] {}", i, if it.done { "x" } else { " " }, it.label);
    }
    Ok(())
}

fn cmd_toggle(store: &Store, id: Option<&String>, idx: Option<&String>) -> Result<()> {
    let id = id.ok_or_else(|| anyhow!("usage: toggle <id> <index>"))?;
    let idx: usize = idx
        .ok_or_else(|| anyhow!("usage: toggle <id> <index>"))?
        .parse()
        .map_err(|_| anyhow!("<index> must be a number"))?;
    let now = store.toggle_item(id, idx)?;
    println!("item {idx} -> {}", if now { "done" } else { "not done" });
    Ok(())
}

fn cmd_config(cfg: &Config) -> Result<()> {
    println!("config file:   {}", config_path().display());
    println!("checklist_dir: {}", cfg.checklist_dir.display());
    Ok(())
}

fn print_usage() {
    println!("{}", usage_text());
}

fn usage_text() -> &'static str {
    "checklists — shared JSON checklist store\n\
     \n\
     USAGE:\n\
     \u{20}  checklists [list]            list every checklist with progress\n\
     \u{20}  checklists show <id>         show one checklist's items\n\
     \u{20}  checklists toggle <id> <i>   toggle item <i> done/undone\n\
     \u{20}  checklists serve             stdio JSON-lines service (for the GUI)\n\
     \u{20}  checklists config            print the config file + store path\n\
     \u{20}  checklists init-config       write config.json with the current/default path\n"
}
