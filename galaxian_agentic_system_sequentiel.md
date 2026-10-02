# SYSTÈME AGENTIQUE — Émulateur Galaxian cycle-accurate en Rust

> Objectif : piloter un ou plusieurs LLM codeurs, étape par étape et sous-étape par sous-étape, pour produire un émulateur de la borne **Galaxian d'origine** (Namco/Midway, 1979), cycle-accurate, en Rust, validé par tests automatisés et par comparaison à des traces de référence MAME.
>
> Ce document contient :
> - **Partie A** — Architecture du système et rôles des agents (avec prompts système)
> - **Partie B** — Protocole de boucle, format de tâche, gestion d'état
> - **Partie C** — Conventions du dépôt Rust
> - **Partie D** — Stratégie de validation (ROMs de test, références MAME)
> - **Partie E** — Base de connaissances (KB) : toute la doc technique, découpée en fiches citables par ID
> - **Partie F** — Plan complet : phases → étapes → sous-étapes, chacune avec son *context pack* (fiches KB), livrables et tests
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
   a. l'en-tête de tâche (Partie F, copie littérale),
   b. le texte intégral des fiches KB listées dans `context`,
   c. la sortie de `cargo xtask api-dump` (signatures publiques existantes),
   d. les chemins des tests à faire passer,
   e. les règles de la Partie C.
3. N'inclus JAMAIS de fiche KB non listée (pour éviter la dilution de contexte).
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
Format de réponse : 1) plan en 5 lignes max ; 2) patch (fichiers complets ou diff) ;
3) commandes de test à lancer ; 4) liste des écarts/hypothèses.
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
(pas de « ticks groupés » si la tâche exige T-state par T-state).
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
  "order": ["T0.1.1", "T0.1.2", "T0.2.1", "..."],   // ordre total canonique, voir F.0
  "current_task": null,                              // verrou : au plus une tâche en cours
  "halted": false,                                   // true = arrêt en attente de l'humain
  "tasks": {
    "T1.3.2": { "status": "TODO|IN_PROGRESS|DONE|BLOCKED", "attempts": 0,
                "commit": null, "report": null, "notes": [] }
  },
  "kb_gaps": [ { "id": "GAP-01", "status": "OPEN|FILLED", "kb_file": null } ]
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
```

## B.4 Règle d'or du cycle-accurate

Le temps est la **donnée centrale**. L'unité de temps de l'émulateur est le **cycle d'horloge Z80** (T-state, 3,072 MHz) ; tout composant (vidéo, NMI, son) est avancé *entre* les accès bus du CPU via `bus.advance(tstates)`. Le CPU **n'exécute jamais une instruction « d'un bloc » puis rattrape** : chaque accès mémoire/E-S est précédé de l'avance de l'horloge du nombre exact de T-states écoulés depuis l'accès précédent.

## B.5 Règles du mode séquentiel

- **Verrou unique.** `current_task` contient au plus un ID. Si `PROGRESS.json` montre plus d'une tâche `IN_PROGRESS`, c'est une anomalie : `halted = true` et intervention humaine.
- **Sélection.** La tâche suivante est la première de `order` en `TODO`. Si une de ses dépendances n'est pas `DONE`, c'est une erreur de plan : arrêt et signalement, pas de saut.
- **Blocage.** Une tâche `BLOCKED` met `halted = true` : plus aucun agent n'est lancé. L'humain corrige (fiche KB, tâche, lacune), remet `status = TODO`, `attempts = 0`, `halted = false`. Seul l'humain peut décider de sauter une tâche bloquée, et uniquement si aucune tâche restante n'en dépend.
- **Lacunes (Partie G).** Si la prochaine tâche dépend d'une lacune `OPEN`, l'Orchestrateur la marque `BLOCKED`, s'arrête et propose d'insérer la tâche de comblement juste avant dans `order`. L'humain valide l'insertion.
- **Reprise après interruption.** Le système peut être arrêté entre deux agents. Au redémarrage, on reprend `current_task` et on relance l'agent suivant le dernier fichier présent dans `out/<id>/` (pas de `task_prompt.md` : Orchestrateur ; pas de `patch` : Codeur ; pas de rapport : Validateur ; pas de `verdict.md` : Relecteur ; sinon : Documentaliste).
- **ROMs de test `TR-*`.** `T0.4.2` est exécutée une seule fois (dossier `tests/roms/`, README de convention). Ensuite, chaque ROM `TR-*` est un livrable de la première tâche qui la cite (source `.asm`, `.bin`, trace MAME, CRC, README), puisque rien n'est écrit « en parallèle » des phases.
- **Pas de concurrence côté exécution.** Si le framework qui lance les agents sait paralléliser (threads, workers, sous-agents), le limiter à **1 worker** pour ce projet.

---

# PARTIE C — Conventions du dépôt Rust

```
galaxian/
├─ Cargo.toml                (workspace)
├─ crates/
│  ├─ z80/                   CPU Z80 cycle-accurate, no_std-compatible, agnostique du hardware
│  ├─ galaxian-core/         bus, timing, mémoire, I/O, vidéo, orchestration
│  ├─ galaxian-audio/        modèle discret du son
│  ├─ galaxian-frontend/     fenêtre, input, audio out (minifb/pixels/cpal) — hors cœur
│  └─ xtask/                 cargo xtask : api-dump, run-zex, mame-diff, test-roms, golden
├─ tests/
│  ├─ roms/                  ROMs de test maison (sources .asm + binaires)
│  ├─ golden/                références MAME (traces, CRC de frames, WAV)
│  └─ third_party/           zexdoc/zexall, z80test, fuse, singlestep (téléchargés par xtask)
├─ roms/                     ROMs commerciales (NON versionnées, fournies par l'utilisateur)
├─ docs/kb/                  fiches KB (Partie E), une par fichier
└─ PROGRESS.json
```

Règles :
- Edition 2021+, `#![forbid(unsafe_code)]`, `clippy -D warnings`, `rustfmt`.
- Dépendances autorisées dans le cœur : `bitflags`, `thiserror`. Dev : `proptest`, `serde_json`, `crc32fast`. Tout ajout = décision humaine.
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
| **V2 — ROMs de test maison** (« TestROM Factory », Phase 0.4) | Petits programmes Z80 écrits en assembleur, ciblant **un seul** mécanisme (NMI, watchdog, VRAM, sprites, étoiles, flip…), à fournir en `.bin` 16 Ko | Chaque bloc matériel isolément, de façon déterministe |
| **V3 — Références MAME (golden)** | MAME lancé en ligne de commande avec scripts Lua : trace CPU (`trace` du debugger), CRC/snapshots d'écran à la frame N, `-wavwrite` | Comparaison cycle/frame par frame de l'émulateur complet |
| **V4 — ROM commerciale Galaxian** | ROMs du set MAME `galaxian` / `galmidw` fournies par l'utilisateur (non redistribuées) | Intégration : boot, attract mode, partie, sons |

## D.2 Génération des références MAME (à scripter dans `xtask golden`)

Options MAME utiles : `-autoboot_script <lua>`, `-seconds_to_run N`, `-nothrottle`, `-video none` / `-sound none` selon le cas, `-wavwrite <fichier>`, `-aviwrite`, `-snapshot_directory`. Trace CPU via la commande debugger `trace <fichier>,maincpu,noloop[,{tracelog "..."}]` (lancée via `-debug` + `-debugscript`). Snapshot d'écran depuis Lua : `manager.machine.screens[":screen"]:snapshot(...)`. *Les noms exacts d'API Lua varient selon la version de MAME : la tâche T0.3 impose de les vérifier sur la version installée.*

Livrables de références à produire (T0.3) :
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
| Audio | **Non bit-exact** (limite du modèle discret, cf. KB-17). Critères : corrélation spectrale et enveloppe vs WAV MAME avec seuils fixés en T5.x |

---

# PARTIE E — BASE DE CONNAISSANCES (KB)

> Chaque fiche est autonome. L'Orchestrateur n'injecte que celles listées dans `context`. Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h), périmètre **Galaxian d'origine** uniquement.

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
**Valeurs dérivées (à vérifier par test T2.x)** :
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
- Fréquence : 60,6 Hz, à l'entrée en VBLANK (début de la ligne 240 selon VBSTART — **à confirmer par trace MAME**, T2.3).
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
(Transcription fidèle de la doc fournie ; l'affectation exacte bit→composante est à **re-vérifier dans galaxian_v.cpp**, GAP-02 : la doc liste « VERT » deux fois pour les bits 5 et 4 alors que le schéma standard est R: bits 0-2, G: bits 3-5, B: bits 6-7.)
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
À extraire de `galaxian_v.cpp` (T0.2) : format de `galaxian_videoram_w` / `galaxian_objram_w`, rôle des 0x40 premiers octets d'OBJRAM (attribut par colonne : scroll vertical + couleur), taille de tile (8×8, 32×32), décodage GFX 2 bitplanes depuis les ROM caractères.

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

## KB-14 — Rendu : ordre de composition (déduit de la doc, à confirmer T4.x)
1. fond noir ; 2. étoiles ; 3. tilemap ; 4. sprites (ordre de priorité : n° faible devant) ; 5. shells/missile. L'ordre exact de superposition étoiles/tilemap/sprites/shells est à **valider contre MAME** (GAP-01).

## KB-15 — Son : vue d'ensemble (moteur DISCRETE de MAME)
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

## KB-20 — État C++ de référence (galaxian_state)
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

## KB-21 — Spécifications Z80 (à fournir en complément)
La doc fournie **ne contient pas** la référence Z80. Pour T1.x, le Codeur doit recevoir : *The Undocumented Z80 Documented* (Sean Young), tableau de timing officiel Zilog (M-cycles/T-states par instruction), description MEMPTR/WZ et registre Q (flags SCF/CCF). Voir GAP-04.

---

# PARTIE F — PLAN : phases → étapes → sous-étapes

Légende : `deps` = dépendances ; `ctx` = fiches KB à injecter ; chaque sous-étape se termine par la commande de validation indiquée. En mode séquentiel, les sous-étapes sont exécutées **une par une, dans l'ordre de F.0**.

## F.0 Ordre d'exécution canonique (`order`)

```
PHASE 0 : T0.1.1 T0.1.2 T0.2.1 T0.2.2 T0.3.1 T0.3.2 T0.3.3 T0.4.1 T0.4.2
PHASE 1 : T1.1.1 T1.1.2 T1.1.3
          T1.2.1 T1.2.2 T1.2.3 T1.2.4 T1.2.5 T1.2.6 T1.2.7 T1.2.8
          T1.3.1 T1.3.2
          T1.4.1 T1.4.2 T1.4.3                       (T1.4.4 : voir PHASE 3)
PHASE 2 : T2.1.1 T2.1.2 T2.2.1 T2.2.2 T2.2.3 T2.2.4 T2.3.1 T2.3.2
PHASE 3 : T3.1.1 T3.1.2 T1.4.4 T3.1.3 T3.1.4
PHASE 4 : T4.1.1 T4.1.2 T4.2.1 T4.2.2 T4.3.1 T4.3.2 T4.3.3 T4.3.4
          T4.4.1 T4.5.1 T4.5.2 T4.5.3 T4.5.4 T4.6.1
PHASE 5 : T5.1.1 T5.2.1 T5.3.1 T5.3.2 T5.3.3 T5.3.4 T5.3.5 T5.4.1
PHASE 6 : T6.1.1 T6.1.2 T6.1.3 T6.1.4
PHASE 7 : T7.1.1 T7.1.2 T7.1.3 T7.1.4 T7.1.5
```

Choix d'ordonnancement :
- `T1.4.4` (trace MAME) a besoin d'une machine qui tourne : elle est placée après `T3.1.2` et avant `T3.1.3`, qui la valide (voir T3.1.3). Les `deps: P1` de `T3.1.2` s'entendent donc « Phase 1 hors `T1.4.4` ».
- La vidéo (phase 4) passe avant l'audio (phase 5), et dans la phase 4 l'ordre est tilemap → sprites → shells → étoiles → composition.
- Tout changement de cet ordre est une décision humaine.

---

## PHASE 0 — Fondations

### Étape 0.1 — Squelette du dépôt
**T0.1.1 — Workspace Cargo.** deps: — · ctx: — · livrables: arborescence Partie C, CI (`cargo fmt --check`, `clippy -D warnings`, `cargo test`). · validation: `cargo build --workspace && cargo test --workspace` (vert, 0 test).
**T0.1.2 — Types de temps.** deps: T0.1.1 · ctx: KB-02 · livrables: `galaxian-core/src/time.rs` : `type Cycles = u64` (T-states Z80), constantes `Z80_HZ=3_072_000`, `CYCLES_PER_LINE=192`, `LINES_PER_FRAME=264`, `CYCLES_PER_FRAME=50_688` ; fonctions de conversion cycle ↔ (ligne, colonne pixel) avec **pixel = 2 × cycle** (6,144 MHz / 3,072 MHz). · tests : unitaires sur les identités (192×264 = 50 688 ; ligne 240 = cycle 46 080). · validation: `cargo test -p galaxian-core time`.

### Étape 0.2 — Comblement des lacunes de doc
**T0.2.1 — Récupérer les sources MAME.** deps: T0.1.1 · ctx: KB-01 · tâche : télécharger les 5 URLs du fichier projet `Mame` (galaxian.cpp/.h, galaxian_a.cpp/.h, galaxian_v.cpp) dans `docs/mame_src/` (lecture seule, non compilé). · validation : fichiers présents + SHA256 consignés.
**T0.2.2 — Extraire les fiches manquantes.** deps: T0.2.1 · ctx: KB-09, KB-14 · tâche : produire `docs/kb/KB-22-gfx-layout.md` (décodage ROMs caractères/sprites, `GFXDECODE`, layouts), `KB-23-videoram-objram.md` (écritures VRAM/OBJRAM, attributs par colonne, scroll), `KB-24-inputs-dips.md` (bits IN0/IN1/IN2, DIP switches), `KB-25-machine-config.md` (config machine : CPU, watchdog timeout, écran, set de ROMs et offsets de chargement), `KB-26-render-order.md`. Chaque affirmation cite fichier + ligne. · validation : revue humaine ; fermer GAP-01, 02, 03.

### Étape 0.3 — Références MAME (golden)
**T0.3.1 — Script de trace CPU.** deps: T0.2.2 · tâche : `xtask golden trace` génère `golden/trace_boot_<N>.log` (PC, opcode, registres, cycles). Vérifier sur la version de MAME installée la syntaxe `trace`.
**T0.3.2 — Script de CRC de frames.** deps: T0.2.2 · tâche : Lua `autoboot_script` qui écrit le CRC32 de l'écran 256×224 aux frames listées (Partie D.2).
**T0.3.3 — Capture audio.** deps: T0.2.2 · tâche : `-wavwrite` sur scénarios (attract, tir, explosion).
validation: fichiers golden présents, regénération reproductible (2 runs = octets identiques).

### Étape 0.4 — TestROM Factory
**T0.4.1 — Chaîne d'assemblage.** deps: T0.1.1 · tâche : intégrer un assembleur Z80 (au choix de l'humain : sjasmplus/z80asm, ou mini-assembleur Rust) ; `xtask test-roms build` produit des `.bin` de 16 Ko (padding 0x00 ou 0xFF documenté).
**T0.4.2 — Catalogue de ROMs de test maison** (écrites progressivement, une par phase, voir tâches `TR-*` ci-dessous). Convention : chaque ROM écrit un **octet de verdict** en RAM (`0x4000` = `0xA5` pass / `0xEE` fail) *et* est comparée à MAME (trace + CRC frame).

---

## PHASE 1 — CPU Z80 cycle-accurate

Principe : machine à états **par M-cycle** avec T-states explicites ; `bus.advance(n)` avant chaque accès. Référence de comportement : MAME `z80.cpp` (non fournie ici : GAP-04).

### Étape 1.1 — Registres, flags, bus
**T1.1.1 — Registres & flags.** ctx: KB-21 · livrables `z80/src/regs.rs` : AF BC DE HL, registres alternés, IX IY SP PC I R IFF1 IFF2 IM, **WZ (MEMPTR)**, **Q** ; flags S Z Y H X P/V N C (bits 7..0 : S Z Y H X PV N C). · tests : unitaires sur accesseurs 8/16 bits.
**T1.1.2 — Trait `Bus` et harnais.** ctx: KB-21 · livrables : trait Partie C + `TestBus` (64 Ko RAM, journal d'événements `(cycle, type, addr, val)`).
**T1.1.3 — Cycles élémentaires.** ctx: KB-21 · fetch M1 = 4 T (R incrémenté, refresh), mem R/W = 3 T, I/O = 4 T. · validation: tests sur durées de ces primitives.

### Étape 1.2 — Jeu d'instructions (par blocs)
Pour chaque sous-étape : implémenter + tests unitaires + passage sur la suite Fuse partielle (cases concernées).
**T1.2.1** NOP, LD r,r' / r,n / r,(HL), LD (HL),r, LD rr,nn.
**T1.2.2** Arithmétique 8 bits (ADD/ADC/SUB/SBC/AND/OR/XOR/CP, INC/DEC), DAA, CPL, NEG, SCF/CCF (**flags X/Y via Q**).
**T1.2.3** Arithmétique 16 bits, PUSH/POP, EX, EXX.
**T1.2.4** Sauts/appels/retours (JP, JR, DJNZ, CALL, RET, RST), conditions ; **MEMPTR** mis à jour.
**T1.2.5** Préfixe CB (rotations, BIT — flags X/Y issus de MEMPTR pour BIT n,(HL) —, SET, RES).
**T1.2.6** Préfixes DD/FD (IX/IY, IXH/IXL non documentés, déplacements), DDCB/FDCB.
**T1.2.7** Préfixe ED (LD I/R, IM, RETN/RETI, block ops LDI/LDD/LDIR/LDDR/CPI/CPD/CPIR/CPDR/INI/IND/INIR/INDR/OUTI/OUTD/OTIR/OTDR, RRD/RLD, IN/OUT (C), ED non documentés).
**T1.2.8** HALT, DI/EI (délai d'activation d'un cycle), IN/OUT (n).
validation générale : `cargo xtask run-fuse` (taux de réussite par bloc) ; objectif final 100 %.

### Étape 1.3 — Interruptions
**T1.3.1 — NMI.** ctx: KB-05, KB-21 · front montant, 11 T-states, push PC, saut à 0x0066, IFF1→IFF2 ; interaction avec HALT et EI.
**T1.3.2 — INT (IM 0/1/2).** ctx: KB-21 · implémentés pour exhaustivité (Galaxian utilise l'NMI).
validation : tests Fuse/z80test concernés + test maison `TR-CPU-NMI`.

### Étape 1.4 — Validation CPU globale
**T1.4.1 — zexdoc/zexall.** deps: 1.2, 1.3 · tâche : shim CP/M minimal (BDOS 2 = putchar, 9 = print string, adresse 0x0005, warm boot 0x0000) · validation: `cargo xtask run-zex zexdoc` et `zexall` → toutes les lignes « OK » (CRC conformes).
**T1.4.2 — z80test (Rak).** · validation : z80doc, z80full, z80ccf, z80memptr → tous passés.
**T1.4.3 — Timing bus par T-state.** · validation : suite Fuse avec événements MR/MW/PR/PW/MC ; si disponible, SingleStepTests/z80 (cycles + bus) → 0 divergence.
**T1.4.4 — Comparaison de trace avec MAME.** deps: T0.3.1 + Phase 3 minimale (exécutée après T3.1.2, voir F.0) · validation : `cargo xtask mame-diff trace` sur N = 1 000 000 instructions de la ROM Galaxian → première divergence = aucune.

---

## PHASE 2 — Horloge système, bus machine, NMI

### Étape 2.1 — Horloge et balayage
**T2.1.1 — Compteurs vidéo.** ctx: KB-02, KB-07 · livrables `galaxian-core/src/raster.rs` : `Raster::advance(cycles)` → (ligne 0..263, pixel 0..383) ; drapeaux `hblank`, `vblank`, `hsync`, `vsync` ; événement « entrée en VBLANK » horodaté. · tests : frame = 50 688 cycles ; ligne visible 16..239 ; pixel visible 0..255.
**T2.1.2 — Retard du compteur V.** ctx: KB-07 · tâche : exposer `v_counter_hw()` reproduisant le retard d'un cran pendant les 48 premiers clocks H du blanking (utilisé par sprites/shells, T4.3).

### Étape 2.2 — Bus machine
**T2.2.1 — Décodage mémoire.** ctx: KB-03, KB-04 · livrables `bus.rs` : mapping exact avec miroirs, `unmap_value_high` (lecture non mappée = 0xFF), écriture ROM ignorée. · tests : table de 100+ adresses (y compris mirrors) vs attendu.
**T2.2.2 — Latches d'écriture.** ctx: KB-04 · `6000-6007`, `6800-6807`, `7001/7004/7006/7007` : 1 bit (D0), adresse masquée `& 7` avec miroir `0x07f8`. · tests : écriture sur miroirs.
**T2.2.3 — Entrées (IN0/IN1/IN2).** ctx: KB-24 (après T0.2.2) · API `Inputs` (coin, start, joystick, tir, DIPs).
**T2.2.4 — Watchdog.** ctx: KB-06, KB-25 · tâche : compteur en cycles ; lecture de 7800 le remet à zéro ; timeout → `reset()` machine. · tests : `TR-WDG` (ROM de test qui n'alimente pas le watchdog → reset détecté).

### Étape 2.3 — NMI VBLANK
**T2.3.1 — Génération de l'NMI.** deps: T1.3.1, T2.1.1, T2.2.2 · ctx: KB-05, KB-19 · tâche : à l'entrée en VBLANK, si `irq_enabled` → ligne NMI haute ; écriture 7001=0 → ligne basse. · tests : ROM `TR-NMI` (compte les NMI dans un registre RAM ; après 600 frames le compteur = 600 ± 0 ; avant écriture de 7001 : 0).
**T2.3.2 — Cycle exact de l'NMI.** deps: T2.3.1, T0.3.1 · validation : le **cycle d'entrée dans le handler 0x0066** est identique à la trace MAME (`TR-NMI` + ROM Galaxian).

---

## PHASE 3 — Machine minimale bootable

**T3.1.1 — Chargement des ROMs.** ctx: KB-25 · lecture d'un répertoire/zip du set `galaxian` ou `galmidw` fourni par l'utilisateur, vérif CRC, placement 0x0000.. et ROMs graphiques/PROM.
**T3.1.2 — Boucle principale.** deps: P1, P2 · `Machine::run_frame()` : exécute exactement 50 688 cycles (instruction non interrompue qui franchit la frontière : cycles excédentaires reportés).
**T3.1.3 — Boot sans vidéo.** · validation : `mame-diff trace` sur les 100 000 premières instructions (T1.4.4 ici validé).
**T3.1.4 — Dump VRAM.** · validation : contenu de VRAM/OBJRAM à la frame 60 identique à MAME (dump Lua).

---

## PHASE 4 — Vidéo pixel-exact

Sortie : framebuffer **256×224 RGB** (après réduction du ×3 interne, ou rendu interne 768×224 pour fidélité des étoiles puis downscale documenté).

### Étape 4.1 — Graphismes
**T4.1.1 — Décodage GFX.** ctx: KB-22 (T0.2.2), KB-09 · tiles 8×8 2bpp, sprites 16×16 2bpp, tables de planes/offsets. · tests : décodage d'une tile connue vs MAME `gfxviewer`/dump.
**T4.1.2 — Palette PROM.** ctx: KB-08 · calcul `compute_resistor_weights` (algo MAME, à recopier depuis la source), 32 entrées, `RGB_MAXIMUM=224`. · tests : valeurs RGB des 32 entrées = MAME.

### Étape 4.2 — Tilemap
**T4.2.1 — Rendu du fond.** ctx: KB-09, KB-23, KB-13 · 32×32 tiles, scroll vertical par colonne, attribut couleur par colonne, flips X/Y. · validation: `TR-TILE` (ROM qui écrit un motif connu) → CRC identique à MAME.
**T4.2.2 — Rendu **par ligne** (timing).** · tâche : le rendu doit utiliser l'état VRAM/OBJRAM **au moment de chaque ligne** (écriture en milieu de frame visible sur la ligne suivante) ; équivalent de `update_partial`. · validation : `TR-RASTER` (modifie la VRAM en milieu de frame) → CRC MAME.

### Étape 4.3 — Sprites (modèle line buffer)
**T4.3.1 — Sélection des sprites par ligne.** ctx: KB-10, T2.1.2 · test vertical `((V+vpos)&0xf0)==0xf0` avec le V matériel retardé.
**T4.3.2 — Line buffer & priorité.** ctx: KB-10 · écriture seulement si pixel = 0 ; sprites 0..7 ; flips ; couleur 3 bits.
**T4.3.3 — Clipping 16 px.** ctx: KB-10 · formule `sprites_clip` (avec flip).
**T4.3.4 — Limite de 7,5 sprites/ligne.** ctx: KB-10 · reproduire le sprite partiellement rendu.
validation : `TR-SPR-*` (1 sprite, 8 sprites alignés, bords, flips) → CRC MAME.

### Étape 4.4 — Shells & missile
**T4.4.1 — Shells/missile.** ctx: KB-11, KB-10 · match vertical `&0xff==0xff`, début à H=$FC, 4 px, « dernier trouvé gagne », 7 blancs + 1 jaune.
validation : `TR-BULLET` → CRC MAME.

### Étape 4.5 — Étoiles
**T4.5.1 — Table LFSR.** ctx: KB-12 · génération de `stars[131071]` exactement comme la doc ; test : CRC de la table = valeur MAME.
**T4.5.2 — Rendu 3:2.** ctx: KB-12, KB-02 · motif 1 px / 2 px alternés, masque `(V1 xor H8)`, couleur par table de 64.
**T4.5.3 — Origine et dérive.** ctx: KB-12, KB-13 · `stars_update_origin`, dérive d'un cran/frame, décalage 2 clocks/frame (bascules 6B).
**T4.5.4 — Flip & étoiles.** ctx: KB-13 · recalage avant flip.
validation : `TR-STARS` (étoiles ON, 0/1/10/100 frames, avec et sans flip) → CRC MAME.

### Étape 4.6 — Composition finale
**T4.6.1 — Ordre de superposition.** ctx: KB-14, KB-26 · validation : CRC de la ROM Galaxian aux frames 1, 2, 5, 10, 30, 60, 120, 300, 600 = `golden/frames_crc.txt`.

---

## PHASE 5 — Audio

> Non bit-exact (KB-17). Approche en 3 niveaux, chacun validé par ses propres critères.

### Étape 5.1 — LFSR partagé
**T5.1.1 — LFSR unique.** ctx: KB-12, KB-16, KB-19 · une seule instance dans `galaxian-core`, lue par vidéo (étoiles) **et** audio. · test : séquence 131 071 états puis retour à l'état initial.

### Étape 5.2 — Registres son
**T5.2.1 — Latches son.** ctx: KB-04, KB-15 · FS1-3, HIT, FIRE, VOL1/2, pitch, `lfo_val` 4 bits.

### Étape 5.3 — Modèle discret minimal
**T5.3.1 — Primitives.** ctx: KB-16 · 555 astable (avec/sans CV), 555 VCO, RC charge/décharge, filtre RC, passe-bande 1er ordre, DAC R-2R, mixeur pondéré. Tests unitaires analytiques (fréquence 555 = 1.44/((R1+2R2)C)).
**T5.3.2 — Bloc PITCH.** ctx: KB-16 · `SOUND_CLOCK/(256−pitch)`, décodage QA/QC/QD.
**T5.3.3 — Bloc BACKGROUND.** ctx: KB-16 · chaîne DAC→VCO→clamp→3×555→mixeur.
**T5.3.4 — Blocs HIT et FIRE.** ctx: KB-16.
**T5.3.5 — Mixage final.** ctx: KB-16 · sortie échantillonnée 44,1/48 kHz.

### Étape 5.4 — Validation audio
**T5.4.1 — Comparaison aux WAV MAME.** ctx: KB-17 · métriques : enveloppe RMS (corrélation ≥ seuil), spectre (fréquence fondamentale à ±1 %), pour chaque scénario golden. Les seuils sont **fixés par l'humain** après un premier essai ; ne pas les ajuster pour « faire passer ».
Tests maison : `TR-SND-FS`, `TR-SND-HIT`, `TR-SND-FIRE`, `TR-SND-PITCH`.

---

## PHASE 6 — Intégration & frontend

**T6.1.1 — Frontend.** fenêtre 256×224 (ratio d'origine, écran vertical : rotation 90° optionnelle), boucle 60,606 Hz (ou sync audio).
**T6.1.2 — Entrées clavier/manette → IN0/IN1.** ctx: KB-24.
**T6.1.3 — Audio out.** file d'échantillons, resampling.
**T6.1.4 — Scénarios de non-régression.** boot, attract, crédit, partie 1 min, flip (cocktail), DIPs ; CRC de frames vs MAME.

---

## PHASE 7 — Durcissement

**T7.1.1 — Déterminisme.** 2 runs identiques bit à bit sur 10 min d'émulation.
**T7.1.2 — Save-states.** sérialisation complète (CPU, RAM, latches, raster, LFSR, étoiles, audio) ; test round-trip.
**T7.1.3 — Performance.** objectif indicatif : ≥ 20× temps réel sur machine moderne, sans dégrader l'exactitude.
**T7.1.4 — Fuzz/proptest.** accès mémoire aléatoires, écritures sur miroirs, séquences de latches.
**T7.1.5 — Documentation finale.** écarts connus vs MAME (audio), instructions de génération des golden.

---

## Catalogue des ROMs de test maison (TestROM Factory)

| ID | Phase | Vérifie |
|---|---|---|
| TR-CPU-NMI | 1 | Entrée NMI, 0x0066, RETN, IFF |
| TR-NMI | 2 | 1 NMI/frame, activation par 7001 |
| TR-WDG | 2 | Reset par watchdog |
| TR-MAP | 2 | Miroirs mémoire, lecture non mappée = 0xFF |
| TR-TILE | 4 | Tilemap, scroll colonne, attributs, flips |
| TR-RASTER | 4 | Écritures VRAM en cours de frame |
| TR-SPR-* | 4 | Sprites : position, flips, priorité, bords, 8/ligne |
| TR-BULLET | 4 | Shells/missile |
| TR-STARS | 4 | Étoiles, dérive, flip |
| TR-SND-* | 5 | FS1-3, HIT, FIRE, pitch, volumes |

Chaque ROM : source `.asm` commentée, `.bin`, trace MAME, CRC de frames, et un README « ce qui est testé / ce qui est attendu ».

---

# PARTIE G — Lacunes de la documentation fournie

| ID | Lacune | Impact | Comment la combler |
|---|---|---|---|
| GAP-01 | Format VRAM/OBJRAM détaillé, attributs par colonne, ordre de composition, taille/layout tilemap | Phase 4 | T0.2.2 (extraction depuis `galaxian_v.cpp`) |
| GAP-02 | Affectation exacte bit→R/G/B de la PROM (la doc liste deux fois « VERT » pour bits 5 et 4) | Palette | Relire `galaxian_palette` dans la source ; test T4.1.2 |
| GAP-03 | Durée du timeout watchdog, table des ROMs (noms, tailles, offsets), config machine | Phases 2-3 | T0.2.2 (`galaxian.cpp` : `WATCHDOG_TIMER`, `ROM_START`) |
| GAP-04 | Référence Z80 (timings par instruction, flags non documentés, MEMPTR, Q) | Phase 1 | Fournir KB-21 depuis sources externes (Sean Young, Zilog, suites de test) |
| GAP-05 | Ports d'entrée IN0/IN1/IN2 et DIPs | Entrées | `INPUT_PORTS_START` dans `galaxian.cpp` (T0.2.2) |
| GAP-06 | Cycle exact de l'NMI par rapport au début du VBLANK (ligne 240 ?) | NMI | Trace MAME (T2.3.2) |
| GAP-07 | Absence de ROMs de test Galaxian publiques | Validation | TestROM Factory (T0.4) |

**Règle** : si une tâche dépend d'une lacune `OPEN`, l'Orchestrateur la marque `BLOCKED`, arrête la chaîne et propose d'insérer la tâche de comblement juste avant dans `order` (voir B.5).
