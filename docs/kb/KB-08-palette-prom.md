# KB-08 — Palette / PROM

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h), périmètre **Galaxian d'origine** uniquement.

Réseau de résistances (PROM 8 bits par entrée) :
```
bit7 -220Ω- BLEU   bit3 -220Ω- VERT
bit6 -470Ω- BLEU   bit2 -1kΩ - VERT
bit5 -220Ω- VERT   bit1 -470Ω- ROUGE
bit4 -470Ω- VERT   bit0 -1kΩ - ROUGE
```
(Transcription fidèle de la doc fournie ; the affectation exacte bit→composante is à **re-vérifier dans galaxian_v.cpp**, GAP-02 : the doc liste « VERT » two times for bits 5 and 4 alors que the schéma standard est R: bits 0-2, G: bits 3-5, B: bits 6-7.)
```cpp
static const int rgb_resistances[3] = {1000, 470, 220};
compute_resistor_weights(0, RGB_MAXIMUM, -1.0,
   3,&rgb_resistances[0],rweights,470,0,
   3,&rgb_resistances[0],gweights,470,0,
   2,&rgb_resistances[1],bweights,470,0);
```
`RGB_MAXIMUM = 224` (marge for étoiles/shells). En parallèle : paire 150Ω/100Ω per composante for the étoiles ; résistance 100Ω for shells/missile. Couleurs of bullets : 7 blanches `(255,255,255)`, the dernière jaune `(255,255,0)`.
Fond : **noir uni** + étoiles.
