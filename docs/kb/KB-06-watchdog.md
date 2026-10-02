# KB-06 — Watchdog

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h), périmètre **Galaxian d'origine** uniquement.

Lecture de 7800 (mirroré sur 0x07ff) = reset du watchdog. Absence de lecture pendant la durée du timeout → reset matériel. **La durée du timeout n'est pas dans la doc fournie** (GAP-03).
