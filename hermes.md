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

## Ce qui est fait (38 tâches DONE sur 355)
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
- **T0.4.12 KB-27 Palette PROM** ✅ (commits `0d8eddd` + hash PROGRESS.json) — livrable `docs/kb/KB-27-palette-prom.md`.
  - Affectation exacte bit→R/G/B de `galaxian_palette` : R = bits 0-2, G = bits 3-5, B = bits 6-7 (commentaire galaxian_v.cpp:243-253 + décodage :286-300 concordants) ; résistances 1k/470/220 Ω par composante. **GAP-02 clos** : la doc fournie (KB-08) listait « VERT » deux fois bits 5/4 — affectation erronée, corrigée ici.
  - Palette étoiles `m_star_color[64]` (bits 5/4 rouge, 3/2 vert, 1/0 bleu @150/100 Ω) + table `starmap[4]={0,194,214,255}` ; coquilles/missile `m_bullet_color[8]` = 7 blanches + jaune (galaxian_v.cpp:355-358). Cross-refs KB-08/KB-25b §3/KB-26/KB-22a/KB-22b.
  - Codeur opencode ; validateur PASS (reports/T0.4.12.json, 37 citations vérifiées) ; relecteur REJECT 1 motif bloquant (terme « color_span » non sourcé — n'apparaît pas dans mame_src) corrigé pré-commit → APPROVE (out/T0.4.12/verdict.md).
- **T0.4.13 Clôture des lacunes** ✅ (commits `763f208` + hash PROGRESS.json `36d41ad`) — livrables `docs/kb/KB-08.md`, `docs/kb/KB-08-palette-prom.md`, `docs/kb/KB-09.md`, `docs/kb/KB-14.md` + `PROGRESS.json` (kb_gaps).
  - KB-08 (×2 fichiers, duplication flagguée pour l'humain — aucun supprimé) : table bit→composante corrigée R bits 0-2 / G bits 3-5 / B bits 6-7 (galaxian_v.cpp:243-253 = décodage :286-300 concordants), doc fournie erronée sur bits 3/2. KB-09 : cross-ref complétée KB-23a/KB-23b/KB-22a/KB-25b (le codeur ne citait que KB-23a/KB-25b). KB-14 : ordre fond→étoiles→tilemap→sprites→shells/missile confirmé par `screen_update_galaxian` (galaxian_v.cpp:460-477) + cross-ref KB-26.
  - **GAP-01/02/03/05 → FILLED** avec kb_file : GAP-01→KB-26-render-order.md, GAP-02→KB-27-palette-prom.md, GAP-03→KB-25a-romset.md, GAP-05→KB-24a-inputs.md.
  - Codeur opencode (run mort en cours de tâche — LM Studio « Model unloaded by user or API request » après les 2 fichiers KB-08 ; resume run `out/T0.4.13/task_prompt_resume.md` a produit le reste). Validateur PASS (reports/T0.4.13.json) ; relecteur APPROVE 2 motifs non bloquants corrigés pré-commit (glissements anglais + cross-ref KB-09 incomplète) → out/T0.4.13/verdict.md.
- **T0.5.1 Environnement MAME** ✅ (commits `5693456` spec v2 + `07d01ed` livrables + hash PROGRESS.json) — livrables `docs/mame_notes.md`, `docs/kb/KB-28a.md`, `docs/kb/KB-28b.md`. **GAP-08 clos** (spec Partie G).
  - MAME v0.289 (mame0289) vérifié sur `tools\mame.exe` ; chaque ⚠ de KB-28a/b tranché, corrections marquées **[T0.5.1]**. Corrections notables : set Midway = **`galaxianm`/`galaxianmo`** (pas `galmidw`) → à répercuter T0.4.9/T3.1.7 ; première ligne de trace **PC=0001** (pas 0000) → corriger la validation T0.5.3 ; compteurs `totalcycles`/`lastinstructioncycles`/`cycles` existent (`symlist maincpu` → debug.log, pas stdout) ; `-no_coin_lockout` inexistant (crédits via champ `Coin 1`) ; bitmap MAME **768×224** (rotate 90, refresh 60.606061) ; WAV **sr=48000** (byte rate 96000 = 48000×2 — pas de discrepancy).
  - Codeur opencode mort en cours de tâche (« No models loaded ») avant d'écrire les livrables → les 3 docs rédigés directement par l'instance parente depuis `golden/tmp/` (aucun re-run). Validateur PASS (reports/T0.5.1.json) ; relecteur APPROVE mené séquentiellement par l'instance parente (solution B.7) → out/T0.5.1/verdict.md.
  - Commit `5693456` = refinements v2 de la spec pré-existants dans l'arborescence de travail (Part E KB-28a/b, étape 0.5 réécrite T0.5.x, GAP-08) — commit séparé pour garder le commit de livrables T0.5.1 dans le budget B.6.
- **T0.5.2 Lanceur MAME** ✅ (commit `c3937e9` + hash PROGRESS.json) — livrable `crates/xtask/src/golden.rs` (`cargo xtask golden run`) + wiring `main.rs` + `.gitignore` (+ `tools/`, `tests/golden/tmp/`).
  - Profil commun KB-28a §3 exact (y compris `-autoboot_delay 0`) ; MAME = `<racine>/tools/mame.exe` avec surcharge `MAME_DIR` ; cwd du processus = dossier de mame.exe ; suppression + recréation de `tests/golden/tmp/run/{cfg,nvram,snap,inp,sta}` avant chaque run réel (pas en dry-run) ; args additionnels après `--` ; `--dry-run` affiche la commande sans lancer MAME ni toucher au filesystem ; code retour de MAME propagé. 3 tests unitaires sans lancer MAME.
  - Codeur opencode (run OK, pas de mort mid-task). Validateur PASS (reports/T0.5.2.json) — subagent frais, seul (pas de concurrence LM Studio) ; relecteur APPROVE mené séquentiellement par l'instance parente (solution B.7) → out/T0.5.2/verdict.md.
- **T0.5.3 Trace CPU brute** ✅ (commit `1645e07` + hash PROGRESS.json) — livrable `tests/golden/mame/trace.dbg` (squelette KB-28b §A corrigé T0.5.1 : action trace avec chemin `/`, `noloop`, compteurs `totalcycles`+`lastinstructioncycles` dans l'action `tracelog` ; `go`). 0 ligne de code Rust touchée.
  - Écart justifié par rapport à la commande littérale du prompt : `-debugger none` retiré — sur le binaire MAME v0.289, `-debugger none` n'exécute JAMAIS `-debugscript` (MAME saute le script entièrement ; `gdbstub` reste bloqué en attente d'un client). Seul le debugger **windows** par défaut exécute le script (fenêtre ouverte brièvement, MAME s'arrête via `-seconds_to_run`). Validation exécutée : `cargo xtask golden run -- -debug -debugscript ../tests/golden/mame/trace.dbg -video none -seconds_to_run 10`.
  - Deux runs exit 0, **3 717 920 lignes** chacune (≥ 1 000), première ligne à PC=0001, aucune ligne `0000:`, dernière ligne complète + newline final ; premières 100 000 lignes byte-identiques entre les deux runs (SHA256 `d9be639a…`).
  - Codeur opencode (run OK). Validateur PASS (reports/T0.5.3.json) — subagent frais, seul ; relecteur APPROVE mené séquentiellement par l'instance parente (solution B.7) → out/T0.5.3/verdict.md.
  - **Commit docs séparé `ec5660a`** : correction du tranché T0.5.1 erroné « utiliser `-debugger none` pour un run headless » présent en 3 endroits (KB-28a:58, KB-28b:25, mame_notes.md §3.8 + conséquence T0.5.2) — le test `runDN` de T0.5.1 ne vérifiait que l'exit code, pas la sortie de trace. À répercuter dans les tâches de trace suivantes (T0.5.4+).
- **T0.5.4 Trace CPU normalisée** ✅ (commit `e8a68c7` + hash PROGRESS.json) — livrable `crates/xtask/src/trace.rs` (nouveau module ~136 lignes d'implémentation + 6 tests unitaires sans MAME) + wiring `main.rs` : commande `cargo xtask golden trace`.
  - Lit `tests/golden/tmp/trace_raw.log` (produit par T0.5.3), tronque à N et convertit en `tests/golden/trace_boot_{1000,100000,1000000}.log` au format normalisé fixe : `<PC> <OP> AF=… BC=… DE=… HL=… SP=… CYC=<LIC>` (CYC = valeur décimale du champ LIC/lastinstructioncycles — tranché T0.5.1, PAS reconstruit depuis KB-21b ; disassemblage supprimé ; un seul passage streaming).
  - 3 goldens versionnés : 1 000 / 100 000 / 1 000 000 lignes, formats identiques (0 ligne déviante), premières 1 000 lignes byte-identiques dans les 3 (SHA256 `3c1b1d4c…`) ; non ignorés par git.
  - Codeur opencode (run OK). Validateur PASS (reports/T0.5.4.json) — subagent frais, seul : goldens régénérés de zéro (supprimés puis recréés), 19/19 tests, clippy `-D warnings` propre ; relecteur APPROVE mené séquentiellement par l'instance parente (solution B.7) → out/T0.5.4/verdict.md.

## Piège B.7 — subagents parallèles sur LM Studio local
- Les **subagents relecteurs lancés en parallèle** via `delegate_task` échouent systématiquement : le serveur LM Studio rejette la requête « context too large » à 16–25k tokens (fenêtre annoncée 135k) — les requêtes concurrentes se disputent la capacité. **Ne pas relancer en parallèle.**
- Solution retenue (conforme A.2 : le Relecteur doit être distinct du Codeur, mais « the same LLM may play several roles in sequence ») : mener la revue **séquentiellement par l'instance parente** sur un `review_pack.md` compact (B.7.2/3). Le codeur étant opencode (processus séparé), le rôle Relecteur reste distinct du Codeur.
- Les erreurs serveur ne consomment pas d'essais de revue (B.7.5) ; rien n'est renvoyé au codeur.

## Ce qu'il reste à faire
- **En cours : T0.5.5 — CRC de frames (script Lua)** (étape 0.5, références MAME golden ; T0.5.6–T0.5.10 ensuite). T0.5.4 DONE (trace normalisée `xtask golden trace` + 3 goldens) ; étape 0.4 terminée (T0.4.3–T0.4.13) : GAP-01/02/03/05 fermés par T0.4.13.
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
