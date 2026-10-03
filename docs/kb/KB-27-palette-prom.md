# KB-27 — Palette PROM (set `galaxian`)

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian.h, galaxian_v.cpp), périmètre **Galaxian d'origine** uniquement. Citations relatives à `docs/mame_src/`.
> Résout GAP-02 (affectation exacte bit→R/G/B de la PROM couleur) ; complète KB-08 et les fiches KB-25b §3 / KB-26 §1b,§3 / KB-22a / KB-22b.

## 1. Affectation bit→R/G/B de la PROM couleur (sprites/tilemap) — `galaxian_palette`

Fonction : `void galaxian_state::galaxian_palette(palette_device &palette)` (galaxian_v.cpp:238), enregistrée comme init du device palette à **32 entrées** : `PALETTE(config, m_palette, FUNC(galaxian_state::galaxian_palette), 32);` (galaxian.cpp:7499).

- Source des couleurs : région mémoire `"proms"` = **0x20 octets**, remplie par la ROM `6l.bpr` (galaxian.cpp:9772-9773) ; `const uint8_t *color_prom = memregion("proms")->base();` (galaxian_v.cpp:240). La boucle décode `len = memregion("proms")->bytes()` = **32 entrées** (galaxian_v.cpp:280-281) et écrit chaque plume via `palette.set_pen_color(i, rgb_t(r, g, b))` (galaxian_v.cpp:302).
- Résistances : `static const int rgb_resistances[3] = { 1000, 470, 220 };` (galaxian_v.cpp:241) — index 0 = 1 kΩ, index 1 = 470 Ω, index 2 = 220 Ω.

Commentaire du driver (galaxian_v.cpp:243-253), câblage bit 7 → bit 0 :

| Bit | Résistance | Composante | Ligne MAME |
|---|---|---|---|
| 7 | 220 Ω | BLEU | galaxian_v.cpp:246 |
| 6 | 470 Ω | BLEU | galaxian_v.cpp:247 |
| 5 | 220 Ω | VERT (GREEN) | galaxian_v.cpp:248 |
| 4 | 470 Ω | VERT (GREEN) | galaxian_v.cpp:249 |
| 3 | 1 kΩ | VERT (GREEN) | galaxian_v.cpp:250 |
| 2 | 220 Ω | ROUGE (RED) | galaxian_v.cpp:251 |
| 1 | 470 Ω | ROUGE (RED) | galaxian_v.cpp:252 |
| 0 | 1 kΩ | ROUGE (RED) | galaxian_v.cpp:253 |

Note du driver : « not all boards have this configuration. Namco PCBs may have 330 ohm resistors instead of 220, but the default setup has also been used by Namco » (galaxian_v.cpp:255-257).

Sources parallèles sur les mêmes bus R/G/B (commentaire galaxian_v.cpp:259-265) :
- **Générateur d'étoiles** : « a pair of 150 ohm and 100 ohm resistors on each R,G,B component that are connected to the star generator » (galaxian_v.cpp:259-261) — cf. §3.
- **Coquilles/missile** : « a set of 100 ohm resistors on each R,G,B component that are enabled when a shell/missile is enabled » (galaxian_v.cpp:263-265) — cf. §4.

### Dérivation des poids — `compute_resistor_weights`

```cpp
double rweights[3], gweights[3], bweights[2];                                                            // galaxian_v.cpp:273
compute_resistor_weights(0, RGB_MAXIMUM, -1.0,                                                           // :274
        3, &rgb_resistances[0], rweights, 470, 0,                                                       // :275
        3, &rgb_resistances[0], gweights, 470, 0,                                                       // :276
        2, &rgb_resistances[1], bweights, 470, 0);                                                      // :277
```

- `RGB_MAXIMUM = 224` (galaxian_v.cpp:229) : « we use RGB_MAXIMUM as the maximum to give headroom for stars and shells/missiles » (galaxian_v.cpp:267-271).
- R et G reçoivent les **3** résistances `{1000, 470, 220}` (`&rgb_resistances[0]`) ; B reçoit les **2 dernières** `{470, 220}` (`&rgb_resistances[1]`) (galaxian_v.cpp:275-277).
- La fonction core `compute_resistor_weights` n'est pas dans le dossier fourni (MAME core) : chaque poids de bit est la contribution du diviseur de tension produit quand ce seul bit est actif, donc inversement proportionnelle à sa résistance (1/R), et l'auto-scaling (`scaler = -1.0`) normalise pour que la somme des bits d'une composante tous actifs atteigne maxval = RGB_MAXIMUM. Valeurs calculées depuis les constantes citées : R/G — bit 1 kΩ ≈ 29,2 ; bit 470 Ω ≈ 62,1 ; bit 220 Ω ≈ 132,7 (somme = 224) ; B — bit 470 Ω ≈ 71,4 ; bit 220 Ω ≈ 152,6 (somme = 224).

### Décodage de la palette (galaxian_v.cpp:279-303)

```cpp
// red component
bit0 = BIT(color_prom[i], 0);   bit1 = BIT(color_prom[i], 1);   bit2 = BIT(color_prom[i], 2);            // :286-288
int const r = combine_weights(rweights, bit0, bit1, bit2);                                                // :289

// green component
bit0 = BIT(color_prom[i], 3);   bit1 = BIT(color_prom[i], 4);   bit2 = BIT(color_prom[i], 5);            // :292-294
int const g = combine_weights(gweights, bit0, bit1, bit2);                                                // :295

// blue component
bit0 = BIT(color_prom[i], 6);   bit1 = BIT(color_prom[i], 7);                                            // :298-299
int const b = combine_weights(bweights, bit0, bit1);                                                      // :300

palette.set_pen_color(i, rgb_t(r, g, b));                                                                // :302
```

(`combine_weights` est aussi du MAME core, hors du dossier fourni.)

**Affectation confirmée : R = bits 0-2 (poids croissants 1 kΩ/470 Ω/220 Ω), G = bits 3-5, B = bits 6-7.** Le commentaire (:243-253) et le code de décodage (:286-300) concordent exactement.

## 2. Correction explicite de GAP-02

La doc fournie (transcrite dans KB-08.md:4-7, identique dans KB-08-palette-prom.md:7-10) liste « VERT » pour les bits **5 et 4** — ainsi que pour les bits 3 et 2 — ne laissant que deux entrées ROUGE (bits 1/0). Cette affectation bit→composante est erronée.

MAME tranche : **R = bits 0-2, G = bits 3-5, B = bits 6-7**.
- Lignes qui tranchent : le rouge est décodé des bits 0/1/2 (galaxian_v.cpp:286-289), le vert des bits 3/4/5 (galaxian_v.cpp:292-295), le bleu des bits 6/7 (galaxian_v.cpp:298-300) ; le commentaire du driver confirme le même câblage bit 7→bit 0 (galaxian_v.cpp:246-253).
- En particulier, la doc a tort sur les bits 3/2 : MAME a **bit3 = VERT @1 kΩ** (galaxian_v.cpp:250) et **bit2 = ROUGE @220 Ω** (galaxian_v.cpp:251), là où la doc écrit « bit3 -220Ω- VERT / bit2 -1kΩ- VERT » (KB-08.md:4-5).
- Conséquence : **3 entrées ROUGE** (bits 0/1/2) et **3 entrées VERT** (bits 3/4/5), pas 2 et 4.

## 3. Palette des étoiles — `m_star_color[64]`

Déclaration : `rgb_t m_star_color[64];` (galaxian.h:450). Remplie dans la même fonction, boucle sur i = 0..63 (galaxian_v.cpp:331-353).

Câblage hardware (commentaire galaxian_v.cpp:259-261) : paire de résistances **150 Ω + 100 Ω** par composante R/G/B, connectées au générateur d'étoiles. Normalisation (galaxian_v.cpp:305-318) : la résistance max sprite/tilemap est ~130 Ω — « 1/(1/1000 + 1/470 + 1/220) » (galaxian_v.cpp:306-308) — et RGB_MAXIMUM y est normalisé ; les étoiles à 150 Ω (LSB) / 100 Ω (MSB) (galaxian_v.cpp:310-311) donneraient idéalement RGB_MAXIMUM×130/150, ×130/100, ×130/60 (galaxian_v.cpp:313-315), trop saturé — MAME « approximate this by compressing the values proportionally into the 194->255 range » (galaxian_v.cpp:317-318).

Table de compression `starmap[4]` (galaxian_v.cpp:320-329) :
```cpp
int const minval = RGB_MAXIMUM * 130 / 150;   // = 194                                          // :320
int const midval = RGB_MAXIMUM * 130 / 100;   // = 291                                          // :321
int const maxval = RGB_MAXIMUM * 130 / 60;    // = 485                                          // :322

uint8_t const starmap[4]{                                                             // :325-329
        0,
        minval,
        minval + (255 - minval) * (midval - minval) / (maxval - minval),              // = 214
        255 };
```
Valeurs calculées : `starmap = {0, 194, 214, 255}`.

Affectation bit→composante des 6 bits d'indice (galaxian_v.cpp:336-349) — LSB de chaque composante à 150 Ω, MSB à 100 Ω :

| Bits | Résistances (LSB/MSB) | Composante | Ligne MAME |
|---|---|---|---|
| bit5 / bit4 | 150 Ω / 100 Ω | ROUGE | galaxian_v.cpp:336-339 (« bit 5 = red @ 150 Ohm, bit 4 = red @ 100 Ohm », :336) |
| bit3 / bit2 | 150 Ω / 100 Ω | VERT | galaxian_v.cpp:341-344 (:341) |
| bit1 / bit0 | 150 Ω / 100 Ω | BLEU | galaxian_v.cpp:346-349 (:346) |

Chaque composante combine ses deux bits via `starmap[(bit_msb << 1) | bit_lsb]` (galaxian_v.cpp:339, 344, 349), puis `m_star_color[i] = rgb_t(r, g, b)` (galaxian_v.cpp:352).

Consommation : l'indice de couleur d'une étoile = les 6 bits bas de l'état du LFSR (`star & 0x3f`), écrit pixel par pixel dans `stars_draw_row` (galaxian_v.cpp:903, 911-912). Mécanisme complet du générateur d'étoiles : KB-25b §3 « Rendu des étoiles », KB-26 §1b.

## 4. Couleurs coquilles/missile — `m_bullet_color[8]`

Déclaration : `rgb_t m_bullet_color[8];` (galaxian.h:454). Câblage hardware (commentaire galaxian_v.cpp:263-265) : jeu de résistances **100 Ω** par composante R/G/B, « enabled when a shell/missile is enabled ».

Valeurs par défaut (galaxian_v.cpp:355-358) :
```cpp
// default bullet colors are white for the first 7, and yellow for the last one          // :355
for (int i = 0; i < 7; i++)
    m_bullet_color[i] = rgb_t(0xff, 0xff, 0xff);                                         // :356-357
m_bullet_color[7] = rgb_t(0xff,0xff,0x00);                                               // :358
```

| Entrée | Couleur par défaut | Ligne MAME |
|---|---|---|
| 0..6 (coquilles) | blanc `(255,255,255)` | galaxian_v.cpp:356-357 |
| 7 (missile) | jaune `(255,255,0)` | galaxian_v.cpp:358 |

Consommation : `galaxian_draw_bullet` (galaxian_v.cpp:1160) — « The first 7 entries are called "shells" and render as white; the final entry is called a "missile" and renders as yellow » (galaxian_v.cpp:1162-1167), tir de **4 pixels** par entrée avec `m_bullet_color[offs]` (galaxian_v.cpp:1169-1172). Sémantique coquilles vs missile : KB-26 §3.

## 5. Cross-références

- **KB-08** (PROM palette) : la fiche qui transcrivait la doc fournie et ouvrait GAP-02 — résolu par cette fiche (§1/§2).
- **KB-25b §3** « Écran (raster) » → sous-section « Rendu des étoiles » : mécanisme du générateur d'étoiles (LFSR 17 bits, horloge asymétrique ×3), couleur `m_star_color[star & 0x3f]`.
- **KB-26** : §1b étoiles (`galaxian_draw_stars`/`stars_draw_row`, galaxian_v.cpp:932, 869) ; §3 coquilles vs missile (entrées 0–6 = coquilles blanches, entrée 7 = missile jaune).
- **KB-22a / KB-22b** : la tilemap de fond 8×8 et les sprites 16×16 consomment cette palette — plumes décodées depuis `proms` via le device `m_palette` (galaxian.cpp:7499) ; profondeur couleur de **8 bits** par tuile/sprite — `GFXDECODE_SCALE("gfx1", 0x0000, galaxian_charlayout / galaxian_spritelayout, 0, 8, GALAXIAN_XSCALE, 1)` (galaxian.cpp:7445-7446).

## Differs (hors périmètre — variantes)

- **moonwar** : `moonwar_palette` = `galaxian_palette` + surcharge `m_bullet_color[7] = rgb_t(0xef, 0xef, 0x97)` — « wire mod to connect the bullet blue output to the 220 ohm resistor » (galaxian_v.cpp:361-367).
- **eagle** : harnais de câblage qui permute RGB → GBR sur toute la palette (galaxian_v.cpp:369-378) ; **sbhoei** : RGB → RBG (galaxian_v.cpp:381-390).
- **mshuttle** : couleurs de coquilles/missile propres, « verified by schematics » — bullets W toujours violettes, bullets Y variables selon H4/H3/H2 (galaxian_v.cpp:1176-1182).
