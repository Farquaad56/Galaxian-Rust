# KB-25 — Config machine (set `galaxian`)

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h), périmètre **Galaxian d'origine** uniquement. Citations relatives à `docs/mame_src/`.

## CPU

- Z80, horloge = `GALAXIAN_PIXEL_CLOCK/3/2` (galaxian.cpp:7489) ; addrmap programme = `galaxian_map` (galaxian.cpp:7493).
- `GALAXIAN_MASTER_CLOCK = 18.432 MHz` (galaxian.h:32), `GALAXIAN_XSCALE = 3` (galaxian.h:37), donc `GALAXIAN_PIXEL_CLOCK = XSCALE*MASTER/3 = 18.432 MHz` (galaxian.h:41).
- **CPU clock = 18.432 MHz / 6 = 3.072 MHz** (galaxian.cpp:7489 + galaxian.h:41).

## Watchdog

- `WATCHDOG_TIMER(config, "watchdog").set_vblank_count("screen", 8)` (galaxian.cpp:7492) : le watchdog est armé par les vblanks de l'écran ; **aucune lecture du watchdog pendant 8 vblanks consécutifs → reset machine**.
- Durée d'un frame = `VTOTAL × HTOTAL / PIXEL_CLOCK` = 264 × 1152 / 18.432 MHz ≈ 16,5 ms (≈ 60,6 Hz) ; **timeout ≈ 8 frames ≈ 132 ms** (galaxian.cpp:7492 + galaxian.h:47-56). En T-states CPU : 8 × (264×1152/6) = 405 504 T-states.
- Lecture de reset du watchdog dans `galaxian_map_base` : `map(0x7800, 0x7800).mirror(0x07ff).r("watchdog", FUNC(watchdog_timer_device::reset_r))` (galaxian.cpp:1766) — toute lecture de **0x7800–0x7fff** réarme le watchdog (`galaxian_map` = base + discrete, galaxian.cpp:1769-1773). Cf. KB-03/KB-06.

## Écran (raster)

- `SCREEN(config, m_screen, SCREEN_TYPE_RASTER)` puis `set_raw(GALAXIAN_PIXEL_CLOCK, GALAXIAN_HTOTAL, GALAXIAN_HBEND, GALAXIAN_HBSTART, GALAXIAN_VTOTAL, GALAXIAN_VBEND, GALAXIAN_VBSTART)` (galaxian.cpp:7501-7502) ; vblank → `vblank_interrupt_w` (galaxian.cpp:7504).
- Valeurs (galaxian.h:47-56) :
  - `GALAXIAN_HTOTAL = 384 × XSCALE = 1152` pixel-clocks/ligne (galaxian.h:47) ; commentaire : compte hardware 128→511, HBLANK 130–250, normalisé ici à 0→383 avec HBLANK 264–383 (galaxian.h:44-46).
  - `GALAXIAN_HBEND = 0` (galaxian.h:48), `GALAXIAN_H0START = 0` (galaxian.h:51), `GALAXIAN_HBSTART = 256 × XSCALE = 768` (galaxian.h:52).
  - `GALAXIAN_VTOTAL = 264` lignes (galaxian.h:54), `GALAXIAN_VBEND = 16` (galaxian.h:55), `GALAXIAN_VBSTART = 224 + 16 = 240` (galaxian.h:56).
- Résolution utile : 384×224 lignes de pixels × XSCALE=3 → rendu 1152×672 ; les étoiles exigent l'échelle ×3 (galaxian.h:36-37).

## ROMs — set `galaxian` (original)

`ROM_START( galaxian )` (galaxian.cpp:9760-9774) :

| région | taille | fichier | offset | size | CRC |
|---|---|---|---|---|---|
| maincpu | 0x4000 | galmidw.u | 0x0000 | 0x0800 | 745e2d61 (galaxian.cpp:9762) |
| maincpu | — | galmidw.v | 0x0800 | 0x0800 | 9c999a40 (galaxian.cpp:9763) |
| maincpu | — | galmidw.w | 0x1000 | 0x0800 | b5894925 (galaxian.cpp:9764) |
| maincpu | — | galmidw.y | 0x1800 | 0x0800 | 6b3ca10b (galaxian.cpp:9765) |
| maincpu | — | 7l | 0x2000 | 0x0800 | 1b933207 (galaxian.cpp:9766) |
| gfx1 | 0x1000 | 1h.bin | 0x0000 | 0x0800 | 39fb43a4 (galaxian.cpp:9769) |
| gfx1 | — | 1k.bin | 0x0800 | 0x0800 | 7e3f56a2 (galaxian.cpp:9770) |
| proms | 0x0020 | 6l.bpr | 0x0000 | 0x0020 | c3ac9467 (galaxian.cpp:9773) |

- Le set original **réutilise les ROM CPU `galmidw.*`** (celles du set `galaxianm`) ; le gfx est `1h.bin`/`1k.bin` (galaxian.cpp:9762-9770).
- « differs » : `ROM_START( galaxianmo )` (galaxian.cpp:9806-9820) charge `galaxian.u/v/w/y` + `7l.bin` en maincpu et `galaxian.j1`/`galaxian.l1` en gfx1 ; même proms `6l.bpr`.

## Palette PROM (GAP-02 — mapping bit→R/G/B exact)

`galaxian_state::galaxian_palette` (galaxian_v.cpp:238), 32 entrées (`PALETTE(..., 32)`, galaxian.cpp:7496), lecture de la région `proms` (galaxian_v.cpp:240) :

- Résistances par composante : `{1000, 470, 220}` ohms (galaxian_v.cpp:241).
- **R = bits 0/1/2** (galaxian_v.cpp:286-289), **G = bits 3/4/5** (galaxian_v.cpp:292-295), **B = bits 6/7** (galaxian_v.cpp:298-300) — résout l'ambiguïté « VERT twice » de la doc : bits 3/4/5 = GREEN, bits 6/7 = BLUE. Le commentaire du driver confirme le même câblage bit7→bit0 (galaxian_v.cpp:243-253).
- Poids calculés par `compute_resistor_weights(0, RGB_MAXIMUM, -1.0, ...)` (galaxian_v.cpp:274-277) ; **`RGB_MAXIMUM = 224`** (`#define`, galaxian_v.cpp:229).
- En parallèle sur chaque R/G/B : paire de résistances **150 Ω / 100 Ω** branchées sur le générateur d'étoiles (galaxian_v.cpp:259-261) ; et **100 Ω** par composante activés quand un shell/missile est actif (galaxian_v.cpp:263-265).
- Valeurs des étoiles : `minval = RGB_MAXIMUM*130/150`, `midval = RGB_MAXIMUM*130/100`, `maxval = RGB_MAXIMUM*130/60` (galaxian_v.cpp:320-322), compressées dans la plage 194→255 (galaxian_v.cpp:317-318) ; table `starmap[4]` {0, minval, mid, 255} (galaxian_v.cpp:325-329).
- Couleurs d'étoiles : 64 valeurs dérivées des bits du générateur — R = bit5 (150 Ω) + bit4 (100 Ω), G = bit3 + bit2, B = bit1 + bit0 (galaxian_v.cpp:336-352).
- Couleurs de balles : 7 blanches `(255,255,255)` puis jaune `(255,255,0)` pour la dernière (galaxian_v.cpp:355-358).

## Config du set original

`galaxian_state::galaxian(machine_config&)` : `galaxian_base(config)` + `GALAXIAN_SOUND(config, "cust", 0)` (galaxian.cpp:7657-7662) ; aucun PPI8255 sur le set original (les ports IN0/IN1/IN2 sont lus directement via la memory map, cf. KB-24).
