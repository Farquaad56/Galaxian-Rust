# KB-26 — Rendu : ordre de composition (confirmé MAME)

> Source : driver MAME `src/mame/galaxian/` (`galaxian_v.cpp`, `galaxian.h`, `galaxian.cpp`), jeu **Galaxian** d'origine uniquement (set `galaxian`, « Galaxian (Namco set 1) », galaxian.cpp:16902). Citations relatives à `docs/mame_src/`.
> Confirme l'hypothèse de KB-14 et clôture GAP-01 (ordre exact de superposition fond/étoiles/tilemap/sprites/coquilles).

## 1. Ordre exact des couches dans `screen_update_galaxian`

Fonction d'update de l'écran, enregistrée par `m_screen->set_screen_update(FUNC(galaxian_state::screen_update_galaxian))` (galaxian.cpp:7503) :

```cpp
uint32_t galaxian_state::screen_update_galaxian(screen_device &screen, bitmap_rgb32 &bitmap, const rectangle &cliprect)  // galaxian_v.cpp:460
{
	/* draw the background layer (including stars) */
	m_draw_background_ptr(bitmap, cliprect);                                                              // :463

	/* draw the tilemap characters over top */
	m_bg_tilemap->draw(screen, bitmap, cliprect, 0, 0);                                                  // :466

	/* render the sprites next. ... */
	for (int i = 0; i < m_numspritegens; i++)                                                            // :469
		sprites_draw(screen, bitmap, cliprect, &m_spriteram[m_sprites_base + i * 0x20]);                 // :470

	/* if we have bullets to draw, render them following */
	if (!m_draw_bullet_ptr.isnull())                                                                     // :473
		bullets_draw(screen, bitmap, cliprect, &m_spriteram[m_bullets_base]);                          // :474

	return 0;                                                                                             // :476
}
```

Ordre du fond vers le premier plan (chaque appel écrase les précédents) :

| # | Couche | Appel | Ligne MAME | Détail critique |
|---|--------|-------|-----------|-----------------|
| 1 | Fond + étoiles | `m_draw_background_ptr(bitmap, cliprect)` | galaxian_v.cpp:463 | Pointeur par défaut = `galaxian_draw_background` : affecté dans `common_init` (galaxian.cpp:8818) et passé explicitement par `init_galaxian` (galaxian.cpp:8833). Voir §1a/§1b. |
| 2 | Tilemap de fond | `m_bg_tilemap->draw(screen, bitmap, cliprect, 0, 0)` | galaxian_v.cpp:466 | Tuiles 8×8, 32 colonnes × 32 lignes (galaxian_v.cpp:405) ; **plume 0 transparente** (`m_bg_tilemap->set_transparent_pen(0)`, galaxian_v.cpp:414) → les étoiles passent au travers. Cf. KB-09, KB-22a. |
| 3 | Sprites (ennemis + joueur) | `sprites_draw(...)` dans la boucle sur `m_numspritegens` | galaxian_v.cpp:469-470 | Set d'origine : **1 générateur** (`m_numspritegens = 1`, galaxian.h:425), base OBJRAM `m_sprites_base = 0x40` (galaxian.h:424) → 8 entrées de 4 octets. Priorité : §2. |
| 4 | Coquilles / missile | `bullets_draw(screen, bitmap, cliprect, &m_spriteram[m_bullets_base])` | galaxian_v.cpp:473-474 | Conditionné par `!m_draw_bullet_ptr.isnull()` (galaxian_v.cpp:473) ; base OBJRAM `m_bullets_base = 0x60` (galaxian.h:423), 8 entrées de 4 octets. Distingué coquilles/missile : §3. |

### 1a. Fond noir (`galaxian_draw_background`)

```cpp
void galaxian_state::galaxian_draw_background(bitmap_rgb32 &bitmap, const rectangle &cliprect)   // galaxian_v.cpp:950
{
	/* erase the background to black first */
	bitmap.fill(rgb_t::black(), cliprect);                                                        // :953

	galaxian_draw_stars(bitmap, cliprect, 256);                                                  // :955
}
```

- Remplissage noir complet **avant** tout pixel coloré (galaxian_v.cpp:952-953) — efface les artefacts de la frame précédente.
- Les étoiles sont ensuite incrustées directement dans la bitmap, sur un champ de 256 colonnes hardware × XSCALE = toute la largeur utile (galaxian_v.cpp:955). Mécanisme complet du générateur d'étoiles : KB-25b §3 « Rendu des étoiles » et KB-12.

### 1b. Étoiles (`galaxian_draw_stars` → `stars_draw_row`)

- `galaxian_draw_stars` (galaxian_v.cpp:932) : met à jour l'origine du RNG (`stars_update_origin`, galaxian_v.cpp:935), puis, si `m_stars_enabled` (galaxian_v.cpp:938), itère scanline par scanline (galaxian_v.cpp:941-945) avec `star_offs = m_star_rng_origin + y * 512` (galaxian_v.cpp:943).
- `stars_draw_row` (galaxian_v.cpp:869) écrit pixel par pixel via `bitmap.pix(...)` (galaxian_v.cpp:903, 911-912), couleur `m_star_color[star & 0x3f]` (galaxian.h:450).

## 2. Priorité des sprites (`sprites_draw`)

Fonction : galaxian_v.cpp:568. Commentaire explicite du driver (galaxian_v.cpp:573-576) :

```cpp
// The line buffer is only written if it contains a '0' currently;
// it is cleared during the visible area, and populated during HBLANK
// To simulate this, we render backwards so that lower numbered sprites
// have priority over higher numbered sprites.
for (int sprnum = 7; sprnum >= 0; sprnum--)                                                     // galaxian_v.cpp:577
```

- **Sémantique hardware** : le buffer de ligne ne reçoit l'écriture d'un sprite que si la case contient encore un `0` — le premier sprite à réclamer un pixel gagne, et le hardware traite les sprites dans l'ordre 0→7. Le sprite n°0 a donc la priorité d'affichage (devant).
- **Simulation MAME** : pour reproduire ce « premier arrivé = devant » avec une écriture qui écrase, MAME dessine en ordre inverse `sprnum = 7 → 0` (galaxian_v.cpp:577) : le sprite n°0 est tracé **en dernier**, donc au-dessus de tous les autres. **Indice faible = premier plan.**
- Chaque sprite est tracé par `m_gfxdecode->gfx(1)->transpen(...)` (galaxian_v.cpp:614-617) avec transparence sur la plume 0 du layout sprite (KB-22b).
- Note parallèle : le Y des sprites suit aussi un match en deux étapes — « the first three sprites match against y-1 » (galaxian_v.cpp:584), `sy = 240 - (base0 - (...(sprnum < 3)))` (galaxian_v.cpp:585) ; cf. §3 pour l'équivalent coquilles/missile.

## 3. Coquilles vs missile (`bullets_draw`) — correction du brouillon

Fonction : galaxian_v.cpp:629. Itération scanline par scanline (galaxian_v.cpp:631-632), avec deux sentinelles `uint8_t shell = 0xff, missile = 0xff;` (galaxian_v.cpp:634).

**Match en deux étapes sur le champ Y de chaque entrée** (`base[which*4+1]`, match si la somme mod 256 vaut `0xff`) :

```cpp
// the first 3 entries match Y-1                                                                  // galaxian_v.cpp:637
effy = m_flipscreen_y ? ((y - 1) ^ 255) : (y - 1);                                               // :638
for (int which = 0; which < 3; which++)                                                          // :639
	if (uint8_t(base[which*4+1] + effy) == 0xff)
		shell = which;                                                                            // :641

// remaining entries match Y                                                                      // :643
effy = m_flipscreen_y ? (y ^ 255) : y;                                                           // :644
for (int which = 3; which < 8; which++)                                                          // :645
	if (uint8_t(base[which*4+1] + effy) == 0xff)
	{
		if (which != 7)                                                                           // :648
			shell = which;                                                                        // :649
		else
			missile = which;                                                                      // :651
	}

// draw the shell
if (shell != 0xff)                                                                                // :655
	m_draw_bullet_ptr(bitmap, cliprect, shell, 255 - base[shell*4+3], y);                         // :656
if (missile != 0xff)                                                                              // :657
	m_draw_bullet_ptr(bitmap, cliprect, missile, 255 - base[missile*4+3], y);                     // :658
```

- **Étapes** : les entrées `which = 0..2` matchent contre **Y−1** (galaxian_v.cpp:637-641) ; les entrées `which = 3..7` matchent contre **Y** (galaxian_v.cpp:643-652).
- **Coquilles vs missile** : les entrées **0 à 6 sont des coquilles** (`shell`, galaxian_v.cpp:641, 649) ; l'entrée **7 est le seul missile** (galaxian_v.cpp:648-651).
- **Correction du brouillon partiel** : la mention « entrées 0–2 (coquilles), entrée 7 (missile) » était fausse — les coquilles sont les **sept** entrées 0–6, seules l'étape de match (Y−1 pour 0–2, Y pour 3–6) et la couleur diffèrent.
- Si plusieurs entrées matchent sur une même ligne, la dernière en indice gagne (`shell` est réécrit, galaxian_v.cpp:641/649) ; le missile est tracé **après** la coquille (galaxian_v.cpp:655-658), donc devant si les deux tombent sur la même ligne.
- Rendu par défaut `galaxian_draw_bullet` (galaxian_v.cpp:1160) : tir de 4 pixels, « The first 7 entries are called "shells" and render as white; the final entry is called a "missile" and renders as yellow » (galaxian_v.cpp:1162-1167) ; couleurs `m_bullet_color[0..6] = blanc`, `m_bullet_color[7] = jaune` (galaxian_v.cpp:355-358). Cf. KB-11, KB-23b.

## 4. Cross-références

- **KB-25b** §3 « Écran (raster) » et §« Rendu des étoiles » : timing écran (`set_raw`, galaxian.cpp:7502), mécanisme du LFSR d'étoiles, horloge asymétrique du RNG.
- **KB-09** / **KB-22a** : tilemap de fond (format VRAM/OBJRAM, attributs par colonne) et layout GFX des tuiles 8×8 (`galaxian_charlayout`, galaxian.cpp:7394).
- **KB-27** (à venir) : palette — couleurs des coquilles/missile (§3) et des étoiles (§1b) dérivées de la palette PROM.
- **KB-14** (hypothèse déduite de la doc) : confirmée point par point par cette fiche ; GAP-01 clos.

## Differs (hors périmètre — variantes)

- Plusieurs générateurs de sprites sur certains PCB (« zigzag, fantastc », galaxian_v.cpp:468) → `m_numspritegens > 1` (galaxian.h:337).
- Hardware SFX : tilemap par colonnes au lieu de lignes (`TILEMAP_SCAN_COLS`, galaxian_v.cpp:410-412), et sens du match Y inversé pour les sprites (galaxian_v.cpp:584-585).
- `null_draw_background` (fond noir sans étoiles, galaxian_v.cpp:925-929) et variantes de couleurs de missile (ex. moonwar, galaxian_v.cpp:361-367 ; mshuttle, galaxian_v.cpp:1176+).
