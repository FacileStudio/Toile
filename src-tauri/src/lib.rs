use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use url::Url;

const PREVIEW_UA: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 \
     (KHTML, like Gecko) Toile-LinkPreview/1.0";
const MAX_HTML: usize = 1_500_000;
const MAX_IMG: usize = 8 * 1024 * 1024;

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
    #[serde(default)]
    font: Option<String>,
}

#[derive(Serialize)]
struct FrontMatter {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: String,
    z: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    font: Option<String>,
}

#[derive(Deserialize, Default)]
struct PartialFm {
    x: Option<f64>,
    y: Option<f64>,
    w: Option<f64>,
    h: Option<f64>,
    color: Option<String>,
    z: Option<f64>,
    font: Option<String>,
}

#[derive(Serialize)]
struct InitData {
    folder: String,
    notes: Vec<Note>,
    unfurl: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct LinkMeta {
    url: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    site_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    favicon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video_id: Option<String>,
}

#[derive(Clone, Serialize)]
struct RemovedPayload {
    id: String,
}

struct SyncState {
    folder: PathBuf,
    unfurl: bool,
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
        "bin".to_string()
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

fn folder_from_config(path: &Path) -> Option<PathBuf> {
    let txt = std::fs::read_to_string(path).ok()?;
    let val = serde_yaml::from_str::<serde_yaml::Value>(&txt).ok()?;
    let f = val.get("folder").and_then(|v| v.as_str())?.trim().to_string();
    if f.is_empty() {
        None
    } else {
        Some(expand_tilde(&f))
    }
}

fn load_config_folder() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let cfg = home.join(".toile.yml");
    let default_folder = home.join("Toile");

    if cfg.exists() {
        return folder_from_config(&cfg).unwrap_or(default_folder);
    }

    let template = format!(
        "# Toile — where your notes live as markdown files.\n\
         # Point this at any folder. To read your notes in Obsidian on your phone,\n\
         # set it to a folder inside an iCloud Obsidian vault, e.g.:\n\
         # folder: ~/Library/Mobile Documents/iCloud~md~obsidian/Documents/MyVault/Toile\n\
         #\n\
         # Every .md file in this folder becomes a postit. A note created by hand\n\
         # (no frontmatter, just text) shows up automatically; Toile only writes\n\
         # layout frontmatter once you move or restyle it in the app.\n\
         folder: {}\n",
        default_folder.display()
    );
    let _ = std::fs::write(&cfg, template);
    default_folder
}

// Optional privacy switch in ~/.toile.yml (`unfurl: false`) to stop link previews
// from reaching out to the network. Defaults on.
fn config_unfurl() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return true,
    };
    std::fs::read_to_string(home.join(".toile.yml"))
        .ok()
        .and_then(|t| serde_yaml::from_str::<serde_yaml::Value>(&t).ok())
        .and_then(|v| v.get("unfurl").and_then(|x| x.as_bool()))
        .unwrap_or(true)
}

fn note_to_md(n: &Note) -> String {
    let fm = FrontMatter {
        x: n.x,
        y: n.y,
        w: n.w,
        h: n.h,
        color: n.color.clone(),
        z: n.z,
        font: n.font.clone(),
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
    (PartialFm::default(), content.trim_end().to_string())
}

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
        font: fm.font,
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
        unfurl: state.unfurl,
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

fn store_bytes(folder: &Path, data: &[u8], ext: &str) -> Result<String, String> {
    let dir = folder.join("assets");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = format!("{:x}.{}", hash_bytes(data), sanitize_ext(ext));
    let path = dir.join(&name);
    if !path.exists() {
        std::fs::write(&path, data).map_err(|e| e.to_string())?;
    }
    Ok(format!("assets/{}", name))
}

#[tauri::command]
fn save_asset(data: Vec<u8>, ext: String, state: State<Arc<SyncState>>) -> Result<String, String> {
    store_bytes(&state.folder, &data, &ext)
}

#[tauri::command]
fn delete_note(id: String, state: State<Arc<SyncState>>) -> Result<(), String> {
    state.deleted.lock().unwrap().insert(id.clone());
    let path = state.folder.join(format!("{}.md", id));
    let _ = std::fs::remove_file(&path);
    Ok(())
}

#[tauri::command]
fn load_strokes(state: State<Arc<SyncState>>) -> Result<String, String> {
    let path = state.folder.join(".toile-drawing.json");
    Ok(std::fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string()))
}

#[tauri::command]
fn save_strokes(data: String, state: State<Arc<SyncState>>) -> Result<(), String> {
    let path = state.folder.join(".toile-drawing.json");
    std::fs::write(&path, data).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_links(state: State<Arc<SyncState>>) -> Result<String, String> {
    let path = state.folder.join(".toile-links.json");
    Ok(std::fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string()))
}

#[tauri::command]
fn save_links(data: String, state: State<Arc<SyncState>>) -> Result<(), String> {
    let path = state.folder.join(".toile-links.json");
    std::fs::write(&path, data).map_err(|e| e.to_string())
}

// ---- link previews: the Tauri core is the (local) unfurl crawler. It fetches
// server-side (no webview CORS) and caches preview images to disk so cards work
// offline. Every hop is SSRF-validated before we connect. ----

fn ip_blocked(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            v.is_loopback()
                || v.is_private()
                || v.is_link_local()
                || v.is_unspecified()
                || v.is_broadcast()
                || v.is_documentation()
                || v.is_multicast()
        }
        IpAddr::V6(v) => {
            v.is_loopback()
                || v.is_unspecified()
                || v.is_multicast()
                || (v.segments()[0] & 0xffc0) == 0xfe80 // link-local
                || (v.segments()[0] & 0xfe00) == 0xfc00 // unique local
        }
    }
}

// Resolve the host and reject if it maps to a loopback/private/link-local address
// (the cloud-metadata endpoint 169.254.169.254 is link-local, so it's covered).
async fn validate(url_str: &str) -> Result<Url, String> {
    let u = Url::parse(url_str).map_err(|e| e.to_string())?;
    if !matches!(u.scheme(), "http" | "https") {
        return Err("unsupported url scheme".into());
    }
    let host = u.host_str().ok_or("missing host")?.to_string();
    let port = u
        .port_or_known_default()
        .unwrap_or(if u.scheme() == "https" { 443 } else { 80 });
    let ips: Vec<IpAddr> = tokio::task::spawn_blocking(move || {
        (host.as_str(), port)
            .to_socket_addrs()
            .map(|it| it.map(|s| s.ip()).collect::<Vec<_>>())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|_| "could not resolve host".to_string())?;
    if ips.is_empty() {
        return Err("host resolved to nothing".into());
    }
    if ips.iter().any(ip_blocked) {
        return Err("address not allowed".into());
    }
    Ok(u)
}

fn http_client() -> Result<Client, String> {
    Client::builder()
        .user_agent(PREVIEW_UA)
        .timeout(Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())
}

// Manual redirect following so every hop is re-validated (an open redirect to
// 127.0.0.1 is the classic SSRF bypass).
async fn get_validated(client: &Client, start: &str, max: u32) -> Result<reqwest::Response, String> {
    let mut current = start.to_string();
    for _ in 0..=max {
        let u = validate(&current).await?;
        let resp = client.get(u.clone()).send().await.map_err(|e| e.to_string())?;
        if resp.status().is_redirection() {
            if let Some(loc) = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
            {
                current = u.join(loc).map_err(|e| e.to_string())?.to_string();
                continue;
            }
        }
        return Ok(resp);
    }
    Err("too many redirects".into())
}

fn ext_from_ct(ct: &str) -> Option<String> {
    let ct = ct.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    let e = match ct.as_str() {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "image/avif" => "avif",
        "image/bmp" => "bmp",
        "image/x-icon" | "image/vnd.microsoft.icon" => "ico",
        _ => return None,
    };
    Some(e.to_string())
}

fn ext_from_url(u: &str) -> Option<String> {
    let path = Url::parse(u).ok()?.path().to_string();
    let name = path.rsplit('/').next()?;
    let ext = name.rsplit_once('.')?.1;
    if ext.is_empty() || ext.len() > 5 {
        None
    } else {
        Some(ext.to_string())
    }
}

async fn download_asset(client: &Client, folder: &Path, url: &str) -> Option<String> {
    let resp = get_validated(client, url, 4).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let ext = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(ext_from_ct)
        .or_else(|| ext_from_url(url))
        .unwrap_or_else(|| "img".to_string());
    let bytes = resp.bytes().await.ok()?;
    if bytes.is_empty() || bytes.len() > MAX_IMG {
        return None;
    }
    store_bytes(folder, &bytes, &ext).ok()
}

struct Parsed {
    title: Option<String>,
    description: Option<String>,
    image: Option<String>,
    site_name: Option<String>,
    accent: Option<String>,
    favicon: Option<String>,
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push('…');
        out
    }
}

fn parse_html(html: &str, base: &Url) -> Parsed {
    let doc = Html::parse_document(html);

    let mut props: HashMap<String, String> = HashMap::new();
    if let Ok(sel) = Selector::parse("meta") {
        for el in doc.select(&sel) {
            let v = el.value();
            let key = v.attr("property").or_else(|| v.attr("name"));
            if let (Some(k), Some(c)) = (key, v.attr("content")) {
                let k = k.trim().to_lowercase();
                let c = c.trim();
                if !k.is_empty() && !c.is_empty() {
                    props.entry(k).or_insert_with(|| c.to_string());
                }
            }
        }
    }

    let pick = |keys: &[&str]| keys.iter().find_map(|k| props.get(*k).cloned());

    let title = pick(&["og:title", "twitter:title"]).or_else(|| {
        Selector::parse("title")
            .ok()
            .and_then(|s| doc.select(&s).next())
            .map(|t| t.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    });
    let description = pick(&["og:description", "twitter:description", "description"])
        .map(|s| truncate(&s, 300));
    let image = pick(&[
        "og:image:secure_url",
        "og:image:url",
        "og:image",
        "twitter:image",
        "twitter:image:src",
    ])
    .and_then(|s| base.join(s.trim()).ok().map(|u| u.to_string()));
    let site_name = pick(&["og:site_name"]);
    let accent = pick(&["theme-color"]);

    let mut favicon = None;
    for q in [
        "link[rel~=\"icon\"]",
        "link[rel=\"shortcut icon\"]",
        "link[rel=\"apple-touch-icon\"]",
    ] {
        if let Ok(sel) = Selector::parse(q) {
            if let Some(href) = doc.select(&sel).next().and_then(|e| e.value().attr("href")) {
                if let Ok(u) = base.join(href.trim()) {
                    favicon = Some(u.to_string());
                    break;
                }
            }
        }
    }
    if favicon.is_none() {
        favicon = base.join("/favicon.ico").ok().map(|u| u.to_string());
    }

    Parsed {
        title,
        description,
        image,
        site_name,
        accent,
        favicon,
    }
}

fn youtube_id(u: &Url) -> Option<String> {
    let host = u.host_str()?.trim_start_matches("www.").to_lowercase();
    let yt = matches!(
        host.as_str(),
        "youtube.com" | "m.youtube.com" | "music.youtube.com" | "youtube-nocookie.com" | "youtu.be"
    );
    if !yt {
        return None;
    }
    let id = if host == "youtu.be" {
        u.path_segments()?.next()?.to_string()
    } else if let Some(v) = u.query_pairs().find(|(k, _)| k == "v").map(|(_, v)| v.to_string()) {
        v
    } else {
        let mut segs = u.path_segments()?;
        let first = segs.next()?;
        if matches!(first, "shorts" | "embed" | "v" | "live") {
            segs.next()?.to_string()
        } else {
            return None;
        }
    };
    if id.len() == 11 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        Some(id)
    } else {
        None
    }
}

async fn youtube_title(client: &Client, page_url: &str) -> Option<String> {
    let mut endpoint = Url::parse("https://www.youtube.com/oembed").ok()?;
    endpoint
        .query_pairs_mut()
        .append_pair("url", page_url)
        .append_pair("format", "json");
    let resp = get_validated(client, endpoint.as_str(), 3).await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().await.ok()?;
    v.get("title")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
}

#[tauri::command]
async fn fetch_link_preview(
    url: String,
    state: State<'_, Arc<SyncState>>,
) -> Result<LinkMeta, String> {
    let folder = state.folder.clone();
    let client = http_client()?;
    let parsed_url = validate(&url).await?;

    if let Some(vid) = youtube_id(&parsed_url) {
        let title = youtube_title(&client, &url).await;
        return Ok(LinkMeta {
            url,
            kind: "youtube".into(),
            title,
            description: None,
            site_name: Some("YouTube".into()),
            image: None,
            favicon: None,
            accent: None,
            video_id: Some(vid),
        });
    }

    let resp = get_validated(&client, &url, 5).await?;
    if !resp.status().is_success() {
        return Err(format!("server returned {}", resp.status()));
    }
    let final_url = resp.url().clone();
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let slice = &bytes[..bytes.len().min(MAX_HTML)];
    let parsed = parse_html(&String::from_utf8_lossy(slice), &final_url);

    let image = match parsed.image {
        Some(ref i) => download_asset(&client, &folder, i).await,
        None => None,
    };
    let favicon = match parsed.favicon {
        Some(ref i) => download_asset(&client, &folder, i).await,
        None => None,
    };

    Ok(LinkMeta {
        url,
        kind: "card".into(),
        title: parsed.title,
        description: parsed.description,
        site_name: parsed.site_name,
        image,
        favicon,
        accent: parsed.accent,
        video_id: None,
    })
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
                    continue;
                }
            }
            state.last.lock().unwrap().insert(key.clone(), content.clone());
            let (fm, body) = parse_md(&content);
            let note = build_note(id, fm, body, state);
            let _ = handle.emit("note-changed", note);
        } else {
            if state.deleted.lock().unwrap().remove(&id) {
                continue;
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
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            init_board,
            write_note,
            delete_note,
            load_strokes,
            save_strokes,
            save_asset,
            load_links,
            save_links,
            fetch_link_preview
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.with_webview(|webview| unsafe {
                    use objc2_web_kit::WKWebView;
                    let wk = &*(webview.inner() as *mut WKWebView);
                    wk.configuration().preferences().setMinimumFontSize(0.0);
                });
            }

            let folder = load_config_folder();
            std::fs::create_dir_all(&folder).ok();

            let state = Arc::new(SyncState {
                folder: folder.clone(),
                unfurl: config_unfurl(),
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
