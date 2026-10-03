# Galaxian-Rust — mémoire projet (hermes.md)

## Plan de route
- Spec : `galaxian_agentic_system_sequentiel.md` (Partie F = plan des tâches, Partie C = conventions, **B.6 = budget d'une sous-étape**).
- État : `PROGRESS.json` (racine du repo). Boucle B.1 : next_task → codeur → validateur → relecteur → commit + DONE.
- Ordre canonique : voir F.0 de la spec (ordre numérique des IDs).
- **Plan v2 : 355 sous-étapes, 44 étapes** (Phase 0 : 49, Phase 1 : 126, Phase 2 : 39, Phase 3 : 19, Phase 4 : 58, Phase 5 : 35, Phase 6 : 15, Phase 7 : 14). Chaque tâche tient dans le budget B.6 : un seul mécanisme, ≤ 2 fichiers source, ≲ 150 lignes, ≤ 4 fiches KB.

## Changement de numérotation (migration de l'ancien plan)
Les commits, `reports/` et `out/` existants portent les **anciens IDs** ; ne pas les renommer. Table de correspondance (aussi dans `legacy_id_map` de `PROGRESS.json`) :

| Ancien ID | Commits | Nouveaux IDs couverts |
|---|---|---|
| T0.1.1 Workspace Cargo | `a1b8e81` + `9944062` | T0.1.1, T0.1.2, T0.1.3, T0.1.7 (+ partie de T0.1.4 et T0.1.6) |
| T0.1.2 Types de temps | `b54e803` + `1ae7b53` | T0.2.1, T0.2.3, T0.2.4 (+ partie de T0.2.2) |
| T0.2.1 Sources MAME | `411b273` + `562270c` | T0.4.1 (+ partie de T0.4.2) |

Anciens IDs encore cités ailleurs : ancien T0.2.2 (extraire KB-22..26) = T0.4.3 → T0.4.13 ; ancien T0.3.x (golden) = T0.5.x ; ancien T0.4.x (TestROM Factory) = T0.6.x ; ancien T1.4.4 (trace MAME) = T3.3.4 ; ancien T3.1.x = T3.1–T3.4 ; ancien T2.3.x (NMI) = T2.5.x.
Les nouveaux commits suivent la convention `T<id>: <titre>` avec les **nouveaux** IDs.

## Ce qui est fait (32 tâches DONE sur 355)
- **T0.1.1 Workspace Cargo / T0.1.2 Lints / T0.1.3 CI / T0.1.7 api-dump** ✅ (ancien T0.1.1 — commits `a1b8e81` + `9944062`)
  - 5 crates : z80, galaxian-core, galaxian-audio, galaxian-frontend, xtask ; edition 2021.
  - Lints par crate (cargo 1.98 refuse le mix workspace+per-crate) : clippy all=deny + pedantic=warn ; `unsafe_code="forbid"` sur z80/galaxian-core/galaxian-audio. *À vérifier au prochain passage : présence de `rustfmt.toml`/`clippy.toml` (exigés par T0.1.2).*
  - CI `.github/workflows/ci.yml` : fmt --check, clippy `--workspace --all-targets -- -D warnings`, test --workspace.
  - Alias xtask fonctionnel via `.cargo/config.toml` (le `[alias]` de Cargo.toml est ignoré par cargo 1.98 → warning manifeste bénin).
  - `cargo xtask api-dump` opérationnel ; run-zex/mame-diff/test-roms/golden = stubs "not implemented yet".
  - Dossiers versionnés : tests/{roms,golden,third_party}/.gitkeep, roms/.gitkeep (roms/* ignoré).
  - Validateur PASS (reports/T0.1.1.json), Relecteur APPROVE sans motif bloquant.
- **T0.2.1 Constantes / T0.2.3 cycle→pixel / T0.2.4 (ligne,pixel)→cycle** ✅ (ancien T0.1.2 — commits `b54e803` + `1ae7b53`)
  - `galaxian-core/src/time.rs` : `type Cycles = u64`, Z80_HZ=3_072_000, HSYNC_HZ=16_000, CYCLES_PER_LINE=192, LINES_PER_FRAME=264, CYCLES_PER_FRAME=50_688, PIXELS_PER_LINE=384, PIXELS_PER_CYCLE=2 ; conversions cycle↔(ligne,col) (pixel = 2 cycles). Stub `timing.rs` supprimé.
  - Validateur PASS (reports/T0.1.2.json), Relecteur APPROVE + 2 findings doc non-bloquants corrigés pré-commit.
- **T0.4.1 Sources MAME** ✅ (ancien T0.2.1 — commits `411b273` + `562270c`)
  - `docs/mame_src/` = 5 fichiers verbatim (galaxian.cpp/.h, galaxian_a.cpp/.h, galaxian_v.cpp) depuis la copie locale `H:/__Emulator__/Arcade_Galaxians - Copie/mame/galaxian/` + SHA256SUMS.txt + README.md. Codeur opencode (`run --auto`).
  - Déviation documentée (out/T0.2.1/task_prompt.md §d) : le "fichier projet Mame" (URLs) n'existe pas sur disque → copie locale verbatim, SHA256 consignés. Validateur PASS (reports/T0.2.1.json), Relecteur APPROVE 0 bloquant.
- **T0.1.8 progress-check** ✅ (commit `6e3dbfa` + PROGRESS.json DONE `f2ae11c`) — livrable `crates/xtask/src/progress.rs` (validateur JSON minimal de PROGRESS.json, 0 dépendance externe).
  - Corrige le bug de parsing : l'objet/tableau n'écrasaient pas le whitespace après une virgule → échec à byte 5412 (`],\r\n`). Fix = `skip_ws` après `,`. Index byte UTF-8 via `c.len_utf8()` (accents français OK).
  - 10/10 tests unitaires passent ; clippy `-D warnings` propre ; `cargo xtask progress-check` exit 0 sur le vrai PROGRESS.json. Validateur PASS (reports/T0.1.8.json).
- **T0.1.9 PROGRESS.json initial** ✅ (commit `4a4f210` + report `reports/T0.1.9.json`) — livrable = le PROGRESS.json lui-même, validé par `cargo xtask progress-check` exit 0.
  - Spéc : « PROGRESS.json initial (toutes TODO) · val: progress-check OK ». Déviation : l'all-TODO a été remplacé par la version migrée (plan v2, 355 tâches) dans le commit `4a4f210`. Le livrable est donc satisfait par le PROGRESS.json actuel.
- **T0.2.2 Type `Cycles` et décomposition** ✅ (commits `e85161b` + `891d20e`) — livrable `crates/galaxian-core/src/time.rs`.
  - `fn split(c: Cycles) -> (frame, ligne, cycle_dans_ligne)` : décompose un cycle absolu en numéro de frame + position dans la frame. Cas de spec vérifiés (0, 191, 192, 50687, 50688) + roundtrip. Tests verts, clippy `-D warnings` propre.
- **T0.4.10 KB-25b Configuration machine** ✅ (commits `472c1a3` + `3303210`) — livrable `docs/kb/KB-25b-machine-config.md` (CPU, watchdog 8 frames ≈ 132 ms / 405504 T-states, écran raster, horloges CPU 3.072 MHz / son 1.536 MHz / HSYNC 16 kHz / VSYNC ~60.6 Hz, composants).
  - Codeur opencode ; validateur PASS (reports/T0.4.10.json) ; relecteur APPROVE par tranches B.7.3 (out/T0.4.10/verdict.partA..D.md + verdict.md). Motif unique (anglais résiduel §2/§Differs) corrigé pré-commit.
- **T0.4.11 KB-26 Ordre de rendu** ✅ (commits `5026d1e` + hash PROGRESS.json) — livrable `docs/kb/KB-26-render-order.md`.
  - Brouillon v1 (commit db86fd5, plan v1) réécrit en entier : erreur « entrées 0–2 coquilles » corrigée (coquilles = entrees 0–6, missile = 7 ; match Y deux étapes which<3 → Y−1 / which≥3 → Y). Items 1–4 du prompt satisfaits. GAP-01 clos.
  - Codeur opencode ; validateur PASS (reports/T0.4.11.json) ; relecteur APPROVE sans motif bloquant (out/T0.4.11/verdict.md).

## Piège B.7 — subagents parallèles sur LM Studio local
- Les **subagents relecteurs lancés en parallèle** via `delegate_task` échouent systématiquement : le serveur LM Studio rejette la requête « context too large » à 16–25k tokens (fenêtre annoncée 135k) — les requêtes concurrentes se disputent la capacité. **Ne pas relancer en parallèle.**
- Solution retenue (conforme A.2 : le Relecteur doit être distinct du Codeur, mais « the same LLM may play several roles in sequence ») : mener la revue **séquentiellement par l'instance parente** sur un `review_pack.md` compact (B.7.2/3). Le codeur étant opencode (processus séparé), le rôle Relecteur reste distinct du Codeur.
- Les erreurs serveur ne consomment pas d'essais de revue (B.7.5) ; rien n'est renvoyé au codeur.

## Ce qu'il reste à faire
- **En cours : T0.4.12 — KB-27 Palette PROM** (`galaxian_palette` : affectation exacte bit→R/G/B, GAP-02). Codeur opencode en cours (out/T0.4.12/task_prompt.md) ; livrable `docs/kb/KB-27-palette-prom.md`.
- **Suivant : T0.4.13 — Clôture des lacunes** (ctx: KB-08, KB-09, KB-14 · corriger ces 3 fiches si les nouvelles fiches les contredisent ; passer GAP-01/02/03/05 à `FILLED` avec `kb_file` dans PROGRESS.json). Dernier tâche de l'étape 0.4.
- Fin d'étape 0.1 : **T0.1.7 api-dump** — DONE (commit `a1b8e81+9****62`, report `reports/T0.1.1.json`). *migré depuis ancien T0.1.1.*
- Restes partiels des tâches « déjà faites » : T0.4.2 (`MANIFEST.md` : version/commit MAME + index fonction→ligne ; SHA256SUMS déjà présent).
- Étape 0.3 (T0.3.1–T0.3.8) : fiches Z80 KB-21a…g — **tâches documentaires, sources à fournir par l'humain** (*The Undocumented Z80 Documented*, tableaux Zilog).
- Étape 0.4 (T0.4.3–T0.4.13) : extraction KB-22a/b, 23a/b, 24a/b, 25a/b, 26, 27 depuis `docs/mame_src/` → fermer GAP-01/02/03/05.
- Étape 0.5 : références MAME golden (T0.5.1–T0.5.10) ; étape 0.6 : TestROM Factory (T0.6.1–T0.6.5, assembleur Z80 **au choix de l'humain**).
- Phases 1..7 : voir F.0 de la spec.

## Conventions / pièges connus
- Terminal = bash (git-bash) ; outils natifs (cargo, git) → chemins forward-slash `H:/...` (pas de conversion MSYS).
- cargo/rustc 1.98.1 : clippy `-D warnings` doit suivre un `--`.
- Ne PAS committer dashboard.html ni galaxian.zip (artefacts session parente / ROMs non versionnées).
- Chaque tâche : codeur → validateur (reports/<ID>.json) → relecteur APPROVE → commit "T<id>: <titre>" + PROGRESS.json DONE.
- Codeur = **opencode** CLI (`opencode run --auto '<prompt>' -f out/<TASK>/task_prompt.md`, workdir=repo). `--auto` requis : le sandbox auto-rejecte reads/writes hors du repo (external_directory) sans it. Validateur/relecteur restent des subagents frais (skills galaxian-validator / galaxian-reviewer).
- **Budget B.6** : l'Orchestrateur vérifie avant de dispatcher (≤ 4 fiches KB, ≤ 3 livrables, 1 mécanisme). Si le codeur répond `## DECOUPAGE`, ou si le budget est dépassé : arrêt de la chaîne, découpage proposé (sous-tâches suffixées `a`, `b`…) à valider par l'humain. Le relecteur REJECT tout patch qui sort du budget ou touche des fichiers hors `livrables`.
- **Context pack** : injecter le `ctx par défaut` de l'étape + `ctx+` de la tâche (pas plus). Les fiches KB-21a…g sont distinctes : ne donner au codeur que celles de la tâche.
- Une tâche = un seul type de travail (coder / rédiger une fiche / écrire une ROM / produire un golden). Les ROMs `TR-*` sont toujours livrées en deux tâches : (a) source+bin+README, (b) golden + validation.
- Échec dans la validation globale CPU (étape 1.9) : ne pas corriger « en place » ; proposer des tâches de correction ciblées par famille (`T1.9.4a`…), validées par l'humain.
- À répercuter dans les skills `galaxian-validator` / `galaxian-reviewer` et dans le prompt opencode (non fournis ici) : règle de budget B.6 pour le relecteur, bloc `## DECOUPAGE` pour le codeur, nouveaux IDs.
