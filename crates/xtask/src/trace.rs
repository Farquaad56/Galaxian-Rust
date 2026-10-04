//! Normalized CPU trace (T0.5.4) — `cargo xtask golden trace`.
//!
//! Reads the raw MAME debugger trace at `tests/golden/tmp/trace_raw.log`
//! (produced by T0.5.3's `tests/golden/mame/trace.dbg`) and writes three
//! versioned goldens, each truncated to N lines:
//! `tests/golden/trace_boot_{1000,100000,1000000}.log`.
//!
//! Normalized line format (fixed, deterministic):
//!
//! ```text
//! <PC> <OP> AF=<af> BC=<bc> DE=<de> HL=<hl> SP=<sp> CYC=<lic>
//! ```
//!
//! `<PC>` is the 4-hex-digit program counter from the raw `NNNN:` field,
//! `<OP>` the verbatim 8-hex-digit opcode bytes at PC, each register pair
//! its 4-hex value, and `CYC` the decimal LIC (`lastinstructioncycles`,
//! T-states of this instruction per MAME's own counter — T0.5.1), not a
//! reconstruction from KB-21b. The disassemblage text is dropped; one
//! normalized line per raw line; no header line; file ends with a newline.

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Truncation sizes (lines) of the three versioned goldens.
const SIZES: [u64; 3] = [1_000, 100_000, 1_000_000];

/// Entry point for `cargo xtask golden trace`. Returns the process exit code.
pub fn run() -> i32 {
    let root = crate::workspace_root();
    let raw = root.join("tests/golden/tmp/trace_raw.log");
    let out_dir = root.join("tests/golden");
    match convert(&raw, &out_dir, &SIZES) {
        Ok(counts) => {
            for (n, count) in SIZES.iter().zip(counts) {
                println!("trace_boot_{n}.log: {count} lines");
            }
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

/// Convert `raw` into one output file per size, each truncated to that many
/// normalized lines. Returns the line count written per file; errors if the
/// raw trace is shorter than a requested size (a silently short golden).
fn convert(raw: &Path, out_dir: &Path, sizes: &[u64]) -> Result<Vec<u64>, String> {
    let file = File::open(raw)
        .map_err(|e| format!("cannot open {}: {e}", raw.display()))?;
    let mut reader = BufReader::new(file);
    let mut writers = Vec::with_capacity(sizes.len());
    for n in sizes {
        let path = out_dir.join(format!("trace_boot_{n}.log"));
        let w = File::create(&path)
            .map_err(|e| format!("cannot create {}: {e}", path.display()))?;
        writers.push(BufWriter::new(w));
    }

    let mut counts = vec![0u64; sizes.len()];
    let mut line = String::new();
    let last = *sizes.last().expect("at least one size");
    for i in 0..last {
        line.clear();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("cannot read {}: {e}", raw.display()))?;
        if line.is_empty() {
            break; // end of file before all sizes were reached
        }
        let norm = normalize_line(line.trim_end_matches(['\r', '\n']))?;
        for (k, w) in writers.iter_mut().enumerate() {
            if i < sizes[k] {
                writeln!(w, "{norm}")
                    .map_err(|e| format!("cannot write trace_boot_{}.log: {e}", sizes[k]))?;
                counts[k] += 1;
            }
        }
    }

    for (w, n) in writers.iter_mut().zip(sizes) {
        w.flush()
            .map_err(|e| format!("cannot flush trace_boot_{n}.log: {e}"))?;
    }
    if counts.iter().zip(sizes).any(|(&c, &n)| c < n) {
        return Err(format!(
            "{} has only {} lines; expected at least {}",
            raw.display(),
            counts.last().copied().unwrap_or(0),
            last
        ));
    }
    Ok(counts)
}

/// Normalize one raw trace line (pure — unit-tested without MAME).
pub fn normalize_line(raw: &str) -> Result<String, String> {
    let t = raw.split(' ').collect::<Vec<_>>();
    if t.len() < 9 {
        return Err(format!("malformed trace line ({len} tokens): {raw}", len = t.len()));
    }
    let lic = digits(t[1], "LIC=")?; // T-states of this instruction (not TOT)
    let af = hex4(t[2], "AF=")?;
    let bc = hex4(t[3], "BC=")?;
    let de = hex4(t[4], "DE=")?;
    let hl = hex4(t[5], "HL=")?;
    let sp = hex4(t[6], "SP=")?;
    let op = value(t[7], "OP=")?;
    if op.len() != 8 || !op.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("bad OP field: {op}"));
    }
    let pc = t[8]
        .strip_suffix(':')
        .ok_or_else(|| format!("missing PC field: {}", t[8]))?;
    if pc.len() != 4 || !pc.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("bad PC field: {pc}"));
    }
    Ok(format!("{pc} {op} AF={af} BC={bc} DE={de} HL={hl} SP={sp} CYC={lic}"))
}

/// Strip a `PREFIX` from a token and validate the remainder is non-empty.
fn value<'a>(token: &'a str, prefix: &str) -> Result<&'a str, String> {
    token
        .strip_prefix(prefix)
        .ok_or_else(|| format!("expected `{prefix}` field, got `{token}`"))
}

/// Validate a 4-hex-digit register pair (e.g. `AF=0044`).
fn hex4<'a>(token: &'a str, prefix: &str) -> Result<&'a str, String> {
    let v = value(token, prefix)?;
    if v.len() != 4 || !v.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("bad register value for `{prefix}`: {v}"));
    }
    Ok(v)
}

/// Validate a non-empty decimal field (e.g. `LIC=13`).
fn digits<'a>(token: &'a str, prefix: &str) -> Result<&'a str, String> {
    let v = value(token, prefix)?;
    if !v.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("bad decimal value for `{prefix}`: {v}"));
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn normal_line_normalizes_exactly() {
        let raw = "TOT=4 LIC=4 AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 OP=320170C3 0001: ld   ($7001),a";
        assert_eq!(
            normalize_line(raw).unwrap(),
            "0001 320170C3 AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 CYC=4"
        );
    }

    #[test]
    fn cyc_comes_from_lic_not_tot() {
        let raw = "TOT=999 LIC=13 AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 OP=C3551AFF 0004: jp   $1A55";
        assert_eq!(
            normalize_line(raw).unwrap(),
            "0004 C3551AFF AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 CYC=13"
        );
    }

    #[test]
    fn extracts_pc_op_and_registers() {
        let raw = "TOT=58 LIC=7 AF=BEEF BC=CAFE DE=1234 HL=ABCD SP=00FF OP=DDCB80E6 1A5C: cb   ($hl),$80";
        assert_eq!(
            normalize_line(raw).unwrap(),
            "1A5C DDCB80E6 AF=BEEF BC=CAFE DE=1234 HL=ABCD SP=00FF CYC=7"
        );
    }

    #[test]
    fn rejects_malformed_lines() {
        assert!(normalize_line("TOT=4 LIC=4").is_err()); // too few tokens
        assert!(normalize_line(
            "TOT=4 AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 OP=320170C3 0001: x"
        )
        .is_err()); // missing LIC
        assert!(normalize_line(
            "TOT=4 LIC=x AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 OP=320170C3 0001: x"
        )
        .is_err()); // non-decimal LIC
        assert!(normalize_line(
            "TOT=4 LIC=4 AF=004 BC=0000 DE=0000 HL=0000 SP=0000 OP=320170C3 0001: x"
        )
        .is_err()); // short register value
        assert!(normalize_line(
            "TOT=4 LIC=4 AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 OP=320170C 0001: x"
        )
        .is_err()); // short OP field
    }

    #[test]
    fn convert_truncates_to_n_and_keeps_prefix() {
        let dir = std::env::temp_dir().join(format!("xtask_trace_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let raw_path = dir.join("trace_raw.log");
        {
            let mut f = File::create(&raw_path).unwrap();
            for i in 0..12u32 {
                write!(
                    f,
                    "TOT={} LIC=4 AF={i:04X} BC=0000 DE=0000 HL=0000 SP=0000 OP=320170C3 {i:04X}: nop\r\n",
                    i + 4
                )
                .unwrap();
            }
        }
        let counts = convert(&raw_path, &dir, &[3, 7]).unwrap();
        assert_eq!(counts, vec![3, 7]);
        let small = fs::read_to_string(dir.join("trace_boot_3.log")).unwrap();
        let large = fs::read_to_string(dir.join("trace_boot_7.log")).unwrap();
        assert_eq!(small.lines().count(), 3);
        assert_eq!(large.lines().count(), 7);
        assert!(large.starts_with(&small)); // same-prefix property across sizes
        assert!(small.ends_with('\n')); // file ends with a newline
        assert_eq!(
            small.lines().next().unwrap(),
            "0000 320170C3 AF=0000 BC=0000 DE=0000 HL=0000 SP=0000 CYC=4"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn convert_errors_when_raw_too_short() {
        let dir = std::env::temp_dir().join(format!("xtask_trace_short_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let raw_path = dir.join("trace_raw.log");
        fs::write(
            &raw_path,
            "TOT=4 LIC=4 AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 OP=320170C3 0001: nop\n",
        )
        .unwrap();
        assert!(convert(&raw_path, &dir, &[5]).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
