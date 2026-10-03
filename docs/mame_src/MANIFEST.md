# MANIFEST — MAME Galaxian driver sources

Read-only reference manifest for the 5 verbatim copies in this directory.
Built by **T0.4.2** (Manifeste et index).

## 1. Fichiers + SHA256

SHA256 recalculés le 2026-10-03 avec `sha256sum` depuis ce dossier ; validés par
`sha256sum -c SHA256SUMS.txt` (les 5 fichiers = OK).

| Fichier | Lignes | SHA256 | Rôle dans le périmètre Galaxian d'origine |
|---|---:|---|---|
| galaxian.cpp | 17339 | `f26195d4895ac07971683a98778e8e619d09140833ff87b95c84312a3dade020` | CPU/memory map, ROM_START(galaxian), INPUT_PORTS_START(galaxian) |
| galaxian.h | 944 | `ab51b3818496b5a3f676cf749484f42453c75fa5454465d930eb6418a378b1a7` | déclarations partagées (state, device) |
| galaxian_a.cpp | 777 | `31dccad2cc6a222aa2731ff4098cbc787d99df85f1cda5a80e480f459ba3ed48` | audio (compteur programmable, 555, LFSR) |
| galaxian_a.h | 77 | `c12bbbb3bfbf24b68b13918d937b8337e673e03aa971c219a37e19f03e169240` | déclarations audio |
| galaxian_v.cpp | 1545 | `bd75a62adc66b9c8666e457008143cbfb6d18c9f4e8627351c18f87725c25403` | vidéo (tilemap, sprites, shells, champ d'étoiles) |

Provenance : MAME upstream `master`, branch de référence
`https://github.com/mamedev/mame/tree/master/src/mame/galaxian` (récupéré le 2026-10-02).
Licence BSD-3-Clause ; copyright holders : Aaron Giles, Couriersud, Stephane Humbert, Robbbert.

## 2. Version / commit MAME

**Non épinglé.** Les fichiers ont été récupérés depuis `master` le 2026-10-02 ;
le commit upstream exact n'est pas consigné dans le dossier (voir README.md §Caveat).
Pour une comparaison de version exacte en Phase 1+, ré-épingler en consignant le
commit SHA upstream avec les hashes.

## 3. Index « fonction → ligne » (Galaxian d'origine)

Seules les fonctions du périmètre **Galaxian d'origine** sont listées ci-dessous ;
les machines dérivées (galaxiana, galaxianm, superg, zerotime, …) sont omises.

### galaxian.cpp

| Fonction / structure | Ligne | Rôle |
|---|---:|---|
| `ROM_START( galaxian )` | 9760 | Définition ROM de base (maincpu 0x4000, gfx1 0x1000, proms 0x20) |
| `INPUT_PORTS_START( galaxian )` | 3072 | Ports d'entrée IN0/IN1/IN2 + DIPs de base |
| `gfx_layout galaxian_charlayout` | 7394 | Layout tiles de caractères (fond) |
| `gfx_layout galaxian_spritelayout` | 7405 | Layout sprites de base |
| `gfx_layout galaxian_charlayout_0x200` | 7416 | Variant layout caractères offset 0x200 |
| `gfx_layout galaxian_spritelayout_0x80` | 7427 | Variant layout sprites offset 0x80 |

### galaxian_a.cpp (audio)

| Fonction | Ligne | Rôle |
|---|---:|---|
| `galaxian_sound_device::device_start()` | 649 | Initialisation du device audio Galaxian |
| `galaxian_sound_device::device_add_mconfig(machine_config&)` | 665 | Wiring machine du device audio |
| `galaxian_sound_device::pitch_w(uint8_t)` | 697 | Fréquence du compteur programmable |
| `galaxian_sound_device::lfo_freq_w(offs_t, uint8_t)` | 702 | Fréquence LFO (4 bits) |
| `galaxian_sound_device::background_enable_w(offs_t, uint8_t)` | 713 | Enable sortie de fond |
| `galaxian_sound_device::noise_enable_w(uint8_t)` | 718 | Enable bruit LFSR |
| `galaxian_sound_device::vol_w(offs_t, uint8_t)` | 723 | Volume |
| `galaxian_sound_device::fire_enable_w(uint8_t)` | 728 | Enable impulsion FIRE |
| `galaxian_sound_device::sound_w(offs_t, uint8_t)` | 734 | Écriture registre son |

### galaxian_v.cpp (vidéo)

| Fonction | Ligne | Rôle |
|---|---:|---|
| `galaxian_state::galaxian_palette(palette_device&)` | 238 | Affectation bit→R/G/B de la PROM + calcul des poids résistifs |
| `galaxian_state::video_start()` | 399 | Initialisation vidéo (VRAM/OBJRAM) |
| `galaxian_state::state_save_register()` | 433 | Enregistrement des états sauvegardables |
| `galaxian_state::galaxian_videoram_w(offs_t, uint8_t)` | 503 | Écriture VRAM (tilemap) |
| `galaxian_state::galaxian_objram_w(offs_t, uint8_t)` | 514 | Écriture OBJRAM (sprites) |
| `galaxian_state::sprites_clip(...)` | 554 | Découpage des sprites |
| `galaxian_state::sprites_draw(...)` | 568 | Dessin des sprites |
| `galaxian_state::bullets_draw(...)` | 629 | Dessin des missiles/shells |
| `galaxian_state::galaxian_flip_screen_x_w(uint8_t)` | 670 | Retournement horizontal |
| `galaxian_state::galaxian_flip_screen_y_w(uint8_t)` | 686 | Retournement vertical |
| `galaxian_state::galaxian_flip_screen_xy_w(uint8_t)` | 697 | Retournement horizontal+vertical |
| `galaxian_state::galaxian_stars_enable_w(uint8_t)` | 711 | Enable champ d'étoiles |
| `galaxian_state::galaxian_gfxbank_w(offs_t, uint8_t)` | 771 | Sélection bank gfx |
| `galaxian_state::stars_init()` | 789 | Initialisation LFSR des étoiles |
| `galaxian_state::stars_update_origin()` | 822 | Mise à jour origine LFSR étoiles |
| `galaxian_state::stars_draw_row(...)` | 869 | Dessin d'une ligne d'étoiles |
| `galaxian_state::galaxian_draw_stars(...)` | 932 | Dessin complet du champ d'étoiles |
| `galaxian_state::galaxian_draw_background(...)` | 950 | Dessin du fond (tilemap) |
| `galaxian_state::background_draw_colorsplit(...)` | 959 | Split couleur horizontal (fond) |
| `galaxian_state::galaxian_draw_bullet(...)` | 1160 | Dessin du missile/shell Galaxian |

## 4. Lacunes ouvertes par ce manifeste

- **GAP-02** (affectation bit→R/G/B exacte de la PROM) : lue dans
  `galaxian_v.cpp` ligne 238 ; à recouper avec KB-27 (T0.4.12).
- **GAP-03** (durée watchdog, table ROMs Galaxian) : `ROM_START(galaxian)` ligne 9760
  donne noms/tailles/offsets/CRC ; durée watchdog dans `galaxian.cpp`
  (`WATCHDOG_TIMER`, T0.4.10).
- **GAP-05** (ports IN0/IN1/IN2 + DIPs) : `INPUT_PORTS_START(galaxian)` ligne 3072
  (T0.4.7, T0.4.8).
