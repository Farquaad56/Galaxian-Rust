//! `cargo xtask` — workspace task runner (Partie C).
//!
//! Subcommands: `api-dump`, `run-zex`, `mame-diff`, `test-roms`, `golden`.
//! Only `api-dump` is functional for T0.1.1; the rest print a stub message.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

/// Public API dump of all workspace crates (T0.1.1).
fn api_dump() {
    let root = workspace_root();
    for member in workspace_members(&root) {
        let crate_dir = root.join(&member);
        let name = crate_dir
            .file_name()
            .map_or_else(|| member.clone(), |s| s.to_string_lossy().into_owned());
        println!("## crate: {name}");
        let lib = crate_dir.join("src/lib.rs");
        if !lib.is_file() {
            continue; // binary-only crates (e.g. xtask) have no public API
        }
        for line in read_lines(&lib) {
            let trimmed = line.trim();
            if is_public_signature(trimmed) {
                println!("{trimmed}");
            }
        }
    }
}

/// A line that declares a public item of this crate (not a `pub use` re-export).
fn is_public_signature(line: &str) -> bool {
    let Some(pos) = line.find("pub ") else {
        return false;
    };
    let rest = &line[pos + "pub ".len()..];
    if rest.starts_with("use ") || rest == "use" {
        return false; // `pub use ...` re-export, not own API
    }
    matches!(
        rest.split_whitespace().next(),
        Some("fn" | "struct" | "enum" | "trait" | "type" | "mod" | "const" | "static")
    )
}

/// Workspace root: the ancestor of this binary that contains `crates/xtask`.
fn workspace_root() -> PathBuf {
    let exe = std::env::current_exe().expect("no current executable");
    for dir in exe.ancestors() {
        if dir.join("crates/xtask/Cargo.toml").is_file() {
            return dir.to_path_buf();
        }
    }
    panic!("could not locate workspace root from {}", exe.display());
}

/// Member paths listed under `members = [ ... ]` in the root Cargo.toml.
fn workspace_members(root: &Path) -> Vec<String> {
    let toml = read_lines(&root.join("Cargo.toml"));
    let Some(start_idx) = toml
        .iter()
        .position(|line| line.trim_start().starts_with("members"))
    else {
        return Vec::new();
    };
    let mut members: Vec<String> = Vec::new();
    for line in toml.iter().skip(start_idx + 1) {
        let trimmed = line.trim();
        if trimmed.starts_with(']') {
            break;
        }
        for part in trimmed.split(',') {
            let token = part.trim();
            if let Some(q) = token.strip_prefix('"').and_then(|t| t.strip_suffix('"')) {
                members.push(q.to_owned());
            }
        }
    }
    members
}

fn read_lines(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .map_or_else(|_| Vec::new(), |s| s.lines().map(str::to_owned).collect())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("api-dump") => api_dump(),
        Some(name @ ("run-zex" | "mame-diff" | "test-roms" | "golden")) => {
            println!("xtask {name}: not implemented yet");
        }
        _ => {
            eprintln!("usage: cargo xtask <api-dump|run-zex|mame-diff|test-roms|golden>");
            std::process::exit(1);
        }
    }
}
