# SYSTÈME AGENTIQUE — Émulateur Galaxian cycle-accurate en Rust

> Objectif : piloter un ou plusieurs LLM codeurs, étape par étape et sous-étape par sous-étape, pour produire un émulateur de la borne **Galaxian d'origine** (Namco/Midway, 1979), cycle-accurate, en Rust, avec un frontend **eframe/egui**, validé par tests automatisés et par comparaison à des références de comportement.
>
> **Principe d'indépendance :** l'émulateur n'est **pas** basé sur MAME. Le code source MAME n'est qu'une **documentation de référence en lecture seule** (voir C.0) : rien n'en est porté, compilé ni lié.
>
> Ce document contient :
> - **Partie A** — Architecture du système et rôles des agents (avec prompts système)
> - **Partie B** — Protocole de boucle, format de tâche, gestion d'état
> - **Partie C** — Décisions d'architecture (C.0) et conventions du dépôt Rust
> - **Partie D** — Stratégie de validation (ROMs de test, références MAME)
> - **Partie E** — Base de connaissances (KB) : toute la doc technique, découpée en fiches citables par ID
> - **Partie F** — Plan complet : phases → étapes → **sous-étapes fines** (budget B.6), chacune avec son *context pack* (fiches KB), livrables et tests
> - **Partie G** — Lacunes connues de la documentation et comment les combler
>
> **Mode d'exécution : SÉQUENTIEL STRICT** — un seul agent actif à la fois, une seule tâche à la fois, dans un ordre total (voir A.4, B.5 et F.0).

---

# PARTIE A — Architecture du système

## A.1 Principe

Le LLM codeur ne reçoit **jamais** tout le projet. Il reçoit **une sous-étape à la fois**, avec :
1. la spécification de la tâche (objectif, API Rust attendue, critères d'acceptation) ;
2. un **context pack** : uniquement les fiches KB nécessaires (Partie E) ;
3. l'état courant du dépôt (arborescence + signatures publiques des modules déjà faits) ;
4. les tests qu'il doit faire passer (écrits **avant** le code quand c'est possible).

La sous-étape est dimensionnée pour tenir **dans une seule réponse** du Codeur : un seul mécanisme, peu de fichiers, peu de fiches (budget B.6).

Une sous-étape n'est « terminée » que lorsque le **Validateur** a exécuté les tests et que le **Relecteur** a donné son accord. Les agents interviennent **l'un après l'autre**, jamais en parallèle (voir A.4).

## A.2 Rôles

| Agent | Rôle | Entrées | Sorties |
|---|---|---|---|
| **Orchestrateur** | Lit `PROGRESS.json`, choisit **la** prochaine tâche de `order` dont les dépendances sont `DONE`, assemble le context pack, la dispatche (une seule à la fois) | `PROGRESS.json`, Partie F, Partie E | Un *task prompt* complet |
| **Codeur** | Implémente une sous-étape | Task prompt | Patch Rust + tests |
| **Testeur / Validateur** | Écrit/lance les tests, compare aux références MAME, produit un rapport JSON | Patch, commandes de test | `reports/<TASK_ID>.json` (pass/fail + diff) |
| **Relecteur** | Vérifie conformité à la doc (KB), style, absence de triche (valeurs codées en dur pour passer un test) | Patch + rapport du Validateur + KB | Verdict `APPROVE` / `REJECT` + motifs |
| **Documentaliste** | Met à jour `docs/` et `PROGRESS.json`, consigne les écarts découverts | Tout | MAJ doc/état |

Un même LLM peut jouer plusieurs rôles en séquence, mais **le Relecteur ne doit pas être la même instance de conversation que le Codeur**.

## A.3 Prompts système

### A.3.1 Orchestrateur

```
Tu es l'ORCHESTRATEUR d'un projet d'émulateur Galaxian cycle-accurate en Rust.
Tu ne codes pas. Tu :
1. Lis PROGRESS.json.
   - Si `current_task` est renseigné (tâche IN_PROGRESS), tu la REPRENDS : tu n'en
     démarres JAMAIS une autre.
   - Sinon, tu choisis UNE SEULE tâche : la première, dans l'ordre de `order`,
     dont le statut est TODO et dont toutes les dépendances sont DONE.
   - Si la prochaine tâche de `order` a une dépendance non DONE, c'est une erreur de
     plan : tu t'arrêtes et tu le signales (tu ne « sautes » pas à la suivante).
2. Génères le TASK PROMPT en concaténant, dans cet ordre :
   a. l'en-tête de tâche (Partie F, copie littérale) **avec les valeurs par défaut de son étape** (`ctx par défaut`, `val par défaut`),
   b. le texte intégral des fiches KB de `context` (défaut de l'étape + `ctx+`),
   c. la sortie de `cargo xtask api-dump` (signatures publiques existantes),
   d. les chemins des tests à faire passer,
   e. les règles de la Partie C.
3. N'inclus JAMAIS de fiche KB non listée (pour éviter la dilution de contexte).
   Avant de produire le TASK PROMPT, vérifie le BUDGET (B.6) : plus de 4 fiches KB, plus de 3 livrables
   ou plus d'un mécanisme = erreur de plan → tu t'arrêtes et proposes un découpage (sous-tâches suffixées
   a, b, c…) que l'humain doit valider ; tu ne lances pas la tâche.
4. Si une tâche échoue 3 fois, tu la passes en BLOCKED, tu produis un rapport
   d'analyse (hypothèses, fiches KB peut-être manquantes) et tu demandes
   l'intervention humaine, puis tu t'ARRÊTES : aucune autre tâche n'est lancée
   tant que l'humain n'a pas répondu.
5. Tu es lancé seul : jamais en même temps qu'un autre agent. Tu ne produis qu'UN
   seul TASK PROMPT par appel.
Format de sortie : un bloc Markdown « TASK PROMPT » et rien d'autre.
```

### A.3.2 Codeur

```
Tu es un ingénieur Rust senior spécialisé en émulation matérielle cycle-accurate.
Règles absolues :
- La documentation technique fournie (fiches KB) est LA source de vérité.
  Si elle est ambiguë ou contradictoire, tu t'arrêtes et tu poses la question
  dans un bloc `## QUESTIONS` au lieu d'inventer.
- Tu ne modifies que les fichiers listés dans `livrables`.
- Tu écris d'abord les tests (ou tu utilises ceux fournis), puis le code.
- Aucun `unsafe`, aucun `unwrap()` hors tests, aucune dépendance non listée.
- Le cœur (`galaxian-core`, `z80`) est déterministe : pas d'horloge système,
  pas de RNG système, pas de threads.
- Chaque constante matérielle porte un commentaire `// KB-xx §y` indiquant sa source.
- Tu ne « triches » pas : interdit de coder en dur une valeur pour faire passer un test.
- MAME n'est qu'une DOCUMENTATION : tu ne portes pas de code C++ MAME, tu n'en reproduis ni la structure
  (classes, délégués, `device_t`, tilemap_t…) ni les noms d'API ; tu implémentes en Rust idiomatique,
  à partir des fiches KB et du comportement matériel qu'elles décrivent.
- Le cœur ne dépend jamais d'`eframe`/`egui` ni d'aucune bibliothèque de fenêtrage ou d'audio :
  seul `galaxian-frontend` les utilise.
Format de réponse : 1) plan en 5 lignes max ; 2) patch (fichiers complets ou diff) ;
3) commandes de test à lancer ; 4) liste des écarts/hypothèses.
Si la tâche te paraît dépasser le budget (plusieurs mécanismes, plus de ~150 lignes de code hors tests,
plus de 3 fichiers), tu NE codes PAS : tu réponds par un bloc `## DECOUPAGE` proposant des sous-tâches
ordonnées, chacune avec ses tests et sa validation.
```

### A.3.3 Validateur

```
Tu es le VALIDATEUR. Tu exécutes exactement les commandes de la section
`validation` de la tâche, tu collectes la sortie brute, et tu produis un JSON :
{ "task": "...", "status": "PASS|FAIL", "tests": [{"name":..., "status":..., "detail":...}],
  "first_divergence": {"cycle":..., "expected":..., "actual":...} | null }
Pour toute comparaison de traces, tu rapportes la PREMIÈRE divergence
(cycle, PC, registres, adresse bus) et 20 lignes de contexte avant.
Tu n'interprètes pas, tu ne corriges pas.
```

### A.3.4 Relecteur

```
Tu es le RELECTEUR. Tu reçois le patch, le rapport PASS du Validateur, la tâche et les fiches KB.
Vérifie : (1) chaque comportement matériel du patch est justifié par une fiche KB ;
(2) aucune constante magique non sourcée ; (3) pas de test contourné ;
(4) respect de la Partie C ; (5) granularité de timing conforme à la tâche
(pas de « ticks groupés » si la tâche exige T-state par T-state) ; (6) le patch reste dans le budget B.6 :
fichiers hors `livrables` ou comportements d'autres tâches = REJECT.
Réponds APPROVE ou REJECT + liste numérotée de motifs actionnables.
```

## A.4 Mode d'exécution : séquentiel strict

1. **Un seul agent actif à la fois.** Un agent n'est lancé que lorsque le précédent a rendu sa sortie complète.
2. **Une seule tâche à la fois.** Au plus une tâche `IN_PROGRESS` dans tout le projet (verrou `current_task` dans `PROGRESS.json`).
3. **Ordre total.** Les tâches sont exécutées dans l'ordre de la liste `order` (voir F.0). Les `deps` ne servent plus à choisir entre plusieurs tâches possibles : elles servent de contrôle de cohérence.
4. **Passage de relais par fichiers.** Chaque agent écrit sa sortie dans un fichier, et le suivant la lit :

| Étape | Agent | Sortie | Lue par |
|---|---|---|---|
| 1 | Orchestrateur | `out/<TASK_ID>/task_prompt.md` | Codeur |
| 2 | Codeur | `out/<TASK_ID>/patch` (+ plan, écarts, questions) | Validateur |
| 3 | Validateur | `reports/<TASK_ID>.json` | Relecteur, Codeur (si FAIL) |
| 4 | Relecteur | `out/<TASK_ID>/verdict.md` | Documentaliste, Codeur (si REJECT) |
| 5 | Documentaliste | commit git + `PROGRESS.json` | Orchestrateur (tâche suivante) |

5. **Pas de tâche suivante avant le commit.** La tâche N+1 ne démarre qu'après le commit du Documentaliste pour la tâche N : le dépôt et la sortie de `cargo xtask api-dump` sont donc toujours à jour pour le Codeur.
6. **Un blocage arrête tout.** Il n'y a pas de travail alternatif à poursuivre (voir B.5).

---

# PARTIE B — Protocole, format de tâche, état

## B.1 Boucle

```
loop {
  t = orchestrateur.next_task()          // reprend current_task s'il existe, sinon 1re tâche TODO de `order` dont deps DONE
  if t == None { break }                 // plan terminé
  progress.current_task = t              // VERROU : aucune autre tâche ne peut démarrer
  prompt = orchestrateur.build(t)        // attendre la fin  -> out/<id>/task_prompt.md
  patch = codeur.run(prompt)             // attendre la fin  -> out/<id>/patch
  report = validateur.run(t.validation)  // lancé APRÈS le codeur -> reports/<id>.json
  if report.FAIL { codeur.retry(report) (max 3) ; continue }     // t reste current_task
  verdict = relecteur.run(patch, report, t)  // lancé APRÈS le validateur -> out/<id>/verdict.md
  if REJECT { codeur.retry(verdict) (max 3) ; continue }          // t reste current_task
  documentaliste.commit(t, patch, report, verdict)  // git commit "T<id>: <titre>", PROGRESS.json -> DONE
  progress.current_task = null           // verrou libéré, seulement maintenant
}
// À 3 échecs sur une tâche : status = BLOCKED, halted = true, ARRÊT COMPLET (voir B.5).
```

## B.2 `PROGRESS.json`

```json
{
  "order": ["T0.1.1", "T0.1.2", "T0.1.3", "..."],   // ordre numérique canonique, voir F.0
  "current_task": null,                              // verrou : au plus une tâche en cours
  "halted": false,                                   // true = arrêt en attente de l'humain
  "legacy_id_map": {},                               // optionnel : anciens IDs -> IDs actuels (voir B.5)
  "tasks": {
    "T1.7.13": { "title": "LDIR / LDDR", "deps": ["T0.3.1","T0.3.6","T1.7.12"], "ctx": ["KB-21a","KB-21e"],
                 "status": "TODO|IN_PROGRESS|DONE|BLOCKED", "attempts": 0,
                 "commit": null, "report": null, "notes": [] }
  },
  "kb_gaps": [ { "id": "GAP-01", "status": "OPEN|FILLED", "closed_by": ["T0.4.3"], "kb_file": null } ]
}
```

## B.3 Gabarit de tâche

```
### T<phase>.<étape>.<sous-étape> — Titre
- deps      : [IDs]
- context   : [KB-xx, ...]            ← fiches à injecter
- livrables : [chemins de fichiers]
- spec      : comportement attendu, API Rust, invariants
- tests     : tests à écrire / à faire passer (chemins)
- validation: commandes exactes + critère de succès
- DoD       : définition de « terminé »
- budget    : respecte B.6 (sinon : découper)
```
Dans la Partie F, les champs `ctx` et `val` communs à une étape sont écrits **une fois** dans l'en-tête de l'étape (`ctx par défaut`, `val par défaut`) et surchargés tâche par tâche si besoin ; la DoD par défaut est : validation verte + `cargo fmt --check` + `clippy -D warnings` + approbation du Relecteur.

## B.4 Règle d'or du cycle-accurate

Le temps est la **donnée centrale**. L'unité de temps de l'émulateur est le **cycle d'horloge Z80** (T-state, 3,072 MHz) ; tout composant (vidéo, NMI, son) est avancé *entre* les accès bus du CPU via `bus.advance(tstates)`. Le CPU **n'exécute jamais une instruction « d'un bloc » puis rattrape** : chaque accès mémoire/E-S est précédé de l'avance de l'horloge du nombre exact de T-states écoulés depuis l'accès précédent.

## B.5 Règles du mode séquentiel

- **Verrou unique.** `current_task` contient au plus un ID. Si `PROGRESS.json` montre plus d'une tâche `IN_PROGRESS`, c'est une anomalie : `halted = true` et intervention humaine.
- **Sélection.** La tâche suivante est la première de `order` en `TODO`. Si une de ses dépendances n'est pas `DONE`, c'est une erreur de plan : arrêt et signalement, pas de saut.
- **Reprise d'un projet commencé sous un ancien plan.** Les tâches déjà réalisées sont marquées `DONE` avec leurs commits et rapports d'origine (les anciens IDs restent valables dans `git log`, `reports/` et `out/`, et sont consignés dans `legacy_id_map` et `notes`). Une tâche `DONE` peut apparaître après des tâches `TODO` dans `order` ; seule la cohérence de ses **dépendances déclarées** est contrôlée. Le sélecteur (« première `TODO` de `order` ») ne change pas.
- **Blocage.** Une tâche `BLOCKED` met `halted = true` : plus aucun agent n'est lancé. L'humain corrige (fiche KB, tâche, lacune), remet `status = TODO`, `attempts = 0`, `halted = false`. Seul l'humain peut décider de sauter une tâche bloquée, et uniquement si aucune tâche restante n'en dépend.
- **Lacunes (Partie G).** Si la prochaine tâche dépend d'une lacune `OPEN`, l'Orchestrateur la marque `BLOCKED`, s'arrête et propose d'insérer la tâche de comblement juste avant dans `order`. L'humain valide l'insertion.
- **Reprise après interruption.** Le système peut être arrêté entre deux agents. Au redémarrage, on reprend `current_task` et on relance l'agent suivant le dernier fichier présent dans `out/<id>/` et `reports/` (pas de `task_prompt.md` : Orchestrateur ; pas de `patch` : Codeur ; pas de `reports/<id>.json` : Validateur ; pas de `verdict.md` : Relecteur ; sinon : Documentaliste).
- **ROMs de test `TR-*`.** Les tâches `T0.6.1`–`T0.6.5` posent une seule fois la chaîne d'assemblage et les conventions (`tests/roms/`). Ensuite, chaque ROM `TR-*` est livrée en **deux sous-étapes consécutives** : (a) source `.asm` + `.bin` + README, (b) golden MAME + validation (voir le catalogue en fin de Partie F). Rien n'est écrit « en parallèle » des phases.
- **Pas de concurrence côté exécution.** Si le framework qui lance les agents sait paralléliser (threads, workers, sous-agents), le limiter à **1 worker** pour ce projet.

## B.6 Budget d'une sous-étape (granularité)

Une sous-étape doit pouvoir être traitée **d'un seul coup** par un LLM codeur, sans perdre le fil :

| Critère | Limite |
|---|---|
| Mécanisme / comportement | **un seul** (ex. « LDIR/LDDR », pas « tout le préfixe ED ») |
| Instructions Z80 | une famille cohérente, **~6 opcodes distincts** au plus |
| Fichiers source modifiés | ≤ 2 (+ 1 fichier de tests) |
| Code ajouté (hors tests) | ≲ 150 lignes |
| Fiches KB injectées | ≤ 4 (context pack ≲ 4 000 tokens) |
| Tests nouveaux | ≲ 12 |
| Commande de validation principale | 1 |
| Nature | **un seul type de travail** : coder, **ou** rédiger une fiche, **ou** écrire une ROM de test, **ou** générer/valider un golden |

Conséquences :
- Une ROM `TR-*` = (a) ROM, (b) golden + validation ; jamais les deux dans la même tâche.
- Les tâches de **validation globale** (zexdoc, z80test, Fuse complet…) sont découpées par famille d'instructions. Si l'une échoue, on **n'enchaîne pas** de correction « en place » : l'Orchestrateur propose d'insérer des tâches de correction ciblées (une par famille en échec), suffixées `a`, `b`, … (ex. `T1.9.4a`), **validées par l'humain**, juste avant la tâche en échec.
- Si un Codeur répond `## DECOUPAGE` ou si l'Orchestrateur détecte un dépassement, la chaîne s'arrête (comme un blocage) jusqu'à validation du découpage par l'humain.
- Tâche trop petite ? On ne fusionne pas pour « gagner du temps » : le coût d'une sous-étape est faible, celui d'une tâche mal comprise est élevé.

---

# PARTIE C — Décisions d'architecture et conventions du dépôt Rust

## C.0 Décisions d'architecture (fixées par l'humain)

| ID | Décision |
|---|---|
| **AD-01** | **L'émulateur n'est pas basé sur MAME.** Les sources MAME (`docs/mame_src/`, lecture seule) servent uniquement de documentation pour extraire des faits matériels (layouts, registres, timings, valeurs R/C). Interdits : copier ou traduire du code MAME, calquer son architecture (`galaxian_state`, tilemap, moteur DISCRETE…), ajouter MAME comme dépendance. L'architecture Rust (`Machine`, `Bus`, `Raster`, `Frame`…) est définie par ce document. |
| **AD-02** | **Le frontend est une application `eframe`/`egui`** (crate `galaxian-frontend`). Le cœur (`z80`, `galaxian-core`, `galaxian-audio`) reste sans dépendance graphique : il expose un `Frame` (tampon RGB 256×224) et des échantillons audio ; le frontend convertit le `Frame` en texture egui. |
| **AD-03** | **MAME comme oracle externe (optionnel).** Les « golden » (traces, CRC de frames, WAV) restent générés en lançant le binaire MAME en ligne de commande (`xtask golden`) : MAME est alors une boîte noire d'observation, comme zexdoc ou Fuse, jamais une base de code. MAME n'est requis que pour **générer ou régénérer** les golden (tâches `golden` des étapes 0.5, 2.x, 3.x, 4.x, 5.5) ; les tests de non-régression comparent aux fichiers golden déjà produits. |
| **AD-04** | Egui ne fournit ni audio ni manette : sortie audio via `cpal` (T6.3.x), manette via `gilrs` (T6.2.2) — ajouts **dans `galaxian-frontend` uniquement**, validés par l'humain. |


```
galaxian/
├─ Cargo.toml                (workspace)
├─ crates/
│  ├─ z80/                   CPU Z80 cycle-accurate, no_std-compatible, agnostique du hardware
│  ├─ galaxian-core/         bus, timing, mémoire, I/O, vidéo, orchestration
│  ├─ galaxian-audio/        modèle discret du son
│  ├─ galaxian-frontend/     application eframe/egui : fenêtre, texture du Frame, input, audio out (cpal) — hors cœur
│  └─ xtask/                 cargo xtask : api-dump, run-fuse, run-zex, mame-diff, test-roms, golden, progress-check, fetch-third-party
├─ .github/workflows/        ci.yml (fmt, clippy, test ; paquets système requis par eframe)
├─ .cargo/config.toml        alias `cargo xtask`
├─ tests/
│  ├─ roms/                  ROMs de test maison (sources .asm + binaires)
│  ├─ golden/                références MAME (traces, CRC de frames, WAV)
│  └─ third_party/           zexdoc/zexall, z80test, fuse, singlestep (téléchargés par xtask)
├─ roms/                     ROMs commerciales (NON versionnées, fournies par l'utilisateur)
├─ docs/kb/                  fiches KB (Partie E), une par fichier
├─ docs/mame_src/            sources MAME — DOCUMENTATION en lecture seule, non compilées
├─ out/<TASK_ID>/            sorties des agents : task_prompt.md, patch, verdict.md (voir A.4)
├─ reports/<TASK_ID>.json    rapports du Validateur (voir A.4)
└─ PROGRESS.json
```

Règles :
- Edition 2021+, `#![forbid(unsafe_code)]`, `clippy -D warnings`, `rustfmt`.
- Dépendances autorisées dans le cœur : `bitflags`, `thiserror`. Dev : `proptest`, `serde_json`, `crc32fast`. Dans `galaxian-frontend` **uniquement** : `eframe` (incluant `egui`), `cpal`, et `gilrs` (T6.2.2) ; versions épinglées en T0.1.4. Tout autre ajout = décision humaine.
- API CPU ↔ bus (imposée) :

```rust
pub trait Bus {
    /// Avance le temps de `tstates` cycles Z80 AVANT l'accès suivant.
    fn advance(&mut self, tstates: u32);
    fn mem_read(&mut self, addr: u16) -> u8;          // cycle M1 exclu
    fn mem_write(&mut self, addr: u16, val: u8);
    fn fetch_opcode(&mut self, addr: u16) -> u8;      // cycle M1 (4 T-states)
    fn io_read(&mut self, port: u16) -> u8;
    fn io_write(&mut self, port: u16, val: u8);
    fn nmi_pending(&mut self) -> bool;                // front montant consommé par le CPU
    fn int_line(&self) -> bool;
}
```
- Chaque phase produit une **fonction `snapshot()`** (état sérialisable) pour comparer aux références.
- Tout test de comparaison renvoie la **première divergence** (cycle + contexte).

---

# PARTIE D — Stratégie de validation

## D.1 Honnêteté sur les « ROMs de test »

Il **n'existe pas, à ma connaissance, de suite de ROMs de test dédiée au hardware Galaxian** (type « test ROM » publique de la borne). La validation repose donc sur **4 familles**, que le système doit mettre en place :

| Famille | Quoi | Sert à valider |
|---|---|---|
| **V1 — Suites CPU publiques** | `zexdoc`/`zexall` (CP/M .COM, nécessite un mini-shim BDOS fonction 2 et 9), `z80test` (Patrik Rak : z80doc, z80full, z80ccf, z80memptr), tests de la suite **Fuse** (`tests.in/tests.expected` avec événements de bus MR/MW/PR/PW par T-state), éventuellement `SingleStepTests/z80` (JSON par opcode avec bus cycle par cycle) — *vérifier la disponibilité/format actuels avant usage* | Z80 : fonctionnel, flags documentés/non documentés, MEMPTR, timing par M-cycle |
| **V2 — ROMs de test maison** (« TestROM Factory », étape 0.6) | Petits programmes Z80 écrits en assembleur, ciblant **un seul** mécanisme (NMI, watchdog, VRAM, sprites, étoiles, flip…), à fournir en `.bin` 16 Ko | Chaque bloc matériel isolément, de façon déterministe |
| **V3 — Références MAME (golden)** | MAME utilisé comme **oracle externe en boîte noire** (AD-03), lancé en ligne de commande avec scripts Lua : trace CPU (`trace` du debugger), CRC/snapshots d'écran à la frame N, `-wavwrite` | Comparaison cycle/frame par frame de l'émulateur complet |
| **V4 — ROM commerciale Galaxian** | ROMs du set MAME `galaxian` / `galmidw` fournies par l'utilisateur (non redistribuées) | Intégration : boot, attract mode, partie, sons |

## D.2 Génération des références MAME (à scripter dans `xtask golden`)

Options MAME utiles : `-autoboot_script <lua>`, `-seconds_to_run N`, `-nothrottle`, `-video none` / `-sound none` selon le cas, `-wavwrite <fichier>`, `-aviwrite`, `-snapshot_directory`. Trace CPU via la commande debugger `trace <fichier>,maincpu,noloop[,{tracelog "..."}]` (lancée via `-debug` + `-debugscript`). Snapshot d'écran depuis Lua : `manager.machine.screens[":screen"]:snapshot(...)`. *Les noms exacts d'API Lua varient selon la version de MAME : la tâche T0.5.1 impose de les vérifier sur la version installée.*

Livrables de références à produire (étape 0.5) :
- `golden/trace_boot_<N>.log` : N premières instructions (PC, opcode, AF BC DE HL SP, cycles) après reset, ROM Galaxian.
- `golden/frames_crc.txt` : CRC32 de l'écran visible (256×224) aux frames 1, 2, 5, 10, 30, 60, 120, 300, 600.
- `golden/audio_<scenario>.wav`.
- Pour chaque ROM de test maison : trace + CRC de frames.

## D.3 Niveaux de tolérance

| Domaine | Critère |
|---|---|
| Z80 | **Exact** : registres, flags (dont X/Y), MEMPTR, T-states par instruction, et ordre/instants des accès bus |
| Timing système | **Exact** : cycle d'entrée en VBLANK, cycle de l'NMI, nombre de cycles/frame (50 688) |
| Vidéo | **Pixel-exact** (CRC identique à MAME), dont étoiles et clipping |
| Audio | **Non bit-exact** (limite du modèle discret, cf. KB-17). Critères : corrélation spectrale et enveloppe vs WAV MAME avec seuils fixés en T5.5.x |

---

# PARTIE E — BASE DE CONNAISSANCES (KB)

> Chaque fiche est autonome. L'Orchestrateur n'injecte que celles listées dans `context`. Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h) **utilisé comme documentation seulement** (AD-01), périmètre **Galaxian d'origine** uniquement. Les extraits C++ des fiches décrivent le matériel ; ils ne constituent pas une structure à reproduire.

## KB-01 — Vue d'ensemble
Un PCB, trois sections : **CPU** (Z80, ROM jusqu'à 16 Ko + RAM 2 Ko décodée), **Son** (circuit discret analogique : compteur programmable + 4× 555 + bruit LFSR), **Vidéo** (tilemap de caractères + sprites + missiles/shells + champ d'étoiles ; LFSR 17 bits partagé avec le son). Des schémas existent et ont servi de base à MAME.

## KB-02 — Horloges et timing (valeurs de référence)

```
XTAL maître            = 18.432 MHz
Z80 (XTAL/6)           = 3.072 MHz
Pixel (XTAL/3)         = 6.144 MHz   (HSYNC = XTAL/3/192/2 = 16 kHz  → 384 pixels/ligne)
VSYNC                  = HSYNC/132/2 = 60.606060 Hz
VBlank                 ≈ 2500 µs
```
Constantes MAME (galaxian.h) :
```cpp
GALAXIAN_XSCALE  = 3                       // rendu interne ×3 horizontal (indispensable pour les étoiles)
GALAXIAN_HTOTAL  = 384 * XSCALE ; HBEND = 0 ; H0START = 0 ; HBSTART = 256 * XSCALE
GALAXIAN_VTOTAL  = 264 ; VBEND = 16 ; VBSTART = 224 + 16 (=240)
```
**Valeurs dérivées (à vérifier par les tests T2.1.x)** :
- Cycles Z80 par ligne : 3 072 000 / 16 000 = **192**.
- Lignes par frame : **264** → **50 688 cycles/frame** (3 072 000 / 60,606 = 50 688, entier).
- Zone visible : lignes 16..239 (224 lignes), pixels 0..255 (256) ; HBLANK pour pixels 256..383 (normalisé).
- Le facteur ×3 découle du rapport 3:2 entre horloge maître et horloge pixel pilotant le générateur d'étoiles (KB-12).

## KB-03 — Carte mémoire (décodage matériel)
```
0000-3fff : ROM programme
4000-47ff : RAM (0x400 décodé, mirroré)
4800-4fff : non connecté
5000-57ff : VRAM
5800-5fff : OBJRAM (sprites/attributs)
6000-67ff : /SW0 (lecture) ou /DRIVER (écriture)
6800-6fff : /SW1 (lecture) ou /SOUND (écriture)
7000-77ff : /DIPSW (lecture) ou LATCH (écriture)
7800-7fff : /WDR watchdog (lecture) ou /PITCH son (écriture)
```
Implémentation MAME de référence :
```cpp
map.unmap_value_high();                        // lectures non mappées → 0xFF
map(0x0000,0x3fff).rom();
map(0x4000,0x43ff).mirror(0x0400).ram();
map(0x5000,0x53ff).mirror(0x0400).ram().w(galaxian_videoram_w).share("videoram");
map(0x5800,0x58ff).mirror(0x0700).ram().w(galaxian_objram_w).share("spriteram");
map(0x6000,0x6000).mirror(0x07ff).portr("IN0");
map(0x6000,0x6001).mirror(0x07f8).w(start_lamp_w);
map(0x6002,0x6002).mirror(0x07f8).w(coin_lock_w);
map(0x6003,0x6003).mirror(0x07f8).w(coin_count_0_w);
map(0x6004,0x6007).mirror(0x07f8).w("cust", lfo_freq_w);
map(0x6800,0x6800).mirror(0x07ff).portr("IN1");
map(0x6800,0x6807).mirror(0x07f8).w("cust", sound_w);
map(0x7000,0x7000).mirror(0x07ff).portr("IN2");
map(0x7001,0x7001).mirror(0x07f8).w(irq_enable_w);
map(0x7004,0x7004).mirror(0x07f8).w(galaxian_stars_enable_w);
map(0x7006,0x7006).mirror(0x07f8).w(galaxian_flip_screen_x_w);
map(0x7007,0x7007).mirror(0x07f8).w(galaxian_flip_screen_y_w);
map(0x7800,0x7800).mirror(0x07ff).r("watchdog", reset_r);
map(0x7800,0x7800).mirror(0x07ff).w("cust", pitch_w);
```
Points d'attention : VRAM = 0x400 octets utiles (mirror 0x400) ; OBJRAM = 0x100 octets (mirror 0x700) ; les écritures VRAM/OBJRAM passent par des fonctions qui **mettent à jour le rendu** (cf. KB-09/10).

## KB-04 — Registres d'écriture (1 bit par adresse, D0)

`/DRIVER` (6000-6007, A0-A2 = adresse du bit) :

| Adr | Fonction | | Adr | Fonction |
|---|---|---|---|---|
| 6000 | lampe 1P START | | 6004 | résistance 1M (555 @9R) |
| 6001 | lampe 2P START | | 6005 | 470k |
| 6002 | COIN LOCKOUT | | 6006 | 220k |
| 6003 | COIN COUNTER | | 6007 | 100k |

`/SOUND` (6800-6807) : 6800 FS1 · 6801 FS2 · 6802 FS3 · 6803 HIT · 6804 n/c · 6805 FIRE · 6806 VOL1 · 6807 VOL2.

`LATCH` (7000-77ff) : 7001 **NMI ON** · 7004 **STARS ON** · 7006 **HFLIP** · 7007 **VFLIP**. (7000, 7002, 7003, 7005 : non utilisés dans la doc fournie.)

Fonctions I/O :
```cpp
start_lamp_w(offset,d): m_lamps[offset] = d & 1;
coin_lock_w(d):  coin_lockout_global_w(~d & 1);
coin_count_0_w(d): coin_counter_w(0, d & 1);
```

## KB-05 — Interruptions
```cpp
void vblank_interrupt_w(int state) {            // appelée sur changement de VBLANK
    if (state && m_irq_enabled) m_maincpu->set_input_line(m_irq_line, ASSERT_LINE);
}
void irq_enable_w(uint8_t data) {               // écriture à 7001
    m_irq_enabled = data & 1;
    if (!m_irq_enabled) m_maincpu->set_input_line(m_irq_line, CLEAR_LINE);
}
```
- `m_irq_line = INPUT_LINE_NMI` par défaut (galaxian.h) → **NMI Z80**.
- Une bascule (6F) est maintenue en preset par « NMI ON » ; tant que le jeu n'a pas écrit 1 à 7001, rien ne se produit.
- Fréquence : 60,6 Hz, à l'entrée en VBLANK (début de la ligne 240 selon VBSTART — **à confirmer par trace MAME**, T2.5.7).
- Le Z80 prend l'NMI sur front : l'émulateur doit modéliser la ligne comme un niveau et détecter le **front montant**.

## KB-06 — Watchdog
Lecture de 7800 (mirroré sur 0x07ff) = reset du watchdog. Absence de lecture pendant la durée de timeout → reset matériel. **La durée du timeout n'est pas dans la doc fournie** (GAP-03).

## KB-07 — Timing vidéo détaillé (compteurs matériels)
Horizontal : compteur H de 128 à 511 (bit fort inversé = 256H), normalisé `000000000→011111111` (actif, 256 px) puis `110000000→111111111` (blanking).
- **HBLANK** : bascule cadencée par 2H, D = `!(64H & 32H & 16H & 8H)` → 1 à H=130, 0 à H=250 → 264 px non blankés (6 px à gauche H=250-255, 256 px principaux, 2 px à droite H=128-129).
- **HSYNC** : bascule cadencée par 16H, D = `!(!64H & 32H)`, /Q → 1 à H=176, 0 à H=208.

Vertical : compteur V de 248 à 511 (264 clocks).
- **La chaîne V est cadencée par HSYNC (pas H)** → pendant les 48 premiers clocks H du blanking, V est **en retard d'un cran** (impacte positionnement exact sprites/missiles).
- **VBLANK** : cadencée par 16V, D = `!(128V & 64V & 32V)` → 1 à V=496, 0 à V=272 → 224 px visibles.
- **VSYNC** = `!256V` → 1 à V=248, 0 à V=256.

## KB-08 — Palette / PROM
Réseau de résistances (PROM 8 bits par entrée) :
```
bit7 -220Ω- BLEU   bit3 -220Ω- VERT
bit6 -470Ω- BLEU   bit2 -1kΩ - VERT
bit5 -220Ω- VERT   bit1 -470Ω- ROUGE
bit4 -470Ω- VERT   bit0 -1kΩ - ROUGE
```
(Transcription fidèle de la doc fournie ; l'affectation exacte bit→composante est à **re-vérifier dans galaxian_v.cpp** (T0.4.12), GAP-02 : la doc liste « VERT » deux fois pour les bits 5 et 4 alors que le schéma standard est R: bits 0-2, G: bits 3-5, B: bits 6-7.)
```cpp
static const int rgb_resistances[3] = {1000, 470, 220};
compute_resistor_weights(0, RGB_MAXIMUM, -1.0,
   3,&rgb_resistances[0],rweights,470,0,
   3,&rgb_resistances[0],gweights,470,0,
   2,&rgb_resistances[1],bweights,470,0);
```
`RGB_MAXIMUM = 224` (marge pour étoiles/shells). En parallèle : paire 150Ω/100Ω par composante pour les étoiles ; résistance 100Ω pour shells/missile. Couleurs de bullets : 7 blanches `(255,255,255)`, la dernière jaune `(255,255,0)`.
Fond : **noir uni** + étoiles.

## KB-09 — Tilemap (fond) *(partiellement hors doc — voir GAP-01)*
Informations de la doc : `m_bg_tilemap` avec flips X/Y ; les **7 premières « entrées » OBJRAM = shells, la dernière = missile** ; `m_sprites_base = 0x40`, `m_bullets_base = 0x60`.
À extraire de `galaxian_v.cpp` (T0.4.3–T0.4.6) : format de `galaxian_videoram_w` / `galaxian_objram_w`, rôle des 0x40 premiers octets d'OBJRAM (attribut par colonne : scroll vertical + couleur), taille de tile (8×8, 32×32), décodage GFX 2 bitplanes depuis les ROM caractères.

## KB-10 — Sprites : mécanisme cycle-exact
Pendant HBLANK jusqu'à **8 sprites** traités (≈ **7,5** rendus complètement : le setup démarre après le début du HBLANK). 8 clocks H de setup, 16 de rendu dans un **line buffer**, phases chevauchées.

OBJRAM :
```
objram[0x40] = Y sprite 0      objram[0x41] = n° image + flip H/V
objram[0x42] = couleur (3 bits bas)   objram[0x43] = X sprite 0
objram[0x61] = Y "shell" 0     objram[0x63] = compteur H avant démarrage du rendu shell 0
```
Matches :
- sprite : `((V + vpos) & 0xf0) == 0xf0`
- shell/missile : `((V + vpos) & 0xff) == 0xff`

Signaux de contrôle (H = compteur horizontal) :

| Signal | Condition | Fonction |
|---|---|---|
| /VPL | `xxxxxx000` | latche V+HPOSI |
| /COL L | `xxxxxx100` | latche HPOSI → couleur (3 bits bas) |
| /LD | `xxxxxx111` | charge reg. à décalage depuis ROM |
| /CNTR CLR | `0xxxx0111` | reset compteur line buffer |
| /OBJ DATA L | `1xxxx0010` | latche n° image |
| /CNTR LD | `1xxxx0111` | latche X (compteur line buffer) |
| /SLD | `1xxxx1111` | latche compteur shell (sauf si /MLD) |
| /MLD | `1x1111111` | latche compteur missile |

Priorité/clipping :
- Le line buffer n'est écrit **que s'il contient '0'** → le premier sprite écrit gagne ; MAME rend à l'envers (7→0) pour que les n° faibles aient priorité.
- **16 des 256 pixels** sont hard-clippés au niveau du line buffer :
```cpp
void sprites_clip(screen, cliprect) {
  clip = screen.visible_area();
  if (m_flipscreen_x) clip.max_x = (256 - (16 + 1)) * m_x_scale - 1;
  else                clip.min_x = ((16 + 1) * m_x_scale);
  cliprect &= clip;
}
```

## KB-11 — Missiles / shells
- 1 seul compteur « shell » + 1 « missile » → **un shell et un missile par ligne max** ; en cas de multiples matches, **le dernier trouvé gagne**.
- Début d'affichage quand le compteur H atteint `$FC`, fin à `$00` → **4 pixels de long**.
```cpp
x -= 4;
draw_pixel(y, x++, color[offs]); ×4   // 4 pixels consécutifs
```

## KB-12 — Générateur d'étoiles (LFSR 17 bits)
```cpp
#define STAR_RNG_PERIOD ((1 << 17) - 1)   // 131071
for (i = 0; i < STAR_RNG_PERIOD; i++) {
  enabled = ((shiftreg & 0x1fe01) == 0x1fe00);        // bits 16..9 = 1, bit 0 = 0
  color   = (~shiftreg & 0x1f8) >> 3;                 // 6 bits
  stars[i] = color | (enabled << 7);
  shiftreg = (shiftreg >> 1) | ((((shiftreg >> 12) ^ ~shiftreg) & 1) << 16);
}
```
- **Même LFSR** que le bruit sonore (KB-15) : un seul circuit.
- Horloge LFSR = horloge maître **ET** pixel → duty 2/3 : **3 clocks maîtres pour 2 clocks RNG** ; le rendu (`stars_draw_row`) produit 1 pixel pour le 1er clock RNG, 2 pixels pour le 2ᵉ (d'où ×3).
- Étoile visible seulement si `(V1 XOR H8) == 1` (damier).
- Période `2^17 − 1` (≠ `2^17`) → l'origine **dérive d'un cran par frame** → scrolling continu. En mode non flippé, une paire de bascules D en 6B retarde le comptage de 2 clocks/frame supplémentaires.
- `stars_update_origin()` doit être appelée **avant** tout changement de flip.
- Étoiles actives seulement si STARS ON (7004).
```cpp
draw_background: bitmap.fill(black); draw_stars(bitmap, cliprect, 256);
```

## KB-13 — Flip screen
```cpp
void flip_screen_x_w(d) {
  if (m_flipscreen_x != (d & 1)) {
    m_screen->update_partial(m_screen->vpos());   // rendre jusqu'à la ligne courante AVANT le changement
    stars_update_origin();                        // recaler l'origine des étoiles AVANT
    m_flipscreen_x = d & 1;
    m_bg_tilemap->set_flip((fx?FLIPX:0)|(fy?FLIPY:0));
  }
}
```
Le nombre de clocks comptés par frame diffère selon le sens de balayage ; sans recalage, les étoiles se désynchronisent.

## KB-14 — Rendu : ordre de composition (déduit de la doc, à confirmer T4.6.x)
1. fond noir ; 2. étoiles ; 3. tilemap ; 4. sprites (ordre de priorité : n° faible devant) ; 5. shells/missile. L'ordre exact de superposition étoiles/tilemap/sprites/shells est à **valider contre MAME** (GAP-01).

## KB-15 — Son : vue d'ensemble (circuit documenté via le moteur DISCRETE de MAME)
> Le graphe DISCRETE ne sert qu'à documenter le circuit (valeurs R/C, topologie). `galaxian-audio` implémente son propre modèle en Rust (AD-01), sans reproduire le moteur DISCRETE.
Graphe de nœuds reproduisant le circuit réel (555, RC, DAC résistif, LFSR, mixeurs) avec valeurs R/C relevées sur schéma.
```cpp
#define SOUND_CLOCK (GALAXIAN_MASTER_CLOCK/6/2)   // 1.536 MHz
#define RNG_RATE    (GALAXIAN_MASTER_CLOCK/3*2)   // 12.288 MHz
```
Entrées logiques (latch 74LS259) :
```
GAL_INP_BG_DAC=NODE_10 (9M Q4-Q7, 4 bits) | FS1=NODE_20 | FS2=NODE_21 | FS3=NODE_22
HIT=NODE_23 | FIRE=NODE_25 | VOL1=NODE_26 | VOL2=NODE_27 | PITCH=NODE_28 (latch 8 bits)
```
Interface :
```cpp
pitch_w(d)               // 7800 : latch 8 bits
sound_w(offset,d)        // 6800-6807
  case 0,1,2: background_enable_w(offset, d)  // FS1/2/3
  case 3: noise_enable_w(d)                   // HIT
  case 4: n/c
  case 5: fire_enable_w(d)                    // FIRE
  case 6,7: vol_w(offset & 1, d)              // VOL1/2
lfo_freq_w(offset,d)     // 6004-6007 : m_lfo_val bit `offset` ← d&1 ; si changé → discrete.write(NODE_10, m_lfo_val)
```

## KB-16 — Son : blocs du graphe
**NOISE (LFSR partagé)**
```cpp
galaxian_lfsr = { DISC_CLK_IS_FREQ, 17 bits, reset 0,
   feedback taps: bit 4 et bit 16, F0 = XOR_INV_IN1, F1 = IN0, F2 = REPLACE,
   shift into bit 0, OUTPUT_F0 };
NODE_150 = LFSR_NOISE(rate = RNG_RATE/100)
NODE_151 = SQUAREWFIX(60*264/2 Hz, duty 0.5)      // signal "2V"
NODE_152 = DFLIPFLOP(clk=NODE_151, d=NODE_150)    // LFSR échantillonné par 2V
```
**BACKGROUND (bourdonnement)** : DAC R-2R 4 bits (1k/470/220/100 kΩ, bias 15k, gnd 330k, Vref 4,4 V) → 555 VCO (R21=100k, C15=1µF) → op-amp mult/add (R31/R32/R33, −5·R33/R31) → clamp 0..5 V → 3× 555 astable à contrôle de tension :
```
FS1: R22=100k,R23=470k,C17=0.01µF | FS2: R25=100k,R26=330k,C18 | FS3: R28=100k,R29=220k,C19
→ MIXER3 (NODE_120)
```
**PITCH** : deux LS164 cascadés rechargés par le latch pitch → fréquence `SOUND_CLOCK/(256 − pitch)` ; seuls QA, QC, QD du 74393 utilisés (`BITS_DECODE(NODE_133, 0..3)`).
**HIT** : `RCDISC5(NODE_155, noise NODE_152, enable HIT, R=R35+R36=150k+22k, C21=2.2µF)` → filtre passe-bande op-amp (R35=150k, R36=22k, R37=470k, C22/C23=0.01µF). *Note devs MAME : le son HIT est trop faible vs enregistrements.*
**FIRE** : `NODE_170 = !FIRE ; NODE_171 = 5V·FIRE ; NODE_172 = 5V·NODE_170 ; NODE_173 = RC(R47=2.2k, C28=47µF)` ; mélange bruit + RC (R46, R48) → 555 VCO (R44=10k, R45=22k, C27=0.01µF) → `RCDISC5(R41=100k, C25=1µF)` = impulsion de durée fixe.
**MIXAGE**
```cpp
NODE_279 = MIXER5(133_00, 133_02, 133_02, 133_03, NODE_120)
NODE_280 = MIXER3(NODE_279, NODE_157(hit), NODE_182(fire))
OUTPUT(NODE_280, 32767/5*5)
```

## KB-17 — Son : limites connues
Le CD4066 (commutateur analogique) rend dynamique l'impédance d'entrée de certains filtres (→ >> 10 MΩ) ; le moteur DISCRETE utilise des résistances statiques → **non reproductible exactement**. Écarts connus : bourdonnement trop rapide, HIT trop faible. **Conséquence : la validation audio ne peut pas être bit-exact.**

## KB-18 — Pas de protection
Galaxian d'origine : aucune puce de protection, pas de chiffrement ROM. Seul mécanisme : watchdog.

## KB-19 — Check-list d'émulation (points de vigilance)
1. Lire 7800 régulièrement (watchdog). 2. Pas d'NMI avant écriture de 7001. 3. NMI exactement au début du VBLANK. 4. Coin lockout/counter câblés. 5. `stars_update_origin()` avant flip. 6. Rendu interne ×3 horizontal pour les étoiles. 7. **Un seul** LFSR partagé son/vidéo.

## KB-20 — État C++ de référence (galaxian_state) — *documentaire uniquement*
> Liste des états matériels à modéliser. L'organisation de l'état en Rust est libre (AD-01) et définie par les tâches ; ne pas calquer cette structure.
```cpp
int  m_bullets_base = 0x60;  int m_sprites_base = 0x40;
uint8_t m_irq_enabled = 0;   int m_irq_line = INPUT_LINE_NMI;
uint8_t m_x_scale = GALAXIAN_XSCALE;  uint8_t m_h0_start = GALAXIAN_H0START;
tilemap_t *m_bg_tilemap;  uint8_t m_flipscreen_x, m_flipscreen_y;
uint32_t m_star_rng_origin, m_star_rng_origin_frame;
rgb_t m_star_color[64];  std::unique_ptr<uint8_t[]> m_stars;  uint8_t m_stars_enabled;
rgb_t m_bullet_color[8];
```
Délégués (tile-info étendu, bullets, background) pointent vers les versions de base pour Galaxian d'origine (aucun banking GFX, aucun fond coloré).

## KB-21 — Spécifications Z80 (à fournir en complément, en 8 fiches)
La doc fournie **ne contient pas** la référence Z80 (GAP-04). Elle est découpée en fiches courtes, rédigées en Phase 0 (étape 0.3) à partir de *The Undocumented Z80 Documented* (Sean Young), des tableaux Zilog et de sources externes, afin que chaque tâche T1.x ne reçoive que la partie utile :
**KB-21a** registres/flags · **KB-21b1** timing 00–7F · **KB-21b2** timing 80–FF · **KB-21c** préfixe CB · **KB-21d** DD/FD/DDCB/FDCB · **KB-21e** ED · **KB-21f** MEMPTR et Q · **KB-21g** interruptions/HALT/EI.
Les fiches `KB-22a/b`, `23a/b`, `24a/b`, `25a/b`, `26`, `27` sont produites en étape 0.4 (extraction depuis `docs/mame_src/`).

---

# PARTIE F — PLAN : phases → étapes → sous-étapes

Légende : `ctx` = fiches KB à injecter (surcharge du `ctx par défaut` de l'étape ; `ctx+` = ajout à ce défaut) ; `livr` = livrables ; `val` = commande de validation (surcharge de la `val par défaut` de l'étape). **Dépendances implicites** : une tâche dépend de la précédente de l'ordre **et** des tâches qui produisent ses fiches KB (`PROGRESS.json` les contient, `cargo xtask progress-check` les vérifie). Chaque sous-étape respecte le **budget B.6**.

## F.0 Ordre d'exécution canonique (`order`)

```
PHASE 0 — Fondations
  Étape 0.1  T0.1.1 → T0.1.9      (9 tâches) Squelette du dépôt
  Étape 0.2  T0.2.1 → T0.2.4      (4 tâches) Types de temps
  Étape 0.3  T0.3.1 → T0.3.8      (8 tâches) Fiches Z80 (comble GAP-04)
  Étape 0.4  T0.4.1 → T0.4.13     (13 tâches) Comblement des lacunes par lecture de la documentation MAME (GAP-01, 02, 03, 05)
  Étape 0.5  T0.5.1 → T0.5.10     (10 tâches) Références MAME (golden)
  Étape 0.6  T0.6.1 → T0.6.5      (5 tâches) TestROM Factory
PHASE 1 — CPU Z80
  Étape 1.1  T1.1.1 → T1.1.19     (19 tâches) Registres, flags, bus, cycles, harnais Fuse
  Étape 1.2  T1.2.1 → T1.2.18     (18 tâches) Chargements et ALU 8 bits (non préfixé)
  Étape 1.3  T1.3.1 → T1.3.8      (8 tâches) 16 bits, pile, échanges, rotations de A
  Étape 1.4  T1.4.1 → T1.4.11     (11 tâches) Flux de contrôle et divers
  Étape 1.5  T1.5.1 → T1.5.10     (10 tâches) Préfixe CB
  Étape 1.6  T1.6.1 → T1.6.13     (13 tâches) Préfixes DD / FD
  Étape 1.7  T1.7.1 → T1.7.19     (19 tâches) Préfixe ED
  Étape 1.8  T1.8.1 → T1.8.8      (8 tâches) Interruptions
  Étape 1.9  T1.9.1 → T1.9.20     (20 tâches) Validation CPU globale
PHASE 2 — Horloge, bus, NMI
  Étape 2.1  T2.1.1 → T2.1.8      (8 tâches) Horloge et balayage
  Étape 2.2  T2.2.1 → T2.2.9      (9 tâches) Mémoire et bus machine
  Étape 2.3  T2.3.1 → T2.3.9      (9 tâches) Registres d'E/S
  Étape 2.4  T2.4.1 → T2.4.5      (5 tâches) Watchdog
  Étape 2.5  T2.5.1 → T2.5.8      (8 tâches) NMI VBLANK
PHASE 3 — Machine bootable
  Étape 3.1  T3.1.1 → T3.1.8      (8 tâches) Chargement des ROMs
  Étape 3.2  T3.2.1 → T3.2.4      (4 tâches) Boucle principale
  Étape 3.3  T3.3.1 → T3.3.4      (4 tâches) Trace contre MAME
  Étape 3.4  T3.4.1 → T3.4.3      (3 tâches) Dump mémoire contre MAME
PHASE 4 — Vidéo
  Étape 4.1  T4.1.1 → T4.1.8      (8 tâches) Graphismes
  Étape 4.2  T4.2.1 → T4.2.13     (13 tâches) Framebuffer et tilemap
  Étape 4.3  T4.3.1 → T4.3.13     (13 tâches) Sprites (modèle line buffer)
  Étape 4.4  T4.4.1 → T4.4.6      (6 tâches) Shells et missile
  Étape 4.5  T4.5.1 → T4.5.13     (13 tâches) Étoiles
  Étape 4.6  T4.6.1 → T4.6.5      (5 tâches) Composition finale
PHASE 5 — Audio
  Étape 5.1  T5.1.1 → T5.1.2      (2 tâches) LFSR partagé
  Étape 5.2  T5.2.1 → T5.2.5      (5 tâches) Registres son
  Étape 5.3  T5.3.1 → T5.3.9      (9 tâches) Primitives discrètes
  Étape 5.4  T5.4.1 → T5.4.12     (12 tâches) Blocs du graphe
  Étape 5.5  T5.5.1 → T5.5.7      (7 tâches) Validation audio
PHASE 6 — Intégration et frontend
  Étape 6.1  T6.1.1 → T6.1.3      (3 tâches) Affichage et temps (eframe/egui)
  Étape 6.2  T6.2.1 → T6.2.3      (3 tâches) Entrées
  Étape 6.3  T6.3.1 → T6.3.3      (3 tâches) Audio
  Étape 6.4  T6.4.1 → T6.4.6      (6 tâches) Scénarios de non-régression
PHASE 7 — Durcissement
  Étape 7.1  T7.1.1 → T7.1.2      (2 tâches) Déterminisme
  Étape 7.2  T7.2.1 → T7.2.5      (5 tâches) Save-states
  Étape 7.3  T7.3.1 → T7.3.2      (2 tâches) Performance
  Étape 7.4  T7.4.1 → T7.4.3      (3 tâches) Fuzz / proptest
  Étape 7.5  T7.5.1 → T7.5.2      (2 tâches) Documentation finale
```
Total : **355 sous-étapes**, dans l'ordre numérique croissant (phase, étape, sous-étape), sans exception.

Choix d'ordonnancement :
- L'ordre est **l'ordre numérique** des identifiants : plus de cas particulier. La trace MAME sur un million d'instructions (ancien T1.4.4) est donc en Phase 3 (T3.3.4), après `run_frame`.
- Les fiches KB manquantes (Z80, MAME) sont produites en Phase 0 (étapes 0.3 et 0.4) **avant** les tâches de code qui les utilisent.
- La vidéo (phase 4) passe avant l'audio (phase 5) ; dans la phase 4 : graphismes → tilemap → sprites → shells → étoiles → composition.
- Tout changement de cet ordre est une décision humaine.

---

## PHASE 0 — Fondations

### Étape 0.1 — Squelette du dépôt
> validation par défaut : `cargo build --workspace && cargo test --workspace` (vert)

- **T0.1.1 — Workspace Cargo vide.** ctx: — · livr: `Cargo.toml` (workspace) + 5 crates (`z80`, `galaxian-core`, `galaxian-audio`, `galaxian-frontend`, `xtask`) avec `lib.rs`/`main.rs` vides, édition 2021.
- **T0.1.2 — Lints et formatage.** livr: `#![forbid(unsafe_code)]` dans chaque crate, `rustfmt.toml`, `clippy.toml` · val: `cargo fmt --check && cargo clippy --workspace -- -D warnings`.
- **T0.1.3 — Intégration continue.** livr: `.github/workflows/ci.yml` (fmt, clippy, test) ; le job Linux installe les paquets système requis par `eframe` (X11/Wayland, xkbcommon, OpenGL) avant le build · val: le fichier de CI exécute localement les 3 commandes sans erreur.
- **T0.1.4 — Dépendances autorisées.** livr: `Cargo.toml` des crates : `bitflags`, `thiserror` (cœur) ; dev : `proptest`, `serde_json`, `crc32fast` ; `galaxian-frontend` : `eframe` (egui), `cpal` (versions épinglées, `default-features` revus par l'humain) ; alias `cargo xtask` dans `.cargo/config.toml`. Aucune dépendance graphique/audio dans `z80`, `galaxian-core`, `galaxian-audio`.
- **T0.1.5 — KB en fichiers.** ctx: KB-01…KB-20 (copie littérale) · livr: `docs/kb/KB-01.md` … `KB-20.md`, un par fichier · val: 20 fichiers présents, contenu identique à la Partie E.
- **T0.1.6 — xtask : squelette CLI.** livr: `crates/xtask/src/main.rs` : sous-commandes stub `api-dump`, `run-fuse`, `run-zex`, `mame-diff`, `test-roms`, `golden`, `progress-check`, `fetch-third-party` (chacune affiche « non implémenté », code retour 0) · val: `cargo xtask --help`.
- **T0.1.7 — xtask : api-dump.** livr: `xtask/src/api_dump.rs` · spec: liste stable (triée) des signatures publiques de chaque crate (outil au choix de l'humain : `cargo public-api` ou `rustdoc --output-format json`) · val: `cargo xtask api-dump` sur un crate contenant 1 fonction publique la liste.
- **T0.1.8 — xtask : progress-check.** livr: `xtask/src/progress.rs` · spec: refuse un `PROGRESS.json` avec >1 `IN_PROGRESS`, un doublon dans `order`, une tâche `DONE`/`IN_PROGRESS` dont une dépendance déclarée n'est pas `DONE`, une dépendance placée après sa tâche dans `order`, `current_task` incohérent · tests: 5 JSON invalides + 1 valide.
- **T0.1.9 — PROGRESS.json initial.** livr: `PROGRESS.json` (fourni avec ce document, toutes tâches `TODO`) · val: `cargo xtask progress-check` OK.

### Étape 0.2 — Types de temps
> ctx par défaut : KB-02 · livr: `galaxian-core/src/time.rs` · val: `cargo test -p galaxian-core time`

- **T0.2.1 — Constantes de temps.** `Z80_HZ=3_072_000`, `CYCLES_PER_LINE=192`, `LINES_PER_FRAME=264`, `CYCLES_PER_FRAME=50_688`, `PIXELS_PER_LINE=384` · tests: 192×264 = 50 688 ; chaque constante commentée `// KB-02`.
- **T0.2.2 — Type `Cycles` et décomposition.** `type Cycles = u64` ; `fn split(c: Cycles) -> (frame, ligne, cycle_dans_ligne)` · tests: cycle 0, 191, 192, 50 687, 50 688.
- **T0.2.3 — Cycle → pixel.** `pixel = 2 × cycle_dans_ligne` (6,144 MHz / 3,072 MHz) ; `fn pixel_of(cycle_in_line) -> u16` · tests: 0→0, 191→382, 192 interdit (appartient à la ligne suivante).
- **T0.2.4 — (ligne, pixel) → cycle.** inverse exact ; ligne 240 = cycle 46 080 · tests: aller-retour sur toutes les lignes (propriété).

### Étape 0.3 — Fiches Z80 (comble GAP-04)
> Tâches **documentaires** : l'humain fournit les sources (*The Undocumented Z80 Documented*, tableaux Zilog). Livrable = une fiche par fichier dans `docs/kb/`. val: revue humaine ; chaque ligne cite sa source (chapitre/page).

- **T0.3.1 — KB-21a Registres et flags.** bits S Z Y H X PV N C, règles de calcul de H/PV/X/Y pour 8 et 16 bits, parité.
- **T0.3.2 — KB-21b1 Timing opcodes non préfixés 00–7F.** M-cycles, T-states, accès bus par opcode.
- **T0.3.3 — KB-21b2 Timing opcodes non préfixés 80–FF.**
- **T0.3.4 — KB-21c Préfixe CB.** timings, flags, `BIT n,(HL)` (X/Y via WZ), SLL non documenté.
- **T0.3.5 — KB-21d Préfixes DD/FD et DDCB/FDCB.** timings, IXH/IXL, chaînes de préfixes, copie registre non documentée.
- **T0.3.6 — KB-21e Préfixe ED.** timings, instructions de bloc (flags X/Y, répétition à 21 T), ED non documentés.
- **T0.3.7 — KB-21f MEMPTR (WZ) et Q.** règle de mise à jour de WZ **par instruction**, règle de Q pour SCF/CCF.
- **T0.3.8 — KB-21g Interruptions, HALT, EI.** NMI 11 T, INT IM 0/1/2 (13/19 T), délai d'EI, interaction HALT, RETN/IFF.

### Étape 0.4 — Comblement des lacunes par lecture de la documentation MAME (GAP-01, 02, 03, 05)
> Tâches **documentaires** (MAME = documentation, AD-01) : chaque affirmation cite fichier + ligne de `docs/mame_src/`. val: revue humaine.

- **T0.4.1 — Récupérer les 5 sources MAME.** ctx: KB-01 · livr: `docs/mame_src/{galaxian.cpp,galaxian.h,galaxian_a.cpp,galaxian_a.h,galaxian_v.cpp}` (documentation en lecture seule, non compilés, non portés) · val: 5 fichiers présents.
- **T0.4.2 — Manifeste et index.** livr: `docs/mame_src/MANIFEST.md` : SHA256 + version/commit MAME + index « fonction → ligne » · val: SHA256 recalculés = manifeste.
- **T0.4.3 — KB-22a Layout GFX tiles.** `GFXDECODE`/`GfxLayout` des caractères (taille, planes, offsets, ROM source). Ferme une partie de GAP-01.
- **T0.4.4 — KB-22b Layout GFX sprites.** idem pour les sprites 16×16.
- **T0.4.5 — KB-23a VRAM et tile-info.** `galaxian_videoram_w`, `get_tile_info`, taille du tilemap, rôle des bits de code tile.
- **T0.4.6 — KB-23b OBJRAM et attributs par colonne.** `galaxian_objram_w`, 0x40 premiers octets (scroll vertical + couleur par colonne), zone sprites/shells.
- **T0.4.7 — KB-24a Entrées IN0/IN1/IN2.** `INPUT_PORTS_START` : bits, polarité (GAP-05).
- **T0.4.8 — KB-24b DIP switches.** bits, valeurs par défaut, effets.
- **T0.4.9 — KB-25a Set de ROMs.** `ROM_START(galaxian)` et `galmidw` : noms, tailles, offsets de chargement, CRC (GAP-03).
- **T0.4.10 — KB-25b Configuration machine.** CPU, `WATCHDOG_TIMER` (durée), écran, horloges (GAP-03).
- **T0.4.11 — KB-26 Ordre de rendu.** `screen_update` : ordre exact fond/étoiles/tilemap/sprites/shells.
- **T0.4.12 — KB-27 Palette PROM.** `galaxian_palette` : affectation exacte bit→R/G/B (GAP-02).
- **T0.4.13 — Clôture des lacunes.** ctx: KB-08, KB-09, KB-14 · tâche : corriger ces 3 fiches si les nouvelles fiches les contredisent ; passer GAP-01/02/03/05 à `FILLED` (avec `kb_file`) dans `PROGRESS.json`.

### Étape 0.5 — Références MAME (golden)
> val par défaut : fichiers golden présents ; régénération reproductible (2 runs = octets identiques). Les scripts MAME sont les mêmes en D.2.

- **T0.5.1 — Environnement MAME.** livr: `docs/mame_notes.md` : version installée, chemin ROMs, **syntaxe réellement vérifiée** de `trace`, de l'API Lua d'écran et de `-wavwrite`.
- **T0.5.2 — Lanceur MAME.** livr: `xtask golden run` : appelle MAME avec options communes (`-seconds_to_run`, `-nothrottle`, répertoires de sortie) · val: lancement à blanc, code retour 0.
- **T0.5.3 — Trace CPU brute.** livr: script debugger `trace` · val: fichier brut produit pour N = 1 000 instructions.
- **T0.5.4 — Trace CPU normalisée.** livr: `xtask golden trace` convertit en `golden/trace_boot_<N>.log` (PC, opcode, AF BC DE HL SP, cycles) ; N = 1 000 puis 100 000 puis 1 000 000 · val: 3 fichiers, formats identiques.
- **T0.5.5 — CRC de frames (script Lua).** livr: Lua qui écrit le CRC32 de l'écran 256×224 aux frames 1, 2, 5, 10, 30, 60, 120, 300, 600 → `golden/frames_crc.txt`.
- **T0.5.6 — Dump VRAM/OBJRAM (script Lua).** livr: dump binaire VRAM + OBJRAM à la frame N → `golden/mem_dump_<N>.bin` (N = 60, 600).
- **T0.5.7 — Audio : attract.** livr: `golden/audio_attract.wav` (`-wavwrite`).
- **T0.5.8 — Audio : tir.** livr: Lua qui injecte coin + start + tir → `golden/audio_fire.wav`.
- **T0.5.9 — Audio : explosion.** livr: `golden/audio_hit.wav`.
- **T0.5.10 — Reproductibilité.** val: `xtask golden all` deux fois → SHA256 identiques ; consigner dans `golden/MANIFEST.md`.

### Étape 0.6 — TestROM Factory
- **T0.6.1 — Assembleur.** livr: intégration de l'assembleur Z80 **choisi par l'humain** (sjasmplus/z80asm ou mini-assembleur Rust) · val: `xtask test-roms build` assemble `tests/roms/hello/hello.asm`.
- **T0.6.2 — Gabarit commun.** livr: `tests/roms/common.inc` : vecteurs reset/NMI, pile, routine « verdict » (`0x4000` = `0xA5` pass / `0xEE` fail), lecture périodique de 7800.
- **T0.6.3 — Sortie 16 Ko.** livr: `.bin` de 16 384 octets, remplissage 0x00 ou 0xFF **documenté** · val: taille exacte.
- **T0.6.4 — Convention README.** livr: `tests/roms/README.md` : structure `.asm`/`.bin`/trace/CRC/README par ROM · val: relecture.
- **T0.6.5 — ROM TR-HELLO.** livr: ROM qui écrit `0xA5` en `0x4000` puis boucle · val: assemblage + vérif manuelle du binaire (exécuté plus tard, T3.x).

---

## PHASE 1 — CPU Z80 cycle-accurate

Principe : machine à états **par M-cycle** avec T-states explicites ; `bus.advance(n)` avant chaque accès. Référence de comportement : fiches KB-21x (T0.3.x). **Chaque sous-étape d'instructions = au plus ~6 opcodes distincts ou une famille cohérente.**

### Étape 1.1 — Registres, flags, bus, cycles, harnais Fuse
> val par défaut : `cargo test -p z80 <module>`

- **T1.1.1 — Flags.** ctx: KB-21a · livr: `z80/src/flags.rs` : `Flags` (bitflags S Z Y H X PV N C) · tests: valeurs de bits.
- **T1.1.2 — Helpers de flags.** ctx: KB-21a · livr: fonctions pures `parity`, `szxy(u8)`, demi-retenue et débordement add/sub 8 et 16 bits · tests: tables de référence.
- **T1.1.3 — Registres 8 bits.** ctx: KB-21a · livr: `regs.rs` : A F B C D E H L · tests: accesseurs.
- **T1.1.4 — Paires 16 bits.** livr: get/set AF BC DE HL · tests: cohérence avec les 8 bits.
- **T1.1.5 — Jeu alterné.** livr: AF' BC' DE' HL' + primitives `ex_af()`, `exx()` · tests: double échange = identité.
- **T1.1.6 — Index et spéciaux.** livr: IX IY SP PC I R (bit 7 de R préservé à l'incrément), IFF1/IFF2, IM, IXH/IXL/IYH/IYL.
- **T1.1.7 — WZ (MEMPTR) et Q.** ctx: KB-21f · livr: champs `wz`, `q` + API `set_q`, règle « Q = F si l'instruction écrit F, sinon 0 ».
- **T1.1.8 — Trait `Bus`.** livr: trait de la Partie C (copie littérale, documenté) · val: `cargo build -p z80`.
- **T1.1.9 — TestBus : mémoire.** livr: `TestBus` 64 Ko + compteur de cycles + `advance` · tests: lecture/écriture.
- **T1.1.10 — TestBus : journal.** livr: `Event{cycle, kind (MR/MW/PR/PW), addr, val}` · tests: ordre des événements.
- **T1.1.11 — Cycle M1.** ctx: KB-21b1 · livr: `fetch_opcode` = 4 T, `R` incrémenté (7 bits bas) · tests: durée 4 T, R+1.
- **T1.1.12 — Cycles mémoire.** livr: lecture/écriture = 3 T · tests: durées.
- **T1.1.13 — Cycles E/S.** livr: `io_read`/`io_write` = 4 T · tests: durées.
- **T1.1.14 — Structure `Cpu` et `step()`.** livr: `cpu.rs` : `Cpu`, table de dispatch 256 entrées, opcode non implémenté → `Err(Unimplemented(op))` (pas de panic) · tests: `step()` sur 0x00 non implémenté renvoie l'erreur.
- **T1.1.15 — Fuse : récupération.** livr: `xtask fetch-third-party fuse` (`tests.in`, `tests.expected`) + SHA256 · val: **vérifier disponibilité/format actuels** avant ; fichiers présents.
- **T1.1.16 — Fuse : parseur `tests.in`.** livr: structure `FuseCase` (registres, mémoire, T-states) · tests: parse 3 cas connus.
- **T1.1.17 — Fuse : parseur `tests.expected`.** livr: registres finaux + événements de bus attendus.
- **T1.1.18 — Fuse : exécuteur d'état.** livr: `xtask run-fuse --opcodes <liste>` : exécute, compare registres/mémoire/T-states ; opcodes non implémentés → `SKIP` · val: taux de réussite affiché.
- **T1.1.19 — Fuse : comparaison de bus.** livr: comparaison des événements MR/MW/PR/PW par T-state avec **première divergence** (+20 lignes de contexte).

### Étape 1.2 — Chargements et ALU 8 bits (non préfixé)
> ctx par défaut : KB-21b1/KB-21b2 (moitié concernée), KB-21a · val par défaut : `cargo test -p z80 <module> && cargo xtask run-fuse --opcodes <liste>` (0 échec sur la liste)

- **T1.2.1 — NOP et LD r,r'.** opcodes 00, 40–7F hors (HL) et 76.
- **T1.2.2 — LD r,n.** 06 0E 16 1E 26 2E 3E.
- **T1.2.3 — LD r,(HL) et LD (HL),r.**
- **T1.2.4 — LD (HL),n.** 36.
- **T1.2.5 — LD rr,nn.** 01 11 21 31.
- **T1.2.6 — LD A,(BC)/(DE) et LD (BC)/(DE),A.** 0A 1A 02 12 · ctx+: KB-21f (WZ).
- **T1.2.7 — LD A,(nn) et LD (nn),A.** 3A 32 · ctx+: KB-21f.
- **T1.2.8 — LD HL,(nn), LD (nn),HL, LD SP,HL.** 2A 22 F9 · ctx+: KB-21f.
- **T1.2.9 — ADD/ADC A,r.** 80–8F.
- **T1.2.10 — SUB/SBC/CP r.** 90–9F, B8–BF (CP : X/Y issus de l'opérande).
- **T1.2.11 — AND/OR/XOR r.** A0–B7 hors CP.
- **T1.2.12 — ALU A,(HL).** 86 8E 96 9E A6 AE B6 BE.
- **T1.2.13 — ALU A,n.** C6 CE D6 DE E6 EE F6 FE.
- **T1.2.14 — INC r / DEC r.** 04 05 … 3C 3D.
- **T1.2.15 — INC (HL) / DEC (HL).** 34 35.
- **T1.2.16 — DAA.** 27.
- **T1.2.17 — CPL.** 2F.
- **T1.2.18 — SCF et CCF.** 37 3F · ctx+: KB-21f (flags X/Y via Q).

### Étape 1.3 — 16 bits, pile, échanges, rotations de A
- **T1.3.1 — INC rr / DEC rr.** 03 0B 13 1B 23 2B 33 3B (aucun flag).
- **T1.3.2 — ADD HL,rr.** 09 19 29 39 · ctx+: KB-21f.
- **T1.3.3 — PUSH rr.** C5 D5 E5 F5.
- **T1.3.4 — POP rr.** C1 D1 E1 F1.
- **T1.3.5 — EX DE,HL ; EX AF,AF' ; EXX.** EB 08 D9.
- **T1.3.6 — EX (SP),HL.** E3 · ctx+: KB-21f.
- **T1.3.7 — RLCA / RRCA.** 07 0F.
- **T1.3.8 — RLA / RRA.** 17 1F.

### Étape 1.4 — Flux de contrôle et divers
- **T1.4.1 — JP nn et JP (HL).** C3 E9 · ctx+: KB-21f.
- **T1.4.2 — JP cc,nn.** C2 CA D2 DA E2 EA F2 FA.
- **T1.4.3 — JR e et JR cc,e.** 18 20 28 30 38 (7 T/12 T).
- **T1.4.4 — DJNZ.** 10 (8 T/13 T).
- **T1.4.5 — CALL nn.** CD.
- **T1.4.6 — CALL cc,nn.** C4 CC D4 DC E4 EC F4 FC (10 T/17 T).
- **T1.4.7 — RET et RET cc.** C9 C0 C8 D0 D8 E0 E8 F0 F8.
- **T1.4.8 — RST p.** C7 CF … FF.
- **T1.4.9 — DI et EI.** F3 FB · ctx+: KB-21g · EI retarde l'acceptation d'INT d'une instruction.
- **T1.4.10 — HALT.** 76 · ctx+: KB-21g · NOP répétés, PC reste sur HALT.
- **T1.4.11 — IN A,(n) et OUT (n),A.** DB D3 · ctx+: KB-21f.

### Étape 1.5 — Préfixe CB
> ctx par défaut : KB-21c, KB-21a

- **T1.5.1 — Infrastructure CB.** décode le 2ᵉ opcode (M1, R incrémenté 2 fois), dispatch 256 · tests: durées de fetch.
- **T1.5.2 — RLC / RRC r.**
- **T1.5.3 — RL / RR r.**
- **T1.5.4 — SLA / SRA / SLL / SRL r.** (SLL non documenté.)
- **T1.5.5 — BIT n,r.** flags X/Y issus de l'opérande.
- **T1.5.6 — SET / RES n,r.**
- **T1.5.7 — RLC/RRC/RL/RR (HL).**
- **T1.5.8 — SLA/SRA/SLL/SRL (HL).**
- **T1.5.9 — SET / RES n,(HL).**
- **T1.5.10 — BIT n,(HL).** ctx+: KB-21f · X/Y issus de la partie haute de WZ.

### Étape 1.6 — Préfixes DD / FD
> ctx par défaut : KB-21d, KB-21a

- **T1.6.1 — Infrastructure DD/FD.** sélection IX/IY, +4 T, préfixe suivi d'un opcode non concerné = comportement NOP documenté, chaînes `DD DD FD …`.
- **T1.6.2 — IXH/IXL : LD (non documentés).** LD r,r' et LD r,n avec IXH/IXL/IYH/IYL.
- **T1.6.3 — IXH/IXL : ALU, INC/DEC.**
- **T1.6.4 — LD r,(IX+d) / LD (IX+d),r.** ctx+: KB-21f.
- **T1.6.5 — LD (IX+d),n.**
- **T1.6.6 — ALU A,(IX+d).** 8 opérations.
- **T1.6.7 — INC/DEC (IX+d).**
- **T1.6.8 — LD IX,nn ; LD IX,(nn) ; LD (nn),IX.**
- **T1.6.9 — ADD IX,rr ; INC/DEC IX.**
- **T1.6.10 — PUSH/POP IX ; EX (SP),IX ; JP (IX) ; LD SP,IX.**
- **T1.6.11 — Infrastructure DDCB/FDCB.** déplacement avant l'opcode, T-states, pas de M1 pour l'opcode final.
- **T1.6.12 — DDCB : rotations, shifts, SET, RES.** (+ copie vers registre, non documentée).
- **T1.6.13 — DDCB : BIT.** ctx+: KB-21f.

### Étape 1.7 — Préfixe ED
> ctx par défaut : KB-21e, KB-21a

- **T1.7.1 — Infrastructure ED.** ED inconnus = NOP de 8 T.
- **T1.7.2 — LD I,A ; LD R,A ; LD A,I ; LD A,R.** (flags P/V = IFF2.)
- **T1.7.3 — NEG** (et ses 7 alias non documentés).
- **T1.7.4 — IM 0/1/2** (et alias).
- **T1.7.5 — RETN / RETI** (IFF1 ← IFF2).
- **T1.7.6 — IN r,(C) et IN (C).** ctx+: KB-21f.
- **T1.7.7 — OUT (C),r et OUT (C),0.**
- **T1.7.8 — ADC HL,rr.**
- **T1.7.9 — SBC HL,rr.**
- **T1.7.10 — LD (nn),rr ; LD rr,(nn).** ctx+: KB-21f.
- **T1.7.11 — RRD / RLD.** ctx+: KB-21f.
- **T1.7.12 — LDI / LDD.**
- **T1.7.13 — LDIR / LDDR.** répétition 21 T, PC −2, flags X/Y de la répétition.
- **T1.7.14 — CPI / CPD.**
- **T1.7.15 — CPIR / CPDR.**
- **T1.7.16 — INI / IND.**
- **T1.7.17 — INIR / INDR.**
- **T1.7.18 — OUTI / OUTD.**
- **T1.7.19 — OTIR / OTDR.**

### Étape 1.8 — Interruptions
> ctx par défaut : KB-21g, KB-05

- **T1.8.1 — NMI : détection de front.** `nmi_pending()` consommé par le CPU ; ligne = niveau, front montant détecté · tests: un seul NMI par front.
- **T1.8.2 — NMI : séquence d'acceptation.** 11 T, push PC, saut 0x0066, IFF1 ← 0 (IFF2 conservé).
- **T1.8.3 — NMI et HALT.** sortie de HALT, PC empilé = adresse suivant HALT.
- **T1.8.4 — NMI, EI et RETN.** NMI juste après EI ; RETN restaure IFF1 depuis IFF2.
- **T1.8.5 — INT en IM 1.** RST 38 (13 T).
- **T1.8.6 — INT en IM 2.** vecteur `I:data bus` (19 T).
- **T1.8.7 — INT en IM 0.** exécution d'un opcode (RST) présenté sur le bus.
- **T1.8.8 — ROM TR-CPU-NMI.** livr: source `.asm`, `.bin`, README · val: exécutée sur `TestBus` avec un NMI injecté à des cycles précis → octet de verdict `0xA5`.

### Étape 1.9 — Validation CPU globale
> **Règle d'échec** : un échec sur une tâche de cette étape n'est **pas** corrigé « en place » : l'Orchestrateur propose d'insérer des tâches de correction ciblées (une par famille d'instructions en échec) avant elle (voir B.6).

- **T1.9.1 — Shim CP/M.** livr: BDOS fonction 2 (putchar) et 9 (print string), appel à 0x0005, warm boot 0x0000 · tests: programme de 10 octets qui affiche « OK ».
- **T1.9.2 — Chargeur .COM et runner.** livr: charge à 0x0100, `xtask run-zex <nom> [--filter]` · val: `.COM` de démonstration exécuté jusqu'au warm boot.
- **T1.9.3 — zexdoc : récupération et runner par test.** livr: `fetch-third-party zexdoc/zexall` + SHA256 ; sortie « une ligne par test » · val: zexdoc s'exécute sans crash (échecs de CRC tolérés ici).
- **T1.9.4 — zexdoc : ALU 8 bits.** filtre : add/adc/sub/sbc/and/or/xor/cp, inc/dec 8 bits, daa, cpl, neg → toutes « OK ».
- **T1.9.5 — zexdoc : 16 bits.** `add hl`, `adc/sbc hl`, `add ix/iy`, inc/dec rr → « OK ».
- **T1.9.6 — zexdoc : chargements.** `ld` toutes formes, `ld (nnnn)`, `ld r,(ix+d)` → « OK ».
- **T1.9.7 — zexdoc : rotations, shifts, bits.** `rld/rrd`, `bit`, `shf/rot`, `set/res` → « OK ».
- **T1.9.8 — zexdoc : blocs.** `ldd/ldi(r)`, `cpd/cpi(r)` → « OK ».
- **T1.9.9 — zexdoc : complet.** toutes les lignes « OK » en une exécution.
- **T1.9.10 — zexall : ALU et 16 bits.**
- **T1.9.11 — zexall : chargements, rotations, bits.**
- **T1.9.12 — zexall : blocs et complet.**
- **T1.9.13 — z80test : récupération et runner.** livr: format d'entrée **à vérifier** (.tap/.com) ; détection du verdict affiché.
- **T1.9.14 — z80doc.**
- **T1.9.15 — z80full.**
- **T1.9.16 — z80ccf.**
- **T1.9.17 — z80memptr.**
- **T1.9.18 — Fuse : timing de bus, tous les cas.** `cargo xtask run-fuse` complet → 0 divergence MR/MW/PR/PW/MC.
- **T1.9.19 — SingleStepTests/z80 : récupération et runner.** **vérifier format actuel** (JSON par opcode, cycles + bus).
- **T1.9.20 — SingleStepTests/z80 : exécution.** 0 divergence (ou liste validée par l'humain).

> La comparaison de trace avec MAME (ancien T1.4.4) est en **Phase 3** (T3.3.x), car elle exige une machine complète.

---

## PHASE 2 — Horloge système, bus machine, NMI

### Étape 2.1 — Horloge et balayage
> ctx par défaut : KB-02, KB-07 · livr: `galaxian-core/src/raster.rs` · val: `cargo test -p galaxian-core raster`

- **T2.1.1 — Structure `Raster` et compteur H.** correspondance pixel (0..383) → compteur H matériel (128..511, KB-07) · tests: pixels 0, 255, 256, 383.
- **T2.1.2 — `Raster::advance(cycles)`.** met à jour (ligne 0..263, pixel 0..383), gère le passage de ligne et de frame · tests: 192 cycles = 1 ligne ; 50 688 = 1 frame.
- **T2.1.3 — HBLANK.** bascule par 2H, 1 à H=130, 0 à H=250 (KB-07) · tests: pixels non blankés = 264.
- **T2.1.4 — HSYNC.** 1 à H=176, 0 à H=208 · tests: largeur et position.
- **T2.1.5 — VBLANK et événement.** 1 à V=496, 0 à V=272 ; événement « entrée en VBLANK » horodaté (cycle absolu) · tests: 224 lignes visibles, événement une fois par frame.
- **T2.1.6 — VSYNC.** 1 à V=248, 0 à V=256.
- **T2.1.7 — Zone visible.** lignes 16..239, pixels 0..255 · tests: bornes.
- **T2.1.8 — Retard du compteur V.** `v_counter_hw()` : en retard d'un cran pendant les 48 premiers clocks H du blanking (utilisé par sprites/shells) · tests: valeurs avant/après le 48ᵉ clock.

### Étape 2.2 — Mémoire et bus machine
> ctx par défaut : KB-03 · livr: `galaxian-core/src/{memory,bus}.rs` · val: `cargo test -p galaxian-core bus`

- **T2.2.1 — `Memory` : ROM et RAM.** ROM 0000-3fff, RAM 4000-43ff miroir 0400 · tests: lecture miroir.
- **T2.2.2 — Zones spéciales.** écriture ROM ignorée, 4800-4fff non connecté, non mappé → 0xFF (`unmap_value_high`).
- **T2.2.3 — VRAM.** 5000-53ff miroir 0400 + hook d'écriture (notifie le rendu) · tests: miroir et hook.
- **T2.2.4 — OBJRAM.** 5800-58ff miroir 0700 + hook d'écriture.
- **T2.2.5 — `Machine` implémente `Bus`.** `advance` fait avancer `Raster` ; `mem_read`/`mem_write` dispatchent par plage · tests: avance du temps à chaque accès.
- **T2.2.6 — Accès `fetch_opcode` et E/S.** `fetch_opcode` = lecture mémoire + 4 T ; `io_read`/`io_write` : retour 0xFF / écriture ignorée (hypothèse consignée en `notes`).
- **T2.2.7 — Test d'adressage complet.** table de 100+ adresses (dont miroirs) vs attendu.
- **T2.2.8 — ROM TR-MAP.** livr: `.asm`, `.bin`, README : écrit/relit chaque miroir, lit une zone non mappée (attend 0xFF), verdict en 0x4000.
- **T2.2.9 — TR-MAP : validation.** exécutée sur `Machine` → verdict `0xA5` ; génère la trace MAME de la ROM (`xtask golden`) et la compare : 0 divergence.

### Étape 2.3 — Registres d'E/S
> ctx par défaut : KB-04 · livr: `galaxian-core/src/io.rs`

- **T2.3.1 — Latch 1 bit.** helper : bit D0, adresse `& 7`, miroir 0x07f8 · tests: toutes adresses de miroir.
- **T2.3.2 — /DRIVER 6000-6007.** lampes, coin lockout/counter, bits `lfo_val` (6004-6007) · tests: écriture sur miroirs.
- **T2.3.3 — /SOUND 6800-6807.** FS1-3, HIT, FIRE, VOL1/2 (6804 n/c) · tests: miroirs.
- **T2.3.4 — LATCH 7000-77ff.** 7001 NMI ON, 7004 STARS ON, 7006 HFLIP, 7007 VFLIP ; autres ignorés.
- **T2.3.5 — Écriture 7800.** latch pitch 8 bits.
- **T2.3.6 — Trait `Inputs` et IN0.** ctx+: KB-24a · lecture 6000 (miroir 07ff).
- **T2.3.7 — IN1.** ctx+: KB-24a · lecture 6800.
- **T2.3.8 — IN2 et DIPs.** ctx+: KB-24a, KB-24b · lecture 7000.
- **T2.3.9 — Test global des E/S.** lecture/écriture de tous les registres via `Machine`.

### Étape 2.4 — Watchdog
> ctx par défaut : KB-06, KB-25b

- **T2.4.1 — Compteur de watchdog.** compteur en cycles, durée du timeout (KB-25b) · tests: timeout atteint au cycle exact.
- **T2.4.2 — Lecture de 7800.** remet le compteur à zéro (miroir 07ff) · tests: lectures périodiques.
- **T2.4.3 — Reset machine sur timeout.** `Machine::reset()` appelé · tests: reset déclenché, registres CPU remis à zéro.
- **T2.4.4 — ROM TR-WDG.** livr: `.asm`, `.bin`, README : alimente le watchdog N fois puis cesse.
- **T2.4.5 — TR-WDG : validation.** reset détecté au bon cycle ; génère la trace MAME de la ROM et la compare : 0 divergence.

### Étape 2.5 — NMI VBLANK
> ctx par défaut : KB-05, KB-19

- **T2.5.1 — `irq_enabled`.** écriture 7001 (D0) · tests: valeur initiale 0.
- **T2.5.2 — Niveau NMI à l'entrée en VBLANK.** si `irq_enabled` → ligne NMI haute (événement de T2.1.5).
- **T2.5.3 — Effacement de la ligne.** écriture 7001=0 → ligne basse.
- **T2.5.4 — Branchement CPU.** `nmi_pending()` renvoie le front montant · tests: 1 front par frame.
- **T2.5.5 — ROM TR-NMI.** livr: `.asm`, `.bin`, README : compte les NMI dans un registre RAM.
- **T2.5.6 — TR-NMI : comptage.** 600 frames → compteur = 600 ± 0 ; avant écriture de 7001 : 0.
- **T2.5.7 — Cycle exact du NMI.** le cycle d'entrée dans le handler 0x0066 est identique à la trace MAME (ferme GAP-06).
- **T2.5.8 — TR-CPU-NMI sur machine.** la ROM de T1.8.8 : trace identique à MAME.

---

## PHASE 3 — Machine minimale bootable

### Étape 3.1 — Chargement des ROMs
> ctx par défaut : KB-25a · livr: `galaxian-core/src/romset.rs` · ROMs fournies par l'utilisateur (non versionnées)

- **T3.1.1 — Lecture d'un dossier.** fichiers du set `galaxian` par nom (KB-25a) · tests: dossier factice.
- **T3.1.2 — Vérification de taille.** erreur typée (`thiserror`) si taille ≠ attendue.
- **T3.1.3 — Vérification CRC32.** compare aux CRC de KB-25a ; erreur claire en cas de désaccord.
- **T3.1.4 — Placement ROM programme.** à 0x0000.. selon les offsets · tests: premiers octets.
- **T3.1.5 — ROMs graphiques.** stockées pour la Phase 4 (non décodées).
- **T3.1.6 — PROM de palette.** stockée pour T4.1.
- **T3.1.7 — Set alternatif `galmidw`.** même API, autres noms/CRC.
- **T3.1.8 — ROM TR-HELLO sur machine.** charge `tests/roms/hello` ; après 1 frame, `0x4000 == 0xA5`.

### Étape 3.2 — Boucle principale
> livr: `galaxian-core/src/machine.rs`

- **T3.2.1 — `run_frame()`.** exécute des instructions jusqu'à franchir 50 688 cycles · tests: nombre de cycles par frame.
- **T3.2.2 — Report des cycles excédentaires.** l'instruction qui franchit la frontière est terminée ; l'excédent est reporté sur la frame suivante · tests: somme sur 100 frames = 100 × 50 688.
- **T3.2.3 — `reset()`.** état initial CPU/E-S/raster · tests: 2 resets = même état.
- **T3.2.4 — `snapshot()`.** état sérialisable de `Machine` (CPU, RAM, latches, raster, watchdog) · tests: égalité après deux `snapshot()` consécutifs.

### Étape 3.3 — Trace contre MAME
> ctx par défaut : — · livr: `xtask mame-diff trace`

- **T3.3.1 — Comparateur de traces.** compare `trace_boot_<N>.log` à la trace de l'émulateur ; rapporte la **première divergence** (cycle, PC, registres) + 20 lignes de contexte.
- **T3.3.2 — Trace 1 000 instructions.** 0 divergence.
- **T3.3.3 — Trace 100 000 instructions.** 0 divergence.
- **T3.3.4 — Trace 1 000 000 instructions.** 0 divergence (ancien T1.4.4).

### Étape 3.4 — Dump mémoire contre MAME
> ctx par défaut : KB-03

- **T3.4.1 — Comparateur de dumps.** livr: `xtask golden diff-mem` (offset du premier octet différent).
- **T3.4.2 — VRAM/OBJRAM à la frame 60.** identique à `golden/mem_dump_60.bin`.
- **T3.4.3 — VRAM/OBJRAM à la frame 600.** identique à `golden/mem_dump_600.bin`.

---

## PHASE 4 — Vidéo pixel-exact

Sortie : framebuffer **256×224 RGB** (rendu interne 768×224 pour fidélité des étoiles, puis réduction documentée). Chaque ROM `TR-*` vidéo est livrée en **deux tâches** : (a) source + binaire + README, (b) golden MAME + validation par CRC.

### Étape 4.1 — Graphismes
> livr: `galaxian-core/src/gfx.rs` · val par défaut : `cargo test -p galaxian-core gfx`

- **T4.1.1 — Structure `GfxLayout` et décodeur de plans.** ctx: KB-22a · décodage générique N bitplanes, offsets de plans/x/y · tests: layout synthétique.
- **T4.1.2 — Tiles 8×8 2 bpp.** ctx: KB-22a · décode toutes les tiles depuis la ROM caractères · tests: nombre de tiles.
- **T4.1.3 — Sprites 16×16 2 bpp.** ctx: KB-22b · décode tous les sprites.
- **T4.1.4 — Tile et sprite connus.** tile n°X et sprite n°Y identiques au dump `gfxviewer` MAME.
- **T4.1.5 — `compute_resistor_weights`.** ctx: KB-08 · implémentation Rust indépendante de la formule de pondération résistive (principe documenté dans `docs/mame_src/`, sans recopier le code) · tests: poids pour {1000,470,220} et {470,220}.
- **T4.1.6 — Palette : 32 entrées.** ctx: KB-08, KB-27 · décodage bit→R/G/B de la PROM, `RGB_MAXIMUM=224`.
- **T4.1.7 — Palette : validation.** les 32 valeurs RGB = MAME.
- **T4.1.8 — Couleurs shells/missile.** ctx: KB-08 · 7 blanches `(255,255,255)` + 1 jaune `(255,255,0)`.

### Étape 4.2 — Framebuffer et tilemap
> livr: `galaxian-core/src/video/` · val par défaut : `cargo test -p galaxian-core video::tilemap`

- **T4.2.1 — Framebuffer interne.** ctx: KB-02 · `Frame` 768×224 RGB, `clear`, accès pixel · tests: bornes.
- **T4.2.2 — Rendu d'une tile.** ctx: KB-09, KB-23a · 8×8 avec palette, sans flip.
- **T4.2.3 — Flips X/Y d'une tile.**
- **T4.2.4 — Tilemap 32×32 statique.** ctx: KB-23a · sans scroll ni attribut · tests: motif connu.
- **T4.2.5 — Scroll vertical par colonne.** ctx: KB-23b.
- **T4.2.6 — Attribut couleur par colonne.** ctx: KB-23b.
- **T4.2.7 — Flip écran sur le tilemap.** ctx: KB-13 · `set_flip(flip_x, flip_y)`.
- **T4.2.8 — ROM TR-TILE (source).** `.asm`, `.bin`, README : motif connu, scroll colonne, attributs, flips.
- **T4.2.9 — TR-TILE : golden et validation.** CRC de frame identique à MAME.
- **T4.2.10 — Rendu par ligne.** `render_line(y)` utilise l'état VRAM/OBJRAM **à cet instant** (équivalent `update_partial`).
- **T4.2.11 — Branchement dans `run_frame`.** `render_line` appelé au bon cycle de chaque ligne visible · tests: écriture VRAM en milieu de frame visible sur la ligne suivante.
- **T4.2.12 — ROM TR-RASTER (source).** `.asm`, `.bin`, README : modifie la VRAM en milieu de frame.
- **T4.2.13 — TR-RASTER : golden et validation.** CRC = MAME.

### Étape 4.3 — Sprites (modèle line buffer)
> livr: `galaxian-core/src/video/sprites.rs` · val par défaut : `cargo test -p galaxian-core video::sprites`

- **T4.3.1 — Lecture d'un sprite depuis OBJRAM.** ctx: KB-10, KB-23b · Y, n° image, flips, couleur (3 bits bas), X.
- **T4.3.2 — Test vertical.** ctx: KB-10, KB-07 · `((V + vpos) & 0xf0) == 0xf0` avec le V matériel retardé (`v_counter_hw`).
- **T4.3.3 — Sélection par ligne.** jusqu'à 8 sprites par ligne, ordre de traitement.
- **T4.3.4 — Line buffer.** structure, remise à zéro par ligne.
- **T4.3.5 — Rendu d'une rangée de sprite.** 16 pixels, couleur, sans flip.
- **T4.3.6 — Flips H et V du sprite.**
- **T4.3.7 — Priorité.** écriture seulement si pixel = 0 ; sprites 0..7 (n° faible devant).
- **T4.3.8 — Clipping 16 px.** formule `sprites_clip`, avec et sans flip X.
- **T4.3.9 — Limite 7,5 sprites par ligne.** le 8ᵉ est rendu partiellement.
- **T4.3.10 — ROM TR-SPR-1 et TR-SPR-FLIP (source).** 1 sprite ; flips H/V.
- **T4.3.11 — TR-SPR-1 et TR-SPR-FLIP : golden et validation.** CRC = MAME.
- **T4.3.12 — ROM TR-SPR-8 et TR-SPR-EDGE (source).** 8 sprites alignés ; bords gauche/droite.
- **T4.3.13 — TR-SPR-8 et TR-SPR-EDGE : golden et validation.** CRC = MAME.

### Étape 4.4 — Shells et missile
> livr: `galaxian-core/src/video/bullets.rs` · ctx par défaut : KB-11, KB-10

- **T4.4.1 — Lecture shells/missile.** OBJRAM 0x60.. : 7 shells + 1 missile · tests: parse.
- **T4.4.2 — Match vertical.** `((V + vpos) & 0xff) == 0xff`.
- **T4.4.3 — Position horizontale.** début à H=`$FC`, 4 pixels, « dernier trouvé gagne ».
- **T4.4.4 — Couleurs.** 7 blancs + 1 jaune (T4.1.8).
- **T4.4.5 — ROM TR-BULLET (source).**
- **T4.4.6 — TR-BULLET : golden et validation.** CRC = MAME.

### Étape 4.5 — Étoiles
> livr: `galaxian-core/src/video/stars.rs` · val par défaut : `cargo test -p galaxian-core video::stars`

- **T4.5.1 — `Lfsr17` partagé.** ctx: KB-12, KB-19 · une instance dans `galaxian-core`, utilisable par vidéo **et** audio · tests: pas du registre.
- **T4.5.2 — Table `stars[131071]`.** ctx: KB-12 · génération exacte.
- **T4.5.3 — CRC de la table.** = valeur MAME.
- **T4.5.4 — Couleurs des étoiles.** ctx: KB-08, KB-12 · 64 entrées (150Ω/100Ω).
- **T4.5.5 — Rendu d'une ligne sans masque.** ctx: KB-12, KB-02 · motif 3:2 (1 px puis 2 px).
- **T4.5.6 — Masque `(V1 xor H8)`.**
- **T4.5.7 — Étoiles activées (7004).** gating par STARS ON.
- **T4.5.8 — Origine et dérive.** `stars_update_origin`, 1 cran par frame.
- **T4.5.9 — Décalage 2 clocks/frame.** bascules 6B, mode non flippé.
- **T4.5.10 — Recalage avant flip.** ctx+: KB-13 · `stars_update_origin()` avant changement de flip.
- **T4.5.11 — ROM TR-STARS (source).** étoiles ON, avec et sans flip.
- **T4.5.12 — TR-STARS sans flip : golden et validation.** 0/1/10/100 frames, CRC = MAME.
- **T4.5.13 — TR-STARS avec flip : golden et validation.** CRC = MAME.

### Étape 4.6 — Composition finale
> ctx par défaut : KB-14, KB-26

- **T4.6.1 — Ordre de superposition.** fond, étoiles, tilemap, sprites, shells (selon KB-26).
- **T4.6.2 — Réduction 768→256.** conversion documentée dans `docs/`, sortie 256×224 RGB · tests: CRC d'une image synthétique.
- **T4.6.3 — Frames 1, 2, 5, 10.** CRC de la ROM Galaxian = `golden/frames_crc.txt`.
- **T4.6.4 — Frames 30, 60, 120.** idem.
- **T4.6.5 — Frames 300, 600.** idem.

---

## PHASE 5 — Audio

> Non bit-exact (KB-17). Trois niveaux : LFSR/registres → primitives → blocs, chacun avec ses critères. Les seuils de comparaison sont **fixés par l'humain** et ne sont pas ajustés pour « faire passer ».

### Étape 5.1 — LFSR partagé
- **T5.1.1 — Une seule instance.** ctx: KB-12, KB-16, KB-19 · `Lfsr17` de T4.5.1 exposé à `galaxian-audio` (pas de copie) · tests: audio et vidéo lisent la même instance.
- **T5.1.2 — Période.** 131 071 états puis retour à l'état initial.

### Étape 5.2 — Registres son
> ctx par défaut : KB-04, KB-15

- **T5.2.1 — FS1-3.** latches 6800-6802.
- **T5.2.2 — HIT et FIRE.** 6803, 6805.
- **T5.2.3 — VOL1/VOL2.** 6806, 6807.
- **T5.2.4 — Pitch.** latch 8 bits (7800).
- **T5.2.5 — `lfo_val`.** 4 bits via 6004-6007 ; notification au graphe seulement si changé.

### Étape 5.3 — Primitives discrètes
> ctx par défaut : KB-16 · livr: `galaxian-audio/src/prims.rs` · val: tests analytiques

- **T5.3.1 — 555 astable.** `f = 1.44/((R1+2R2)C)` · tests: fréquence à ±0,1 %.
- **T5.3.2 — 555 astable à contrôle de tension.**
- **T5.3.3 — 555 VCO.**
- **T5.3.4 — RC charge/décharge.** (`RCDISC5`, `RC`) · tests: constante de temps.
- **T5.3.5 — Filtre RC.**
- **T5.3.6 — Passe-bande 1er ordre.**
- **T5.3.7 — DAC R-2R 4 bits.** · tests: 16 niveaux.
- **T5.3.8 — Mixeur pondéré.** (`MIXER3`, `MIXER5`)
- **T5.3.9 — Bascule D et carré à fréquence fixe.** (`DFLIPFLOP`, `SQUAREWFIX`)

### Étape 5.4 — Blocs du graphe
> ctx par défaut : KB-16 · livr: `galaxian-audio/src/blocks/`

- **T5.4.1 — PITCH : diviseur.** `SOUND_CLOCK/(256 − pitch)`.
- **T5.4.2 — PITCH : décodage QA/QC/QD.** `BITS_DECODE`.
- **T5.4.3 — BACKGROUND : DAC → VCO.**
- **T5.4.4 — BACKGROUND : op-amp et clamp 0..5 V.**
- **T5.4.5 — BACKGROUND : 3×555 et mixeur.** FS1/FS2/FS3.
- **T5.4.6 — NOISE.** `LFSR_NOISE` + bascule échantillonnée par « 2V ».
- **T5.4.7 — HIT : `RCDISC5`.**
- **T5.4.8 — HIT : passe-bande.**
- **T5.4.9 — FIRE : bruit et RC.**
- **T5.4.10 — FIRE : 555 VCO et `RCDISC5`.** impulsion de durée fixe.
- **T5.4.11 — Mixage final.** `MIXER5` puis `MIXER3`.
- **T5.4.12 — Échantillonnage.** sortie 44,1/48 kHz, API « tirer N échantillons ».

### Étape 5.5 — Validation audio
> ctx par défaut : KB-17

- **T5.5.1 — Outil de métriques.** `xtask` : enveloppe RMS (corrélation), spectre (fondamentale à ±1 %) entre deux WAV.
- **T5.5.2 — ROMs TR-SND-FS et TR-SND-PITCH (source).**
- **T5.5.3 — ROMs TR-SND-HIT et TR-SND-FIRE (source).**
- **T5.5.4 — TR-SND-* : golden et mesures.** WAV MAME pour chaque ROM ; **l'humain fixe les seuils** après un premier essai.
- **T5.5.5 — Scénario attract.** comparaison à `golden/audio_attract.wav`.
- **T5.5.6 — Scénario tir.**
- **T5.5.7 — Scénario explosion.**

---

## PHASE 6 — Intégration et frontend

### Étape 6.1 — Affichage et temps (eframe/egui)
> livr: `galaxian-frontend/` · val par défaut : `cargo build -p galaxian-frontend && cargo test -p galaxian-frontend`

- **T6.1.1 — Application eframe et affichage 256×224.** `impl eframe::App` ; fonction pure `frame_to_color_image(&Frame) -> egui::ColorImage` (testée sans fenêtre) ; texture `TextureHandle` mise à jour à chaque frame, filtrage *nearest*, mise à l'échelle entière.
- **T6.1.2 — Rotation 90° optionnelle.** (écran vertical d'origine) rotation appliquée à l'affichage (UV/transformation du rendu egui), jamais dans le cœur ; fonction de rotation pure testée.
- **T6.1.3 — Boucle 60,606 Hz.** pas de temps fixe par accumulateur : le cœur avance de N frames machine selon le temps écoulé, indépendamment de la fréquence de rafraîchissement d'egui ; `ctx.request_repaint()` / `request_repaint_after` ; test de l'accumulateur avec une horloge simulée.

### Étape 6.2 — Entrées
> ctx par défaut : KB-24a, KB-24b

- **T6.2.1 — Clavier → IN0/IN1.** coin, start, gauche/droite, tir, lus via `ctx.input` d'egui ; table de correspondance pure testée.
- **T6.2.2 — Manette.** via `gilrs` (egui n'a pas de gestion de manette ; dépendance validée par l'humain).
- **T6.2.3 — DIP switches configurables.** fichier de config (et panneau egui optionnel), sans toucher au cœur.

### Étape 6.3 — Audio
- **T6.3.1 — Sortie audio.** file d'échantillons + `cpal` (egui ne fournit pas d'audio).
- **T6.3.2 — Resampling.**
- **T6.3.3 — Synchronisation audio/vidéo.** sync sur l'audio ou la vidéo (option).

### Étape 6.4 — Scénarios de non-régression
> val: CRC de frames vs MAME

- **T6.4.1 — Boot.**
- **T6.4.2 — Attract mode.**
- **T6.4.3 — Crédit.**
- **T6.4.4 — Partie d'une minute.** entrées scriptées.
- **T6.4.5 — Flip (cocktail).**
- **T6.4.6 — DIPs.**

---

## PHASE 7 — Durcissement

### Étape 7.1 — Déterminisme
- **T7.1.1 — Hash d'état.** `Machine::state_hash()`.
- **T7.1.2 — Deux runs de 10 min.** identiques bit à bit.

### Étape 7.2 — Save-states
- **T7.2.1 — Sérialisation CPU.**
- **T7.2.2 — Sérialisation RAM, VRAM, latches.**
- **T7.2.3 — Sérialisation raster, LFSR, étoiles.**
- **T7.2.4 — Sérialisation audio.**
- **T7.2.5 — Round-trip.** save → load → même `state_hash` et mêmes frames.

### Étape 7.3 — Performance
- **T7.3.1 — Profilage.** rapport, **aucun changement de code**.
- **T7.3.2 — Optimisations ciblées.** objectif indicatif ≥ 20× temps réel · val: CRC golden inchangés.

### Étape 7.4 — Fuzz / proptest
- **T7.4.1 — Accès mémoire aléatoires.**
- **T7.4.2 — Écritures sur miroirs.**
- **T7.4.3 — Séquences de latches.**

### Étape 7.5 — Documentation finale
- **T7.5.1 — Écarts connus vs MAME.** (audio notamment)
- **T7.5.2 — Génération des golden.** instructions reproductibles.

---

## Catalogue des ROMs de test maison (TestROM Factory)

| ID | Tâche (source) | Vérifie |
|---|---|---|
| TR-HELLO | T0.6.5 | Chaîne d'assemblage, verdict |
| TR-CPU-NMI | T1.8.8 | Entrée NMI, 0x0066, RETN, IFF |
| TR-MAP | T2.2.8 | Miroirs mémoire, lecture non mappée = 0xFF |
| TR-WDG | T2.4.4 | Reset par watchdog |
| TR-NMI | T2.5.5 | 1 NMI/frame, activation par 7001 |
| TR-TILE | T4.2.8 | Tilemap, scroll colonne, attributs, flips |
| TR-RASTER | T4.2.12 | Écritures VRAM en cours de frame |
| TR-SPR-1/FLIP | T4.3.10 | 1 sprite, flips |
| TR-SPR-8/EDGE | T4.3.12 | 8 sprites par ligne, bords |
| TR-BULLET | T4.4.5 | Shells/missile |
| TR-STARS | T4.5.11 | Étoiles, dérive, flip |
| TR-SND-FS/PITCH | T5.5.2 | FS1-3, pitch |
| TR-SND-HIT/FIRE | T5.5.3 | HIT, FIRE |

Chaque ROM : source `.asm` commentée, `.bin`, trace MAME, CRC de frames, et un README « ce qui est testé / ce qui est attendu ».

---

---

# PARTIE G — Lacunes de la documentation fournie

| ID | Lacune | Impact | Comment la combler |
|---|---|---|---|
| GAP-01 | Format VRAM/OBJRAM détaillé, attributs par colonne, ordre de composition, taille/layout tilemap | Phase 4 | T0.4.3–T0.4.6, T0.4.11 (extraction depuis `galaxian_v.cpp`) |
| GAP-02 | Affectation exacte bit→R/G/B de la PROM (la doc liste deux fois « VERT » pour bits 5 et 4) | Palette | Relire `galaxian_palette` (T0.4.12) ; test T4.1.7 |
| GAP-03 | Durée du timeout watchdog, table des ROMs (noms, tailles, offsets), config machine | Phases 2-3 | T0.4.9, T0.4.10 (`galaxian.cpp` : `WATCHDOG_TIMER`, `ROM_START`) |
| GAP-04 | Référence Z80 (timings par instruction, flags non documentés, MEMPTR, Q) | Phase 1 | T0.3.1–T0.3.8 : fiches KB-21a…g depuis sources externes (Sean Young, Zilog, suites de test) |
| GAP-05 | Ports d'entrée IN0/IN1/IN2 et DIPs | Entrées | `INPUT_PORTS_START` dans `galaxian.cpp` (T0.4.7, T0.4.8) |
| GAP-06 | Cycle exact de l'NMI par rapport au début du VBLANK (ligne 240 ?) | NMI | Trace MAME (T2.5.7) |
| GAP-07 | Absence de ROMs de test Galaxian publiques | Validation | TestROM Factory (étape 0.6 + ROMs du catalogue) |

**Règle** : si une tâche dépend d'une lacune `OPEN`, l'Orchestrateur la marque `BLOCKED`, arrête la chaîne et propose d'insérer la tâche de comblement juste avant dans `order` (voir B.5).
