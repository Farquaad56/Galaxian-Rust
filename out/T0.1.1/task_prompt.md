# TASK PROMPT — T0.1.1

## a. En-tête de tâche (Partie F, copie littérale)

**T0.1.1 — Workspace Cargo.** deps: — · ctx: — · livrables : arborescence Partie C, CI (`cargo fmt --check`, `clippy -D warnings`, `cargo test`). · validation: `cargo build --workspace && cargo test --workspace` (vert, 0 test).

## b. Fiches KB à injecter

Aucune (ctx vide pour cette tâche). Ne t'appuie que sur la Partie C ci-dessous et l'en-tête de tâche.

## c. API existante (`cargo xtask api-dump`)

Aucun code Rust n'existe encore — c'est la première tâche du projet. Le dépôt contient uniquement :
- `galaxian_agentic_system_sequentiel.md` (spec complète, Partie C = conventions)
- `PROGRESS.json` (état du plan)
- `docs/kb/KB-01..KB-08` (fiches KB déjà extraites)
- `README.md`, git repo avec remote origin.

## d. Tests à faire passer / validation exacte

Commandes de validation (à exécuter depuis la racine du dépôt, toutes doivent être vertes) :

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -D warnings
cargo build --workspace && cargo test --workspace   # vert, 0 test
```

## e. Règles Partie C (conventions du dépôt Rust)

Arborescence attendue à la racine du dépôt (`H:/__Emulator__/Arcade_Galaxian_Rust`) :

```
Cargo.toml                (workspace)
crates/
├─ z80/                   CPU Z80 cycle-accurate, no_std-compatible, agnostique du hardware
├─ galaxian-core/         bus, timing, mémoire, I/O, vidéo, orchestration
├─ galaxian-audio/        modèle discret du son
├─ galaxian-frontend/     fenêtre, input, audio out (minifb/pixels/cpal) — hors cœur
└─ xtask/                 cargo xtask : api-dump, run-zex, mame-diff, test-roms, golden
tests/
├─ roms/                  ROMs de test maison (sources .asm + binaires)
├─ golden/                références MAME (traces, CRC de frames, WAV)
└─ third_party/           zexdoc/zexall, z80test, fuse, singlestep (téléchargés by xtask)
roms/                     ROMs commerciales (NON versionnées, fournies par the user) → .gitignore
docs/kb/                  fiches KB (Partie E), une par fichier  (déjà présent)
PROGRESS.json             (déjà présent)
```

Règles :
- Edition 2021+, `#![forbid(unsafe_code)]` dans les crates cœur (`z80`, `galaxian-core`, `galaxian-audio`), `clippy -D warnings`, `rustfmt`.
- Dépendances autorisées dans le cœur : `bitflags`, `thiserror`. Dev : `proptest`, `serde_json`, `crc32fast`. Tout ajout = décision humaine (ne pas ajouter de dépendance pour this task).
- API CPU ↔ bus imposée (à exposer plus tard, not now — just keep the workspace clean for it):

```rust
pub trait Bus {
    /// Avances le time of `tstates` cycles Z80 AVANT the access following.
    fn advance(&mut self, tstates: u32);
    fn mem_read(&mut self, addr: u16) -> u8;          // cycle M1 excluded
    fn mem_write(&mut self, addr: u16, val: u8);
    fn fetch_opcode(&mut self, addr: u16) -> u8;      // cycle M1 (4 T-states)
    fn io_read(&mut self, port: u16) -> u8;
    fn io_write(&mut self, port: u16, val: u8);
    fn nmi_pending(&mut self) -> bool;                // front montant consumed by the CPU
    fn int_line(&self) -> bool;
}
```

- Each phase produces a **function `snapshot()`** (état sérialisable).
- All comparison tests return the **first divergence point** (cycle + context).

## Livrables concrets de T0.1.1

1. Workspace Cargo à la racine : membres = les 5 crates ci-dessus, workspace lints (`clippy` pedantic-level at least `-D warnings`, `unsafe_code = "forbid"` for the core crates), shared dev-dependencies if useful.
2. Les 5 crates existantes avec un minimal compilable `lib.rs` (or `main.rs` for xtask) :
   - `z80`: lib, no_std-compatible (`#![no_std]` + `forbid(unsafe_code)`), module skeleton only (empty modules OK).
   - `galaxian-core`: lib, `forbid(unsafe_code)`, module skeleton only.
   - `galaxian-audio`: lib, `forbid(unsafe_code)`, module skeleton only.
   - `galaxian-frontend`: lib, no dependency yet (minifb/pixels/cpal will be added by T6.1.1 — do not add them now).
   - `xtask`: bin crate wired as `[alias] xtask = "run --package xtask --"` in the workspace Cargo.toml; subcommands `api-dump`, `run-zex`, `mame-diff`, `test-roms`, `golden` — for this task only `api-dump` must be functional (prints public signatures of all workspace crates, e.g. via `cargo metadata` + a simple parse or doc comments; keep it dependency-free if possible); the other four may print "not implemented yet" and exit 0.
3. Dossiers vides versionnés : `tests/roms/.gitkeep`, `tests/golden/.gitkeep`, `tests/third_party/.gitkeep`; `.gitignore` for `target/`, `roms/*` (except .gitkeep), build artifacts.
4. CI : `.github/workflows/ci.yml` running on push/PR to main: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace`.

## Contraintes d'exécution

- Repo : `H:/__Emulator__/Arcade_Galaxian_Rust` (git, branch main). Do NOT commit — the parent session commits after review.
- Run the validation commands yourself before answering; they must all pass.
- No new dependencies beyond those listed in Partie C.
- If a rule above is ambiguous or contradictory: stop and ask in a `## QUESTIONS` block instead of inventing.
