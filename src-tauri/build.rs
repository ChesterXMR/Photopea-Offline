// Embeds Photopea (../www) into the exe.
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, PathBuf)>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        let rel = p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
        if p.is_dir() {
            walk(&p, root, out);
        } else {
            out.push((rel, p));
        }
    }
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../www").canonicalize().unwrap();
    let mut files = vec![];
    walk(&root, &root, &mut files);
    files.sort();
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("assets.rs");
    let mut f = fs::File::create(out).unwrap();
    writeln!(f, "pub static ASSETS: &[(&str, &[u8])] = &[").unwrap();
    for (rel, p) in &files {
        writeln!(f, "    ({:?}, include_bytes!({:?})),", format!("/{rel}"), p.to_string_lossy()).unwrap();
    }
    writeln!(f, "];").unwrap();
    println!("cargo:rerun-if-changed=../www");
    tauri_build::build();
}
