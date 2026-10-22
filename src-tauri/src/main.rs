// Self-contained offline Photopea (personal use). The exe serves the embedded copy of www/ on 127.0.0.1:8887 — the
// port the editor requires off photopea.com — with a Content-Security-Policy that allows no other host, and shows it
// in a window that cannot navigate anywhere else. The Text tool uses the fonts installed on the computer.
#![windows_subsystem = "windows"]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

const PORT: u16 = 8887;

const CSP: &str = "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' blob:; style-src 'self' 'unsafe-inline' blob: data:; img-src 'self' data: blob:; font-src 'self' data: blob:; media-src 'self' data: blob:; connect-src 'self' data: blob:; worker-src 'self' blob: data:; frame-src 'self' blob: data:; child-src 'self' blob: data:; object-src 'none'; base-uri 'self'; form-action 'self'; manifest-src 'none'";

fn content_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "png" => "image/png",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        "svg" => "image/svg+xml",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

/// Percent-decoding of the request path (like decodeURIComponent; None on a malformed escape)
fn decode(p: &str) -> Option<String> {
    let b = p.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let h = std::str::from_utf8(b.get(i + 1..i + 3)?).ok()?;
            out.push(u8::from_str_radix(h, 16).ok()?);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// The fonts installed on this computer, as Photopea's font list. Served as `/sysfonts/list.js` (sets FNTS.list and
/// FNTS.sysfallback, loaded right after Photopea's database) and `/sysfonts/<n>` (the font files themselves).
struct SysFonts {
    files: Vec<PathBuf>,
    list_js: String,
}

static SYS_FONTS: std::sync::OnceLock<SysFonts> = std::sync::OnceLock::new();

fn sys_fonts() -> &'static SysFonts {
    SYS_FONTS.get_or_init(scan_fonts)
}

/// The system's font folders
fn font_folders() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = vec![];
    let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
    #[cfg(windows)]
    {
        v.extend(env("WINDIR").map(|w| w.join("Fonts")));
        v.extend(env("LOCALAPPDATA").map(|l| l.join("Microsoft").join("Windows").join("Fonts")));
    }
    #[cfg(target_os = "macos")]
    {
        v.push("/System/Library/Fonts".into());
        v.push("/Library/Fonts".into());
        v.extend(env("HOME").map(|h| h.join("Library/Fonts")));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        v.push("/usr/share/fonts".into());
        v.push("/usr/local/share/fonts".into());
        v.extend(env("XDG_DATA_HOME").map(|x| x.join("fonts")));
        v.extend(env("HOME").map(|h| h.join(".local/share/fonts")));
        v.extend(env("HOME").map(|h| h.join(".fonts")));
    }
    v
}

fn font_files(dir: &std::path::Path, out: &mut Vec<PathBuf>, depth: u32) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if depth < 8 {
                font_files(&p, out, depth + 1);
            }
        } else if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
            if matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc" | "otc") {
                out.push(p);
            }
        }
    }
}

fn read_at(f: &mut std::fs::File, off: u64, len: usize) -> Option<Vec<u8>> {
    use std::io::Seek;
    f.seek(std::io::SeekFrom::Start(off)).ok()?;
    let mut b = vec![0; len];
    f.read_exact(&mut b).ok()?;
    Some(b)
}

fn u16_at(b: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_be_bytes(b.get(o..o + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(o..o + 4)?.try_into().ok()?))
}

/// (family, style, PostScript name) of every font in a font file (several in a .ttc collection)
fn font_names(path: &std::path::Path) -> Vec<(String, String, String)> {
    let Ok(mut f) = std::fs::File::open(path) else { return vec![] };
    let Some(head) = read_at(&mut f, 0, 12) else { return vec![] };
    let offsets: Vec<u64> = if &head[0..4] == b"ttcf" {
        let n = u32_at(&head, 8).unwrap_or(0).min(256) as usize;
        read_at(&mut f, 12, 4 * n).map_or(vec![], |t| (0..n).filter_map(|i| u32_at(&t, 4 * i)).map(u64::from).collect())
    } else {
        vec![0]
    };
    offsets.into_iter().filter_map(|o| face_names(&mut f, o)).collect()
}

/// Names from one font's `name` table: typographic family/style (IDs 16/17) when present, else 1/2, and the
/// PostScript name (6). Prefers Windows-platform English records, then Unicode, then Mac Roman.
fn face_names(f: &mut std::fs::File, off: u64) -> Option<(String, String, String)> {
    let head = read_at(f, off, 12)?;
    let n = u16_at(&head, 4)? as usize;
    let dir = read_at(f, off + 12, 16 * n)?;
    let rec = dir.chunks_exact(16).find(|r| &r[0..4] == b"name")?;
    let t = read_at(f, u32_at(rec, 8)? as u64, (u32_at(rec, 12)? as usize).min(1 << 20))?;
    let (count, strings) = (u16_at(&t, 2)? as usize, u16_at(&t, 4)? as usize);
    let utf16 = |b: &[u8]| String::from_utf16_lossy(&b.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect::<Vec<_>>());
    let mut best: [(u8, String); 18] = Default::default();
    for i in 0..count {
        let r = 6 + 12 * i;
        let (Some(pid), Some(eid), Some(lang), Some(id), Some(len), Some(at)) =
            (u16_at(&t, r), u16_at(&t, r + 2), u16_at(&t, r + 4), u16_at(&t, r + 6), u16_at(&t, r + 8), u16_at(&t, r + 10))
        else {
            break;
        };
        let id = id as usize;
        if !matches!(id, 1 | 2 | 6 | 16 | 17) {
            continue;
        }
        let Some(raw) = t.get(strings + at as usize..strings + at as usize + len as usize) else { continue };
        let (score, text) = match (pid, eid) {
            (3, 1) | (3, 10) => (if lang == 0x409 { 5 } else { 4 }, utf16(raw)),
            (0, _) => (3, utf16(raw)),
            (1, 0) => (if lang == 0 { 2 } else { 1 }, raw.iter().map(|&c| c as char).collect()),
            _ => continue,
        };
        let text = text.trim().replace(',', " ");
        if !text.is_empty() && score > best[id].0 {
            best[id] = (score, text);
        }
    }
    let get = |id: usize| Some(best[id].1.clone()).filter(|s| !s.is_empty());
    let family = get(16).or_else(|| get(1))?;
    let style = get(17).or_else(|| get(2)).unwrap_or_else(|| "Regular".into());
    let ps = get(6).unwrap_or_else(|| format!("{family}-{style}").replace(' ', ""));
    Some((family, style, ps))
}

/// Category in Photopea's font menu filter (FNTS.cats), guessed from the family name
fn font_category(family: &str) -> u32 {
    let f = family.to_ascii_lowercase();
    let has = |keys: &[&str]| keys.iter().any(|k| f.contains(k));
    if has(&["mono", "code", "courier", "consol", "terminal", "fixed"]) {
        7 // Monospaced
    } else if has(&["symbol", "wingding", "webding", "dingbat", "emoji", "marlett"]) {
        3 // Dingbat
    } else if has(&["script", "brush", "hand", "callig", "print", "mistral", "vivaldi"]) {
        13 // Script
    } else if !f.contains("sans") && has(&["serif", "times", "georgia", "garamond", "cambria", "antiqua", "palatino", "century", "baskerville", "bodoni", "didot", "constantia"]) {
        14 // Serif
    } else {
        12 // Sans Serif
    }
}

/// Fonts used when a document asks for one that isn't installed (Photopea's own choice was DejaVu Sans/Serif, which
/// came with its font library): sans regular/bold/italic/bold italic, then serif. The first is also the default font.
fn fallback_fonts(have: &std::collections::HashSet<String>, first: &str) -> Vec<String> {
    let sans: &[[&str; 4]] = &[
        ["ArialMT", "Arial-BoldMT", "Arial-ItalicMT", "Arial-BoldItalicMT"],
        ["Helvetica", "Helvetica-Bold", "Helvetica-Oblique", "Helvetica-BoldOblique"],
        ["DejaVuSans", "DejaVuSans-Bold", "DejaVuSans-Oblique", "DejaVuSans-BoldOblique"],
        ["LiberationSans", "LiberationSans-Bold", "LiberationSans-Italic", "LiberationSans-BoldItalic"],
        ["NotoSans-Regular", "NotoSans-Bold", "NotoSans-Italic", "NotoSans-BoldItalic"],
    ];
    let serif: &[[&str; 4]] = &[
        ["TimesNewRomanPSMT", "TimesNewRomanPS-BoldMT", "TimesNewRomanPS-ItalicMT", "TimesNewRomanPS-BoldItalicMT"],
        ["Times-Roman", "Times-Bold", "Times-Italic", "Times-BoldItalic"],
        ["DejaVuSerif", "DejaVuSerif-Bold", "DejaVuSerif-Italic", "DejaVuSerif-BoldItalic"],
        ["LiberationSerif", "LiberationSerif-Bold", "LiberationSerif-Italic", "LiberationSerif-BoldItalic"],
        ["NotoSerif-Regular", "NotoSerif-Bold", "NotoSerif-Italic", "NotoSerif-BoldItalic"],
    ];
    let pick = |sets: &[[&str; 4]]| -> Option<Vec<String>> {
        let set = sets.iter().find(|s| have.contains(s[0]))?;
        Some(set.iter().map(|n| if have.contains(*n) { n } else { set[0] }.to_string()).collect())
    };
    let s = pick(sans).unwrap_or_else(|| vec![first.to_string(); 4]);
    let r = pick(serif).unwrap_or_else(|| s.clone());
    s.into_iter().chain(r).collect()
}

fn scan_fonts() -> SysFonts {
    let mut paths = vec![];
    for d in font_folders() {
        font_files(&d, &mut paths, 0);
    }
    let mut files = vec![];
    let mut faces = vec![]; // (family, style, ps, file index)
    let mut seen = std::collections::HashSet::new();
    for p in paths {
        let names = font_names(&p);
        if names.is_empty() {
            continue;
        }
        for (family, style, ps) in names {
            if seen.insert(ps.clone()) {
                faces.push((family, style, ps, files.len()));
            }
        }
        files.push(p);
    }
    faces.sort_by(|a, b| (a.0.to_lowercase(), &a.1).cmp(&(b.0.to_lowercase(), &b.1)));
    // every subset bit set: the menu's script filter then never hides a system font
    let list: Vec<String> = faces
        .iter()
        .map(|(family, style, ps, file)| format!("{:?}", format!("{family},{style},{ps},131071,{},sysfonts/{file}", font_category(family))))
        .collect();
    let first = faces.first().map_or("DejaVuSans", |f| f.2.as_str());
    let fallback: Vec<String> = fallback_fonts(&seen, first).iter().map(|n| format!("{n:?}")).collect();
    let list_js = format!("FNTS.list=[{}];FNTS.sysfallback=[{}];FNTS.map=FNTS.mapFS=FNTS.mapSub=null;\n", list.join(","), fallback.join(","));
    SysFonts { files, list_js }
}

fn lookup(path: &str) -> Option<Vec<u8>> {
    if let Some(rest) = path.strip_prefix("/sysfonts/") {
        let fonts = sys_fonts();
        if rest == "list.js" {
            return Some(fonts.list_js.clone().into_bytes());
        }
        return std::fs::read(fonts.files.get(rest.parse::<usize>().ok()?)?).ok();
    }
    ASSETS.iter().find(|(p, _)| *p == path).map(|(_, b)| b.to_vec())
}

fn respond(s: &mut TcpStream, status: &str, ctype: &str, body: &[u8], head_only: bool) -> std::io::Result<()> {
    let hdr = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nContent-Security-Policy: {CSP}\r\nReferrer-Policy: no-referrer\r\nX-DNS-Prefetch-Control: off\r\nX-Content-Type-Options: nosniff\r\nConnection: keep-alive\r\n\r\n",
        body.len()
    );
    s.write_all(hdr.as_bytes())?;
    if !head_only {
        s.write_all(body)?;
    }
    s.flush()
}

fn serve(mut s: TcpStream) {
    let Ok(rd) = s.try_clone() else { return };
    let mut r = BufReader::new(rd);
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
        let mut len = 0usize;
        loop {
            let mut h = String::new();
            if r.read_line(&mut h).unwrap_or(0) == 0 {
                return;
            }
            let h = h.trim_end();
            if h.is_empty() {
                break;
            }
            if let Some((k, v)) = h.split_once(':') {
                if k.eq_ignore_ascii_case("content-length") {
                    len = v.trim().parse().unwrap_or(0);
                }
            }
        }
        if len > 0 {
            let mut skip = vec![0u8; len];
            if r.read_exact(&mut skip).is_err() {
                return;
            }
        }
        let mut parts = line.split_whitespace();
        let method = parts.next().unwrap_or("");
        let target = parts.next().unwrap_or("/");
        let res = if method != "GET" && method != "HEAD" {
            respond(&mut s, "405 Method Not Allowed", "text/plain", b"405", false)
        } else {
            match decode(target.split('?').next().unwrap_or("/")) {
                None => respond(&mut s, "400 Bad Request", "text/plain", b"400: bad URL", false),
                Some(mut p) => {
                    if p == "/" {
                        p = "/index.html".into();
                    }
                    match lookup(&p) {
                        Some(b) => respond(&mut s, "200 OK", content_type(&p), &b, method == "HEAD"),
                        None => respond(&mut s, "404 Not Found", "text/plain", format!("404: {p}").as_bytes(), false),
                    }
                }
            }
        };
        if res.is_err() {
            return;
        }
    }
}

fn start_server() {
    // read the installed fonts' names while the window starts up
    std::thread::spawn(|| {
        sys_fonts();
    });
    // if the port is taken (another copy of the app is running), that copy's server is used
    if let Ok(l) = TcpListener::bind(("127.0.0.1", PORT)) {
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                std::thread::spawn(move || serve(s));
            }
        });
    }
}

/// Offline lock: windows never leave the local server (in-page data:/blob: URLs are fine)
fn local_only(u: &tauri::Url) -> bool {
    matches!(u.scheme(), "data" | "blob" | "about")
        || (u.scheme() == "http" && u.host_str() == Some("127.0.0.1") && u.port() == Some(PORT))
}

fn main() {
    start_server();
    tauri::Builder::default()
        .setup(|app| {
            let url: tauri::Url = format!("http://127.0.0.1:{PORT}/#").parse().unwrap();
            let handle = app.handle().clone();
            let mut main = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
                .title("Photopea")
                .inner_size(1400.0, 900.0)
                .maximized(true)
                .on_navigation(local_only)
                // dropped files go to the page (Photopea opens them), not to Tauri
                .disable_drag_drop_handler()
                // reading the clipboard (Edit > Paste, right-click Paste) is always allowed: no "wants to see text
                // and images copied to the clipboard" prompt (Windows: auto-answers WebView2's request; Linux:
                // WebKitGTK clipboard access on; macOS: always allowed)
                .enable_clipboard_access()
                // File > Print writes the image into a blank popup and prints it; no other popups
                .on_new_window(move |u, features| {
                    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                    if u.as_str() != "about:blank" {
                        return tauri::webview::NewWindowResponse::Deny;
                    }
                    let label = format!("popup{}", N.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
                    match tauri::WebviewWindowBuilder::new(&handle, label, tauri::WebviewUrl::External(u))
                        .window_features(features)
                        .title("Photopea")
                        .on_navigation(local_only)
                        .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
                        .build()
                    {
                        Ok(window) => tauri::webview::NewWindowResponse::Create { window },
                        Err(_) => tauri::webview::NewWindowResponse::Deny,
                    }
                });
            // Windows: as in a browser, a touchpad pinch reaches the page as Ctrl+wheel and Photopea zooms the canvas.
            // Not on macOS/Linux: there this option injects a whole-page zoom on Ctrl/Cmd+wheel, and their WebKit
            // already sends Photopea the pinch gestures it handles itself.
            #[cfg(windows)]
            {
                main = main.zoom_hotkeys_enabled(true);
            }
            // macOS/Linux: their webviews only save a download the app accepts (WebView2 on Windows saves by itself).
            // Accept the files Photopea generates (blob:/data:) into the Downloads folder, nothing else.
            #[cfg(not(windows))]
            {
                main = main.on_download(|_, event| match event {
                    tauri::webview::DownloadEvent::Requested { url, .. } => matches!(url.scheme(), "blob" | "data"),
                    _ => true,
                });
            }
            // test runs (scripts/test-app.mjs) keep their window off the user's screen
            if std::env::var_os("PHOTOPEA_TEST_OFFSCREEN").is_some() {
                main = main.maximized(false).position(-4000.0, 0.0);
            }
            main.build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Photopea");
}
