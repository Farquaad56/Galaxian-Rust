# Notes MAME — environnement installé (T0.5.1)

> Tâche : **T0.5.1 — Environnement MAME**. Ferme **GAP-08** (syntaxe MAME réelle non vérifiée sur `tools\mame.exe`).
> Binaire testé : `tools\mame.exe` = **MAME v0.289 (mame0289)**, lancé depuis la racine du dépôt (`H:\__Emulator__\Arcade_Galaxian_Rust`) avec `-rompath "roms;tools/roms"`.
> Toutes les sorties ci-dessous ont été produites sur ce binaire ; les artefacts bruts sont dans `golden/tmp/` (non versionné). Les fiches `KB-28a.md` / `KB-28b.md` en portent la correction.

## 1. Version installée

```
$ tools\mame.exe -help
MAME v0.289 (mame0289)
Copyright MAMEdev and contributors
...
Usage:  mame [machine] [media] [software] [options]
        mame -showusage    for a list of options
        mame -showconfig   to show current configuration in mame.ini format
```

`tools\whatsnew.txt` : `0.289 2026-07-31`. La doc locale `tools\docs\MAME.pdf` + `tools\docs\man\` correspondent à cette version ; c'est elle qui fait foi (pas docs.mamedev.org).

## 2. Sorties des commandes de vérification (KB-28a §2)

### `-verifyroms galaxian`
```
romset galaxian is good
1 romsets found, 1 were OK.
```
✔ Le set `galaxian` est complet et valide dans `tools\roms\galaxian.zip`.

### `-listfull "galaxian*"` (noms réels des sets)
```
Name:             Description:
galaxian          "Galaxian (Namco set 1)"
galaxiana         "Galaxian (Namco set 2)"
galaxianbl        "Galaxian (bootleg, set 2)"
galaxianbl2       "Galaxian (bootleg, set 4)"
galaxianbl3       "Galaxian (Spanish bootleg)"
galaxianem        "Galaxian (Electromar Spanish bootleg)"
galaxiani         "Galaxian (Irem)"
galaxianiii       "Galaxian III (bootleg of Galaxian)"
galaxianm         "Galaxian (Midway set 1)"
galaxianmo        "Galaxian (Midway set 2)"
galaxianoly       "Galaxian (Olympia bootleg)"
galaxianrp        "Galaxian (Rene Pierre bootleg)"
galaxiant         "Galaxian (Taito)"
galaxian_sound    "Galaxian Custom Sound"
```

### `-listroms galaxian` / `-listcrc galaxian`
Le driver `galaxian` (Namco set 1) requiert : `galmidw.u/.v/.w/.y`, `7l`, `1h.bin`, `1k.bin`, `6l.bpr`. CRC/SHA1 consignés dans `golden/tmp/listroms.txt` et `listcrc.txt`.

### `-listdevices galaxian` (tags des périphériques)
```
Driver galaxian (Galaxian (Namco set 1)):
   <root>                         Galaxian (Namco set 1)
     cust                         Galaxian Custom Sound
       discrete                   Discrete Sound
     gfxdecode                    gfxdecode
     maincpu                      Zilog Z80 @ 3.07 MHz
     palette                      palette
     screen                       Video Screen @ 18.43 MHz
     speaker                      Speaker
     watchdog                     Watchdog Timer
```
Tags utiles : `:maincpu`, `:screen`. (Les ports d'entrée `:IN0/:IN1/:IN2` ne sont pas listés ici — voir `-listxml`.)

### `-listxml galaxian` → `golden/galaxian.xml`
Écran et ports consignés dans `golden/galaxian.xml` :
```xml
<chip type="cpu" tag="maincpu" name="Zilog Z80" clock="3072000"/>
<display tag="screen" type="raster" rotate="90" width="768" height="224" refresh="60.606061"
         pixclock="18432000" htotal="1152" hbend="0" hbstart="768" vtotal="264" vbend="16" vbstart="240"/>
```
- `width=768 height=224` → le bitmap MAME est **768×224** (rendu ×3 horizontal, cf. KB-02 `GALAXIAN_XSCALE=3`). ✔
- `rotate="90"` → la capture PNG est tournée : 224×768 au repos (voir §3.11).
- Les blocs `<port tag=":IN0">` / `:IN1` / `:IN2` sont **vides** dans le XML ; les libellés réels des champs viennent de la doc Lua (§3.9) et de KB-24a, pas du XML.

### `-showusage` → `golden/tmp/showusage.txt`
Toutes les options du profil commun (KB-28a §3) sont présentes : `-seconds_to_run`, `-nothrottle`, `-frameskip`, `-autoboot_script`, `-autoboot_delay`, `-debug`, `-debugscript`, `-debugger`, `-wavwrite`, `-samplerate`, `-snapname/-snapsize/-snapview`, `-record/-playback/-exit_after_playback`, `-log/-oslog/-verbose`, `-watchdog`, `-bench`, `-rompath`, `-noreadconfig`, `-cfg_directory/-nvram_directory/-snapshot_directory/-input_directory/-state_directory`, `-skip_gameinfo`. ✔
`-showconfig` → `golden/tmp/showconfig.ini` (484 lignes) : défauts effectifs `frameskip 0`, `throttle 1`, `samplerate 48000`, `video auto`.

## 3. Tranchement de chaque ⚠ de KB-28a/b

Légende : **✔** confirmé sur le binaire · **✘** infirmé (correction donnée) · **⚠→** écart constaté, consigné.

### 3.1 Nom réel du set Midway — ✔ tranché
Le plan écrit `galmidw` ; la doc liste `galaxianm`/`galaxianmo`. `-listfull` confirme : le **set Midway s'appelle `galaxianm`** (Midway set 1) et `galaxianmo` (Midway set 2). Il n'existe **pas** de set nommé `galmidw` — `galmidw.*` sont seulement les *noms de ROM fichiers* du driver `galaxian`. → Corriger T0.4.9 / T3.1.7 : utiliser `galaxianm`, pas `galmidw`.

### 3.2 `mame.exe` bloque-t-il l'invite ? — ✔
Oui. `tools\mame.exe … -seconds_to_run N` **bloque jusqu'à la fin** de l'exécution, puis retourne le code d'erreur dans `$LASTEXITCODE` (0 = succès). Pas besoin de `Start-Process -Wait`. Exemples : run de 10 s → exit 0 ; run arrêté par le debugger → exit 0.

### 3.3 Syntaxe `trace`/`tracelog`, `symlist maincpu` — ✔
Commandes du debugger (fichier `-debugscript`) : `trace {fichier|OFF}[,cpu[,[noloop|logerror][,action]]]`, `tracesym`, `traceflush`, `symlist <cpu>`, `source`, `gvblank`, `gtime`, `quit`.
- **`symlist maincpu`** : la sortie va dans **`debug.log`** (avec `-debuglog`), **pas** sur stdout. Extrait (`golden/tmp/debug.log`) :
  ```
  **** CPU ':maincpu' symbols ****
  a af af2 b bc bc2 c curflags curpc cycles d de de2 e f h halt hl hl2 i iff1 iff2 im ix iy l lastinstructioncycles logunmap pc r sp totalcycles wz
  ```
- **Compteur de cycles : OUI, il existe.** Trois symboles en lecture seule : `totalcycles` (cumul T-states), `lastinstructioncycles` (T-states de la dernière instruction), et `cycles` (compte à rebours irrégulier qui se réinitialise sur les événements vidéo — **ne pas** l'utiliser). → La colonne `cycles` de la trace normalisée (T0.5.4) peut être remplie depuis `totalcycles`/`lastinstructioncycles`, **pas** reconstruite depuis KB-21b.

### 3.4 Format d'une ligne de trace — ✔
Avec l'action `{tracelog "AF=%04X BC=%04X DE=%04X HL=%04X SP=%04X OP=%02X%02X%02X%02X ",af,bc,de,hl,sp,b@pc,b@(pc+1),b@(pc+2),b@(pc+3)}`, chaque ligne est :
```
AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 OP=320170C3 0001: ld   ($7001),a
```
c'est-à-dire `<texte de l'action> <PC>: <désassemblage>`. Les 4 octets `OP=` couvrent les opcodes préfixés (CB/DD/ED/FD + DDCB/FDCB). Exemple réel : `golden/tmp/trace_raw.log` (1 212 469 lignes pour ~3 s émulees, ≈8 T-states/instruction en moyenne).

### 3.5 Premier PC de la trace — ⚠→ écart
La première ligne est à **PC = 0001** (`ld ($7001),a`), **pas 0000**. Il n'y a aucune ligne `0000:` dans le fichier (vérifié sur les 4 traces). Le reset logiciel de MAME démarre l'exécution au vecteur suivant ; la trace s'ouvre avant la *première instruction exécutée* (PC=0001), pas à PC=0000. → **T0.5.3** : la validation « première ligne à PC = 0000 » doit être corrigée en « première ligne à PC = 0001 ».

### 3.6 Lignes après `go` — ✔
Les lignes **après** `go` dans un `-debugscript` s'exécutent **immédiatement** (sans attendre l'émulation). Test : `trace … / go / gtime 2` → MAME s'est arrêté à ~2 s émulees (843 267 lignes), pas à la fin du run. → Ne rien mettre après `go` ; la fin du run vient de `-seconds_to_run`.

### 3.7 Fichier de trace complet à la sortie ? — ✔
Oui. Le fichier se termine par une ligne **complète** + `\n` (dernier octet = `0x0A`). Exemple : dernière ligne `AF=0120 … OP=0FD80E10 2077: rrca`. Pas besoin de `traceflush`.

### 3.8 Fenêtre debugger avec `-seconds_to_run` — ✔
Le debugger **Windows** s'ouvre par défaut (`-debugger windows`). ~~Pour un run headless, utiliser `-debugger none` : MAME tourne sans fenêtre et le fichier de trace est écrit normalement (test `golden/tmp/runDN`, exit 0).~~ → **corrigé [T0.5.3]** : `-debugger none` n'exécute JAMAIS `-debugscript` — le test `runDN` ne vérifiait que l'exit code, pas la sortie de trace ; MAME saute le script entièrement et le fichier de trace n'est PAS écrit. Seul le debugger **windows** par défaut exécute le script (la fenêtre s'ouvre brièvement) ; `-seconds_to_run` arrête MAME correctement sous debugger.

### 3.9 API Lua — ✔
Confirmé sur le binaire (`golden/tmp/lua_api*.lua` + sorties) :
- `manager.machine.screens[":screen"]` → objet écran ; `.width`=768, `.height`=224.
- `scr:frame_number()` → **base 0** (vaut 0 au boot). ✔
- `scr:pixels()` → chaîne d'octets, longueur = width×height×4 = **688128**. ✔
- `manager.machine.devices[":maincpu"].spaces["program"]` : `.read_u8(addr)` **existe** (et `.read_i8`). ✔
- `manager.machine.ioport.ports` → table ; `ports[":IN0"]`, `ports[":IN1"]`, `ports[":IN2"]`. Chaque port a `.fields` (table) et `:read()` (renvoie le byte du port).
  - Libellés réels des champs : **`:IN0`** = `P1 Button 1, Coin 1, Service Mode, Service 1, P1 Right, P1 Left, Cabinet, Coin 2` ; **`:IN1`** = `2 Players Start, Coinage, P2 Button 1, 1 Player Start, P2 Left, P2 Right` ; **`:IN2`** = `Unused, Bonus Life, Lives`.
  - `field:set_value(n)` **fonctionne**. Il n'y a **pas** de read-back par champ (`:value()`/`:get_value()`/`:read()` sur un champ → nil). Pour lire l'état d'un port, utiliser `port:read()`.
- `manager.machine:exit()` → termine le run. ✔
- `io.open` / `io` **disponibles** dans le sandbox Lua (scripts écrivent des fichiers normalement). ✔
- `emu.register_frame_done(fn, "frame")` → callback par frame. ✔
- `emu.app_version()` → `"0.289"`. ✔

### 3.10 `first_frame_seen` avec `-autoboot_delay 0` — ✔
Avec `-autoboot_delay 0 -autoboot_script f.lua`, le script démarre à **frame 0** : la ligne d'en-tête est `# first_frame_seen=0 size=768x224 mame=0.289`. La frame 1 n'est pas manquée. (Test `golden/tmp/ffs.txt`.)

### 3.11 Dimensions réelles du bitmap — ✔
Le bitmap MAME est **768×224** (`scr.width`/`scr.height`, et `<display width=768 height=224>`). La capture PNG par défaut est tournée à cause de `rotate="90"` : **224×768** au repos. Le CRC golden porte sur le bitmap MAME tel quel (768×224) ; la comparaison avec la sortie 256×224 de l'émulateur reste une décision humaine (T0.5.5).

### 3.12 Effet de `-video none` / `-sound none` — ✔
- **`-video none`** : le bitmap est **bien rendu**. `pixels()` est tout zéro au boot mais devient non-noir dès que des frames avancent (CRC ≠ CRC d'un écran noir à partir de frame ~10). → Conserver `-video none`. MAME émet un avertissement `Warning: -video none doesn't make much sense without -seconds_to_run` — sans objet si on termine par `-seconds_to_run` ou `machine:exit()`.
- **`-sound none`** : le son reste émulé ; `-wavwrite` produit un WAV **non vide**. → Conserver `-sound none`.

### 3.13 Dossier de sortie de `-wavwrite` — ✔
Le fichier est écrit au **chemin donné, relatif au répertoire courant** (pas dans le dossier snap). Test : `-wavwrite golden/tmp/audio_none.wav` → `golden/tmp/audio_none.wav` créé. En-tête vérifié sur 4 WAV (`audio_1s/48k/def/none`) : mono, 16 bits, **sample rate = 48000 Hz** (valeur demandée par `-samplerate`, y compris le défaut) ; le *byte rate* de l'en-tête vaut 96000 (= 48000 × 2 octets), à ne pas confondre avec la fréquence d'échantillonnage. La durée du WAV = temps émulé (ex. `audio_none.wav` : 240 001 frames ≈ 5 s). Consigner : comparer les octets de données, pas seulement l'en-tête.

### 3.14 `-no_coin_lockout` — ✘ infirmé
L'option **n'existe pas** dans MAME 0.289 (`Error: unknown option: -no_coin_lockout`). Pour forcer des crédits sans coin lockout, fixer le champ `Coin 1` via Lua (§3.9), pas via un flag CLI.

## 4. Conséquences pour les tâches suivantes
- **T0.5.2** (lanceur) : le profil commun de KB-28a §3 est valide tel quel ; ~~ajouter `-debugger none` aux runs headless~~ → **corrigé [T0.5.3]** : ne PAS ajouter `-debugger none` — il saute le script de trace entièrement ; conserver le debugger `windows` par défaut pour les runs de trace (la fenêtre s'ouvre brièvement, MAME s'arrête via `-seconds_to_run`).
- **T0.5.3** (trace brute) : première ligne à PC=0001 (pas 0000) — corriger la validation.
- **T0.5.4** (trace normalisée) : colonne `cycles` remplissable depuis `totalcycles`/`lastinstructioncycles`.
- **T0.5.5** (CRC frames) : bitmap 768×224 ; `first_frame_seen=0` attendu avec `-autoboot_delay 0`.
- **T0.5.7–0.5.9** (audio) : WAV non vide avec `-sound none` ; comparer les octets de données, pas l'en-tête sr.
- **T0.4.9 / T3.1.7** : le set Midway est `galaxianm`, pas `galmidw`.
