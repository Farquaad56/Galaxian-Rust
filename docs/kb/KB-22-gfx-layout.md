# KB-22 — Décodeurs graphiques (GFX) et layouts tile/sprites

> Source : driver MAME `src/mame/galaxian/` (`galaxian.cpp`), jeu **Galaxian** (set `galaxian`). Chaque affirmation cite un fichier + ligne.
> Périmètre : set d'origine uniquement (les variantes sidam/namenayo/gmgalax/tenspot… existent dans le driver mais relèguées en « differs »).

## Tuiles 8×8 — `galaxian_charlayout`

```cpp
// galaxian.cpp:7394
static const gfx_layout galaxian_charlayout = {
    8, 8,
    RGN_FRAC(1, 2),                          // nombre total de tiles (moitié gfx1)
    2,                                        // nb_plan_planes (2 bitplanes)
    { RGN_FRAC(0, 2), RGN_FRAC(1, 2) },     // offsets des plans dans la ROM
    { STEP8(0, 1) },                          // pas de stride spécifique (bit-plans entiers)
    { STEP8(0, 8) },                          // adresse de départ du plan 0
    8 * 8                                       // pitch (octets par ligne de tile)
};
```

- Taille d'une tuile : **8 × 8 pixels**.
- Profondeur de couleur : **2 bpp** (deux plans de bits).
- Les données sont dans la région `gfx1` (voir plus bas), entrée par plan.

## Sprites 16×16 — `galaxian_spritelayout`

```cpp
// galaxian.cpp:7405
static const gfx_layout galaxian_spritelayout = {
    16, 16,
    RGN_FRAC(1, 2),                          // même région que les tuiles (gfx1)
    2,                                         // nb_plan_planes
    { RGN_FRAC(0, 2), RGN_FRAC(1, 2) },      // plans dans la ROM
    { STEP8(0, 1), STEP8(8 * 8, 1) },         // offset du plan 1 (décalé de 8 octets → 64)
    { STEP8(0, 8), STEP8(16 * 8, 8) },        // pas inter-plan (pas de entrecroisement)
    16 * 16                                    // pitch (octets par rangée sprite)
};
```

- Taille d'un sprite : **16 × 16 pixels**.
- Même convention de plans que les tuiles, mais organisation mémoire différente (`STEP8` avec décalages).
- Le pas de 16×8 correspond au fait que le premier plan est lu séquentiellement sur 8 octets par colonne, puis le deuxième plan prend la relais à l'adresse suivante.

## Table de décodage `gfx_galaxian`

```cpp
// galaxian.cpp:7444
static GFXDECODE_START(gfx_galaxian)
    GFXDECODE_SCALE("gfx1", 0x0000, galaxian_charlayout,   0, 8, GALAXIAN_XSCALE, 1)
    GFXDECODE_SCALE("gfx1", 0x0000, galaxian_spritelayout, 0, 8, GALAXIAN_XSCALE, 1)
GFXDECODE_END
```

- **Région** : `gfx1` (mêmes ROM `1h.bin`/`1k.bin`, voir KB-25 pour la carte ROM).
- **Offset de départ** : `0x0000`.
- **Échelle horizontale** : `GALAXIAN_XSCALE = 3` (le rendu est ×3 en X, cf. KB-02/KB-25).
- **Pas d'offset par défaut** (`0`) ; les entrées ne se chevauchent pas car elles référencent des layouts différents.

## Usage dans le driver de base

```cpp
// galaxian.cpp:7498 (dans la machine `galaxian`)
GFXDECODE(config, m_gfxdecode, m_palette, gfx_galaxian);
```

- Initialise les deux générateurs (`m_gfxdecode`), liés au décodeur de palette (`m_palette`, 32 entrées).
- Le driver utilise ensuite `gfx_decode_tile()`/`gfx_blend_overlay()` avec ces layouts pour produire les tuiles et sprites.

## Variantes (non-normatives, hors jeu original)

Le driver définit aussi des variantes `gfx_sidam`, `gfx_gmgalax`, `gfx_namenayo`, `gfx_pacmanbl`, `gfx_tenspot`, `gfx_videight` pour d'autres sets ; elles modifient l'échelle (`SIDAM_XSCALE`), l'offset du sprite (32) ou le pitch. Elles sont **hors périmètre** de cette fiche.
