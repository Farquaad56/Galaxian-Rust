# KB-08 — Palette / PROM

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h), périmètre **Galaxian d'origine** uniquement.

Réseau de résistances (PROM 8 bits par entrée) :
```
bit7 -220Ω- BLEU   bit3 -1kΩ - VERT
bit6 -470Ω- BLEU   bit2 -220Ω- ROUGE
bit5 -220Ω- VERT   bit1 -470Ω- ROUGE
bit4 -470Ω- VERT   bit0 -1kΩ - ROUGE
```
Affectation exacte bit→composante tranchée par MAME (GAP-02 résolu) : **R = bits 0-2, G = bits 3-5, B = bits 6-7** — commentaire du driver (galaxian_v.cpp:243-253) et décodage de la palette (galaxian_v.cpp:286-300), concordants. La doc fournie se trompait sur les bits 3/2 (« bit3 -220Ω- VERT / bit2 -1kΩ- VERT » au lieu de « bit3 -1kΩ- VERT / bit2 -220Ω- ROUGE », galaxian_v.cpp:250-251) : il y a **3 entrées ROUGE** (bits 0/1/2), pas 2.
```cpp
static const int rgb_resistances[3] = {1000, 470, 220};   // galaxian_v.cpp:241
compute_resistor_weights(0, RGB_MAXIMUM, -1.0,            // galaxian_v.cpp:274-277
    3,&rgb_resistances[0],rweights,470,0,
    3,&rgb_resistances[0],gweights,470,0,
    2,&rgb_resistances[1],bweights,470,0);
```
`RGB_MAXIMUM = 224` (galaxian_v.cpp:229 ; marge pour étoiles/shells). En parallèle : paire 150Ω/100Ω par composante pour les étoiles (galaxian_v.cpp:259-261) ; résistance 100Ω pour shells/missile (galaxian_v.cpp:263-265). Couleurs des bullets : 7 blanches `(255,255,255)` (galaxian_v.cpp:356-357), la dernière jaune `(255,255,0)` (galaxian_v.cpp:358).
Fond : **noir uni** + étoiles.

Cross-référence : KB-27 (fiche complète — affectation bit→R/G/B, palette des étoiles `m_star_color[64]`, couleurs coquilles/missile `m_bullet_color[8]`).
