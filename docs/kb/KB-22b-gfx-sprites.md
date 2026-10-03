# KB-22b — Layout GFX des sprites (16×16)

> Source: driver MAME `src/mame/galaxian/` (galaxian.cpp), **périmètre Galaxian d'origine** uniquement.
> Ferme une partie de GAP-01 (layout tilemap, attributs par colonne).

## Layout `galaxian_spritelayout` (galaxian.cpp:7405)

```c
static const gfx_layout galaxian_spritelayout =
{
    16, 16,                          // width=16, height=16 (px)
    RGN_FRAC(1,2),                   // total entries (~512 sprites)
    2,                               // bits per pixel par plan (→ 4 pen values)
    { RGN_FRAC(0,2), RGN_FRAC(1,2) },// color table: base=0, span=RGN_FRAC(1,2)=512
    { STEP8(0,1), STEP8(8*8,1) },    // offsets plans: [chip0]=0, [chip1]=64 par rangee
    { STEP8(0,8), STEP8(16*8,8) },   // row steps:      [chip0]=8o/rangee, [chip1]=128o/rangee
    16*16                            // tile size in bytes (256 o)
};
```

### Décodage des champs GfxLayout

| Position | Champ | Valeur sprite | Interprétation |
|---|---|---|---|
| 0 | width | `16` | Largeur de sprite en px |
| 1 | height | `16` | Hauteur de sprite en px |
| 2 | entries | `RGN_FRAC(1,2)` | Nombre d'entrées dans la région gfx (≈512) |
| 3 | planes | `2` | Nombre de plans (1 bit/plane → 4 états pixel) |
| 4 | color_base | `{ RGN_FRAC(0,2), RGN_FRAC(1,2) }` | Base table couleurs = 0 ; span = 512 |
| 5 | plan_offsets | `{ STEP8(0,1), STEP8(64,1) }` | Offset plan 0 = 0 ; offset plan 1 = 64 (inter-plan) |
| 6 | row_steps | `{ STEP8(0,8), STEP8(128,8) }` | Pas par rangee : chip0=8 o/rangee, chip1=128 o/rangee |
| 7 | pixel_size | `16*16` | 256 octets/sprite dans la ROM |

## Inter-plan (différence clé avec les tuiles)

Contrairement aux caractères (KB-22a, un seul plan par puce), le sprite combine **deux plans** :
- plan 0 (chip A, `1h.bin`) : offset 0, row-step 8 octets/rangée ;
- plan 1 (chip B, `1k.bin`) : offset `0x40` (64), row-step 128 octets/rangée.

Le décalage de `0x40` sur chip 1 fait que chaque rangée sprite lit **deux octets différents**
du jeu ROM : la première moitié des données (chip 0) et la seconde (chip 1), puis les recombine.
Chaque pixel = `(plan0_bit << 1) | plan1_bit`.

## Région d'images : `gfx1`

Même région que les tuiles — `gfx1`, remplie par deux puces ROM dans `ROM_START(galaxian)` :

| Fichier ROM | Taille | Offset gfx1 | Rôle (plan) |
|---|---:|---|---|
| `1h.bin` | 2 KiB (`0x0800`) | `0x0000`–`0x07FF` | Chip A — plan 0 (poids forts) |
| `1k.bin` | 2 KiB (`0x0800`) | `0x0000`–`0x07FF` | Chip B — plan 1 (poids faibles) |

Chaque sprite occupe **256 octets** dans la ROM (16 rayées × 16 octets), répartis sur les deux
plans entrelacés. La région `gfx1` fait au total `0x1000` octets.

## Variantes de layout

`galaxian_spritelayout_0x80` (galaxian.cpp:7427) : identique à `galaxian_spritelayout`
mais avec `entries = 0x80` au lieu de `RGN_FRAC(1,2)` — utilisé par des machines dérivées
où la région gfx est plus petite. Périmètre Galaxian d'origine : non utilisé (seulement
`galaxian_spritelayout`).

## Décodeur GFXDECODE (galaxian.cpp:7445)

```c
static GFXDECODE_START(gfx_galaxian)
    GFXDECODE_SCALE("gfx1", 0x0000, galaxian_spritelayout, 0, 8, GALAXIAN_XSCALE, 1)
GFXDECODE_END
```

| Paramètre | Valeur sprite | Signification |
|---|---|---|
| Region | `"gfx1"` | Région mémoire des données graphiques |
| Offset région | `0x0000` | Décalage dans la région |
| Layout | `galaxian_spritelayout` | Structure de décodage |
| color_start | `0` | Index de début table couleurs |
| color_span | `8` | 4 valeurs × 2 (plans) = 8 péniches |
| xscale | `GALAXIAN_XSCALE` (=3, galaxian.h:37) | Expansion horizontale du sprite |
| yscale | `1` | Pas de scaling vertical |

> **Remarque** : `xscale=3` multiplie les coordonnées X des sprites par 3 lors du rendu.
> Les sprites sont rendus après la tilemap et avant les shells (KB-26).

## Résumé

| Élément | Valeur |
|---|---|
| Layout | `galaxian_spritelayout` (16×16 px) |
| Sprites | ≈512 (`RGN_FRAC(1,2)`) |
| Plans | 2 (inter-plan : offset chip1 = 0x40), 1 bit/plane → 4 états pixel |
| Octets/sprite | 256 |
| Région gfx | `gfx1` (0x1000 o : `1h.bin` + `1k.bin`) |
| color_span | 8 péniches |
| xscale / yscale | 3 / 1 |
