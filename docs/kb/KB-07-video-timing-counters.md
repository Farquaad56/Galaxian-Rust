# KB-07 — Timing vidéo détaillé (compteurs matériels)

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h), périmètre **Galaxian d'origine** uniquement.

Horizontal : compteur H de 128 à 511 (bit fort inversé = 256H), normalisé `000000000→011111111` (actif, 256 px) puis `110000000→111111111` (blanking).
- **HBLANK** : bascule cadencée by 2H, D = `!(64H & 32H & 16H & 8H)` → 1 à H=130, 0 à H=250 → 264 px non blankés (6 px à gauche H=250-255, 256 px principaux, 2 px à droite H=128-129).
- **HSYNC** : bascule cadencée by 16H, D = `!(!64H & 32H)`, /Q → 1 à H=176, 0 à H=208.

Vertical : compteur V de 248 à 511 (264 clocks).
- **The chaîne V est cadencée by HSYNC (pas H)** → pendant the 48 premiers clocks H du blanking, V is **en retard d'un cran** (impacte positionnement exact sprites/missiles).
- **VBLANK** : cadencée by 16V, D = `!(128V & 64V & 32V)` → 1 à V=496, 0 à V=272 → 224 px visibles.
- **VSYNC** = `!256V` → 1 à V=248, 0 à V=256.
