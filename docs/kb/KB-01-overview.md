# KB-01 — Vue d'ensemble

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h), périmètre **Galaxian d'origine** uniquement.

Un PCB, trois sections : **CPU** (Z80, ROM jusqu'à 16 Ko + RAM 2 Ko décodée), **Son** (circuit discret analogique : compteur programmable + 4× 555 + bruit LFSR), **Vidéo** (tilemap de caractères + sprites + missiles/shells + champ d'étoiles ; LFSR 17 bits partagé avec le son). Des schémas existent et ont servi de base à MAME.
