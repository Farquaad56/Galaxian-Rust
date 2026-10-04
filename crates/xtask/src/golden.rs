//! Golden MAME launcher (T0.5.2) — `cargo xtask golden run`.
//!
//! Builds and runs the common MAME profile from KB-28a §3, redirecting all
//! state into a fresh `tests/golden/tmp/run/` directory. Std-only.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Entry point for the `golden` subcommand. Returns the process exit code:
/// MAME's own code on success, or non-zero on a launcher error.
pub fn run(args: &[String]) -> i32 {
    let Some(sub) = args.first().map(String::as_str) else {
        eprintln!("usage: cargo xtask golden run [--dry-run] [-- <mame-args>...]");
        return 1;
    };
    if sub != "run" {
        eprintln!("unknown golden subcommand `{sub}` (expected `run`)");
        return 1;
    }

    let rest = &args[1..];
    let dry_run = rest.iter().any(|a| a == "--dry-run");
    // Extra MAME args: everything after the first `--`, minus any `--dry-run`.
    let extra: Vec<String> = match rest.iter().position(|a| a == "--") {
        Some(idx) => rest[idx + 1..]
            .iter()
            .filter(|a| *a != "--dry-run")
            .cloned()
            .collect(),
        None => Vec::new(),
    };

    let root = crate::workspace_root();
    let mame = mame_path(&root);
    if !mame.is_file() {
        eprintln!("MAME executable not found: {}", mame.display());
        return 1;
    }
    // The MAME process runs from the folder that contains mame.exe.
    let cwd = mame
        .parent()
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let run_dir = root.join("tests").join("golden").join("tmp").join("run");
    let args_list = build_args(&run_dir, &extra);

    if dry_run {
        print_command(&mame, &cwd, &args_list);
        return 0;
    }

    if let Err(e) = reset_run_dirs(&run_dir) {
        eprintln!("{e}");
        return 1;
    }

    match Command::new(&mame)
        .current_dir(&cwd)
        .args(&args_list)
        .status()
    {
        Ok(status) => status.code().unwrap_or(1),
        Err(e) => {
            eprintln!("failed to launch MAME: {e}");
            1
        }
    }
}

/// Path to mame.exe: `<root>/tools/mame.exe`, or `$MAME_DIR/mame.exe` if set.
fn mame_path(root: &Path) -> PathBuf {
    match std::env::var("MAME_DIR") {
        Ok(dir) if !dir.is_empty() => PathBuf::from(dir).join("mame.exe"),
        _ => root.join("tools").join("mame.exe"),
    }
}

/// The common profile from KB-28a §3, in order, with `$RUN` = `run_dir`.
fn common_profile(run_dir: &Path) -> Vec<String> {
    let dir = |name: &str| run_dir.join(name).to_string_lossy().into_owned();
    vec![
        "galaxian".to_string(),
        "-rompath".to_string(),
        "roms".to_string(),
        "-noreadconfig".to_string(),
        "-cfg_directory".to_string(),
        dir("cfg"),
        "-nvram_directory".to_string(),
        dir("nvram"),
        "-snapshot_directory".to_string(),
        dir("snap"),
        "-input_directory".to_string(),
        dir("inp"),
        "-state_directory".to_string(),
        dir("sta"),
        "-skip_gameinfo".to_string(),
        "-nothrottle".to_string(),
        "-frameskip".to_string(),
        "0".to_string(),
        "-noautoframeskip".to_string(),
        "-norewind".to_string(),
        "-noautosave".to_string(),
        "-autoboot_delay".to_string(),
        "0".to_string(),
    ]
}

/// Full argument list: common profile followed by the extra args, in order.
fn build_args(run_dir: &Path, extra: &[String]) -> Vec<String> {
    let mut args = common_profile(run_dir);
    args.extend(extra.iter().cloned());
    args
}

/// Delete then recreate `run_dir/{cfg,nvram,snap,inp,sta}` for a fresh run.
fn reset_run_dirs(run_dir: &Path) -> Result<(), String> {
    let _ = fs::remove_dir_all(run_dir); // best-effort; absent is fine
    for sub in ["cfg", "nvram", "snap", "inp", "sta"] {
        let path = run_dir.join(sub);
        fs::create_dir_all(&path)
            .map_err(|e| format!("failed to create {}: {e}", path.display()))?;
    }
    Ok(())
}

/// Print the exact command that would be executed (dry-run), without launching.
fn print_command(mame: &Path, cwd: &Path, args: &[String]) {
    println!("[dry-run] MAME will not be launched");
    println!("executable : {}", mame.display());
    println!("working dir: {}", cwd.display());
    println!("arguments  :");
    for arg in args {
        println!("    {arg}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_profile_matches_kb28a() {
        let run_dir = PathBuf::from("tests/golden/tmp/run");
        let s = |t: &str| t.to_string();
        let d = |name: &str| run_dir.join(name).to_string_lossy().into_owned();
        assert_eq!(
            common_profile(&run_dir),
            vec![
                s("galaxian"),
                s("-rompath"),
                s("roms"),
                s("-noreadconfig"),
                s("-cfg_directory"),
                d("cfg"),
                s("-nvram_directory"),
                d("nvram"),
                s("-snapshot_directory"),
                d("snap"),
                s("-input_directory"),
                d("inp"),
                s("-state_directory"),
                d("sta"),
                s("-skip_gameinfo"),
                s("-nothrottle"),
                s("-frameskip"),
                s("0"),
                s("-noautoframeskip"),
                s("-norewind"),
                s("-noautosave"),
                s("-autoboot_delay"),
                s("0"),
            ]
        );
    }

    #[test]
    fn extra_args_appended_in_order() {
        let run_dir = PathBuf::from("/repo/tests/golden/tmp/run");
        let profile = common_profile(&run_dir);
        let extra = vec!["-verifyroms".to_string(), "galaxian".to_string()];
        let args = build_args(&run_dir, &extra);
        assert_eq!(&args[..profile.len()], &profile[..]);
        assert_eq!(&args[profile.len()..], ["-verifyroms", "galaxian"]);
    }

    #[test]
    fn mame_path_honors_mame_dir_override() {
        let root = PathBuf::from("/repo");
        std::env::set_var("MAME_DIR", "/custom/dir");
        assert_eq!(
            mame_path(&root),
            PathBuf::from("/custom/dir").join("mame.exe")
        );
        std::env::remove_var("MAME_DIR");
        assert_eq!(mame_path(&root), root.join("tools").join("mame.exe"));
    }
}
