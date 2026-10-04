# KB-28a — Mémento CLI MAME (Windows / PowerShell) pour les golden

*Fiche fournie (pas extraite de `galaxian.cpp`). Source : documentation MAME (docs.mamedev.org, pages « Command-line », relevées sur la version 0.289) ; le binaire du dépôt est `tools\mame.exe` et sa documentation locale est dans `tools\docs\` (c'est elle, avec `tools\whatsnew.txt`, qui fait foi pour la version installée). Légende : **✔** = confirmé par la doc ; **⚠** = à vérifier sur le binaire installé (T0.5.1) ; tant qu'un ⚠ n'est pas tranché, le traiter comme faux.*

> **Corrigée en T0.5.1** : chaque ⚠ a été tranché sur `tools\mame.exe` v0.289 (sorties consignées dans `docs/mame_notes.md`). Les corrections sont marquées **[T0.5.1]** ; les ⚠ résolus passent en ✔/✘ avec la conclusion.

**1. Environnement**
```
H:\__Emulator__\Arcade_Galaxian_Rust\          racine du dépôt (R)
├─ tools\            mame.exe + dossiers MAME (roms, cfg, snap, ini, hash, plugins, docs…) — NON versionné
├─ tests\golden\     sorties golden (versionnées) ; golden\mame\ = scripts .dbg/.lua ; golden\tmp\ = jetable (ignoré par git)
```
- **Répertoire courant = `tools\`** pour tout lancement (✔ chemins relatifs résolus depuis le répertoire courant). Les ROMs sont dans `tools\roms` (`-rompath roms`, défaut ✔).
- Binaire Windows natif : les options SDL (`-videodriver`, `-audiodriver`, `-scalemode`, `-keymap*`, `-gl_lib`, `-sdlvideofps`…) **ne s'appliquent pas** ; les options Windows (`-priority`, `-profile`, `-triplebuffer`, `-full_screen_*`, `-dual_lightgun`) sont sans utilité pour le golden.
- Les dossiers existants `tools\cfg`, `tools\snap` (configuration et captures de l'utilisateur) ne sont **jamais** réutilisés par le golden : le profil du §3 redirige tout vers `tests\golden\tmp\run\`.

**2. Vérifications préalables** (✔ ; lancer depuis `tools\`)
```powershell
Set-Location H:\__Emulator__\Arcade_Galaxian_Rust\tools
.\mame.exe -help                          # version de MAME
.\mame.exe -verifyroms galaxian           # attendu : « romset galaxian is good »
.\mame.exe -listfull "galaxian*"          # noms réels des sets (Namco, Midway…)
.\mame.exe -listroms galaxian             # noms, tailles, CRC, SHA1
.\mame.exe -listcrc galaxian
.\mame.exe -listdevices galaxian          # tags des périphériques (:maincpu, :screen…)
.\mame.exe -listxml galaxian | Out-File -Encoding utf8 ..\tests\golden\galaxian.xml   # écran, ports IN0/IN1/IN2, DIP
.\mame.exe -showusage | Out-File -Encoding utf8 ..\tests\golden\tmp\showusage.txt     # options réellement présentes
.\mame.exe -showconfig | Out-File -Encoding utf8 ..\tests\golden\tmp\showconfig.ini   # configuration effective
```
**[T0.5.1]** Sorties consignées dans `docs/mame_notes.md` §2 : `-help` → v0.289 (mame0289) ; `-verifyroms galaxian` → « romset galaxian is good » ✔ ; `-listfull "galaxian*"` → 14 sets, dont `galaxianm` / `galaxianmo` (Midway set 1/2).
⚠ Le plan écrit `galmidw` pour le set Midway (T0.4.9, T3.1.7) ; la doc liste `galaxianm` et `galaxianmo` (Midway set 1/2) : confirmer avec `-listfull` (GAP-08).
**[T0.5.1] Tranché** : le set Midway s'appelle **`galaxianm`** (et `galaxianmo`). Il n'existe pas de set nommé `galmidw` — `galmidw.*` sont seulement les *noms de fichiers ROM* du driver `galaxian`. Corriger T0.4.9 / T3.1.7 : utiliser `galaxianm`, pas `galmidw`.

**3. Profil de lancement commun** (que toutes les recettes de KB-28b étendent) :
```powershell
Set-Location H:\__Emulator__\Arcade_Galaxian_Rust\tools
$G   = "..\tests\golden"
$RUN = "$G\tmp\run"
Remove-Item -Recurse -Force $RUN -ErrorAction SilentlyContinue      # état neuf avant CHAQUE exécution
New-Item -ItemType Directory -Force "$RUN\cfg","$RUN\nvram","$RUN\snap","$RUN\inp","$RUN\sta" | Out-Null
$common = @("galaxian","-rompath","roms","-noreadconfig",
  "-cfg_directory","$RUN\cfg","-nvram_directory","$RUN\nvram","-snapshot_directory","$RUN\snap",
  "-input_directory","$RUN\inp","-state_directory","$RUN\sta",
  "-skip_gameinfo","-nothrottle","-frameskip","0","-noautoframeskip","-norewind","-noautosave",
  "-autoboot_delay","0")
# usage : & .\mame.exe @common <options propres à la recette> ; puis $LASTEXITCODE
```
`$RUN` est supprimé puis recréé à chaque exécution : `cfg\` conserve les DIP, les assignations d'entrées et la configuration d'écran (✔) ; un état résiduel fausse la reproductibilité. Le `@common` (splatting) passe le tableau comme arguments séparés.

| Option | Effet (✔ doc) | Usage golden |
|---|---|---|
| `-seconds_to_run S` (`-str`) | arrête après S secondes **émulées** ; écrit une capture d'écran à la sortie dans le dossier snap | fin de run déterministe |
| `-nothrottle` | n'adapte plus la vitesse au temps réel | exécution rapide, temps émulé inchangé |
| `-frameskip 0`, `-noautoframeskip` | aucune frame sautée | indispensable aux CRC d'écran |
| `-autoboot_script f.lua` | charge un script Lua | CRC, dumps, entrées |
| `-autoboot_delay 0` | la doc indique que le chargement du script est retardé de quelques secondes par défaut ; `0` = immédiat | **obligatoire avec tout script Lua** — **[T0.5.1] ✔ confirmé** : le script démarre à frame 0 (`first_frame_seen=0`) |
| `-debug`, `-debugscript f` | active le debugger ; exécute le fichier de commandes au démarrage | traces CPU |
| `-debugger windows` | module de debugger ; sous Windows, `windows` (fenêtre Win32) est le défaut ✔, `qt` n'est pas inclus par défaut sous Windows ✔ | la trace ouvre une fenêtre de debugger ⚠ — **[T0.5.1] Tranché** : pour un run headless, utiliser **`-debugger none`** (MAME tourne sans fenêtre, le fichier de trace est écrit normalement) ; `-seconds_to_run` arrête MAME correctement sous debugger |
| `-video none` / `-sound none` | pas de fenêtre / pas de sortie audio (le son reste émulé) | ⚠ vérifier que `screen:pixels()` et `-wavwrite` fonctionnent encore ainsi ; sinon retirer l'option — **[T0.5.1] ✔ confirmé** : le bitmap est bien rendu (`pixels()` non-noir dès frame ~10), `-wavwrite` produit un WAV non vide → conserver les deux options |
| `-wavwrite f.wav` | écrit la sortie finale du mixeur | audio ; durée du WAV = temps émulé ; ⚠ dossier de sortie (voir ci-dessous) — **[T0.5.1] Tranché** : le fichier est écrit au **chemin donné, relatif au répertoire courant** (pas dans le dossier snap). En-tête vérifié sur 4 WAV : mono, 16 bits, sample rate = 48000 Hz ; comparer les octets de données |
| `-samplerate 48000` | défaut 48000 ✔ | le fixer explicitement ; ne pas toucher `-volume` (défaut 0 dB) — **[T0.5.1] Tranché** : l'en-tête WAV porte bien la fréquence demandée (48000 Hz, y compris par défaut) ; son *byte rate* vaut 96000 (= 48000 × 2 octets), à ne pas confondre avec la fréquence d'échantillonnage |
| `-record f`, `-playback f`, `-exit_after_playback` | enregistre/rejoue les entrées | alternative aux entrées Lua ; la doc prévient que cela « ne marche pas de façon fiable for all systems » et se désynchronise si cfg/nvram diffèrent |
| `-log`, `-oslog`, `-verbose` | `error.log`, sortie système, diagnostics | à activer pour toute anomalie — **[T0.5.1] note** : la sortie du debugger (`symlist`, etc.) va dans **`debug.log`** (avec `-debuglog`), pas sur stdout |
| `-watchdog S` | tue MAME si aucune frame n'est mise à jour pendant S s | CI : 30 |
| `-bench N` | équivaut à `-str N -video none -sound none -nothrottle` | mesure de perf MAME, pas pour le golden |

⚠ **Dossier de sortie de `-wavwrite`** : la doc ne précise pas s'il est relatif au répertoire courant ou au dossier snap. À constater en T0.5.1 (essai `-wavwrite ..\tests\golden\tmp\t.wav` puis recherche du fichier dans `tools\`, `<RUN>\snap\`, `tests\golden\tmp\`) et à consigner.
**[T0.5.1] Tranché** : relatif au **répertoire courant**. Test : `-wavwrite golden/tmp/audio_none.wav` → `golden/tmp/audio_none.wav` créé (pas dans le dossier snap).

**4. Pièges PowerShell**
- Redirection `>` : Windows PowerShell 5.1 écrit en **UTF-16** (inutilisable pour diff/parseurs). Utiliser `| Out-File -Encoding utf8` (BOM en 5.1, sans BOM en PowerShell 7) ou `cmd /c ".\mame.exe … > fichier"` pour obtenir les octets bruts.
- `;` sépare les commandes dans PowerShell : tout argument qui en contient (ex. `-rompath "a;b"`) doit être entre guillemets.
- Code retour : `$LASTEXITCODE` après `& .\mame.exe …`. ⚠ Si l'invite revient avant la fin de MAME (exécutable GUI), ajouter `| Out-Null` ou utiliser `Start-Process -Wait -NoNewWindow -PassThru` ; T0.5.1 constate le comportement.
**[T0.5.1] Tranché** : `mame.exe` **bloque jusqu'à la fin** de l'exécution, puis retourne le code d'erreur dans `$LASTEXITCODE` (0 = succès). Pas besoin de `Start-Process -Wait`.
- Dans les scripts `.dbg` et `.lua`, écrire les chemins avec `/` (`../tests/golden/…`), acceptés par Windows, sans échappement de `\`.

**5. Reproductibilité** (T0.5.10) : deux runs successifs avec le même binaire, le même set ROM (`-verifyroms` OK) et un `<RUN>` neuf doivent produire des fichiers **identiques à l'octet**. Hachage SHA256 via `Get-FileHash -Algorithm SHA256 <fichier>` (pas de dépendance Rust supplémentaire : un crate `sha2` serait une décision humaine, T0.1.4). Ne jamais utiliser `-rewind`, `-autosave`, `-state` pour un golden.
