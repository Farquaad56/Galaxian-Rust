# KB-22 — GFX decoding : layouts caractères / sprites et région d'images

> Source: driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian.h), **périmètre Galaxian d'origine** uniquement.

## Vue synoptique

| Élément | Tuiles | Entrées totales | Bits/plane | Accès ROM par rangée (16 px) |
|---|---|---|---|---|
| `galaxian_charlayout` ([galaxian.cpp:7394](../mame_src/galaxian.cpp#L7394)) | 8 × 8 px | `RGN_FRAC(1,2)` (≈ 512 entrées) | 2 plans par puce | `STEP8(0,1)` + `STEP8(0,8)` |
| `galaxian_spritelayout` ([galaxian.cpp:7405](../mame_src/galaxian.cpp#L7405)) | 16 × 16 px | `RGN_FRAC(1,2)` (≈ 512 entrées) | 2 plans par puce | `STEP8(0,1), STEP8(8*8,1)` + `STEP8(0,8), STEP8(16*8,8)` |

## Détail des champs de `gfx_layout`

```c
// galaxian.cpp:7394-7403
static const gfx_layout galaxian_charlayout = {
    8, 8,                        // width=8, height=8 (px)
    RGN_FRAC(1,2),               // total entries (~512)
    2,                            // bits per pixel par plan (→ 4 pen values)
    { RGN_FRAC(0,2), RGN_FRAC(1,2) }, // color table: base=0, span=RGN_FRAC(1,2)
    { STEP8(0,1) },               // offsets: [chip0] = 0 par rangée
    { STEP8(0,8) },               // row steps:      [chip0] = 8 octets/rangée de 8 px
    8*8                           // tile size in bytes (64 o)
};

// galaxian.cpp:7405-7414
static const gfx_layout galaxian_spritelayout = {
    16, 16,                       // width=16, height=16 (px)
    RGN_FRAC(1,2),               // total entries (~512)
    2,                            // bits per pixel par plan
    { RGN_FRAC(0,2), RGN_FRAC(1,2) },
    { STEP8(0,1), STEP8(8*8,1) },// offsets: chip0→0, chip1→64 (par rangée de 16 px)
    { STEP8(0,8), STEP8(16*8,8) },// row steps: chip0=8o/rangée, chip1=128o/rangée
    16*16                         // tile size in bytes (256 o)
};
```

## Interprétation des offsets / strides par plan

| Layout | Offset chip 0 | Offset chip 1 | Row-step chip 0 | Row-step chip 1 | Combinaison |
|---|---|---|---|---|---|
| `charlayout` | `0` (pas d'inter-plan) | — | `1` o/rangée | — | simple pas-à-pas par rangées de 8 px |
| `spritelayout` | `0` | `0x40` (64) | `8` o/rangée | `8` o/rangée | les deux plans sont combinés ligne par ligne |

> **Note** : le décalage de `0x40` sur chip 1 dans `spritelayout` fait que chaque rangée sprite lit **deux octets différents** du jeu ROM (`1h.bin` / `1k.bin`) — la première moitié des données (chip 0) et la seconde (chip 1), puis les recombine.

## Region d'images : `gfx1`

| Fichier ROM | Taille | Offset dans `gfx1` | Rôle |
|---|---|---|---|
| `1h.bin` | 2 KiB (`0x0800`) | `0x0000`–`0x07FF` | **Chip A** — bitplane 0 (poids forts) |
| `1k.bin` | 2 KiB (`0x0800`) | `0x0000`–`0x07FF` | **Chip B** — bitplane 1 (poids faibles) |

Chaque tuile de caractères ou sprite est composée de deux plans issus de chips différents, lus de façon entrelacée. La région complète fait donc `0x1000` octets.

## `GFXDECODE_START(gfx_galaxian)` (galaxian.cpp:7444)

```c
static GFXDECODE_START(gfx_galaxian)
    GFXDECODE_SCALE("gfx1", 0x0000, galaxian_charlayout,   0, 8, GALAXIAN_XSCALE, 1)
    GFXDECODE_SCALE("gfx1", 0x0000, galaxian_spritelayout, 0, 8, GALAXIAN_XSCALE, 1)
GFXDECODE_END
```

| Paramètre | Valeur char | Valeur sprite | Signification |
|---|---|---|---|
| Region | `"gfx1"` | `"gfx1"` | Région mémoire des données graphiques |
| Offset région | `0x0000` | `0x0000` | Décalage dans la région (même jeu) |
| Layout | `galaxian_charlayout` | `galaxian_spritelayout` | Structure de décodage correspondante |
| `color_start` | `0` | `0` | Index de début dans la table de couleurs |
| `color_span` | `8` | `8` | Taille de la plage (4 valeurs × 2 = 8) |
| `xscale` | `GALAXIAN_XSCALE` (=3) | — | Facteur d'expansion horizontale |
| `yscale` | `1` | — | Facteur d'expansion verticale (pas de scaling Y) |

> **Remarque** : `GALAXIAN_XSCALE = 3` est défini dans [galaxian.h:37](../mame_src/galaxian.h#L37). Le facteur de largeur `xscale=3` multiplie les coordonnées X par 3 lors du rendu.

## Comptage des péniches (pen count)

Avec **2 plans × 1 bit/pla n** = **4 états de pixels** (`00`, `01`, `10`, `11`) par combinaison de lignes :

- La table de couleurs fait 8 entrées (`color_span=8`), ce qui correspond à un décalage de 3 bits (puisque 2³ = 8).
- Les péniches sont indexées via `(chip0_data ^ chip1_data) & 7`, produisant des indices **0…7** directement lisibles dans la palette.

Aucune table supplémentaire n'est nécessaire : les `color_start` et `color_span` suffisent à mapper chaque combinaison de plans sur une entrée de palette de 8 pixels.

## Résumé des tailles de tuile

| Type | Dimensions (px) | Entrées | Octets/tuile |
|---|---|---|---|
| Caractère | 8 × 8 | `RGN_FRAC(1,2)` ≈ 512 | 64 |
| Sprite | 16 × 16 | `RGN_FRAC(1,2)` ≈ 512 | 256 |

Chaque entrée de la région `gfx1` (taille totale = 0x1000 octets) contient une séquence de plans lue ligne par ligne. Les deux bits extraits à chaque pixel se combinent pour donner l'indice de couleur final après décalage par `color_start`.
