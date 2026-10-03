# KB-23a — VRAM et tile-info (tilemap fond)

> Source: driver MAME `src/mame/galaxian/` (galaxian_v.cpp, galaxian.h), **périmètre Galaxian d'origine** uniquement.
> Ferme une partie de GAP-01 (format VRAM/OBJRAM détaillé).

## Écriture VRAM : `galaxian_videoram_w` (galaxian_v.cpp:503)

```c
void galaxian_state::galaxian_videoram_w(offs_t offset, uint8_t data)
{
    // update any video up to the current scanline
    m_screen->update_partial(m_screen->vpos());

    // store the data and mark the corresponding tile dirty
    m_videoram[offset] = data;
    m_bg_tilemap->mark_tile_dirty(offset);
}
```

- Écriture d'un octet dans la VRAM à `offset`.
- Met à jour partiellement l'écran jusqu'à la ligne courante (`update_partial`).
- Marque le tile correspondant comme dirty pour recalcul au prochain refresh.

## Création du tilemap : `video_start` (galaxian_v.cpp:399)

```c
m_bg_tilemap = &machine().tilemap().create(
    *m_gfxdecode,
    tilemap_get_info_delegate(*this, FUNC(galaxian_state::bg_get_tile_info)),
    TILEMAP_SCAN_ROWS, m_x_scale*8, 8, 32, 32);
m_bg_tilemap->set_scroll_cols(32);
```

| Paramètre | Valeur | Signification |
|---|---|---|
| scan type | `TILEMAP_SCAN_ROWS` | Scan par rangées (Galaxian d'origine) |
| tile width | `m_x_scale*8` = 24 px | Largeur tuile après scaling X (3×8) |
| tile height | `8` px | Hauteur tuile |
| cols | `32` | Nombre de colonnes de tiles |
| rows | `32` | Nombre de rangées de tiles |
| scroll | `set_scroll_cols(32)` | Scroll vertical par colonne (32 colonnes) |

> **Note** : le tilemap fait 32×32 tuiles = 1024 entrées VRAM. Chaque entrée est un octet
> contenant le code de tile + attributs.

## Lecture tile-info : `bg_get_tile_info` (galaxian_v.cpp:487)

```c
TILE_GET_INFO_MEMBER(galaxian_state::bg_get_tile_info)
{
    uint8_t *videoram = m_videoram;
    uint8_t x = tile_index & 0x1f;
    uint8_t y = tile_index >> 5;

    uint16_t code = videoram[tile_index];
    uint8_t attrib = m_spriteram[x*2+1];
    uint8_t color = attrib & 7;

    m_extend_tile_info_ptr(&code, &color, attrib, x, y);

    tileinfo.set(0, code, color, 0);
}
```

### Décodage des bits VRAM

Chaque octet VRAM contient :
- **bits 0-7** : code de tile (index dans la ROM gfx1) ;
- le code est lu tel quel (`videoram[tile_index]`) et passé à `tileinfo.set`.

### Attributs par colonne (OBJRAM)

L'attribut de couleur n'est pas stocké dans la VRAM mais dans l'**OBJRAM** :
- `m_spriteram[x*2+1]` : octet d'attribut pour la colonne x ;
- bits 0-2 (`& 7`) = index de couleur (palette) ;
- les autres bits sont utilisés par le delegate `m_extend_tile_info_ptr`.

> **Important** : chaque colonne a son propre attribut de couleur, stocké dans l'OBJRAM.
> C'est pourquoi la VRAM ne contient que le code de tile — la couleur est déterminée
> par la position X (colonne) et l'attribut correspondant dans l'OBJRAM.

## Déclaration VRAM (galaxian.h:419)

```c
required_shared_ptr<uint8_t> m_videoram;
```

La VRAM est un buffer partagé de 1024 octets (32×32 tuiles), déclaré dans le state
de la machine. L'écriture passe par `galaxian_videoram_w` qui met à jour le rendu.

## Résumé

| Élément | Valeur |
|---|---|
| VRAM size | 1024 octets (32×32 tuiles) |
| Tilemap scan | `TILEMAP_SCAN_ROWS` |
| Tuile dimensions | 8×8 px (24×8 après scaling X=3) |
| Colonnes/rangées | 32 / 32 |
| Scroll | Vertical par colonne (`set_scroll_cols(32)`) |
| VRAM content | Code de tile uniquement (1 octet/tile) |
| Couleur | Attribut par colonne dans l'OBJRAM (`m_spriteram[x*2+1] & 7`) |
