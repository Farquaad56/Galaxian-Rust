# KB-02 — Horloges et timing (valeurs de référence)

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h), périmètre **Galaxian d'origine** uniquement.

```
XTAL maître            = 18.432 MHz
Z80 (XTAL/6)           = 3.072 MHz
Pixel (XTAL/3)         = 6.144 MHz   (HSYNC = XTAL/3/192/2 = 16 kHz  → 384 pixels/ligne)
VSYNC                  = HSYNC/132/2 = 60.606060 Hz
VBlank                 ≈ 2500 µs
```
Constantes MAME (galaxian.h) :
```cpp
GALAXIAN_XSCALE  = 3                       // rendu interne ×3 horizontal (indispensable pour les étoiles)
GALAXIAN_HTOTAL  = 384 * XSCALE ; HBEND = 0 ; H0START = 0 ; HBSTART = 256 * XSCALE
GALAXIAN_VTOTAL  = 264 ; VBEND = 16 ; VBSTART = 224 + 16 (=240)
```
**Valeurs dérivées (à vérifier par test T2.x)** :
- Cycles Z80 par ligne : 3 072 000 / 16 000 = **192**.
- Lignes par frame : **264** → **50 688 cycles/frame** (3 072 000 / 60,606 = 50 688, entier).
- Zone visible : lignes 16..239 (224 lignes), pixels 0..255 (256) ; HBLANK pour pixels 256..383 (normalisé).
- Le facteur ×3 découle du rapport 3:2 entre horloge maître et horloge pixel pilotant le générateur d'étoiles (KB-12).
