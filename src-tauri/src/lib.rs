use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use notify::{RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

const PALETTE: [&str; 6] = [
    "#ffe8a3", "#ffc9c9", "#c9e8ca", "#bfe0f2", "#e2cdf2", "#ffd6b0",
];
const DEFAULT_SIZE: f64 = 224.0;

#[derive(Serialize, Deserialize, Clone)]
struct Note {
    id: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: String,
    z: f64,
    #[serde(default)]
    text: String,
}

// Written to disk: pure layout metadata. The filename is the note's identity.
#[derive(Serialize)]
struct FrontMatter {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: String,
    z: f64,
}

// Read from disk: every field optional, so hand-written / partial notes still parse.
#[derive(Deserialize, Default)]
struct PartialFm {
    x: Option<f64>,
    y: Option<f64>,
    w: Option<f64>,
    h: Option<f64>,
    color: Option<String>,
    z: Option<f64>,
}

#[derive(Serialize)]
struct InitData {
    folder: String,
    notes: Vec<Note>,
}

#[derive(Clone, Serialize)]
struct RemovedPayload {
    id: String,
}

struct SyncState {
    folder: PathBuf,
    last: Mutex<HashMap<String, String>>,
    deleted: Mutex<HashSet<String>>,
    ztop: Mutex<f64>,
}

fn hash_str(s: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

fn hash_bytes(b: &[u8]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    b.hash(&mut h);
    h.finish()
}

fn sanitize_ext(ext: &str) -> String {
    let cleaned: String = ext
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(5)
        .collect::<String>()
        .to_ascii_lowercase();
    if cleaned.is_empty() {
        "png".to_string()
    } else {
        cleaned
    }
}

fn expand_tilde(p: &str) -> PathBuf {
    if p == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    if let Some(stripped) = p.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    PathBuf::from(p)
}

fn load_config_folder() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let cfg = home.join(".tableau.yml");
    let default_folder = home.join("Tableau");

    if let Ok(txt) = std::fs::read_to_string(&cfg) {
        if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(&txt) {
            if let Some(f) = val.get("folder").and_then(|v| v.as_str()) {
                let f = f.trim();
                if !f.is_empty() {
                    return expand_tilde(f);
                }
            }
        }
        return default_folder;
    }

    let template = format!(
        "# Tableau — where your notes live as markdown files.\n\
         # Point this at any folder. To read your notes in Obsidian on your phone,\n\
         # set it to a folder inside an iCloud Obsidian vault, e.g.:\n\
         # folder: ~/Library/Mobile Documents/iCloud~md~obsidian/Documents/MyVault/Tableau\n\
         #\n\
         # Every .md file in this folder becomes a postit. A note created by hand\n\
         # (no frontmatter, just text) shows up automatically; Tableau only writes\n\
         # layout frontmatter once you move or restyle it in the app.\n\
         folder: {}\n",
        default_folder.display()
    );
    let _ = std::fs::write(&cfg, template);
    default_folder
}

fn note_to_md(n: &Note) -> String {
    let fm = FrontMatter {
        x: n.x,
        y: n.y,
        w: n.w,
        h: n.h,
        color: n.color.clone(),
        z: n.z,
    };
    let yaml = serde_yaml::to_string(&fm).unwrap_or_default();
    format!("---\n{}---\n{}\n", yaml, n.text)
}

fn parse_md(content: &str) -> (PartialFm, String) {
    let stripped = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"));
    if let Some(rest) = stripped {
        if let Some(idx) = rest.find("\n---") {
            let fm_str = &rest[..idx];
            let body = rest[idx + 4..].trim_start_matches(['\r', '\n']);
            let fm = serde_yaml::from_str::<PartialFm>(fm_str).unwrap_or_default();
            return (fm, body.trim_end().to_string());
        }
    }
    // No (valid) frontmatter: the whole file is the note body.
    (PartialFm::default(), content.trim_end().to_string())
}

// Build a Note from a parsed file, filling any missing field with a stable
// default derived from the filename so hand-written notes get a consistent
// position/color without ever rewriting the file.
fn build_note(id: String, fm: PartialFm, body: String, state: &SyncState) -> Note {
    let h = hash_str(&id);
    let x = fm.x.unwrap_or((h % 900) as f64 - 450.0);
    let y = fm.y.unwrap_or(((h / 900) % 700) as f64 - 350.0);
    let w = fm.w.unwrap_or(DEFAULT_SIZE);
    let height = fm.h.unwrap_or(DEFAULT_SIZE);
    let color = fm
        .color
        .unwrap_or_else(|| PALETTE[(h % 6) as usize].to_string());

    let z = {
        let mut top = state.ztop.lock().unwrap();
        match fm.z {
            Some(z) => {
                if z > *top {
                    *top = z;
                }
                z
            }
            None => {
                *top += 1.0;
                *top
            }
        }
    };

    Note {
        id,
        x,
        y,
        w,
        h: height,
        color,
        z,
        text: body,
    }
}

fn is_md(p: &Path) -> bool {
    p.extension().and_then(|e| e.to_str()) == Some("md")
}

fn note_id(p: &Path) -> Option<String> {
    p.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
}

#[tauri::command]
fn init_board(state: State<Arc<SyncState>>) -> Result<InitData, String> {
    let mut notes = Vec::new();
    let mut last = state.last.lock().unwrap();
    if let Ok(rd) = std::fs::read_dir(&state.folder) {
        for entry in rd.flatten() {
            let p = entry.path();
            if !is_md(&p) {
                continue;
            }
            let id = match note_id(&p) {
                Some(id) => id,
                None => continue,
            };
            if let Ok(content) = std::fs::read_to_string(&p) {
                last.insert(p.to_string_lossy().to_string(), content.clone());
                let (fm, body) = parse_md(&content);
                notes.push(build_note(id, fm, body, &state));
            }
        }
    }
    Ok(InitData {
        folder: state.folder.to_string_lossy().to_string(),
        notes,
    })
}

#[tauri::command]
fn write_note(note: Note, state: State<Arc<SyncState>>) -> Result<(), String> {
    {
        let mut top = state.ztop.lock().unwrap();
        if note.z > *top {
            *top = note.z;
        }
    }
    let content = note_to_md(&note);
    let path = state.folder.join(format!("{}.md", note.id));
    state
        .last
        .lock()
        .unwrap()
        .insert(path.to_string_lossy().to_string(), content.clone());
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

// Store an image as a content-hashed file under <folder>/assets and hand back
// the relative path. Same bytes => same name, so paste-the-same-image is free.
// The note body just gets a portable `![](assets/…)` ref — Obsidian reads it too.
#[tauri::command]
fn save_image(data: Vec<u8>, ext: String, state: State<Arc<SyncState>>) -> Result<String, String> {
    let dir = state.folder.join("assets");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = format!("{:x}.{}", hash_bytes(&data), sanitize_ext(&ext));
    let path = dir.join(&name);
    if !path.exists() {
        std::fs::write(&path, &data).map_err(|e| e.to_string())?;
    }
    Ok(format!("assets/{}", name))
}

#[tauri::command]
fn delete_note(id: String, state: State<Arc<SyncState>>) -> Result<(), String> {
    state.deleted.lock().unwrap().insert(id.clone());
    let path = state.folder.join(format!("{}.md", id));
    let _ = std::fs::remove_file(&path);
    Ok(())
}

fn handle_fs_event(handle: &AppHandle, state: &Arc<SyncState>, paths: Vec<PathBuf>) {
    for path in paths {
        if !is_md(&path) {
            continue;
        }
        let key = path.to_string_lossy().to_string();
        let id = match note_id(&path) {
            Some(id) => id,
            None => continue,
        };

        if path.exists() {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            {
                let last = state.last.lock().unwrap();
                if last.get(&key).map(|c| c == &content).unwrap_or(false) {
                    continue; // echo of our own write
                }
            }
            state.last.lock().unwrap().insert(key.clone(), content.clone());
            let (fm, body) = parse_md(&content);
            let note = build_note(id, fm, body, state);
            let _ = handle.emit("note-changed", note);
        } else {
            if state.deleted.lock().unwrap().remove(&id) {
                continue; // our own delete
            }
            state.last.lock().unwrap().remove(&key);
            let _ = handle.emit("note-removed", RemovedPayload { id });
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            init_board,
            write_note,
            delete_note,
            save_image
        ])
        .setup(|app| {
            let folder = load_config_folder();
            std::fs::create_dir_all(&folder).ok();

            let state = Arc::new(SyncState {
                folder: folder.clone(),
                last: Mutex::new(HashMap::new()),
                deleted: Mutex::new(HashSet::new()),
                ztop: Mutex::new(1.0),
            });
            app.manage(state.clone());

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
                let mut watcher = match notify::recommended_watcher(move |res| {
                    let _ = tx.send(res);
                }) {
                    Ok(w) => w,
                    Err(_) => return,
                };
                if watcher.watch(&folder, RecursiveMode::NonRecursive).is_err() {
                    return;
                }
                for res in rx {
                    if let Ok(event) = res {
                        handle_fs_event(&handle, &state, event.paths);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
