# KB-25b — Configuration machine (set `galaxian`)

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h, galaxian_v.cpp), périmètre **Galaxian d'origine** uniquement. Citations relatives à `docs/mame_src/`.
> Partie « config machine » de la fiche KB-25 ; les ROMs sont dans KB-25a. Résout GAP-03 (durée du watchdog) et complète KB-02/KB-03/KB-06.

## 1. CPU

- Type : **Z80**, instancié par `galaxian_base` : `Z80(config, m_maincpu, GALAXIAN_PIXEL_CLOCK/3/2);` (galaxian.cpp:7492).
- Horloge exacte et dérivation des constantes :
  - `GALAXIAN_MASTER_CLOCK = 18.432 MHz XTAL` (galaxian.h:32) ;
  - `GALAXIAN_XSCALE = 3` — « we scale horizontally by 3 to render stars correctly » (galaxian.h:36-37) ;
  - `GALAXIAN_PIXEL_CLOCK = GALAXIAN_XSCALE*GALAXIAN_MASTER_CLOCK / 3 = 3×18.432 MHz/3 = 18.432 MHz` (galaxian.h:41) — c'est l'horloge de *rendu* MAME, pas l'horloge pixel hardware (cf. §4) ;
  - **CPU clock = PIXEL_CLOCK/3/2 = 18.432 MHz/6 = 3.072 MHz** (galaxian.cpp:7492 + galaxian.h:41).
- Addrmap programme : `m_maincpu->set_addrmap(AS_PROGRAM, &galaxian_state::galaxian_map);` (galaxian.cpp:7493) ; `galaxian_map` = `galaxian_map_base` + `galaxian_map_discrete` (galaxian.cpp:1772-1776). Décodage complet : KB-03.

## 2. Watchdog (`WATCHDOG_TIMER`)

- Armement : `WATCHDOG_TIMER(config, "watchdog").set_vblank_count("screen", 8);` (galaxian.cpp:7495) — le watchdog est armé par les **vblanks de l'écran** tagué `"screen"` ; s'il n'est pas réarmé pendant **8 vblanks consécutifs**, il déclenche un reset machine (sémantique du `watchdog_timer_device` MAME core, hors du dossier fourni).
- Durée du timeout : 1 frame = HTOTAL×VTOTAL/PIXEL_CLOCK = 1152×264/18.432 MHz = 304 128 pixel-clocks = **16.5 ms** (≈ 60.6 Hz) (galaxian.h:47,54,41) → timeout = 8 frames ≈ **132 ms**. En T-states CPU (CPU = PIXEL_CLOCK/6) : 8 × (304 128/6) = 8 × 50 688 = **405 504 T-states** (galaxian.cpp:7492,7495 + galaxian.h:41).
- Zone mémoire dont une lecture réarme le watchdog : dans `galaxian_map_base` — `map(0x7800, 0x7800).mirror(0x07ff).r("watchdog", FUNC(watchdog_timer_device::reset_r));` (galaxian.cpp:1769) → toute lecture de **0x7800–0x7fff** réarme le watchdog. La même zone est également écrite par le son (`pitch_w`, galaxian.cpp:1746, via `galaxian_map_discrete` 1742-1747) : lecture = watchdog, écriture = pitch. Cf. KB-03 (ligne 7800-7fff), KB-06 (qui notait la durée manquante — GAP-03).

## 3. Écran (raster) — `SCREEN` + `set_raw(...)`

```cpp
SCREEN(config, m_screen);                                                              // galaxian.cpp:7501
m_screen->set_raw(GALAXIAN_PIXEL_CLOCK, GALAXIAN_HTOTAL, GALAXIAN_HBEND,              // galaxian.cpp:7502
                  GALAXIAN_HBSTART, GALAXIAN_VTOTAL, GALAXIAN_VBEND, GALAXIAN_VBSTART);
m_screen->set_screen_update(FUNC(galaxian_state::screen_update_galaxian));            // galaxian.cpp:7503
m_screen->screen_vblank().set(FUNC(galaxian_state::vblank_interrupt_w));              // galaxian.cpp:7504
```

- vblank → interrupt : le callback `vblank_interrupt_w` (galaxian.cpp:7504) assert la ligne d'interrupt du Z80 si `m_irq_enabled` (flip-flop 6F, NMI ON) — impl. galaxian.cpp:764-770. Cf. KB-05.

### Constantes de timing (`GALAXIAN_*`, valeurs numériques)

| Constante | Définition | Valeur | Ligne |
|---|---|---:|---|
| `GALAXIAN_HTOTAL`  | `384 * GALAXIAN_XSCALE` | **1152** pixel-clocks/ligne | galaxian.h:47 |
| `GALAXIAN_HBEND`   | `0 * GALAXIAN_XSCALE`   | **0** | galaxian.h:48 |
| `GALAXIAN_H0START` | `0 * GALAXIAN_XSCALE`   | **0** (l'ancienne valeur `6*XSCALE` est commentée) | galaxian.h:51 (cf. 49) |
| `GALAXIAN_HBSTART` | `256 * GALAXIAN_XSCALE` | **768** (l'ancienne valeur `264*XSCALE` = 792 est commentée) | galaxian.h:52 (cf. 50) |
| `GALAXIAN_VTOTAL`  | `264`                   | **264** lignes/frame | galaxian.h:54 |
| `GALAXIAN_VBEND`   | `16`                    | **16** | galaxian.h:55 |
| `GALAXIAN_VBSTART` | `224 + 16`              | **240** | galaxian.h:56 |

Commentaire de normalisation (galaxian.h:44-46) : « H counts from 128->511, HBLANK starts at 130 and ends at 250 / we normalize this here so that we count 0->383 with HBLANK from 264-383 ».

### Timing hardware (commentaires du driver, galaxian_v.cpp:9-66)

- Horloge vidéo : le XTAL 18.432 MHz est divisé par 3 (J/K flip-flops) en **6.144 MHz** qui pilote la plupart de la logique vidéo ; le circuit diviseur a un duty cycle de **66 %** — « important for accurate stars rendering » (galaxian_v.cpp:11-16).
- Horizontal : H compte 010000000 (128) → 111111111 (511), soit **384 H clocks/scanline** ; le bit haut est inversé en 256H, donc blanking = 110000000→111111111 puis « main portion of screen = 256 pixels » (galaxian_v.cpp:21-28). HBLANK (flip-flop clocké par 2H) : à 1 quand H=130, à 0 quand H=250 → **264 total non-blanked pixels** = 6 px à gauche (H=250..255) + 256 px zone principale (H=256..511) + 2 px à droite (H=128..129) (galaxian_v.cpp:30-38). HSYNC clocké par 16H, haut de H=176 à H=208 (galaxian_v.cpp:40-45).
- Vertical : V compte 011111000 (248) → 111111111 (511), soit **264 V clocks/frame** ; la chaîne de sync V est clockée par HSYNC (V en retard d'un count pendant les 48 premiers H clocks du blanking — important pour sprites/missiles) (galaxian_v.cpp:50-56). VBLANK : à 1 quand V=496, à 0 quand V=272 → **224 total non-blanked lines** (galaxian_v.cpp:58-62). VSYNC = !256V, haut de V=248 à V=256 (galaxian_v.cpp:64-66).

### Résolution utile vs normalisée (XSCALE)

- Frame complète normalisée : **1152×264** pixel-clocks (HTOTAL×VTOTAL, galaxian.h:47,54), à PIXEL_CLOCK = 18.432 MHz (galaxian.cpp:7502 + galaxian.h:41).
- Zone utile selon les constantes passées à `set_raw` : largeur = HBSTART−HBEND = 768−0 = **768 px**, hauteur = VBSTART−VBEND = 240−16 = **224 lignes** (galaxian.h:48,52,55,56) — mapping visarea = [hbend..hbstart) × [vbend..vbstart), sémantique de `screen_device::set_raw` du MAME core (`src/emu/screen.h`, hors du dossier fourni). En unités hardware (÷XSCALE=3) : **256 colonnes × 224 lignes**, ce qui correspond à la « main portion of screen = 256 pixels » (galaxian_v.cpp:28) et à la largeur du champ d'étoiles 256×m_x_scale (galaxian_v.cpp:955, galaxian.h:432).
- Écart comment/constante : le commentaire dit HBLANK normalisé 264–383 (galaxian.h:44-46) et « 264 total non-blanked pixels » (galaxian_v.cpp:35), mais `HBSTART = 768` (galaxian.h:52) ≠ 264×XSCALE = 792 (commenté, galaxian.h:50) : MAME ne rend que les 256 colonnes principales, pas les 8 pixels non-blankés supplémentaires des bords (galaxian_v.cpp:36-38).

### Rendu des étoiles

- `m_x_scale` = `GALAXIAN_XSCALE` par défaut (galaxian.h:432), réglable via `set_x_scale` (galaxian.h:338) ; l'échelle ×3 horizontale est indispensable au rendu correct des étoiles (galaxian.h:36-37).
- Ordre de rendu : `screen_update_galaxian` = fond + étoiles → tilemap background → sprites → bullets (galaxian_v.cpp:460-477) ; le fond par défaut est `galaxian_draw_background` (galaxian.cpp:8818) = remplissage noir + `galaxian_draw_stars(bitmap, cliprect, 256)` — le champ d'étoiles couvre les **256 colonnes hardware** × XSCALE = toute la largeur utile de 768 px (galaxian_v.cpp:950-956).
- Mécanisme : LFSR 17 bits, période `STAR_RNG_PERIOD = (1<<17)-1` (galaxian_v.cpp:228), précalculée dans `stars_init` — une étoile est active si les 8 bits hauts sont à 1 et le bit bas à 0, couleur = les 6 bits sous les 8 hauts (galaxian_v.cpp:796-811). Chaque ligne avance l'offset de 512 (`star_offs = m_star_rng_origin + y*512`, galaxian_v.cpp:943) ; le registre est clocké 512×256 = 2^17 fois par frame, et l'écart d'un clock par frame (selon `m_flipscreen_x`) produit le défilement horizontal des étoiles (galaxian_v.cpp:822-846).
- Horloge asymétrique du RNG : le RNG est clocké par MASTER(18 MHz) AND PIXEL(6 MHz) ; avec le duty 2/3, chaque pixel hardware consomme 2 clocks RNG inégaux — d'où l'expansion ×3 de MAME : 1er clock RNG → 1 px normalisé (`m_x_scale*x+0`), 2e clock → 2 px (`+1`,`+2`) (galaxian_v.cpp:874-915, commentaire 881-896). Une étoile est dessinée seulement si `V1 ^ H8 == 1` : `enable_star = (y ^ (x >> 3)) & 1` (galaxian_v.cpp:877-878), avec couleur `m_star_color[star & 0x3f]` (galaxian.h:450).
- Couleurs d'étoiles : 64 valeurs dérivées de la palette PROM — R = bit5 (150 Ω) + bit4 (100 Ω), G = bit3 + bit2, B = bit1 + bit0, via `starmap[4]` ; « The stars are at 150 Ohms for the LSB, and 100 Ohms for the MSB » (galaxian_v.cpp:310-311, 325-352). Cf. KB-08.

## 4. Horloges — set d'origine

| Horloge | Valeur | Dérivation / source |
|---|---:|---|
| XTAL maître (`GALAXIAN_MASTER_CLOCK`) | **18.432 MHz** | galaxian.h:32 |
| Pixel-clocks de rendu MAME (`GALAXIAN_PIXEL_CLOCK`) | **18.432 MHz** | `XSCALE*MASTER/3 = 3×18.432/3` (galaxian.h:41,37,32) ; passé à `set_raw` (galaxian.cpp:7502) |
| Logique vidéo hardware | **6.144 MHz** (duty 66 %) | MASTER/3 par J/K flip-flops (galaxian_v.cpp:11-15) |
| Z80 CPU | **3.072 MHz** | `PIXEL_CLOCK/3/2 = MASTER/6` (galaxian.cpp:7492 + galaxian.h:41) |
| Son discret (`SOUND_CLOCK`) | **1.536 MHz** | `GALAXIAN_MASTER_CLOCK/6/2` (galaxian_a.cpp:36) |
| HSYNC (débit de ligne) | **16 kHz**, pulse 32 H clocks (≈ 5.2 µs) | 6.144 MHz/384 ; flip-flop clocké par 16H, haut H=176..208 (galaxian_v.cpp:21-22,40-45) |
| VSYNC (framerate) | **≈ 60.6 Hz**, pulse 8 lignes (500 µs) | chaîne V clockée par HSYNC : 16 kHz/264 ; ligne = 62.5 µs ; haut V=248..256 (galaxian_v.cpp:50-51,53-56,64-66) |
| `KONAMI_SOUND_CLOCK` = 14.318181 MHz XTAL | défini mais **non utilisé** par le set d'origine | galaxian.h:33 ; utilisé seulement par les sets à AY8910 (`konami_sound_1x_ay8910`, galaxian.cpp:7546-7556 ; `konami_sound_2x_ay8910`, 7586-7605) — « differs » |

## 5. Composants configurés (config de base du set `galaxian`)

Le set d'origine = `galaxian_base(config)` + `GALAXIAN_SOUND(config, "cust")` (galaxian.cpp:7660-7665) :

| Composant | Ligne | Détail / cross-ref |
|---|---|---|
| `Z80` maincpu | galaxian.cpp:7492 | 3.072 MHz, addrmap `galaxian_map` (7493) — §1, KB-03 |
| `WATCHDOG_TIMER` "watchdog" | galaxian.cpp:7495 | vblank count 8 sur "screen", réarmé par lecture 0x7800–0x7fff (1769) — §2, KB-06 |
| `GFXDECODE` m_gfxdecode | galaxian.cpp:7498 | layout `gfx_galaxian` : charlayout 8×8 (galaxian.cpp:7394-7403) + spritelayout 16×16 (7405-7414), tous deux échellés ×XSCALE=3 (7444-7447) — KB-22a/KB-22b |
| `PALETTE` m_palette, 32 entrées | galaxian.cpp:7499 | `galaxian_palette` lit la région `proms` (galaxian_v.cpp:238-240) — KB-08 |
| `SCREEN` m_screen | galaxian.cpp:7501-7504 | raster, `set_raw(...)` + `screen_update_galaxian` + vblank → `vblank_interrupt_w` — §3, KB-07/KB-26 |
| `SPEAKER` "speaker" | galaxian.cpp:7507 | `.front_center()` |
| Son : `GALAXIAN_SOUND(config, "cust")` | galaxian.cpp:7664 | device type « Galaxian Custom Sound » (galaxian_a.cpp:613, déclaré galaxian_a.h:72) ; ajoute un son DISCRETE avec interface `galaxian_discrete` routée vers ":speaker" (galaxian_a.cpp:665-669, 335), dont le circuit est modélisé à SOUND_CLOCK = MASTER/6/2 = 1.536 MHz (galaxian_a.cpp:36, cf. galaxian_a.cpp:406) — KB-27 (à venir) |

Le set d'origine n'a **pas** de PPI8255 : les ports IN0/IN1/IN2 sont lus directement via la memory map (KB-03, KB-24a) ; le PPI n'apparaît que dans `konami_base` pour les autres sets (galaxian.cpp:7528-7543).

## Differs (hors périmètre — variantes)

- **sidam** (bootleg) : XTAL 12 MHz (`SIDAM_MASTER_CLOCK`, galaxian.h:34), CPU `set_clock(12_MHz_XTAL/2/2)` = 3 MHz, `gfx_sidam` et écran reconfiguré à 12 MHz avec `SIDAM_*` (HTOTAL=768, XSCALE=2) — galaxian.cpp:7511-7525, galaxian.h:39,42,58-61.
- **gmgalax** : mêmes `galaxian_charlayout` (8×8) et `galaxian_spritelayout` (16×16), mais avec le nombre de tuiles/sprites doublé à 16 au lieu de 8 (`gfx_gmgalax`, galaxian.cpp:7454-7457, vs `gfx_galaxian` d'origine à 7444-7447).
- **namenayo** : color PROMs séparés, sprites décalées d'offset 32 (`gfx_namenayo`, galaxian.cpp:7460-7463).
- **tenspot** : charlayout offset 0x200 + région gfx2 séparée pour les sprites (`gfx_tenspot`, galaxian.cpp:7471-7474, layouts 7416-7436).
