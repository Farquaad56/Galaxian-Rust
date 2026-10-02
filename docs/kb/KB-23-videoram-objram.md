# KB-23 — VRAM / OBJRAM : écritures, attributs par colonne et scroll

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp), périmètre **Galaxian d'origine**.

## VRAM (videoram)

| Élément | Valeur |
|---|---|
| Adresse physique | 0x5000–0x53FF (64 Mwords × 8) |
| Taille effective | 0x400 octets, miroir sur 0x400 |
| Accès écriture | `galaxian_videoram_w` ([galaxian_v.cpp:503](../mame_src/galaxian_v.cpp#L503)) |

Format d'un mot VRAM (chaque adresse = une case de la tuilemap 32×32) :
```
Bit(s)   Champ            Description
------   ---------------- ------------------------------------------
0x1f:0   tile_index       indice de tuile (8 bits)
```

`galaxian_videoram_w(offset, data)` ([galaxian_v.cpp:509](../mame_src/galaxian_v.cpp#L509)) répercute l'écriture sur `m_videoram[offset]` et appelle `mark_tile_dirty(offset)` pour que le prochain frame redécode la tuile correspondante.

## Tilemap : création

```cpp
// galaxian_v.cpp:405 — mode normal (Galaxian) : rowscroll individuel
m_bg_tilemap = tilemap_create(*m_gfxdecode, bg_get_tile_info,
    TILEMAP_SCAN_ROWS, m_x_scale*8, 8, 32, 32);
m_bg_tilemap->set_scroll_cols(32);

// galaxian_v.cpp:411 — mode SFX (column scroll) : rowscroll individuel par ligne
m_bg_tilemap = tilemap_create(*m_gfxdecode, bg_get_tile_info,
    TILEMAP_SCAN_COLS, m_x_scale*8, 8, 32, 32);
m_bg_tilemap->set_scroll_rows(32);
```

La fonction `bg_get_tile_info` ([galaxian_v.cpp:487](../mame_src/galaxian_v.cpp#L487)) lit le code dans VRAM et l'attribut couleur dans OBJRAM (`m_spriteram[x*2+1]`) ; un appel `m_extend_tile_info_ptr` optionnel peut modifier `code`/`color`.

## OBJRAM (spriteram) — structure des 0x100 octets

| Décalage | Rôle | Détail |
|---|---|---|
| `0x00–0x3F` | Attributs par colonne (tilemap) | Même format que KB-09 |
| `0x40–0xBF` | Sprites (8 entrées × 4 octets) | 1er sprite generator |
| `0xC0`+ | Shells / missiles | Base configurable via `set_bullets_base()` |

## Premiers 0x40 octets : attributs par colonne du tilemap

Chaque écriture OBJRAM à un décalage `< 0x40` met à jour soit le scroll vertical d'une colonne, soit la couleur de fond associée ([galaxian_v.cpp:514](../mame_src/galaxian_v.cpp#L514)) :

| Offset | Fonction | Mécanisme |
|---|---|---|
| `offset` pair | Scroll Y (colonne normale) ou X (SFX) | `set_scrolly(offset >> 1, data)` — [galaxian_v.cpp:532](../mame_src/galaxian_v.cpp#L532) |
| `offset` impair | Recolore la rangée correspondante | Marque jusqu'à 32 tuiles comme dirty :<br>`for (offset >>= 1; offset < 0x0400; offset += 32)` — [galaxian_v.cpp:540](../mame_src/galaxian_v.cpp#L540) |

Notes :
- `offset >> 1` = indice de rangée (0..63). Les 32 premières colonnes utilisent les paires pair/impair successives.
- Sur Frogger (`m_frogger_adjust`) : le haut et le bas de l'octet de scroll sont échangés (`(data >> 4) | (data << 4)`), voir [galaxian_v.cpp:529](../mame_src/galaxian_v.cpp#L529).
- La couleur extraite du bit de poids fort de l'attribif contrôle aussi le transparent via `set_transparent_pen(0)`.

## Entrées sprite (démarrage à 0x40)

| Offset dans la bloc | Champ         | Format                                                    |
|---------------------|---------------|----------------------------------------------------------|
| `[0]`                | Y             | position verticale (8 bits, flip Y inclus)              |
| `[1]`                | Code + Flip   | `code & 0x3f`, `flip_x = bit6`, `flip_y = bit7`         |
| `[2]`                | Couleur       | `color & 7`                                             |
| `[3]`                | X             | position horizontale (8 bits)                           |

Chaque bloc fait 4 octets, soit 0x20 octets par sprite générateur. Le code final est ajouté au résultat du générateur GFX via le point de fonction `m_extend_sprite_info_ptr`, appelé dans [`sprites_draw`](../mame_src/galaxian_v.cpp#L597).

## Entrées shell / missile (démarrage à 0xC0)

| Offset dans la bloc | Champ         | Usage                                     |
|---------------------|---------------|------------------------------------------|
| `[0]`                | Y             | même format que sprite, comparaison Y-1 |
| `[1]`                | Code + Flip   | bits 6/7 = flip X/Y                     |
| `[2]`                | Couleur       | non utilisé pour le tir                 |
| `[3]`                | X             | position horizontale du tir              |

Le rendu des coquilles et missiles se fait dans [`bullets_draw`](../mame_src/galaxian_v.cpp#L629) : les entrées 0-2 utilisent `effy = y-1`, les entrées 3-6 utilisent `effy = y`, l'entrée 7 est un missile.

## Résumé des offsets OBJRAM (valeur par défaut)

| Offset | Taille   | Contenu                                           |
|--------|----------|--------------------------------------------------|
| 0x00   | 0x40     | Attributs par colonne tilemap (scroll + couleur) |
| 0x40   | 0x80     | Sprites (16 entrées × 4 octets, mais 8 visibles) |
| 0xC0   | variable | Shells / missile (base configurable, défaut 0xC0)|

L'ordre de rendu est donc : **fond noir/étoiles** → **tilemap** → **sprites** → **coquilles/missile**, comme confirmé par [`screen_update_galaxian`](../mame_src/galaxian_v.cpp#L460).
