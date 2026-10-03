# KB-22a — Layout GFX des tuiles de caractères (8×8)

> Source: driver MAME `src/mame/galaxian/` (galaxian.cpp), **périmètre Galaxian d'origine** uniquement.
> Ferme une partie de GAP-01 (format VRAM/OBJRAM, attributs par colonne, taille/layout tilemap).

## Layout `galaxian_charlayout` (galaxian.cpp:7394)

```c
static const gfx_layout galaxian_charlayout =
{
    8, 8,                          // width=8, height=8 (px)
    RGN_FRAC(1,2),                 // total entries (~512 tuiles)
    2,                             // bits per pixel par plan (→ 4 pen values)
    { RGN_FRAC(0,2), RGN_FRAC(1,2) },// color table: base=0, span=RGN_FRAC(1,2)=512
    { STEP8(0,1) },                // offsets plans: [chip0]=0 par rangée
    { STEP8(0,8) },                // row steps:      [chip0]=8 octets/rangée de 8 px
    8*8                            // tile size in bytes (64 o)
};
```

### Décodage des champs GfxLayout

| Position | Champ | Valeur char | Interprétation |
|---|---|---|---|
| 0 | width | `8` | Largeur de tuile en px |
| 1 | height | `8` | Hauteur de tuile en px |
| 2 | entries | `RGN_FRAC(1,2)` | Nombre d'entrées dans la région gfx (≈512) |
| 3 | planes | `2` | Nombre de plans (bits/plane = 1 → 4 états pixel) |
| 4 | color_base | `{ RGN_FRAC(0,2), RGN_FRAC(1,2) }` | Base table couleurs = 0 ; span = 512 |
| 5 | plan_offsets | `{ STEP8(0,1) }` | Offset du plan 0 = 0 (pas d'inter-plan pour les caractères) |
| 6 | row_steps | `{ STEP8(0,8) }` | Pas par rangée = 8 octets (un octet de plan par pixel sur 8 px) |
| 7 | pixel_size | `8*8` | 64 octets/tuile dans la ROM |

## Région d'images : `gfx1`

Les tuiles de caractères sont stockées dans la région `gfx1`, remplie par deux puces ROM :

| Fichier ROM | Taille | Offset gfx1 | Rôle (plan) |
|---|---:|---|---|
| `1h.bin` | 2 KiB (`0x0800`) | `0x0000`–`0x07FF` | Chip A — plan 0 (poids forts) |
| `1k.bin` | 2 KiB (`0x0800`) | `0x0000`–`0x07FF` | Chip B — plan 1 (poids faibles) |

Chaque tuile de caractères occupe **64 octets** dans la ROM (8 rangées × 8 octets),
composée de deux plans entrelacés issus de chips différents. La région `gfx1` fait
au total `0x1000` octets.

## Décodage par tuile

Pour chaque rangée y ∈ [0,7] :
- le plan 0 lit l'octet à `base + y*8 + x` (x ∈ [0,7]) → bit de poids fort ;
- le plan 1 lit l'octet à `base + y*8 + x` du second chip → bit de poids faible ;
- les deux bits forment un indice de couleur (pen) dans la palette via `(p0<<1)|p1`.

Aucun décalage entre plans pour les caractères (`STEP8(0,1)` = offset 0, `STEP8(0,8)` = pas de 8).
Contrairement aux sprites (KB-22b), il n'y a **qu'un seul plan par puce** et **pas d'inter-plan**.

## Variantes de layout

`galaxian_charlayout_0x200` (galaxian.cpp:7416) : identique à `galaxian_charlayout`
mais avec `entries = 0x200` au lieu de `RGN_FRAC(1,2)` — utilisé par des machines
dérivées où la région gfx est plus petite. Périmètre Galaxian d'origine : non utilisé
(seulement `galaxian_charlayout`).

## Décodeur GFXDECODE (galaxian.cpp:7444)

```c
static GFXDECODE_START(gfx_galaxian)
    GFXDECODE_SCALE("gfx1", 0x0000, galaxian_charlayout,   0, 8, GALAXIAN_XSCALE, 1)
GFXDECODE_END
```

| Paramètre | Valeur char | Signification |
|---|---|---|
| Region | `"gfx1"` | Région mémoire des données graphiques |
| Offset région | `0x0000` | Décalage dans la région |
| Layout | `galaxian_charlayout` | Structure de décodage |
| color_start | `0` | Index de début table couleurs |
| color_span | `8` | 4 valeurs × 2 (plans) = 8 péniches |
| xscale | `GALAXIAN_XSCALE` (=3, galaxian.h:37) | Expansion horizontale du tile |
| yscale | `1` | Pas de scaling vertical |

> **Remarque** : `xscale=3` multiplie les coordonnées X des tuiles par 3 lors du rendu.
> Voir KB-26 (ordre de rendu) pour la place des tuiles dans la composition écran.

## Résumé

| Élément | Valeur |
|---|---|
| Layout | `galaxian_charlayout` (8×8 px) |
| Tuiles | ≈512 (`RGN_FRAC(1,2)`) |
| Plans | 2 (un par puce), 1 bit/plane → 4 états pixel |
| Octets/tuile | 64 |
| Région gfx | `gfx1` (0x1000 o : `1h.bin` + `1k.bin`) |
| color_span | 8 péniches |
| xscale / yscale | 3 / 1 |
