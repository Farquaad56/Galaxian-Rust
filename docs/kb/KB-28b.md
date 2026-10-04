# KB-28b — Scripts golden : trace debugger, Lua (CRC, dumps, entrées), audio

*Fiche fournie. Même légende que KB-28a. Les scripts ci-dessous sont des **squelettes** à valider en T0.5.1, pas du code final. Ils supposent : répertoire courant = `tools\`, profil `$common` de KB-28a §3 chargé, dossier `tests\golden\mame\` existant.*

> **Corrigée en T0.5.1** : chaque ⚠ a été tranché sur `tools\mame.exe` v0.289 (sorties consignées dans `docs/mame_notes.md`). Les corrections sont marquées **[T0.5.1]** ; les ⚠ résolus passent en ✔/✘ avec la conclusion.

**A. Trace CPU (debugger)** : fichier `tests/golden/mame/trace.dbg` :
```
trace ../tests/golden/tmp/trace_raw.log,maincpu,noloop,{tracelog "AF=%04X BC=%04X DE=%04X HL=%04X SP=%04X OP=%02X%02X%02X%02X ",af,bc,de,hl,sp,b@pc,b@(pc+1),b@(pc+2),b@(pc+3)}
go
```
```powershell
& .\mame.exe @common -debug -debugscript ..\tests\golden\mame\trace.dbg -seconds_to_run 10
```
- ✔ `trace {fichier|OFF}[,cpu[,[noloop|logerror][,action]]]` ; **sans `noloop`, les boucles sont condensées en une ligne : toujours `noloop`.** L'`action` entre accolades est exécutée avant chaque ligne ; `tracelog "format",args` écrit dans le fichier de trace ouvert (sans effet si aucun n'est ouvert). Autres commandes : `tracesym`, `traceflush`, `symlist <cpu>`, `source`, `gvblank`, `gtime`, `quit`.
- ✔ Avec `-debug`, MAME s'arrête dans le debugger après le reset logiciel initial : la trace est ouverte **avant** la première instruction (PC = 0000) ; `go` lance l'exécution, `-seconds_to_run` y met fin.
**[T0.5.1] Écart constaté** : la première ligne est en réalité à **PC = 0001** (`ld ($7001),a`), pas 0000 — il n'y a aucune ligne `0000:` dans le fichier (vérifié sur les 4 traces). Le reset logiciel démarre l'exécution au vecteur suivant. → **T0.5.3** : corriger la validation « première ligne à PC = 0000 » en « première ligne à PC = 0001 ».
- ⚠ Les lignes **après** `go` dans un `-debugscript` sont probablement exécutées immédiatement (sans attendre l'émulation) : ne rien mettre après `go` ; la fin du run vient de `-seconds_to_run`. À constater : la dernière ligne du fichier est-elle complète (fichier fermé à la sortie) ? Sinon tester `traceflush`.
**[T0.5.1] Tranché** : les lignes après `go` s'exécutent **immédiatement** — test `trace … / go / gtime 2` → MAME s'est arrêté à ~2 s émulees (843 267 lignes), pas à la fin du run. La dernière ligne du fichier est **complète** + `\n` (dernier octet = `0x0A`) ; pas besoin de `traceflush`.
- ⚠ Syntaxe des expressions : `af bc de hl sp pc` (symboles du Z80, vérifier avec `symlist maincpu`), `b@adresse` (octet en mémoire). Une ligne de trace = texte de l'`action` puis `pc: désassemblage` (format exact à relever). Les 4 octets à `pc` couvrent les opcodes préfixés (CB/DD/ED/FD + DDCB/FDCB).
**[T0.5.1] Tranché** : format d'une ligne = `<texte de l'action> <PC>: <désassemblage>`, ex. `AF=0044 BC=0000 DE=0000 HL=0000 SP=0000 OP=320170C3 0001: ld   ($7001),a`. Les symboles Z80 (`af bc de hl sp pc`, `iff1/iff2/im/r/wz/halt`) et les compteurs de cycles existent (voir ci-dessous).
- ⚠ **Cycles** : MAME n'écrit pas les cycles dans une trace par défaut. Chercher un symbole de compteur de cycles (`symlist maincpu`) ; s'il existe, l'ajouter à la `action`. **S'il n'existe pas, ne pas reconstruire les cycles à partir des tables KB-21b (raisonnement circulaire)** : laisser la colonne vide, comparer les instants via T2.5.7 ; décision humaine à consigner.
**[T0.5.1] Tranché — le compteur de cycles EXISTE.** `symlist maincpu` (sortie dans **`debug.log`** avec `-debuglog`, pas sur stdout) expose trois symboles en lecture seule : `totalcycles` (cumul T-states), `lastinstructioncycles` (T-states de la dernière instruction), et `cycles` (compte à rebours irrégulier qui se réinitialise aux événements vidéo — **ne pas** l'utiliser). → La colonne `cycles` de la trace normalisée (T0.5.4) peut être remplie depuis `totalcycles`/`lastinstructioncycles`, **pas** reconstruite depuis KB-21b. Exemple : `{tracelog "TOT=%d LIC=%d AF=%04X PC=%04X ",totalcycles,lastinstructioncycles,af,pc}` → `TOT=4 LIC=4 AF=0044 PC=0001 0001: ld   ($7001),a`.
- ⚠ La fenêtre du debugger Windows s'ouvre ; `-video none` peut changer son comportement. Si `-seconds_to_run` n'arrête pas MAME sous debugger, utiliser `-watchdog` en filet et consigner.
**[T0.5.1] Tranché** : le debugger **Windows** s'ouvre par défaut (`-debugger windows`). ~~Pour un run headless, utiliser `-debugger none` (MAME tourne sans fenêtre, le fichier de trace est écrit normalement).~~ → **corrigé [T0.5.3]** : `-debugger none` n'exécute JAMAIS `-debugscript` — MAME saute le script entièrement et le fichier de trace n'est PAS écrit ; `gdbstub` reste bloqué en attente d'un client. Pour exécuter réellement le script de trace, conserver le debugger **windows** par défaut : la fenêtre s'ouvre brièvement et `-seconds_to_run` arrête MAME correctement sous debugger.
- Pas d'option « N instructions » : produire plus de lignes que nécessaire, puis tronquer à N dans `xtask golden trace`. Ordre de grandeur : ~8 T-states par instruction en moyenne → 1 000 000 d'instructions ≈ 3 à 4 s émulees (⚠ à mesurer) ; viser `-seconds_to_run 10`.
**[T0.5.1] Mesuré** : un run de ~3 s émulees produit **1 212 469 lignes** (`golden/tmp/trace_raw.log`) → ≈8 T-states/instruction en moyenne confirmé. Pour 1 000 000 d'instructions, viser `-seconds_to_run 3` (ou 10 pour marge).

**B. Lua : CRC des frames** : fichier `tests/golden/mame/frames_crc.lua` :
```lua
local OUT  = "../tests/golden/frames_crc.txt"
local WANT = {1,2,5,10,30,60,120,300,600}
local LAST = 600
local want = {}
for _, n in ipairs(WANT) do want[n] = true end
local tbl = {}
for i = 0, 255 do
  local c = i
  for _ = 1, 8 do
    if c & 1 == 1 then c = (c >> 1) ~ 0xEDB88320 else c = c >> 1 end
  end
  tbl[i] = c
end
local function crc32(s)
  local c = 0xFFFFFFFF
  for i = 1, #s do c = tbl[(c ~ s:byte(i)) & 0xFF] ~ (c >> 8) end
  return c ~ 0xFFFFFFFF
end
local scr = manager.machine.screens[":screen"]
local out = assert(io.open(OUT, "w"))
local first = nil
local function on_frame()
  local n = scr:frame_number()
  if first == nil then
    first = n
    out:write(string.format("# first_frame_seen=%d size=%dx%d mame=%s\n", n, scr.width, scr.height, emu.app_version()))
  end
  if want[n] then
    out:write(string.format("%d %08X %dx%d\n", n, crc32(scr:pixels()), scr.width, scr.height))
    out:flush()
  end
  if n >= LAST then out:close(); manager.machine:exit() end
end
emu.register_frame_done(on_frame, "frame")
```
```powershell
& .\mame.exe @common -video none -sound none -autoboot_script ..\tests\golden\mame\frames_crc.lua -seconds_to_run 15
```
- ✔ `manager.machine.screens[":screen"]` et `emu.register_frame_done(fn, "frame")` figurent dans la doc Lua. ⚠ à confirmer sur le binaire : `scr:frame_number()` (et sa base, 0 ou 1), `scr:pixels()` (chaîne d'octets, 32 bits/pixel), `scr.width`/`scr.height`, `emu.app_version()`, `manager.machine:exit()`, disponibilité de `io` dans la sandbox Lua.
**[T0.5.1] Tranché — tout confirmé sur le binaire** : `scr:frame_number()` existe, **base 0** (vaut 0 au boot) ; `scr:pixels()` → chaîne d'octets de longueur width×height×4 = **688128** ; `scr.width`=768 / `scr.height`=224 ; `emu.app_version()` → `"0.289"` ; `manager.machine:exit()` termine le run ; `io.open`/`io` disponibles dans la sandbox Lua (les scripts écrivent des fichiers normalement).
- La première ligne `# first_frame_seen=…` prouve que le script a démarré à temps : si elle est > 1, `-autoboot_delay 0` n'a pas eu l'effet attendu et la frame 1 est manquée (tâche en échec).
**[T0.5.1] Tranché** : avec `-autoboot_delay 0`, la ligne d'en-tête est `# first_frame_seen=0 size=768x224 mame=0.289` — le script démarre à frame 0, la frame 1 n'est pas manquée (test `golden/tmp/ffs.txt`).
- ⚠ **Dimensions** : KB-02 indique un rendu interne ×3 en horizontal (`GALAXIAN_XSCALE = 3`) : le bitmap MAME visible est probablement **768×224**, pas 256×224. D'où la colonne `WxH`. Le CRC golden porte sur le bitmap MAME tel quel ; la comparaison avec la sortie 256×224 de l'émulateur (Phase 4) est une décision humaine (T0.5.5).
**[T0.5.1] Tranché — confirmé** : le bitmap MAME est **768×224** (`scr.width`/`scr.height`, et `<display width=768 height=224>` dans `golden/galaxian.xml`). La capture PNG par défaut est tournée à cause de `rotate="90"` (224×768 au repos). Le CRC porte sur le bitmap 768×224 tel quel.
- ⚠ `-video none` : vérifier que le bitmap est bien rendu (CRC ≠ CRC d'un écran noir) ; sinon retirer l'option (fenêtre visible).
**[T0.5.1] Tranché — confirmé** : le bitmap est bien rendu avec `-video none` (`pixels()` tout zéro au boot mais non-noir dès frame ~10, CRC ≠ CRC d'un écran noir). Conserver `-video none`. MAME émet `Warning: -video none doesn't make much sense without -seconds_to_run` — sans objet si l'on termine par `-seconds_to_run` ou `machine:exit()`.

**C. Lua : dump VRAM + OBJRAM** : fichier `tests/golden/mame/mem_dump.lua` (adresses de KB-03) :
```lua
local DUMPS = {[60]=true, [600]=true}
local LAST  = 600
local scr = manager.machine.screens[":screen"]
local mem = manager.machine.devices[":maincpu"].spaces["program"]
local function dump(path)
  local t = {}
  for a = 0x5000, 0x53FF do t[#t+1] = string.char(mem:read_u8(a)) end   -- VRAM : 0x400 octets
  for a = 0x5800, 0x58FF do t[#t+1] = string.char(mem:read_u8(a)) end   -- OBJRAM : 0x100 octets
  local f = assert(io.open(path, "wb")); f:write(table.concat(t)); f:close()
end
local function on_frame()
  local n = scr:frame_number()
  if DUMPS[n] then dump(string.format("../tests/golden/mem_dump_%d.bin", n)) end
  if n >= LAST then manager.machine:exit() end
end
emu.register_frame_done(on_frame, "frame")
```
```powershell
& .\mame.exe @common -video none -sound none -autoboot_script ..\tests\golden\mame\mem_dump.lua -seconds_to_run 15
```
Fichier = 0x500 (1 280) octets, VRAM d'abord, OBJRAM ensuite. ⚠ `read_u8` (la doc montre `read_i8` ; les variantes non signées se listent par complétion dans la console Lua, `-console`).
**[T0.5.1] Tranché — confirmé** : `mem:read_u8(addr)` **existe** (et `.read_i8`) sur `manager.machine.devices[":maincpu"].spaces["program"]`. Test : `addr 0000: read_u8=AF read_i8=-81`, `addr FFFC: read_u8=FF read_i8=-1`.

**D. Audio** :
```powershell
& .\mame.exe @common -video none -sound none -wavwrite ..\tests\golden\audio_attract.wav -samplerate 48000 -seconds_to_run 20
```
✔ `-wavwrite` écrit la sortie finale du mixeur. ⚠ avec `-sound none`, vérifier que le WAV n'est pas vide ; sinon retirer l'option (module audio par défaut, `wasapi` sous Windows ✔). ⚠ dossier de sortie du WAV (KB-28a §3). Durée S fixée une fois pour toutes (T0.5.7).
**[T0.5.1] Tranché — confirmé** : avec `-sound none`, le WAV est **non vide** (test `golden/tmp/audio_none.wav`, 480 046 octets, données non nulles) → conserver `-sound none`. Le fichier est écrit au chemin donné, relatif au répertoire courant. En-tête vérifié sur 4 WAV : mono, 16 bits, **sample rate = 48000 Hz** (valeur demandée par `-samplerate`, y compris le défaut) ; le *byte rate* de l'en-tête vaut 96000 (= 48000 × 2 octets), à ne pas confondre avec la fréquence d'échantillonnage — comparer les octets de données, pas seulement l'en-tête.

**E. Entrées scriptées (scénarios tir/explosion)** : fichier `tests/golden/mame/scenario_fire.lua` ; actions par numéro de frame, jamais par temps réel :
```lua
local scr   = manager.machine.screens[":screen"]
local ports = manager.machine.ioport.ports
local LAST  = 1500
local ACTIONS = {                       -- frame = liste de {port, champ, valeur}
  [120] = {{":IN0", "Coin 1", 1}},      -- ⚠ tags et libellés réels : KB-24a et tests/golden/galaxian.xml
  [125] = {{":IN0", "Coin 1", 0}},
}
local function on_frame()
  local n = scr:frame_number()
  for _, a in ipairs(ACTIONS[n] or {}) do ports[a[1]].fields[a[2]]:set_value(a[3]) end
  if n >= LAST then manager.machine:exit() end
end
emu.register_frame_done(on_frame, "frame")
```
```powershell
& .\mame.exe @common -video none -sound none -autoboot_script ..\tests\golden\mame\scenario_fire.lua -wavwrite ..\tests\golden\audio_fire.wav -samplerate 48000 -seconds_to_run 40
```
⚠ `ioport.ports[...].fields[...]:set_value`. Ne pas deviner les tags/libellés : les lire dans `-listxml` et KB-24a. Alternative ✔ : `-record` en jouant à la main puis `-playback <nom> -exit_after_playback` (cfg/nvram vierges obligatoires) ; moins fiable, à n'utiliser que si Lua échoue.
**[T0.5.1] Tranché — confirmé** : `ports[":IN0"].fields["Coin 1"]:set_value(1)` **fonctionne**. Les libellés réels des champs (vérifiés sur le binaire) :
- **`:IN0`** = `P1 Button 1, Coin 1, Service Mode, Service 1, P1 Right, P1 Left, Cabinet, Coin 2`
- **`:IN1`** = `2 Players Start, Coinage, P2 Button 1, 1 Player Start, P2 Left, P2 Right`
- **`:IN2`** = `Unused, Bonus Life, Lives`

Il n'y a **pas** de read-back par champ (`:value()`/`:get_value()`/`:read()` sur un field → nil) ; pour lire l'état d'un port, utiliser `port:read()`. Les effets sont visibles au frame suivant (fixé à la frame N → `port:read()` modifié à la frame N+1). Note : les blocs `<port tag=":IN0">` du XML `-listxml` sont **vides** — les libellés viennent de la doc Lua / KB-24a, pas du XML. L'option `-no_coin_lockout` n'existe pas dans MAME 0.289 (`Error: unknown option`) ; forcer des crédits via le champ `Coin 1`, pas via un flag CLI.
